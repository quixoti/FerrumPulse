// tests/network_test.rs - Unit tests for Network QoS & Ping Diagnostics Engine

use ferrumpulse::network::{
    calculate_ping_jitter, compute_orphaned_policy_keys, sanitize_policy_name,
};
use ferrumpulse::profiles::{GameProfile, InstallSource};

#[test]
fn test_sanitize_policy_name() {
    assert_eq!(sanitize_policy_name("Counter-Strike 2"), "FerrumPulse_Counter_Strike_2");
    assert_eq!(sanitize_policy_name("Apex Legends!"), "FerrumPulse_Apex_Legends_");
    assert_eq!(sanitize_policy_name("Valorant"), "FerrumPulse_Valorant");
}

#[test]
fn test_calculate_ping_jitter() {
    assert_eq!(calculate_ping_jitter(&[]), 0.0);
    assert_eq!(calculate_ping_jitter(&[15.0]), 0.0);

    // Identical pings = 0.0 jitter
    let steady = vec![12.0, 12.0, 12.0, 12.0];
    assert_eq!(calculate_ping_jitter(&steady), 0.0);

    // Known variance: 10.0 and 20.0 -> mean 15.0, diffs -5, +5, variance 25, sqrt = 5.0
    let fluctuating = vec![10.0, 20.0];
    assert!((calculate_ping_jitter(&fluctuating) - 5.0).abs() < 1e-6);
}

#[test]
fn test_game_profile_dscp_qos_default_true() {
    let prof = GameProfile::new(
        "test-cs2".into(),
        "Counter-Strike 2".into(),
        vec!["cs2.exe".into()],
        InstallSource::Steam,
    );
    assert!(prof.dscp_qos_enabled);

    // Test backwards compatibility: JSON without dscp_qos_enabled field defaults to true
    let legacy_json = r#"{
        "id": "legacy-game",
        "display_name": "Legacy Game",
        "executables": ["legacy.exe"],
        "source": "Manual",
        "icon_path": null,
        "power_scheme": "high",
        "core_unparking_percent": 100,
        "lock_timer_resolution": true,
        "target_refresh_rate": null,
        "suspend_launcher_webviews": true,
        "boost_priority": true,
        "post_game_ram_sweep": true
    }"#;

    let parsed: GameProfile = serde_json::from_str(legacy_json).expect("Failed to deserialize legacy profile JSON");
    assert!(parsed.dscp_qos_enabled);
}

#[test]
fn test_compute_orphaned_qos_policy_keys() {
    let cs2 = GameProfile::new(
        "cs2".into(),
        "Counter-Strike 2".into(),
        vec!["cs2.exe".into()],
        InstallSource::Steam,
    );
    let mut valorant = GameProfile::new(
        "val".into(),
        "Valorant".into(),
        vec!["valorant.exe".into()],
        InstallSource::Riot,
    );
    valorant.dscp_qos_enabled = false;

    let existing = vec![
        "FerrumPulse_Counter_Strike_2".to_string(),
        "FerrumPulse_Valorant".to_string(),
        "FerrumPulse_Deleted_Game".to_string(),
        "CustomUserPolicy".to_string(),
    ];

    let orphans = compute_orphaned_policy_keys(&existing, &[cs2, valorant]);
    assert!(orphans.contains(&"FerrumPulse_Valorant".to_string()));
    assert!(orphans.contains(&"FerrumPulse_Deleted_Game".to_string()));
    assert!(!orphans.contains(&"FerrumPulse_Counter_Strike_2".to_string()));
    assert!(!orphans.contains(&"CustomUserPolicy".to_string()));
}

