use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::cleaner::{clean_storage as do_clean_storage, empty_windows_recycle_bin, scan_storage as do_scan_storage, StorageCleanResult, StorageScanResult};
use crate::memory::{get_memory_stats as do_get_memory_stats, purge_working_sets as do_purge_working_sets, MemoryStats, MemoryTrimResult};
use crate::power::{get_power_status as do_get_power_status, set_core_unparking as do_set_core_unparking, set_power_scheme as do_set_power_scheme, PowerStatus};
use crate::process::{DaemonConfig, DaemonStatus, ProcessPacingManager};
use crate::tweaks::{
    apply_all_scheduler_tweaks as do_apply_all_scheduler_tweaks,
    get_tweaks_status as do_get_tweaks_status, toggle_autostart as do_toggle_autostart,
    toggle_diagtrack as do_toggle_diagtrack, toggle_gamedvr as do_toggle_gamedvr,
    toggle_global_timer_resolution as do_toggle_global_timer_resolution,
    toggle_mmcss_gaming_priority as do_toggle_mmcss_gaming_priority,
    toggle_network_pacing as do_toggle_network_pacing,
    toggle_raw_mouse_input as do_toggle_raw_mouse_input,
    toggle_system_responsiveness as do_toggle_system_responsiveness,
    toggle_win32_priority_separation as do_toggle_win32_priority_separation,
    TweakActionResult, TweaksStatus,
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
pub fn cmd_toggle_mmcss_gaming_priority(enable: bool) -> TweakActionResult {
    do_toggle_mmcss_gaming_priority(enable)
}

#[tauri::command]
pub fn cmd_toggle_win32_priority_separation(optimize: bool) -> TweakActionResult {
    do_toggle_win32_priority_separation(optimize)
}

#[tauri::command]
pub fn cmd_toggle_system_responsiveness(optimize: bool) -> TweakActionResult {
    do_toggle_system_responsiveness(optimize)
}

#[tauri::command]
pub fn cmd_apply_all_scheduler_tweaks() -> Vec<TweakActionResult> {
    do_apply_all_scheduler_tweaks()
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
pub fn cmd_get_battery_status(
    state: State<'_, Arc<ProcessPacingManager>>,
) -> crate::battery::BatteryTelemetry {
    let cfg = state.get_battery_config();
    crate::battery::get_battery_telemetry_with_config(&cfg)
}

#[tauri::command]
pub fn cmd_set_battery_eco(
    enable: bool,
    state: State<'_, Arc<ProcessPacingManager>>,
) -> Result<(), String> {
    let mut cfg = state.get_battery_config();
    if enable {
        if let Some(cur_hz) = crate::battery::get_current_display_refresh_rate() {
            if cur_hz > 60 {
                cfg.saved_ac_refresh_rate = Some(cur_hz);
                let _ = crate::battery::save_battery_config(&cfg);
                state.update_battery_config(cfg.clone());
            }
        }
    }
    crate::battery::apply_battery_eco_profile_with_config(enable, &cfg)
}

#[tauri::command]
pub fn cmd_get_battery_config(
    state: State<'_, Arc<ProcessPacingManager>>,
) -> crate::battery::BatteryConfig {
    state.get_battery_config()
}

#[tauri::command]
pub fn cmd_update_battery_config(
    config: crate::battery::BatteryConfig,
    state: State<'_, Arc<ProcessPacingManager>>,
) -> Result<(), String> {
    crate::battery::save_battery_config(&config)?;
    state.update_battery_config(config);
    Ok(())
}

#[tauri::command]
pub fn cmd_get_audio_tuning_status(
    state: State<'_, Arc<ProcessPacingManager>>,
) -> crate::audio::AudioTuningStatus {
    let cfg = state.get_audio_config();
    crate::audio::get_audio_tuning_status(&cfg)
}

#[tauri::command]
pub fn cmd_toggle_audio_engine_tuning(
    enable: bool,
    state: State<'_, Arc<ProcessPacingManager>>,
) -> crate::tweaks::TweakActionResult {
    let res = crate::audio::toggle_audio_engine_tuning(enable);
    state.update_audio_config(crate::audio::AudioTuningConfig { enabled: enable });
    res
}

#[tauri::command]
pub fn cmd_get_memory_guard_config(
    state: State<'_, Arc<ProcessPacingManager>>,
) -> crate::memory::MemoryGuardConfig {
    state.get_memory_guard_config()
}

#[tauri::command]
pub fn cmd_update_memory_guard_config(
    config: crate::memory::MemoryGuardConfig,
    state: State<'_, Arc<ProcessPacingManager>>,
) -> Result<(), String> {
    crate::memory::save_memory_guard_config(&config)?;
    state.update_memory_guard_config(config);
    Ok(())
}

#[tauri::command]
pub fn cmd_set_display_refresh_rate(hz: u32) -> Result<(), String> {
    crate::battery::set_display_refresh_rate(hz)
}

#[tauri::command]
pub fn cmd_scan_game_libraries() -> Vec<crate::scanner::DiscoveredGame> {
    let scanner = crate::scanner::CompositeLibraryScanner::default_suite();
    scanner.scan_all()
}

#[tauri::command]
pub fn cmd_list_profiles(
    state: State<'_, Arc<ProcessPacingManager>>,
) -> Result<crate::profiles::ProfilesStore, String> {
    let shared = state.get_profiles_store();
    let guard = shared
        .read()
        .map_err(|_| "Failed to lock shared profiles store".to_string())?;
    Ok(guard.clone())
}

#[tauri::command]
pub fn cmd_save_profile(
    mut profile: crate::profiles::GameProfile,
    state: State<'_, Arc<ProcessPacingManager>>,
) -> Result<(), String> {
    use crate::profiles::{sanitize_executables, JsonFileProfileRepository, ProfileRepository};
    profile.executables = sanitize_executables(profile.executables);
    if profile.executables.is_empty() {
        return Err("At least one valid executable name (e.g. game.exe) is required.".into());
    }
    let repo = JsonFileProfileRepository::new();
    let mut store = repo.load().unwrap_or_default();
    store.insert(profile);
    repo.save(&store)?;
    state.refresh_profiles_store(store.clone());
    let profiles: Vec<crate::profiles::GameProfile> = store.profiles.into_values().collect();
    let _ = crate::network::sync_profiles_qos_policies(&profiles);
    Ok(())
}

#[tauri::command]
pub fn cmd_delete_profile(
    profile_id: String,
    state: State<'_, Arc<ProcessPacingManager>>,
) -> Result<(), String> {
    use crate::profiles::{JsonFileProfileRepository, ProfileRepository};
    let repo = JsonFileProfileRepository::new();
    let mut store = repo.load().unwrap_or_default();
    if store.remove(&profile_id) {
        repo.save(&store)?;
        state.refresh_profiles_store(store.clone());
        let profiles: Vec<crate::profiles::GameProfile> = store.profiles.into_values().collect();
        let _ = crate::network::sync_profiles_qos_policies(&profiles);
        Ok(())
    } else {
        Err("Profile not found".into())
    }
}

#[tauri::command]
pub fn cmd_get_supported_refresh_rates() -> Vec<u32> {
    crate::battery::get_supported_refresh_rates()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickedGameExe {
    pub display_name: String,
    pub executable_name: String,
    pub executable_path: String,
}

#[cfg(windows)]
#[repr(C)]
struct OpenFileNameW {
    l_struct_size: u32,
    hwnd_owner: *mut std::ffi::c_void,
    h_instance: *mut std::ffi::c_void,
    lpstr_filter: *const u16,
    lpstr_custom_filter: *mut u16,
    n_max_cust_filter: u32,
    n_filter_index: u32,
    lpstr_file: *mut u16,
    n_max_file: u32,
    lpstr_file_title: *mut u16,
    n_max_file_title: u32,
    lpstr_initial_dir: *const u16,
    lpstr_title: *const u16,
    flags: u32,
    n_file_offset: u16,
    n_file_extension: u16,
    lpstr_def_ext: *const u16,
    l_cust_data: isize,
    lpfn_hook: *const std::ffi::c_void,
    lp_template_name: *const u16,
    pv_reserved: *mut std::ffi::c_void,
    dw_reserved: u32,
    flags_ex: u32,
}

#[cfg(windows)]
#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(lpofn: *mut OpenFileNameW) -> i32;
    fn GetSaveFileNameW(lpofn: *mut OpenFileNameW) -> i32;
}

#[cfg(windows)]
fn win32_file_dialog(save: bool, filter: &str, title: &str, default_name: &str) -> Option<String> {
    let mut file_buf = vec![0u16; 1024];
    for (i, ch) in default_name.encode_utf16().take(1020).enumerate() {
        file_buf[i] = ch;
    }

    let filter_u16: Vec<u16> = filter.encode_utf16().chain(std::iter::once(0)).collect();
    let title_u16: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let def_ext: Vec<u16> = if save {
        "json\0".encode_utf16().collect()
    } else {
        "exe\0".encode_utf16().collect()
    };

    // OFN_PATHMUSTEXIST (0x0800) | OFN_NOCHANGEDIR (0x0008) | (OFN_FILEMUSTEXIST 0x1000 / OFN_OVERWRITEPROMPT 0x0002)
    let flags = if save {
        0x0800 | 0x0008 | 0x0002
    } else {
        0x0800 | 0x0008 | 0x1000
    };

    let mut ofn = OpenFileNameW {
        l_struct_size: std::mem::size_of::<OpenFileNameW>() as u32,
        hwnd_owner: std::ptr::null_mut(),
        h_instance: std::ptr::null_mut(),
        lpstr_filter: filter_u16.as_ptr(),
        lpstr_custom_filter: std::ptr::null_mut(),
        n_max_cust_filter: 0,
        n_filter_index: 1,
        lpstr_file: file_buf.as_mut_ptr(),
        n_max_file: file_buf.len() as u32,
        lpstr_file_title: std::ptr::null_mut(),
        n_max_file_title: 0,
        lpstr_initial_dir: std::ptr::null(),
        lpstr_title: title_u16.as_ptr(),
        flags,
        n_file_offset: 0,
        n_file_extension: 0,
        lpstr_def_ext: def_ext.as_ptr(),
        l_cust_data: 0,
        lpfn_hook: std::ptr::null(),
        lp_template_name: std::ptr::null(),
        pv_reserved: std::ptr::null_mut(),
        dw_reserved: 0,
        flags_ex: 0,
    };

    let ok = unsafe {
        if save {
            GetSaveFileNameW(&mut ofn)
        } else {
            GetOpenFileNameW(&mut ofn)
        }
    };

    if ok != 0 {
        let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
        Some(String::from_utf16_lossy(&file_buf[..len]))
    } else {
        None
    }
}

#[tauri::command]
pub async fn cmd_pick_game_executable() -> Result<Option<PickedGameExe>, String> {
    #[cfg(windows)]
    {
        let picked = tokio::task::spawn_blocking(|| {
            win32_file_dialog(
                false,
                "Game Executables (*.exe)\0*.exe\0All Files (*.*)\0*.*\0",
                "Select Game Executable",
                "",
            )
        })
        .await
        .map_err(|e| format!("File dialog failed: {}", e))?;

        Ok(picked.map(|full_path| {
            let p = std::path::Path::new(&full_path);
            let exe_name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("game.exe")
                .to_string();
            let stem = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Custom Game")
                .replace(['_', '-'], " ");
            PickedGameExe {
                display_name: stem,
                executable_name: exe_name,
                executable_path: full_path,
            }
        }))
    }
    #[cfg(not(windows))]
    Ok(None)
}

#[tauri::command]
pub async fn cmd_export_profiles_to_file(
    state: State<'_, Arc<ProcessPacingManager>>,
) -> Result<Option<String>, String> {
    let json = {
        let shared = state.get_profiles_store();
        let guard = shared
            .read()
            .map_err(|_| "Failed to lock shared profiles store".to_string())?;
        crate::profiles::export_profiles_json(&guard)?
    };

    #[cfg(windows)]
    {
        let dest = tokio::task::spawn_blocking(|| {
            win32_file_dialog(
                true,
                "FerrumPulse Profiles (*.json)\0*.json\0All Files (*.*)\0*.*\0",
                "Export FerrumPulse Game Profiles",
                "ferrumpulse-profiles.json",
            )
        })
        .await
        .map_err(|e| format!("Export dialog failed: {}", e))?;

        if let Some(path) = dest {
            std::fs::write(&path, json).map_err(|e| format!("Failed to write export file: {}", e))?;
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }
    #[cfg(not(windows))]
    {
        let _ = json;
        Ok(None)
    }
}

#[tauri::command]
pub async fn cmd_import_profiles_from_file(
    state: State<'_, Arc<ProcessPacingManager>>,
) -> Result<Option<usize>, String> {
    #[cfg(windows)]
    {
        let src = tokio::task::spawn_blocking(|| {
            win32_file_dialog(
                false,
                "FerrumPulse Profiles (*.json)\0*.json\0All Files (*.*)\0*.*\0",
                "Import FerrumPulse Game Profiles",
                "",
            )
        })
        .await
        .map_err(|e| format!("Import dialog failed: {}", e))?;

        if let Some(path) = src {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read import file: {}", e))?;
            use crate::profiles::{import_profiles_from_json, JsonFileProfileRepository, ProfileRepository};
            let repo = JsonFileProfileRepository::new();
            let mut store = repo.load().unwrap_or_default();
            let imported = import_profiles_from_json(&content, &mut store)?;
            repo.save(&store)?;
            state.refresh_profiles_store(store.clone());
            let profiles: Vec<crate::profiles::GameProfile> = store.profiles.into_values().collect();
            let _ = crate::network::sync_profiles_qos_policies(&profiles);
            Ok(Some(imported))
        } else {
            Ok(None)
        }
    }
    #[cfg(not(windows))]
    {
        let _ = state;
        Ok(None)
    }
}

#[tauri::command]
pub fn cmd_quit_app(
    app: tauri::AppHandle,
    state: State<'_, Arc<ProcessPacingManager>>,
) {
    state.stop_and_cleanup();
    app.exit(0);
}

#[tauri::command]
pub fn cmd_audit_win11_gaming_features() -> Result<crate::auditor::Win11AuditorReport, String> {
    Ok(crate::auditor::audit_win11_gaming_features())
}

#[tauri::command]
pub fn cmd_toggle_hags(enable: bool) -> crate::tweaks::TweakActionResult {
    crate::auditor::toggle_hags(enable)
}

#[tauri::command]
pub fn cmd_toggle_windowed_optimizations(enable: bool) -> crate::tweaks::TweakActionResult {
    crate::auditor::toggle_windowed_optimizations(enable)
}

#[tauri::command]
pub fn cmd_toggle_game_mode(enable: bool) -> crate::tweaks::TweakActionResult {
    crate::auditor::toggle_game_mode(enable)
}

#[tauri::command]
pub fn cmd_open_windows_settings(uri: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        let _ = Command::new("cmd")
            .args(&["/c", "start", &uri])
            .spawn()
            .map_err(|e| format!("Failed to launch settings: {}", e))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = uri;
        Ok(())
    }
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

#[tauri::command]
pub async fn cmd_run_latency_benchmark(
    window: tauri::WebviewWindow,
) -> Result<crate::latency::LatencyBenchmarkReport, String> {
    use tauri::Emitter;
    let cancel_token = crate::latency::reset_cancel_flag();
    let win = window.clone();

    crate::latency::run_benchmark_session(cancel_token, move |progress| {
        let _ = win.emit("latency_progress", progress);
    })
    .await
}

#[tauri::command]
pub fn cmd_cancel_latency_benchmark() -> Result<bool, String> {
    crate::latency::cancel_current_benchmark();
    Ok(true)
}

#[tauri::command]
pub fn cmd_get_network_tweaks_status() -> Result<crate::network::NetworkTweaksStatus, String> {
    Ok(crate::network::get_network_tweaks_status())
}

#[tauri::command]
pub fn cmd_toggle_tcp_latency_optimization(optimize: bool) -> Result<crate::tweaks::TweakActionResult, String> {
    Ok(crate::network::toggle_tcp_latency_optimization(optimize))
}

#[tauri::command]
pub fn cmd_sync_all_qos_policies() -> Result<usize, String> {
    use crate::profiles::{JsonFileProfileRepository, ProfileRepository};
    let repo = JsonFileProfileRepository::new();
    let store = repo.load()?;
    let profiles: Vec<crate::profiles::GameProfile> = store.profiles.into_values().collect();
    crate::network::sync_profiles_qos_policies(&profiles)
}

#[tauri::command]
pub async fn cmd_run_ping_jitter_probe(target_host: Option<String>) -> Result<crate::network::PingProbeReport, String> {
    let host = target_host.unwrap_or_else(|| "1.1.1.1".to_string());
    crate::network::run_ping_jitter_probe(&host).await
}


