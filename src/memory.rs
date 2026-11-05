use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_ram_mb: u64,
    pub available_ram_mb: u64,
    pub used_ram_mb: u64,
    pub memory_load_percent: u32,
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
        })
    }

    #[cfg(not(windows))]
    {
        Ok(MemoryStats {
            total_ram_mb: 32768,
            available_ram_mb: 20480,
            used_ram_mb: 12288,
            memory_load_percent: 38,
        })
    }
}

/// Iterates running processes and purges inactive working sets via K32EmptyWorkingSet.
pub fn purge_working_sets() -> Result<MemoryTrimResult, String> {
    let before_stats = get_memory_stats()?;
    let mut trimmed_count: u32 = 0;

    #[cfg(windows)]
    {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        for pid in sys.processes().keys() {
            let raw_pid = pid.as_u32();

            // Skip Idle, System, and kernel PIDs to maintain OS stability
            if raw_pid <= 4 {
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
                // from the process working set and commits them to the paging system.
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
