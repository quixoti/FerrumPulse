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
    // Verify all preset fields can be checked safely without throwing panics
    println!("DiagTrack disabled: {}", st.diagtrack_disabled);
    println!("GameDVR disabled: {}", st.gamedvr_disabled);
    println!("Network pacing optimized: {}", st.network_pacing_optimized);
    println!("Global timer enabled: {}", st.global_timer_enabled);
    println!("Raw mouse enabled: {}", st.raw_mouse_enabled);
    println!("Autostart enabled: {}", st.autostart_enabled);
    println!("MMCSS gaming priority: {}", st.mmcss_gaming_priority);
    println!("Win32 priority separation: {}", st.win32_priority_separation);
    println!("SystemResponsiveness optimized: {}", st.system_responsiveness_optimized);
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
    assert!(cfg.throttled_apps.contains(&"spotify.exe".to_string()));
    // Verify browsers and companion apps are strictly NEVER throttled by default
    assert!(!cfg.throttled_apps.contains(&"chrome.exe".to_string()));
    assert!(!cfg.throttled_apps.contains(&"msedge.exe".to_string()));
    assert!(!cfg.throttled_apps.contains(&"discord.exe".to_string()));
    assert!(!cfg.throttled_apps.contains(&"epicgameslauncher.exe".to_string()));
}

#[test]
fn test_protected_throttling_whitelist() {
    use ferrumpulse::process::PROTECTED_THROTTLING_WHITELIST;
    assert!(PROTECTED_THROTTLING_WHITELIST.contains(&"chrome.exe"));
    assert!(PROTECTED_THROTTLING_WHITELIST.contains(&"msedge.exe"));
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
    assert!(PROTECTED_SYSTEM_PROCESSES.contains(&"chrome.exe"));
    assert!(PROTECTED_SYSTEM_PROCESSES.contains(&"msedge.exe"));
    assert!(PROTECTED_SYSTEM_PROCESSES.contains(&"audiodg.exe"));
}

#[test]
fn test_battery_config_persistence_and_memory_guard_cooldown() {
    use ferrumpulse::battery::{load_battery_config_from, save_battery_config_to, BatteryConfig};
    use ferrumpulse::memory::{should_auto_trim_ram, MemoryGuardConfig};

    let tmp = std::env::temp_dir().join(format!("ferrumpulse_bat_cfg_{}.json", std::process::id()));
    let _ = std::fs::remove_file(&tmp);

    let mut cfg = BatteryConfig::default();
    cfg.refresh_rate_saver = false;
    cfg.saved_ac_refresh_rate = Some(240);
    save_battery_config_to(&tmp, &cfg).expect("Should persist battery config");

    let loaded = load_battery_config_from(&tmp);
    assert!(!loaded.refresh_rate_saver);
    assert_eq!(loaded.saved_ac_refresh_rate, Some(240));
    let _ = std::fs::remove_file(&tmp);

    // Memory guard: off by default
    let mut mg = MemoryGuardConfig::default();
    assert!(!should_auto_trim_ram(95, &mg, None));

    // Enabled: triggers >= 88%, respects 60s cooldown
    mg.auto_trim_at_high_pressure = true;
    assert!(!should_auto_trim_ram(87, &mg, None));
    assert!(should_auto_trim_ram(88, &mg, None));
    assert!(!should_auto_trim_ram(92, &mg, Some(30)));
    assert!(should_auto_trim_ram(92, &mg, Some(60)));
}

