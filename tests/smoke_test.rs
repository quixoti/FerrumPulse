use ferrumpulse::cleaner::scan_storage;
use ferrumpulse::memory::get_memory_stats;
use ferrumpulse::power::get_power_status;
use ferrumpulse::process::ProcessPacingManager;
use ferrumpulse::tweaks::get_tweaks_status;

#[test]
fn test_power_status_query() {
    let status = get_power_status();
    assert!(status.is_ok(), "Power status query should succeed");
    let s = status.unwrap();
    assert!(!s.active_scheme_name.is_empty());
    assert!(!s.active_scheme_guid.is_empty());
}

#[test]
fn test_memory_stats_query() {
    let stats = get_memory_stats();
    assert!(stats.is_ok(), "Memory stats query should succeed");
    let m = stats.unwrap();
    assert!(m.total_ram_mb > 0);
    assert!(m.available_ram_mb > 0);
    assert!(m.memory_load_percent <= 100);
}

#[test]
fn test_tweaks_status_query() {
    let st = get_tweaks_status();
    // Verify all 6 preset fields can be checked safely without throwing panics
    println!("DiagTrack disabled: {}", st.diagtrack_disabled);
    println!("GameDVR disabled: {}", st.gamedvr_disabled);
    println!("Network pacing optimized: {}", st.network_pacing_optimized);
    println!("Global timer enabled: {}", st.global_timer_enabled);
    println!("Raw mouse enabled: {}", st.raw_mouse_enabled);
    println!("Autostart enabled: {}", st.autostart_enabled);
}

#[test]
fn test_raw_mouse_toggle_idempotent() {
    use ferrumpulse::tweaks::{is_raw_mouse_input_enabled, toggle_raw_mouse_input};
    let initial = is_raw_mouse_input_enabled();
    // Toggle should return clean result and not panic
    let res = toggle_raw_mouse_input(initial);
    assert!(res.success);
}

#[test]
fn test_battery_telemetry_query() {
    use ferrumpulse::battery::get_battery_telemetry;
    let b = get_battery_telemetry();
    println!("Has battery: {}, AC connected: {}, Pct: {}%", b.has_battery, b.ac_connected, b.battery_percent);
    assert!(b.battery_percent <= 100);
}

#[test]
fn test_storage_scanner() {
    let scan = scan_storage();
    assert!(scan.total_cleanable_mb >= 0.0);
    assert!(!scan.shader_cache_warning.is_empty());
}

#[test]
fn test_daemon_config_defaults() {
    let manager = ProcessPacingManager::new();
    let cfg = manager.get_config();
    assert!(cfg.enabled);
    assert!(cfg.boost_priority);
    assert!(cfg.throttle_background);
    assert!(cfg.target_games.contains(&"cs2.exe".to_string()));
    assert!(cfg.throttled_apps.contains(&"chrome.exe".to_string()));
    // Verify companion apps are never throttled by default
    assert!(!cfg.throttled_apps.contains(&"discord.exe".to_string()));
    assert!(!cfg.throttled_apps.contains(&"epicgameslauncher.exe".to_string()));
}

#[test]
fn test_protected_throttling_whitelist() {
    use ferrumpulse::process::PROTECTED_THROTTLING_WHITELIST;
    assert!(PROTECTED_THROTTLING_WHITELIST.contains(&"discord.exe"));
    assert!(PROTECTED_THROTTLING_WHITELIST.contains(&"obs64.exe"));
    assert!(PROTECTED_THROTTLING_WHITELIST.contains(&"audiodg.exe"));
    assert!(PROTECTED_THROTTLING_WHITELIST.contains(&"dwm.exe"));
    assert!(PROTECTED_THROTTLING_WHITELIST.contains(&"steam.exe"));
}

#[test]
fn test_protected_memory_processes() {
    use ferrumpulse::memory::PROTECTED_SYSTEM_PROCESSES;
    assert!(PROTECTED_SYSTEM_PROCESSES.contains(&"csrss.exe"));
    assert!(PROTECTED_SYSTEM_PROCESSES.contains(&"dwm.exe"));
    assert!(PROTECTED_SYSTEM_PROCESSES.contains(&"audiodg.exe"));
}

