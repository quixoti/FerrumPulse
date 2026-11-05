pub mod battery;
pub mod cleaner;
pub mod commands;
pub mod memory;
pub mod power;
pub mod process;
pub mod tray;
pub mod tweaks;

use std::sync::Arc;
use tauri::Manager;
use process::ProcessPacingManager;

pub fn run() {
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
            commands::cmd_scan_storage,
            commands::cmd_clean_storage,
            commands::cmd_empty_recycle_bin,
            commands::cmd_one_click_boost,
            commands::cmd_get_battery_status,
            commands::cmd_set_battery_eco,
            commands::cmd_set_display_refresh_rate,
            commands::cmd_minimize_window,
            commands::cmd_toggle_maximize_window,
            commands::cmd_hide_window,
            commands::cmd_drag_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FerrumPulse application");
}
