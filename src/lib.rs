pub mod audio;
pub mod auditor;
pub mod battery;
pub mod cleaner;
pub mod commands;
pub mod latency;
pub mod memory;
pub mod network;
pub mod power;
pub mod process;
pub mod profiles;
pub mod scanner;
pub mod suspender;
pub mod tray;
pub mod tweaks;

use std::sync::Arc;
use tauri::Manager;
use process::ProcessPacingManager;

pub fn run() {
    // Recover any webviews that may have remained suspended after an unexpected crash
    suspender::recover_orphaned_suspended_processes();

    // Forcibly clear any residual EcoQoS / Efficiency Mode flags on web browsers
    process::unthrottle_all_browsers();

    let pacing_manager = Arc::new(ProcessPacingManager::new());
    pacing_manager.start_daemon();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(pacing_manager)
        .setup(|app| {
            let is_minimized = std::env::args().any(|arg| arg == "--minimized");
            if let Some(window) = app.get_webview_window("main") {
                if is_minimized {
                    let _ = window.hide();
                }

                #[cfg(target_os = "windows")]
                {
                    if let Err(e) = window_vibrancy::apply_acrylic(&window, None) {
                        eprintln!("[FerrumPulse] DWM Acrylic blur error: {:?}", e);
                    }

                    if let Ok(hwnd) = window.hwnd() {
                        use windows::Win32::Foundation::HWND;
                        use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};
                        // SAFETY: Setting DWM window corner preference on a valid HWND returned by Tauri.
                        // Passing a valid stack-allocated DWMWCP_ROUND enum and its exact byte size.
                        unsafe {
                            let target_hwnd = HWND(hwnd.0 as _);
                            let corner_pref = DWMWCP_ROUND;
                            let _ = DwmSetWindowAttribute(
                                target_hwnd,
                                DWMWA_WINDOW_CORNER_PREFERENCE,
                                &corner_pref as *const _ as _,
                                std::mem::size_of_val(&corner_pref) as _,
                            );
                        }
                    }
                }
            }

            // Setup system tray menu & click events
            if let Err(e) = tray::setup_tray(app.handle()) {
                eprintln!("Tray setup error: {}", e);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Minimize to system tray on window close
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::cmd_get_power_status,
            commands::cmd_set_core_unparking,
            commands::cmd_set_power_scheme,
            commands::cmd_get_daemon_status,
            commands::cmd_get_daemon_config,
            commands::cmd_update_daemon_config,
            commands::cmd_get_memory_stats,
            commands::cmd_purge_working_sets,
            commands::cmd_get_tweaks_status,
            commands::cmd_toggle_diagtrack,
            commands::cmd_toggle_gamedvr,
            commands::cmd_toggle_network_pacing,
            commands::cmd_toggle_global_timer,
            commands::cmd_toggle_raw_mouse,
            commands::cmd_toggle_autostart,
            commands::cmd_toggle_mmcss_gaming_priority,
            commands::cmd_toggle_win32_priority_separation,
            commands::cmd_toggle_system_responsiveness,
            commands::cmd_apply_all_scheduler_tweaks,
            commands::cmd_scan_storage,
            commands::cmd_clean_storage,
            commands::cmd_empty_recycle_bin,
            commands::cmd_one_click_boost,
            commands::cmd_get_battery_status,
            commands::cmd_set_battery_eco,
            commands::cmd_get_battery_config,
            commands::cmd_update_battery_config,
            commands::cmd_get_audio_tuning_status,
            commands::cmd_toggle_audio_engine_tuning,
            commands::cmd_get_memory_guard_config,
            commands::cmd_update_memory_guard_config,
            commands::cmd_set_display_refresh_rate,
            commands::cmd_scan_game_libraries,
            commands::cmd_list_profiles,
            commands::cmd_save_profile,
            commands::cmd_delete_profile,
            commands::cmd_get_supported_refresh_rates,
            commands::cmd_pick_game_executable,
            commands::cmd_export_profiles_to_file,
            commands::cmd_import_profiles_from_file,
            commands::cmd_quit_app,
            commands::cmd_audit_win11_gaming_features,
            commands::cmd_toggle_hags,
            commands::cmd_toggle_windowed_optimizations,
            commands::cmd_toggle_game_mode,
            commands::cmd_open_windows_settings,
            commands::cmd_minimize_window,
            commands::cmd_toggle_maximize_window,
            commands::cmd_hide_window,
            commands::cmd_drag_window,
            commands::cmd_run_latency_benchmark,
            commands::cmd_cancel_latency_benchmark,
            commands::cmd_get_network_tweaks_status,
            commands::cmd_toggle_tcp_latency_optimization,
            commands::cmd_sync_all_qos_policies,
            commands::cmd_run_ping_jitter_probe,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FerrumPulse application");
}
