use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows::Win32::System::ProcessStatus::K32EmptyWorkingSet;
#[cfg(windows)]
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
#[cfg(windows)]
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA,
};

pub const RAM_HIGH_PRESSURE_THRESHOLD: u32 = 88;
pub const RAM_AUTO_TRIM_COOLDOWN_SECS: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGuardConfig {
    pub auto_trim_at_high_pressure: bool,
    pub threshold_percent: u32,
}

impl Default for MemoryGuardConfig {
    fn default() -> Self {
        Self {
            auto_trim_at_high_pressure: false,
            threshold_percent: RAM_HIGH_PRESSURE_THRESHOLD,
        }
    }
}

pub type SharedMemoryGuardConfig = Arc<RwLock<MemoryGuardConfig>>;

fn memory_guard_path() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("FerrumPulse").join("memory_guard.json")
}

pub fn load_memory_guard_config_from(path: &PathBuf) -> MemoryGuardConfig {
    if !path.exists() {
        return MemoryGuardConfig::default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_memory_guard_config_to(
    path: &PathBuf,
    cfg: &MemoryGuardConfig,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialize memory guard config: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write memory guard config: {}", e))
}

pub fn load_memory_guard_config() -> MemoryGuardConfig {
    load_memory_guard_config_from(&memory_guard_path())
}

pub fn save_memory_guard_config(cfg: &MemoryGuardConfig) -> Result<(), String> {
    save_memory_guard_config_to(&memory_guard_path(), cfg)
}

/// Pure policy check deciding whether background RAM auto-trim should trigger.
pub fn should_auto_trim_ram(
    memory_load_percent: u32,
    cfg: &MemoryGuardConfig,
    secs_since_last_trim: Option<u64>,
) -> bool {
    if !cfg.auto_trim_at_high_pressure {
        return false;
    }
    let threshold = cfg.threshold_percent.clamp(70, 98);
    if memory_load_percent < threshold {
        return false;
    }
    match secs_since_last_trim {
        Some(elapsed) => elapsed >= RAM_AUTO_TRIM_COOLDOWN_SECS,
        None => true,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_ram_mb: u64,
    pub available_ram_mb: u64,
    pub used_ram_mb: u64,
    pub memory_load_percent: u32,
    pub high_pressure_warning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrimResult {
    pub freed_mb: u64,
    pub processes_trimmed: u32,
    pub before_used_mb: u64,
    pub after_used_mb: u64,
    pub available_mb: u64,
}

/// Retrieves current physical memory metrics using GlobalMemoryStatusEx.
pub fn get_memory_stats() -> Result<MemoryStats, String> {
    #[cfg(windows)]
    {
        let mut status = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };

        // SAFETY: GlobalMemoryStatusEx writes to pre-sized MEMORYSTATUSEX structure
        let res = unsafe { GlobalMemoryStatusEx(&mut status) };
        if res.is_err() {
            return Err("Failed to query system memory status.".into());
        }

        let total_mb = status.ullTotalPhys / (1024 * 1024);
        let avail_mb = status.ullAvailPhys / (1024 * 1024);
        let used_mb = total_mb.saturating_sub(avail_mb);

        Ok(MemoryStats {
            total_ram_mb: total_mb,
            available_ram_mb: avail_mb,
            used_ram_mb: used_mb,
            memory_load_percent: status.dwMemoryLoad,
            high_pressure_warning: status.dwMemoryLoad >= RAM_HIGH_PRESSURE_THRESHOLD,
        })
    }

    #[cfg(not(windows))]
    {
        Ok(MemoryStats {
            total_ram_mb: 32768,
            available_ram_mb: 20480,
            used_ram_mb: 12288,
            memory_load_percent: 38,
            high_pressure_warning: false,
        })
    }
}

/// Windows system processes and web browsers that must NEVER have their working sets flushed
/// to prevent OS instability and browser tab freezing / hard page faults.
pub const PROTECTED_SYSTEM_PROCESSES: &[&str] = &[
    // Core Windows & Audio
    "csrss.exe",
    "lsass.exe",
    "smss.exe",
    "dwm.exe",
    "audiodg.exe",
    "services.exe",
    "explorer.exe",
    "systemsettings.exe",
    // Web Browsers (Flushing causes 2-5s tab freezes and massive pagefile churn)
    "chrome.exe",
    "msedge.exe",
    "brave.exe",
    "firefox.exe",
    "opera.exe",
    "vivaldi.exe",
    "arc.exe",
];

/// Iterates running processes and purges inactive working sets via K32EmptyWorkingSet.
/// Strictly enforces hard page fault prevention: the active foreground game and critical OS processes are never flushed.
pub fn purge_working_sets() -> Result<MemoryTrimResult, String> {
    let before_stats = get_memory_stats()?;
    let mut trimmed_count: u32 = 0;

    #[cfg(windows)]
    {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        // HARD GUARD: Query foreground process PID to prevent flushing the active game window
        let foreground_pid = crate::process::get_foreground_process().map(|(pid, _)| pid);

        for (pid, process) in sys.processes() {
            let raw_pid = pid.as_u32();

            // Skip Idle, System, and kernel PIDs to maintain OS stability
            if raw_pid <= 4 {
                continue;
            }

            // HARD GUARD: NEVER flush the active foreground game process!
            // Doing so purges active textures/geometry into pagefile/standby, causing hard page faults and massive frame stutter.
            if Some(raw_pid) == foreground_pid {
                continue;
            }

            // HARD GUARD: Skip protected Windows system and audio subsystem processes
            let proc_name = process.name().to_string_lossy().to_lowercase();
            if PROTECTED_SYSTEM_PROCESSES.iter().any(|&p| proc_name == p || proc_name.ends_with(p)) {
                continue;
            }

            // SAFETY: PROCESS_SET_QUOTA | PROCESS_QUERY_LIMITED_INFORMATION is sufficient
            // to flush the working set without attaching debuggers or violating security boundaries.
            let handle_res = unsafe {
                OpenProcess(
                    PROCESS_SET_QUOTA | PROCESS_QUERY_LIMITED_INFORMATION,
                    false,
                    raw_pid,
                )
            };

            if let Ok(handle) = handle_res {
                // SAFETY: K32EmptyWorkingSet cleanly removes inactive physical pages
                // from the background process working set and commits them to the paging system.
                let purge_res = unsafe { K32EmptyWorkingSet(handle) };
                if purge_res.as_bool() {
                    trimmed_count += 1;
                }

                unsafe {
                    let _ = CloseHandle(handle);
                };
            }
        }
    }

    // Refresh memory status to calculate freed RAM
    let after_stats = get_memory_stats()?;
    let freed_mb = before_stats.used_ram_mb.saturating_sub(after_stats.used_ram_mb);

    Ok(MemoryTrimResult {
        freed_mb,
        processes_trimmed: trimmed_count,
        before_used_mb: before_stats.used_ram_mb,
        after_used_mb: after_stats.used_ram_mb,
        available_mb: after_stats.available_ram_mb,
    })
}
