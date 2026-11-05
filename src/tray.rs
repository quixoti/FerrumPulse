use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let boost_i = MenuItem::with_id(app, "boost", "⚡ Instant Game Boost", true, None::<&str>)?;
    let eco_i = MenuItem::with_id(app, "eco", "🔋 Battery Saver Mode (60Hz + Eco)", true, None::<&str>)?;
    let trim_i = MenuItem::with_id(app, "trim", "🧹 Purge RAM Working Sets", true, None::<&str>)?;
    let high_i = MenuItem::with_id(app, "high_perf", "🚀 High Performance Plan", true, None::<&str>)?;
    let open_i = MenuItem::with_id(app, "open", "Open Dashboard", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit FerrumPulse", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&boost_i, &eco_i, &trim_i, &high_i, &open_i, &quit_i])?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("FerrumPulse | Gaming & PC Optimizer")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "boost" => {
                std::thread::spawn(|| {
                    let _ = crate::power::set_core_unparking(true);
                    let _ = crate::power::set_power_scheme("high");
                    let _ = crate::memory::purge_working_sets();
                    let _ = crate::tweaks::toggle_network_pacing(true);
                });
            }
            "eco" => {
                std::thread::spawn(|| {
                    let _ = crate::battery::apply_battery_eco_profile(true);
                    let _ = crate::battery::set_display_refresh_rate(60);
                });
            }
            "trim" => {
                std::thread::spawn(|| {
                    let _ = crate::memory::purge_working_sets();
                });
            }
            "high_perf" => {
                std::thread::spawn(|| {
                    let _ = crate::power::set_power_scheme("high");
                });
            }
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.destroy();
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(visible) = window.is_visible() {
                        if visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
