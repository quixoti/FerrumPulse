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
                "chrome.exe".into(),
                "msedge.exe".into(),
                "brave.exe".into(),
                "firefox.exe".into(),
                "steamwebhelper.exe".into(),
                "battlenet.exe".into(),
                "spotify.exe".into(),
            ],
        }
    }
}

/// Hardcoded protected application whitelist that must NEVER be throttled under any circumstance.
/// Protects voice communications, live streaming captures, core Windows audio, and primary launchers.
pub const PROTECTED_THROTTLING_WHITELIST: &[&str] = &[
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
}

pub struct ProcessPacingManager {
    config: Arc<Mutex<DaemonConfig>>,
    running: Arc<AtomicBool>,
    status: Arc<Mutex<DaemonStatus>>,
}

impl ProcessPacingManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(DaemonConfig::default())),
            running: Arc::new(AtomicBool::new(false)),
            status: Arc::new(Mutex::new(DaemonStatus {
                is_running: false,
                active_game: None,
                boosted_pid: None,
                throttled_count: 0,
            })),
        }
    }
}

impl Default for ProcessPacingManager {
    fn default() -> Self {
        Self::new()
    }
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

        let config_clone = Arc::clone(&self.config);
        let running_clone = Arc::clone(&self.running);
        let status_clone = Arc::clone(&self.status);

        std::thread::spawn(move || {
            let mut last_boosted_pid: Option<u32> = None;
            let mut last_throttled_pids: HashSet<u32> = HashSet::new();
            let mut access_denied_blacklist: HashSet<u32> = HashSet::new();
            let mut was_ac_connected = true;
            let mut timer_active = false;

            while running_clone.load(Ordering::SeqCst) {
                let current_cfg = { config_clone.lock().unwrap().clone() };
                let battery_telem = crate::battery::get_battery_telemetry();

                // Laptop AC/DC Power Failsafe:
                // When transitioning from AC to Battery power, immediately revert core parking to stock 50%,
                // switch to Balanced power scheme, and release any multimedia timer locks.
                if was_ac_connected && !battery_telem.ac_connected {
                    crate::battery::enforce_battery_and_standby_failsafe();
                    if timer_active {
                        unlock_high_resolution_timer();
                        timer_active = false;
                    }
                    was_ac_connected = false;
                } else if !was_ac_connected && battery_telem.ac_connected {
                    was_ac_connected = true;
                }

                if !current_cfg.enabled {
                    // Revert any changes if disabled
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
                    let mut st = status_clone.lock().unwrap();
                    st.is_running = false;
                    st.active_game = None;
                    st.boosted_pid = None;
                    st.throttled_count = 0;

                    let sleep_ms = if battery_telem.ac_connected { 1000 } else { 3000 };
                    std::thread::sleep(Duration::from_millis(sleep_ms));
                    continue;
                }

                // Check foreground window process
                let fg_proc = get_foreground_process();

                let is_game = if let Some((_, ref name)) = fg_proc {
                    let lower = name.to_lowercase();
                    current_cfg
                        .target_games
                        .iter()
                        .any(|g| lower == g.to_lowercase() || lower.ends_with(&g.to_lowercase()))
                } else {
                    false
                };

                if is_game {
                    // Lock scheduler timer to 1.0ms for sub-millisecond input/frame pacing (only while on AC or enabled)
                    if !timer_active && battery_telem.ac_connected {
                        lock_high_resolution_timer();
                        timer_active = true;
                    }

                    if let Some((fg_pid, ref fg_name)) = fg_proc {
                        // Boost foreground game with anti-cheat graceful fallback and blacklisting
                        if current_cfg.boost_priority && last_boosted_pid != Some(fg_pid) {
                            if access_denied_blacklist.contains(&fg_pid) {
                                // Anti-cheat protected process: skip quietly without tight-loop syscall retries
                            } else {
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

                        // Throttle background apps (skips protected companion whitelist)
                        let mut throttled_count = 0;
                        if current_cfg.throttle_background {
                            let newly_throttled = throttle_background_processes(
                                fg_pid,
                                &current_cfg.throttled_apps,
                            );
                            throttled_count = newly_throttled.len();
                            last_throttled_pids = newly_throttled;
                        }

                        let mut st = status_clone.lock().unwrap();
                        st.is_running = true;
                        st.active_game = Some(fg_name.clone());
                        st.boosted_pid = last_boosted_pid;
                        st.throttled_count = throttled_count;
                    }
                } else {
                    // Not gaming, release timer lock and revert elevated priorities and throttling
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

                    let mut st = status_clone.lock().unwrap();
                    st.is_running = true;
                    st.active_game = None;
                    st.boosted_pid = None;
                    st.throttled_count = 0;
                }

                let sleep_ms = if crate::battery::get_battery_telemetry().ac_connected {
                    1000
                } else {
                    3000
                };
                std::thread::sleep(Duration::from_millis(sleep_ms));
            }

            // Cleanup when daemon stops
            if let Some(pid) = last_boosted_pid {
                revert_priority(pid);
            }
            for pid in last_throttled_pids {
                revert_throttling(pid);
            }
        });
    }

    pub fn stop_daemon(&self) {
        self.running.store(false, Ordering::SeqCst);
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

/// Sets ProcessPowerThrottling (EcoQoS / Execution Speed) on background launchers and browsers.
pub fn throttle_background_processes(
    exclude_pid: u32,
    target_names: &[String],
) -> HashSet<u32> {
    let mut throttled = HashSet::new();

    #[cfg(windows)]
    {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

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
        let _ = (exclude_pid, target_names);
    }

    throttled
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

