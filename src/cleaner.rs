use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::UI::Shell::{
    SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageScanResult {
    pub user_temp_mb: f64,
    pub windows_temp_mb: f64,
    pub directx_cache_mb: f64,
    pub total_cleanable_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCleanResult {
    pub freed_mb: f64,
    pub files_deleted: u32,
    pub files_skipped: u32,
    pub recycle_bin_emptied: bool,
}

/// Scans temp folders and DirectX shader caches to calculate cleanable disk space.
pub fn scan_storage() -> StorageScanResult {
    let user_temp = get_user_temp_dir();
    let win_temp = get_windows_temp_dir();
    let d3d_cache = get_directx_cache_dir();

    let user_bytes = user_temp.as_ref().map(|p| calculate_dir_size(p)).unwrap_or(0);
    let win_bytes = win_temp.as_ref().map(|p| calculate_dir_size(p)).unwrap_or(0);
    let d3d_bytes = d3d_cache.as_ref().map(|p| calculate_dir_size(p)).unwrap_or(0);
    let total_bytes = user_bytes + win_bytes + d3d_bytes;

    StorageScanResult {
        user_temp_mb: to_mb(user_bytes),
        windows_temp_mb: to_mb(win_bytes),
        directx_cache_mb: to_mb(d3d_bytes),
        total_cleanable_mb: to_mb(total_bytes),
    }
}

/// Performs safe, non-destructive cleanup across selected targets.
pub fn clean_storage(
    clean_user_temp: bool,
    clean_win_temp: bool,
    clean_d3d_cache: bool,
    empty_recycle_bin: bool,
) -> StorageCleanResult {
    let mut total_freed_bytes: u64 = 0;
    let mut total_deleted: u32 = 0;
    let mut total_skipped: u32 = 0;

    if clean_user_temp {
        if let Some(path) = get_user_temp_dir() {
            let (freed, del, skip) = clean_directory_contents(&path);
            total_freed_bytes += freed;
            total_deleted += del;
            total_skipped += skip;
        }
    }

    if clean_win_temp {
        if let Some(path) = get_windows_temp_dir() {
            let (freed, del, skip) = clean_directory_contents(&path);
            total_freed_bytes += freed;
            total_deleted += del;
            total_skipped += skip;
        }
    }

    if clean_d3d_cache {
        if let Some(path) = get_directx_cache_dir() {
            let (freed, del, skip) = clean_directory_contents(&path);
            total_freed_bytes += freed;
            total_deleted += del;
            total_skipped += skip;
        }

        // Also clean NVIDIA / AMD DXCache if present
        if let Some(local_app) = std::env::var_os("LOCALAPPDATA") {
            let nv_cache = PathBuf::from(&local_app).join("NVIDIA").join("DXCache");
            if nv_cache.exists() {
                let (freed, del, skip) = clean_directory_contents(&nv_cache);
                total_freed_bytes += freed;
                total_deleted += del;
                total_skipped += skip;
            }
            let amd_cache = PathBuf::from(&local_app).join("AMD").join("DxCache");
            if amd_cache.exists() {
                let (freed, del, skip) = clean_directory_contents(&amd_cache);
                total_freed_bytes += freed;
                total_deleted += del;
                total_skipped += skip;
            }
        }
    }

    let bin_cleaned = if empty_recycle_bin {
        empty_windows_recycle_bin().is_ok()
    } else {
        false
    };

    StorageCleanResult {
        freed_mb: to_mb(total_freed_bytes),
        files_deleted: total_deleted,
        files_skipped: total_skipped,
        recycle_bin_emptied: bin_cleaned,
    }
}

/// Empties Windows Recycle Bin silently without dialog prompts.
pub fn empty_windows_recycle_bin() -> Result<(), String> {
    #[cfg(windows)]
    {
        // SAFETY: SHEmptyRecycleBinW flushes recycle bin with flags suppressing UI popups
        let res = unsafe {
            SHEmptyRecycleBinW(
                None,
                PCWSTR::null(),
                SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
            )
        };

        if res.is_ok() {
            Ok(())
        } else {
            Err(format!("Recycle bin clean returned: {:?}", res))
        }
    }

    #[cfg(not(windows))]
    {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_user_temp_dir() -> Option<PathBuf> {
    std::env::var_os("TEMP").map(PathBuf::from)
}

fn get_windows_temp_dir() -> Option<PathBuf> {
    let p = PathBuf::from("C:\\Windows\\Temp");
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

fn get_directx_cache_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|local| PathBuf::from(local).join("D3DSCache"))
}

fn to_mb(bytes: u64) -> f64 {
    ((bytes as f64) / (1024.0 * 1024.0) * 10.0).round() / 10.0
}

fn calculate_dir_size(path: &Path) -> u64 {
    let mut total_size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Ok(meta) = entry.metadata() {
                    total_size += meta.len();
                }
            } else if p.is_dir() {
                total_size += calculate_dir_size(&p);
            }
        }
    }
    total_size
}

fn clean_directory_contents(dir: &Path) -> (u64, u32, u32) {
    let mut freed_bytes = 0;
    let mut deleted_count = 0;
    let mut skipped_count = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                // Gracefully ignore files locked by current running processes/games
                match fs::remove_file(&path) {
                    Ok(_) => {
                        freed_bytes += size;
                        deleted_count += 1;
                    }
                    Err(_) => {
                        skipped_count += 1;
                    }
                }
            } else if path.is_dir() {
                let (sub_freed, sub_del, sub_skip) = clean_directory_contents(&path);
                freed_bytes += sub_freed;
                deleted_count += sub_del;
                skipped_count += sub_skip;
                let _ = fs::remove_dir(&path);
            }
        }
    }

    (freed_bytes, deleted_count, skipped_count)
}

