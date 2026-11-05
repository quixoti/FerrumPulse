use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::cleaner::{clean_storage as do_clean_storage, empty_windows_recycle_bin, scan_storage as do_scan_storage, StorageCleanResult, StorageScanResult};
use crate::memory::{get_memory_stats as do_get_memory_stats, purge_working_sets as do_purge_working_sets, MemoryStats, MemoryTrimResult};
use crate::power::{get_power_status as do_get_power_status, set_core_unparking as do_set_core_unparking, set_power_scheme as do_set_power_scheme, PowerStatus};
use crate::process::{DaemonConfig, DaemonStatus, ProcessPacingManager};
use crate::tweaks::{
    get_tweaks_status as do_get_tweaks_status, toggle_autostart as do_toggle_autostart,
    toggle_diagtrack as do_toggle_diagtrack, toggle_gamedvr as do_toggle_gamedvr,
    toggle_global_timer_resolution as do_toggle_global_timer_resolution,
    toggle_network_pacing as do_toggle_network_pacing,
    toggle_raw_mouse_input as do_toggle_raw_mouse_input, TweakActionResult, TweaksStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneClickBoostResult {
    pub unparked: bool,
    pub power_plan: String,
    pub network_optimized: bool,
    pub memory_freed_mb: u64,
}

#[tauri::command]
pub fn cmd_get_power_status() -> Result<PowerStatus, String> {
    do_get_power_status()
}

#[tauri::command]
pub fn cmd_set_core_unparking(unpark: bool) -> Result<u32, String> {
    do_set_core_unparking(unpark)
}

#[tauri::command]
pub fn cmd_set_power_scheme(scheme_type: String) -> Result<String, String> {
    do_set_power_scheme(&scheme_type)
}

#[tauri::command]
pub fn cmd_get_daemon_status(state: State<'_, Arc<ProcessPacingManager>>) -> DaemonStatus {
    state.get_status()
}

#[tauri::command]
pub fn cmd_get_daemon_config(state: State<'_, Arc<ProcessPacingManager>>) -> DaemonConfig {
    state.get_config()
}

#[tauri::command]
pub fn cmd_update_daemon_config(config: DaemonConfig, state: State<'_, Arc<ProcessPacingManager>>) -> Result<(), String> {
    state.update_config(config);
    Ok(())
}

#[tauri::command]
pub fn cmd_get_memory_stats() -> Result<MemoryStats, String> {
    do_get_memory_stats()
}

#[tauri::command]
pub fn cmd_purge_working_sets() -> Result<MemoryTrimResult, String> {
    do_purge_working_sets()
}

#[tauri::command]
pub fn cmd_get_tweaks_status() -> TweaksStatus {
    do_get_tweaks_status()
}

#[tauri::command]
pub fn cmd_toggle_diagtrack(disable: bool) -> TweakActionResult {
    do_toggle_diagtrack(disable)
}

#[tauri::command]
pub fn cmd_toggle_gamedvr(disable: bool) -> TweakActionResult {
    do_toggle_gamedvr(disable)
}

#[tauri::command]
pub fn cmd_toggle_network_pacing(optimize: bool) -> TweakActionResult {
    do_toggle_network_pacing(optimize)
}

#[tauri::command]
pub fn cmd_toggle_global_timer(enable: bool) -> TweakActionResult {
    do_toggle_global_timer_resolution(enable)
}

#[tauri::command]
pub fn cmd_toggle_raw_mouse(enable: bool) -> TweakActionResult {
    do_toggle_raw_mouse_input(enable)
}

#[tauri::command]
pub fn cmd_toggle_autostart(enable: bool) -> TweakActionResult {
    do_toggle_autostart(enable)
}

#[tauri::command]
pub fn cmd_scan_storage() -> StorageScanResult {
    do_scan_storage()
}

#[tauri::command]
pub fn cmd_clean_storage(
    clean_user_temp: bool,
    clean_win_temp: bool,
    clean_d3d_cache: bool,
    empty_recycle_bin: bool,
) -> StorageCleanResult {
    do_clean_storage(clean_user_temp, clean_win_temp, clean_d3d_cache, empty_recycle_bin)
}

#[tauri::command]
pub fn cmd_empty_recycle_bin() -> Result<(), String> {
    empty_windows_recycle_bin()
}

#[tauri::command]
pub fn cmd_one_click_boost(state: State<'_, Arc<ProcessPacingManager>>) -> Result<OneClickBoostResult, String> {
    // 1. Unpark CPU Cores to 100%
    let _ = do_set_core_unparking(true);

    // 2. Switch to High Performance plan
    let plan_res = do_set_power_scheme("high").unwrap_or_else(|_| "Balanced".into());

    // 3. Optimize Multimedia & Network Pacing
    let net_res = do_toggle_network_pacing(true);

    // 4. Ensure process pacing daemon is active
    let mut cfg = state.get_config();
    cfg.enabled = true;
    cfg.boost_priority = true;
    cfg.throttle_background = true;
    state.update_config(cfg);

    // 5. Purge inactive memory working sets
    let mem_res = do_purge_working_sets().map(|r| r.freed_mb).unwrap_or(0);

    Ok(OneClickBoostResult {
        unparked: true,
        power_plan: plan_res,
        network_optimized: net_res.success,
        memory_freed_mb: mem_res,
    })
}

#[tauri::command]
pub fn cmd_get_battery_status() -> crate::battery::BatteryTelemetry {
    crate::battery::get_battery_telemetry()
}

#[tauri::command]
pub fn cmd_set_battery_eco(enable: bool) -> Result<(), String> {
    crate::battery::apply_battery_eco_profile(enable)
}

#[tauri::command]
pub fn cmd_set_display_refresh_rate(hz: u32) -> Result<(), String> {
    crate::battery::set_display_refresh_rate(hz)
}

#[tauri::command]
pub fn cmd_minimize_window(window: tauri::WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn cmd_toggle_maximize_window(window: tauri::WebviewWindow) {
    if let Ok(is_max) = window.is_maximized() {
        if is_max {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
pub fn cmd_hide_window(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

#[tauri::command]
pub fn cmd_drag_window(window: tauri::WebviewWindow) {
    let _ = window.start_dragging();
}

