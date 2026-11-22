// src/suspender.rs - Intelligent Launcher & Webview Suspender
// Targets ONLY non-essential Chromium renderer helpers (--type=renderer) during gameplay.
// Strictly preserves master launchers, overlays, cloud saves, downloads, browsers, Discord, and OBS.

use crate::process::PROTECTED_THROTTLING_WHITELIST;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS};

#[cfg(windows)]
#[link(name = "ntdll")]
extern "system" {
    fn NtSuspendProcess(process_handle: *mut std::ffi::c_void) -> i32;
    fn NtResumeProcess(process_handle: *mut std::ffi::c_void) -> i32;
}

/// Supported game launcher ecosystems for targeted webview suspension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LauncherFamily {
    Steam,
    Epic,
    BattleNet,
    EaApp,
    Ubisoft,
    Riot,
}

/// Persisted record of a suspended process used for crash recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuspendedWebviewRecord {
    pub pid: u32,
    pub exe_name: String,
    pub start_time: u64,
    pub family: LauncherFamily,
}

/// Smart Resume evaluation outcome for each daemon tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeDecision {
    Hold,
    ResumeFamily(LauncherFamily),
    ResumeAll,
}

const DEFOCUS_GRACE_PERIOD: Duration = Duration::from_secs(10);

/// Pure classifier determining whether a process is a non-essential storefront Chromium renderer.
/// Requires the process name to match a known launcher webview helper AND command line to include
/// `--type=renderer` (excluding GPU, utility, audio, crashpad, and protected companion apps).
pub fn classify_suspendable_renderer(proc_name: &str, cmd_args: &[String]) -> Option<LauncherFamily> {
    let lower_name = proc_name.trim().to_lowercase();
    if lower_name.is_empty() {
        return None;
    }

    // Hard guard: never suspend anything in the protected companion/browser/OS whitelist.
    if PROTECTED_THROTTLING_WHITELIST
        .iter()
        .any(|&p| lower_name == p || lower_name.ends_with(p))
    {
        return None;
    }

    let family = match lower_name.as_str() {
        "steamwebhelper.exe" => LauncherFamily::Steam,
        "epicwebhelper.exe" => LauncherFamily::Epic,
        "battle.net.exe" | "blizzardbrowser.exe" => LauncherFamily::BattleNet,
        "eacefsubprocess.exe" | "eadesktop.exe" => LauncherFamily::EaApp,
        "uplaywebcore.exe" => LauncherFamily::Ubisoft,
        "riotclientuxrender.exe" => LauncherFamily::Riot,
        _ => return None,
    };

    // Inspect command-line arguments: ONLY freeze Chromium `--type=renderer` helpers.
    let mut has_renderer = false;
    for arg in cmd_args {
        let lower_arg = arg.to_lowercase();
        if lower_arg.contains("--type=gpu-process")
            || lower_arg.contains("--type=utility")
            || lower_arg.contains("--type=crashpad-handler")
            || lower_arg.contains("--type=zygote")
            || lower_arg.contains("--type=ppapi")
        {
            return None;
        }
        if lower_arg == "--type=renderer" || lower_arg.starts_with("--type=renderer ") {
            has_renderer = true;
        }
    }

    if has_renderer {
        Some(family)
    } else {
        None
    }
}

/// Maps a foreground executable name back to its LauncherFamily so Alt-Tabbing directly
/// to a launcher window immediately wakes its storefront UI.
pub fn launcher_family_for_foreground(fg_name: &str) -> Option<LauncherFamily> {
    match fg_name.trim().to_lowercase().as_str() {
        "steam.exe" | "steamwebhelper.exe" => Some(LauncherFamily::Steam),
        "epicgameslauncher.exe" | "epicwebhelper.exe" => Some(LauncherFamily::Epic),
        "battle.net.exe" | "battle.net launcher.exe" | "blizzardbrowser.exe" => {
            Some(LauncherFamily::BattleNet)
        }
        "eadesktop.exe" | "eaapp.exe" | "eacefsubprocess.exe" => Some(LauncherFamily::EaApp),
        "upc.exe" | "ubisoftconnect.exe" | "uplaywebcore.exe" => Some(LauncherFamily::Ubisoft),
        "riotclientservices.exe" | "riotclientux.exe" | "riotclientuxrender.exe" => {
            Some(LauncherFamily::Riot)
        }
        _ => None,
    }
}

/// Pure Smart Resume policy evaluation (deterministic and testable without timers/Win32).
pub fn evaluate_smart_resume(
    game_alive: bool,
    game_focused: bool,
    fg_launcher: Option<LauncherFamily>,
    defocused_elapsed: Option<Duration>,
    grace_period: Duration,
) -> ResumeDecision {
    if !game_alive {
        return ResumeDecision::ResumeAll;
    }
    if game_focused {
        return ResumeDecision::Hold;
    }
    if let Some(family) = fg_launcher {
        return ResumeDecision::ResumeFamily(family);
    }
    if let Some(elapsed) = defocused_elapsed {
        if elapsed >= grace_period {
            return ResumeDecision::ResumeAll;
        }
    }
    ResumeDecision::Hold
}

#[cfg(windows)]
fn suspend_pid_win32(pid: u32) -> bool {
    // Minimal rights: PROCESS_SUSPEND_RESUME (0x0800) | PROCESS_QUERY_LIMITED_INFORMATION (0x1000)
    const ACCESS: PROCESS_ACCESS_RIGHTS = PROCESS_ACCESS_RIGHTS(0x0800 | 0x1000);
    // SAFETY: Requesting minimal suspend permission; handle closed immediately after NtSuspendProcess.
    if let Ok(handle) = unsafe { OpenProcess(ACCESS, false, pid) } {
        let status = unsafe { NtSuspendProcess(handle.0) };
        unsafe {
            let _ = CloseHandle(handle);
        }
        status == 0
    } else {
        false
    }
}

#[cfg(not(windows))]
fn suspend_pid_win32(_pid: u32) -> bool {
    true
}

#[cfg(windows)]
fn resume_pid_win32(pid: u32) -> bool {
    const ACCESS: PROCESS_ACCESS_RIGHTS = PROCESS_ACCESS_RIGHTS(0x0800 | 0x1000);
    // SAFETY: Requesting minimal resume permission; handle closed immediately after NtResumeProcess.
    if let Ok(handle) = unsafe { OpenProcess(ACCESS, false, pid) } {
        let status = unsafe { NtResumeProcess(handle.0) };
        unsafe {
            let _ = CloseHandle(handle);
        }
        status == 0
    } else {
        false
    }
}

#[cfg(not(windows))]
fn resume_pid_win32(_pid: u32) -> bool {
    true
}

fn recovery_file_path() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("FerrumPulse").join("suspended_pids.json")
}

pub fn save_recovery_records_to(path: &PathBuf, records: &[SuspendedWebviewRecord]) {
    if records.is_empty() {
        let _ = std::fs::remove_file(path);
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(records) {
        let _ = std::fs::write(path, json);
    }
}

pub fn load_recovery_records_from(path: &PathBuf) -> Vec<SuspendedWebviewRecord> {
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Filters recovery records against live running processes matching BOTH PID and process start time
/// so OS PID reuse cannot accidentally touch an unrelated process.
pub fn filter_matching_orphans(
    records: &[SuspendedWebviewRecord],
    live_processes: &HashMap<u32, (String, u64)>,
) -> Vec<u32> {
    records
        .iter()
        .filter_map(|rec| {
            live_processes.get(&rec.pid).and_then(|(name, start)| {
                if *start == rec.start_time && name.eq_ignore_ascii_case(&rec.exe_name) {
                    Some(rec.pid)
                } else {
                    None
                }
            })
        })
        .collect()
}

/// On application startup, checks `%APPDATA%\FerrumPulse\suspended_pids.json` and resumes any
/// orphaned launcher webviews if FerrumPulse exited unexpectedly in a previous session.
pub fn recover_orphaned_suspended_processes() -> usize {
    let path = recovery_file_path();
    let records = load_recovery_records_from(&path);
    if records.is_empty() {
        return 0;
    }

    let mut resumed = 0;
    #[cfg(windows)]
    {
        let mut sys = sysinfo::System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let mut live = HashMap::new();
        for (pid, proc_info) in sys.processes() {
            live.insert(
                pid.as_u32(),
                (
                    proc_info.name().to_string_lossy().to_string(),
                    proc_info.start_time(),
                ),
            );
        }
        for pid in filter_matching_orphans(&records, &live) {
            if resume_pid_win32(pid) {
                resumed += 1;
            }
        }
    }

    let _ = std::fs::remove_file(&path);
    resumed
}

/// Manages active suspended webview PIDs with Smart Resume tracking and disk crash recovery.
pub struct LauncherSuspender {
    suspended: HashMap<u32, SuspendedWebviewRecord>,
    defocus_started_at: Option<Instant>,
    failed_pids: HashSet<u32>,
    recovery_path: PathBuf,
}

impl LauncherSuspender {
    pub fn new() -> Self {
        Self {
            suspended: HashMap::new(),
            defocus_started_at: None,
            failed_pids: HashSet::new(),
            recovery_path: recovery_file_path(),
        }
    }

    pub fn suspended_count(&self) -> usize {
        self.suspended.len()
    }

    /// Scans `sys` (caller-reused `sysinfo::System`) and suspends newly discovered `--type=renderer`
    /// launcher helper processes.
    pub fn suspend_eligible(&mut self, sys: &sysinfo::System, exclude_pid: Option<u32>) -> usize {
        let mut newly_suspended = 0;

        // Prune any PIDs that exited externally while tracked.
        let before_prune = self.suspended.len();
        self.suspended
            .retain(|pid, _| sys.process(sysinfo::Pid::from_u32(*pid)).is_some());
        let mut changed = self.suspended.len() != before_prune;

        for (pid, process) in sys.processes() {
            let raw_pid = pid.as_u32();
            if exclude_pid == Some(raw_pid)
                || self.suspended.contains_key(&raw_pid)
                || self.failed_pids.contains(&raw_pid)
            {
                continue;
            }

            let proc_name = process.name().to_string_lossy().to_string();
            let cmd_args: Vec<String> = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();

            if let Some(family) = classify_suspendable_renderer(&proc_name, &cmd_args) {
                if suspend_pid_win32(raw_pid) {
                    self.suspended.insert(
                        raw_pid,
                        SuspendedWebviewRecord {
                            pid: raw_pid,
                            exe_name: proc_name,
                            start_time: process.start_time(),
                            family,
                        },
                    );
                    newly_suspended += 1;
                    changed = true;
                } else {
                    self.failed_pids.insert(raw_pid);
                }
            }
        }

        if changed {
            self.persist_state();
        }
        newly_suspended
    }

    /// Evaluates Smart Resume rules on every daemon tick and resumes processes as needed.
    pub fn update_lifecycle(
        &mut self,
        sys: &sysinfo::System,
        game_alive: bool,
        game_focused: bool,
        fg_name: Option<&str>,
        allow_suspension: bool,
        active_game_pid: Option<u32>,
    ) -> usize {
        if !allow_suspension || !game_alive {
            self.defocus_started_at = None;
            return self.resume_all();
        }

        if game_focused {
            self.defocus_started_at = None;
            self.suspend_eligible(sys, active_game_pid);
            return self.suspended.len();
        }

        // Game is alive in the background (Alt-Tabbed).
        if self.defocus_started_at.is_none() {
            self.defocus_started_at = Some(Instant::now());
        }

        let fg_launcher = fg_name.and_then(launcher_family_for_foreground);
        let elapsed = self.defocus_started_at.map(|t| t.elapsed());

        match evaluate_smart_resume(
            game_alive,
            game_focused,
            fg_launcher,
            elapsed,
            DEFOCUS_GRACE_PERIOD,
        ) {
            ResumeDecision::Hold => {}
            ResumeDecision::ResumeFamily(family) => {
                self.resume_family(family);
            }
            ResumeDecision::ResumeAll => {
                self.resume_all();
            }
        }

        self.suspended.len()
    }

    pub fn resume_family(&mut self, family: LauncherFamily) -> usize {
        let target_pids: Vec<u32> = self
            .suspended
            .iter()
            .filter(|(_, rec)| rec.family == family)
            .map(|(pid, _)| *pid)
            .collect();

        if target_pids.is_empty() {
            return 0;
        }

        let mut resumed = 0;
        for pid in target_pids {
            if resume_pid_win32(pid) {
                resumed += 1;
            }
            self.suspended.remove(&pid);
        }
        self.persist_state();
        resumed
    }

    pub fn resume_all(&mut self) -> usize {
        if self.suspended.is_empty() {
            return 0;
        }
        let pids: Vec<u32> = self.suspended.keys().copied().collect();
        let mut resumed = 0;
        for pid in pids {
            if resume_pid_win32(pid) {
                resumed += 1;
            }
        }
        self.suspended.clear();
        self.persist_state();
        resumed
    }

    fn persist_state(&self) {
        let records: Vec<SuspendedWebviewRecord> = self.suspended.values().cloned().collect();
        save_recovery_records_to(&self.recovery_path, &records);
    }
}

impl Default for LauncherSuspender {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LauncherSuspender {
    fn drop(&mut self) {
        self.resume_all();
    }
}
