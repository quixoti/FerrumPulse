use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HWND};
#[cfg(windows)]
use windows::Win32::System::Threading::{
    OpenProcess, SetPriorityClass, SetProcessInformation, HIGH_PRIORITY_CLASS,
    NORMAL_PRIORITY_CLASS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION, ProcessPowerThrottling,
    QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

#[cfg(windows)]
#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(uPeriod: u32) -> u32;
    fn timeEndPeriod(uPeriod: u32) -> u32;
}

pub fn lock_high_resolution_timer() {
    #[cfg(windows)]
    unsafe {
        let _ = timeBeginPeriod(1);
    }
}

pub fn unlock_high_resolution_timer() {
    #[cfg(windows)]
    unsafe {
        let _ = timeEndPeriod(1);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub enabled: bool,
    pub boost_priority: bool,
    pub throttle_background: bool,
    pub target_games: Vec<String>,
    pub throttled_apps: Vec<String>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            boost_priority: true,
            throttle_background: true,
            target_games: vec![
                "cs2.exe".into(),
                "valorant.exe".into(),
                "fortniteclient-win64-shipping.exe".into(),
                "apex.exe".into(),
                "r5apex.exe".into(),
                "overwatch.exe".into(),
                "cyberpunk2077.exe".into(),
                "gta5.exe".into(),
                "dota2.exe".into(),
                "league of legends.exe".into(),
                "cod.exe".into(),
                "destiny2.exe".into(),
            ],
            throttled_apps: vec![
                "steamwebhelper.exe".into(),
                "battlenet.exe".into(),
                "spotify.exe".into(),
            ],
        }
    }
}

/// Hardcoded protected application whitelist that must NEVER be throttled under any circumstance.
/// Protects web browsers, voice communications, live streaming captures, core Windows audio, and primary launchers.
pub const PROTECTED_THROTTLING_WHITELIST: &[&str] = &[
    // Web Browsers (Strictly protected to prevent video stutters, tab freezing, or audio desync)
    "chrome.exe",
    "msedge.exe",
    "brave.exe",
    "firefox.exe",
    "opera.exe",
    "vivaldi.exe",
    "arc.exe",
    // Voice & Communication
    "discord.exe",
    "discordcanary.exe",
    "ts3client_win64.exe",
    // Broadcast & Capture
    "obs64.exe",
    "obs32.exe",
    "streamlabs obs.exe",
    // Windows Core Audio & System
    "audiodg.exe",
    "dwm.exe",
    "explorer.exe",
    "systemsettings.exe",
    // Game Launchers
    "steam.exe",
    "epicgameslauncher.exe",
    "riotclientservices.exe",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoostResult {
    Success,
    AccessDenied,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub is_running: bool,
    pub active_game: Option<String>,
    pub boosted_pid: Option<u32>,
    pub throttled_count: usize,
    pub suspended_webviews_count: usize,
}

pub struct ProcessPacingManager {
    config: Arc<Mutex<DaemonConfig>>,
    profiles_store: crate::profiles::SharedProfilesStore,
    battery_config: crate::battery::SharedBatteryConfig,
    audio_config: crate::audio::SharedAudioConfig,
    memory_guard_config: crate::memory::SharedMemoryGuardConfig,
    running: Arc<AtomicBool>,
    stop_signal: Arc<(Mutex<bool>, std::sync::Condvar)>,
    status: Arc<Mutex<DaemonStatus>>,
    worker_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl ProcessPacingManager {
    pub fn new() -> Self {
        use crate::profiles::ProfileRepository;
        let initial_store = crate::profiles::JsonFileProfileRepository::new()
            .load()
            .unwrap_or_default();
        Self::with_profiles_store(Arc::new(std::sync::RwLock::new(initial_store)))
    }

    pub fn with_profiles_store(profiles_store: crate::profiles::SharedProfilesStore) -> Self {
        Self {
            config: Arc::new(Mutex::new(DaemonConfig::default())),
            profiles_store,
            battery_config: Arc::new(std::sync::RwLock::new(crate::battery::load_battery_config())),
            audio_config: Arc::new(std::sync::RwLock::new(crate::audio::load_audio_config())),
            memory_guard_config: Arc::new(std::sync::RwLock::new(
                crate::memory::load_memory_guard_config(),
            )),
            running: Arc::new(AtomicBool::new(false)),
            stop_signal: Arc::new((Mutex::new(false), std::sync::Condvar::new())),
            status: Arc::new(Mutex::new(DaemonStatus {
                is_running: false,
                active_game: None,
                boosted_pid: None,
                throttled_count: 0,
                suspended_webviews_count: 0,
            })),
            worker_handle: Mutex::new(None),
        }
    }

    pub fn get_profiles_store(&self) -> crate::profiles::SharedProfilesStore {
        Arc::clone(&self.profiles_store)
    }

    pub fn refresh_profiles_store(&self, store: crate::profiles::ProfilesStore) {
        if let Ok(mut guard) = self.profiles_store.write() {
            *guard = store;
        }
    }

    pub fn get_battery_config(&self) -> crate::battery::BatteryConfig {
        self.battery_config
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn update_battery_config(&self, cfg: crate::battery::BatteryConfig) {
        if let Ok(mut guard) = self.battery_config.write() {
            *guard = cfg;
        }
    }

    pub fn get_audio_config(&self) -> crate::audio::AudioTuningConfig {
        self.audio_config
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn update_audio_config(&self, cfg: crate::audio::AudioTuningConfig) {
        if let Ok(mut guard) = self.audio_config.write() {
            *guard = cfg;
        }
    }

    pub fn get_memory_guard_config(&self) -> crate::memory::MemoryGuardConfig {
        self.memory_guard_config
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn update_memory_guard_config(&self, cfg: crate::memory::MemoryGuardConfig) {
        if let Ok(mut guard) = self.memory_guard_config.write() {
            *guard = cfg;
        }
    }
}

impl Default for ProcessPacingManager {
    fn default() -> Self {
        Self::new()
    }
}

struct ActiveGameSession {
    profile: Option<crate::profiles::GameProfile>,
    game_pid: u32,
    previous_refresh_rate: Option<u32>,
    power_applied: bool,
}

impl ProcessPacingManager {

    pub fn get_config(&self) -> DaemonConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn update_config(&self, new_cfg: DaemonConfig) {
        let mut cfg = self.config.lock().unwrap();
        *cfg = new_cfg;
    }

    pub fn get_status(&self) -> DaemonStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn start_daemon(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            return; // Already running
        }
        {
            let mut stopped = self.stop_signal.0.lock().unwrap();
            *stopped = false;
        }

        let config_clone = Arc::clone(&self.config);
        let profiles_clone = Arc::clone(&self.profiles_store);
        let battery_cfg_clone = Arc::clone(&self.battery_config);
        let audio_cfg_clone = Arc::clone(&self.audio_config);
        let mem_guard_clone = Arc::clone(&self.memory_guard_config);
        let running_clone = Arc::clone(&self.running);
        let stop_signal_clone = Arc::clone(&self.stop_signal);
        let status_clone = Arc::clone(&self.status);

        let handle = std::thread::spawn(move || {
            // Forcibly clear any previous EcoQoS clamps on web browsers on daemon launch
            unthrottle_all_browsers();

            let mut sys = sysinfo::System::new();
            let mut suspender = crate::suspender::LauncherSuspender::new();
            let mut active_session: Option<ActiveGameSession> = None;

            let mut last_boosted_pid: Option<u32> = None;
            let mut last_throttled_pids: HashSet<u32> = HashSet::new();
            let mut access_denied_blacklist: HashSet<u32> = HashSet::new();
            let mut last_tuned_audiodg_pid: Option<u32> = None;
            let mut last_auto_trim_at: Option<std::time::Instant> = None;
            let mut was_ac_connected = true;
            let mut timer_active = false;

            while running_clone.load(Ordering::SeqCst) {
                let current_cfg = { config_clone.lock().unwrap().clone() };
                let mut bat_cfg = battery_cfg_clone
                    .read()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                let audio_cfg = audio_cfg_clone
                    .read()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                let mem_guard_cfg = mem_guard_clone
                    .read()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                let battery_telem = crate::battery::get_battery_telemetry_with_config(&bat_cfg);
                let profiles_store = profiles_clone
                    .read()
                    .map(|g| g.clone())
                    .unwrap_or_default();

                // Reuse single sysinfo::System instance across ticks to avoid per-second allocations
                sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

                // Ambient Audio Engine Guard: auto-reapply audiodg.exe core pinning & priority if restarted
                crate::audio::reapply_if_audiodg_restarted(
                    &sys,
                    audio_cfg.enabled,
                    &mut last_tuned_audiodg_pid,
                );

                // Opt-in >88% RAM Pressure Guardrail (60s cooldown, strictly skips active game/browsers/OS)
                if mem_guard_cfg.auto_trim_at_high_pressure {
                    if let Ok(mem_stats) = crate::memory::get_memory_stats() {
                        let elapsed_secs = last_auto_trim_at.map(|t| t.elapsed().as_secs());
                        if crate::memory::should_auto_trim_ram(
                            mem_stats.memory_load_percent,
                            &mem_guard_cfg,
                            elapsed_secs,
                        ) {
                            let _ = crate::memory::purge_working_sets();
                            last_auto_trim_at = Some(std::time::Instant::now());
                        }
                    }
                }

                // Laptop AC/DC Power Symmetry & Failsafe:
                // AC -> DC: Save pre-battery AC refresh rate, resume webviews, release timer, apply Eco policies.
                // DC -> AC: Auto-restore saved AC refresh rate and AC Balanced baseline if auto_switch is enabled.
                if was_ac_connected && !battery_telem.ac_connected {
                    suspender.resume_all();
                    crate::battery::enforce_battery_and_standby_failsafe();
                    if timer_active {
                        unlock_high_resolution_timer();
                        timer_active = false;
                    }
                    if let Some(ref mut sess) = active_session {
                        if let Some(orig_hz) = sess.previous_refresh_rate.take() {
                            let _ = crate::battery::set_display_refresh_rate(orig_hz);
                        }
                        sess.power_applied = false;
                    }
                    if bat_cfg.auto_switch {
                        if let Some(cur_hz) = crate::battery::get_current_display_refresh_rate() {
                            if cur_hz > 60 {
                                bat_cfg.saved_ac_refresh_rate = Some(cur_hz);
                                let _ = crate::battery::save_battery_config(&bat_cfg);
                                if let Ok(mut guard) = battery_cfg_clone.write() {
                                    *guard = bat_cfg.clone();
                                }
                            }
                        }
                        let _ = crate::battery::apply_battery_eco_profile_with_config(true, &bat_cfg);
                    }
                    was_ac_connected = false;
                } else if !was_ac_connected && battery_telem.ac_connected {
                    if bat_cfg.auto_switch {
                        let _ = crate::battery::apply_battery_eco_profile_with_config(false, &bat_cfg);
                    }
                    was_ac_connected = true;
                }

                if !current_cfg.enabled {
                    suspender.resume_all();
                    if timer_active {
                        unlock_high_resolution_timer();
                        timer_active = false;
                    }
                    if let Some(pid) = last_boosted_pid.take() {
                        revert_priority(pid);
                    }
                    for pid in last_throttled_pids.drain() {
                        revert_throttling(pid);
                    }
                    if let Some(sess) = active_session.take() {
                        if let Some(orig_hz) = sess.previous_refresh_rate {
                            let _ = crate::battery::set_display_refresh_rate(orig_hz);
                        }
                        if sess.power_applied {
                            let _ = crate::power::set_power_scheme(&profiles_store.default_desktop.power_scheme);
                            let _ = crate::power::set_core_unparking(profiles_store.default_desktop.core_unparking_percent == 100);
                        }
                    }

                    let mut st = status_clone.lock().unwrap();
                    st.is_running = false;
                    st.active_game = None;
                    st.boosted_pid = None;
                    st.throttled_count = 0;
                    st.suspended_webviews_count = 0;

                    let sleep_ms = if battery_telem.ac_connected { 1000 } else { 3000 };
                    let (lock, cvar) = &*stop_signal_clone;
                    let guard = lock.lock().unwrap();
                    if !*guard {
                        let _ = cvar.wait_timeout(guard, Duration::from_millis(sleep_ms));
                    }
                    continue;
                }

                // Check foreground window process
                let fg_proc = get_foreground_process();

                let matched_profile = if let Some((_, ref name)) = fg_proc {
                    profiles_store.find_by_executable(name).cloned()
                } else {
                    None
                };

                let fg_is_game = if matched_profile.is_some() {
                    true
                } else if let Some((_, ref name)) = fg_proc {
                    let lower = name.to_lowercase();
                    current_cfg
                        .target_games
                        .iter()
                        .any(|g| lower == g.to_lowercase() || lower.ends_with(&g.to_lowercase()))
                } else {
                    false
                };

                // Detect if an existing tracked game session's PID has actually terminated
                let tracked_alive = active_session
                    .as_ref()
                    .map(|s| sys.process(sysinfo::Pid::from_u32(s.game_pid)).is_some())
                    .unwrap_or(false);

                if active_session.is_some() && !tracked_alive {
                    // Tracked game exited! Perform complete session rollback once.
                    suspender.resume_all();
                    if let Some(finished_sess) = active_session.take() {
                        if let Some(orig_hz) = finished_sess.previous_refresh_rate {
                            let _ = crate::battery::set_display_refresh_rate(orig_hz);
                        }
                        if finished_sess.power_applied {
                            let _ = crate::power::set_power_scheme(&profiles_store.default_desktop.power_scheme);
                            let _ = crate::power::set_core_unparking(
                                profiles_store.default_desktop.core_unparking_percent == 100,
                            );
                        }
                        if finished_sess
                            .profile
                            .as_ref()
                            .map(|p| p.post_game_ram_sweep)
                            .unwrap_or(false)
                        {
                            std::thread::spawn(|| {
                                std::thread::sleep(Duration::from_secs(5));
                                let _ = crate::memory::purge_working_sets();
                            });
                        }
                    }
                }

                if fg_is_game {
                    if let Some((fg_pid, ref fg_name)) = fg_proc {
                        // Start or update active game session if switching to a new game PID
                        let is_new_session = active_session
                            .as_ref()
                            .map(|s| s.game_pid != fg_pid)
                            .unwrap_or(true);

                        if is_new_session {
                            // Clean up prior session if switching straight from one game to another
                            if let Some(prev) = active_session.take() {
                                if let Some(orig_hz) = prev.previous_refresh_rate {
                                    let _ = crate::battery::set_display_refresh_rate(orig_hz);
                                }
                            }

                            let mut previous_refresh_rate = None;
                            let mut power_applied = false;

                            if let Some(ref prof) = matched_profile {
                                if battery_telem.ac_connected {
                                    let _ = crate::power::set_power_scheme(&prof.power_scheme);
                                    let _ = crate::power::set_core_unparking(prof.core_unparking_percent == 100);
                                    power_applied = true;
                                }
                                if let Some(hz) = prof.target_refresh_rate {
                                    let current_hz = crate::battery::get_current_display_refresh_rate();
                                    if current_hz != Some(hz) {
                                        if crate::battery::set_display_refresh_rate(hz).is_ok() {
                                            previous_refresh_rate = current_hz;
                                        }
                                    }
                                }
                            }

                            active_session = Some(ActiveGameSession {
                                profile: matched_profile.clone(),
                                game_pid: fg_pid,
                                previous_refresh_rate,
                                power_applied,
                            });
                        }

                        // Timer lock
                        let should_lock_timer = matched_profile
                            .as_ref()
                            .map(|p| p.lock_timer_resolution)
                            .unwrap_or(true);

                        if should_lock_timer && !timer_active && battery_telem.ac_connected {
                            lock_high_resolution_timer();
                            timer_active = true;
                        }

                        // Priority elevation
                        let should_boost = matched_profile
                            .as_ref()
                            .map(|p| p.boost_priority)
                            .unwrap_or(current_cfg.boost_priority);

                        if should_boost && last_boosted_pid != Some(fg_pid) {
                            if !access_denied_blacklist.contains(&fg_pid) {
                                if let Some(old_pid) = last_boosted_pid {
                                    revert_priority(old_pid);
                                }
                                match boost_process_priority(fg_pid) {
                                    BoostResult::Success => {
                                        last_boosted_pid = Some(fg_pid);
                                    }
                                    BoostResult::AccessDenied => {
                                        eprintln!(
                                            "[FerrumPulse] Anti-cheat detected on PID {} ({}). Blacklisting from priority elevation.",
                                            fg_pid, fg_name
                                        );
                                        access_denied_blacklist.insert(fg_pid);
                                        last_boosted_pid = None;
                                    }
                                    BoostResult::Failed => {
                                        last_boosted_pid = None;
                                    }
                                }
                            }
                        }

                        // Background throttling using reused `sys`
                        let mut throttled_count = 0;
                        if current_cfg.throttle_background {
                            let newly_throttled = throttle_background_processes_with_sys(
                                &sys,
                                fg_pid,
                                &current_cfg.throttled_apps,
                            );
                            throttled_count = newly_throttled.len();
                            last_throttled_pids = newly_throttled;
                        }

                        // Launcher webview suspension (default true when on AC)
                        let allow_suspend = battery_telem.ac_connected
                            && matched_profile
                                .as_ref()
                                .map(|p| p.suspend_launcher_webviews)
                                .unwrap_or(true);

                        let suspended_count = suspender.update_lifecycle(
                            &sys,
                            true,
                            true,
                            Some(fg_name),
                            allow_suspend,
                            Some(fg_pid),
                        );

                        let mut st = status_clone.lock().unwrap();
                        st.is_running = true;
                        st.active_game = Some(fg_name.clone());
                        st.boosted_pid = last_boosted_pid;
                        st.throttled_count = throttled_count;
                        st.suspended_webviews_count = suspended_count;
                    }
                } else {
                    // Not currently focused on a game window: release timer & elevated priorities immediately
                    if timer_active {
                        unlock_high_resolution_timer();
                        timer_active = false;
                    }

                    if let Some(old_pid) = last_boosted_pid.take() {
                        revert_priority(old_pid);
                    }
                    if !last_throttled_pids.is_empty() {
                        for pid in last_throttled_pids.drain() {
                            revert_throttling(pid);
                        }
                    }

                    // Smart Resume lifecycle check if game is still alive in the background (Alt-Tabbed)
                    let session_alive = active_session.is_some();
                    let allow_suspend = battery_telem.ac_connected
                        && active_session
                            .as_ref()
                            .and_then(|s| s.profile.as_ref())
                            .map(|p| p.suspend_launcher_webviews)
                            .unwrap_or(true);

                    let fg_name_opt = fg_proc.as_ref().map(|(_, n)| n.as_str());
                    let suspended_count = suspender.update_lifecycle(
                        &sys,
                        session_alive,
                        false,
                        fg_name_opt,
                        allow_suspend,
                        active_session.as_ref().map(|s| s.game_pid),
                    );

                    let mut st = status_clone.lock().unwrap();
                    st.is_running = true;
                    st.active_game = None;
                    st.boosted_pid = None;
                    st.throttled_count = 0;
                    st.suspended_webviews_count = suspended_count;
                }

                let sleep_ms = if battery_telem.ac_connected { 1000 } else { 3000 };
                let (lock, cvar) = &*stop_signal_clone;
                let guard = lock.lock().unwrap();
                if !*guard {
                    let _ = cvar.wait_timeout(guard, Duration::from_millis(sleep_ms));
                }
            }

            // Guaranteed deterministic cleanup before thread exits
            suspender.resume_all();
            if timer_active {
                unlock_high_resolution_timer();
            }
            if let Some(pid) = last_boosted_pid {
                revert_priority(pid);
            }
            for pid in last_throttled_pids {
                revert_throttling(pid);
            }
            if let Some(sess) = active_session {
                if let Some(orig_hz) = sess.previous_refresh_rate {
                    let _ = crate::battery::set_display_refresh_rate(orig_hz);
                }
                if sess.power_applied {
                    if let Ok(store) = profiles_clone.read() {
                        let _ = crate::power::set_power_scheme(&store.default_desktop.power_scheme);
                        let _ = crate::power::set_core_unparking(
                            store.default_desktop.core_unparking_percent == 100,
                        );
                    }
                }
            }
        });

        *self.worker_handle.lock().unwrap() = Some(handle);
    }

    pub fn stop_daemon(&self) {
        self.running.store(false, Ordering::SeqCst);
        {
            let (lock, cvar) = &*self.stop_signal;
            let mut stopped = lock.lock().unwrap();
            *stopped = true;
            cvar.notify_all();
        }
        if let Some(handle) = self.worker_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    /// Graceful full shutdown: stops daemon, waits for deterministic rollback & webview resume.
    pub fn stop_and_cleanup(&self) {
        self.stop_daemon();
    }
}

impl Drop for ProcessPacingManager {
    fn drop(&mut self) {
        self.stop_daemon();
    }
}

/// Retrieves the PID and executable filename of the current active foreground window.
pub fn get_foreground_process() -> Option<(u32, String)> {
    #[cfg(windows)]
    {
        // SAFETY: GetForegroundWindow returns the top-level window handle or null
        let hwnd: HWND = unsafe { GetForegroundWindow() };
        if hwnd.0.is_null() {
            return None;
        }

        let mut pid: u32 = 0;
        // SAFETY: Querying the thread and process ID owning the HWND
        let thread_id = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if thread_id == 0 || pid == 0 {
            return None;
        }

        // Query process executable name
        // SAFETY: Opening process with query limited info rights
        let handle_res = unsafe {
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
        };

        if let Ok(handle) = handle_res {
            let mut buffer = [0u16; 1024];
            let mut size: u32 = buffer.len() as u32;

            // SAFETY: Reading image path into pre-allocated UTF-16 buffer
            let query_res = unsafe {
                QueryFullProcessImageNameW(
                    handle,
                    PROCESS_NAME_FORMAT(0),
                    windows::core::PWSTR(buffer.as_mut_ptr()),
                    &mut size,
                )
            };

            // SAFETY: Cleanly closing the opened process handle via CloseHandle
            unsafe {
                let _ = CloseHandle(handle);
            };

            if query_res.is_ok() && size > 0 {
                let raw_path = String::from_utf16_lossy(&buffer[..size as usize]);
                let file_name = Path::new(&raw_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&raw_path)
                    .to_string();
                return Some((pid, file_name));
            }
        }

        None
    }

    #[cfg(not(windows))]
    {
        None
    }
}

/// Sets process priority to HIGH_PRIORITY_CLASS with defensive error checking and handle limits.
/// Restricts boost ceiling strictly to HIGH_PRIORITY_CLASS (REALTIME is banned to prevent thread starvation of DWM/Audio).
pub fn boost_process_priority(pid: u32) -> BoostResult {
    #[cfg(windows)]
    {
        // SAFETY: Requesting minimum required rights: PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION.
        // Never requests PROCESS_ALL_ACCESS to respect anti-cheat boundaries.
        let handle_res = unsafe {
            OpenProcess(
                PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION,
                false,
                pid,
            )
        };

        match handle_res {
            Ok(handle) => {
                // SAFETY: Elevating game priority strictly to HIGH_PRIORITY_CLASS.
                // Under NO circumstances is REALTIME_PRIORITY_CLASS permitted, preventing thread starvation
                // of DWM, core audio (audiodg.exe), and input subsystems.
                let res = unsafe { SetPriorityClass(handle, HIGH_PRIORITY_CLASS) };
                unsafe {
                    let _ = CloseHandle(handle);
                };
                if res.is_ok() {
                    BoostResult::Success
                } else {
                    BoostResult::Failed
                }
            }
            Err(e) => {
                // Check for ERROR_ACCESS_DENIED (0x80070005) commonly returned by Vanguard, EAC, or BattlEye
                if e.code().0 as u32 == 0x80070005 {
                    BoostResult::AccessDenied
                } else {
                    BoostResult::Failed
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = pid;
        BoostResult::Success
    }
}

/// Reverts process priority back to NORMAL_PRIORITY_CLASS.
pub fn revert_priority(pid: u32) -> bool {
    #[cfg(windows)]
    {
        // SAFETY: Opening process to restore default NORMAL priority
        let handle_res = unsafe {
            OpenProcess(PROCESS_SET_INFORMATION, false, pid)
        };

        if let Ok(handle) = handle_res {
            let res = unsafe { SetPriorityClass(handle, NORMAL_PRIORITY_CLASS) };
            unsafe {
                let _ = CloseHandle(handle);
            };
            return res.is_ok();
        }
        false
    }

    #[cfg(not(windows))]
    {
        let _ = pid;
        true
    }
}

/// Sets ProcessPowerThrottling (EcoQoS / Execution Speed) using an existing sysinfo::System snapshot.
pub fn throttle_background_processes_with_sys(
    sys: &sysinfo::System,
    exclude_pid: u32,
    target_names: &[String],
) -> HashSet<u32> {
    let mut throttled = HashSet::new();

    #[cfg(windows)]
    {
        for (pid, process) in sys.processes() {
            let raw_pid = pid.as_u32();
            if raw_pid == exclude_pid {
                continue;
            }

            let proc_name = process.name().to_string_lossy().to_lowercase();

            // HARD GUARD: Skip apps in the protected companion whitelist (Voice, OBS, Audio, System, Launchers)
            if PROTECTED_THROTTLING_WHITELIST.iter().any(|&p| proc_name == p || proc_name.ends_with(p)) {
                continue;
            }

            let matches = target_names.iter().any(|target| {
                let t_lower = target.to_lowercase();
                proc_name == t_lower || proc_name.ends_with(&t_lower)
            });

            if matches {
                // SAFETY: Opening background process for SET_INFORMATION
                let handle_res = unsafe {
                    OpenProcess(PROCESS_SET_INFORMATION, false, raw_pid)
                };

                if let Ok(handle) = handle_res {
                    let mut throttling = PROCESS_POWER_THROTTLING_STATE {
                        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
                        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
                        StateMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
                    };

                    // SAFETY: Enabling power throttling on background task
                    let res = unsafe {
                        SetProcessInformation(
                            handle,
                            ProcessPowerThrottling,
                            &mut throttling as *mut _ as _,
                            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
                        )
                    };

                    unsafe {
                        let _ = CloseHandle(handle);
                    };

                    if res.is_ok() {
                        throttled.insert(raw_pid);
                    }
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (sys, exclude_pid, target_names);
    }

    throttled
}

/// Sets ProcessPowerThrottling (EcoQoS / Execution Speed) on background launchers and companion apps.
pub fn throttle_background_processes(
    exclude_pid: u32,
    target_names: &[String],
) -> HashSet<u32> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    throttle_background_processes_with_sys(&sys, exclude_pid, target_names)
}

/// Reverts process power throttling back to normal execution speed.
pub fn revert_throttling(pid: u32) -> bool {
    #[cfg(windows)]
    {
        // SAFETY: Opening process to disable power throttling
        let handle_res = unsafe {
            OpenProcess(PROCESS_SET_INFORMATION, false, pid)
        };

        if let Ok(handle) = handle_res {
            let mut throttling = PROCESS_POWER_THROTTLING_STATE {
                Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
                ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
                StateMask: 0, // Cleared -> unthrottled
            };

            let res = unsafe {
                SetProcessInformation(
                    handle,
                    ProcessPowerThrottling,
                    &mut throttling as *mut _ as _,
                    std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
                )
            };

            unsafe {
                let _ = CloseHandle(handle);
            };

            return res.is_ok();
        }
        false
    }

    #[cfg(not(windows))]
    {
        let _ = pid;
        true
    }
}

/// Discovers all running web browser processes and clears EcoQoS / Efficiency Mode flags.
pub fn unthrottle_all_browsers() -> usize {
    let mut cleared = 0;
    #[cfg(windows)]
    {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let browsers = [
            "chrome.exe",
            "msedge.exe",
            "brave.exe",
            "firefox.exe",
            "opera.exe",
            "vivaldi.exe",
            "arc.exe",
        ];

        for (pid, process) in sys.processes() {
            let proc_name = process.name().to_string_lossy().to_lowercase();
            if browsers.iter().any(|&b| proc_name == b || proc_name.ends_with(b)) {
                if revert_throttling(pid.as_u32()) {
                    cleared += 1;
                }
            }
        }
    }
    cleared
}


