// tests/audio_test.rs - Unit tests for Audio Engine (audiodg.exe) Optimization & WASAPI Glitch Telemetry

use ferrumpulse::audio::{
    compute_audio_core_affinity_mask, count_dpc_fallback_glitches, is_wasapi_deadline_missed,
    load_audio_config_from, save_audio_config_to, AudioTuningConfig,
};

#[test]
fn test_compute_audio_core_affinity_mask_across_cpu_topologies() {
    assert_eq!(compute_audio_core_affinity_mask(1), None);
    assert_eq!(compute_audio_core_affinity_mask(2), Some((1usize << 1, 1)));
    assert_eq!(compute_audio_core_affinity_mask(4), Some((1usize << 2, 2)));
    assert_eq!(compute_audio_core_affinity_mask(16), Some((1usize << 2, 2)));
    assert_eq!(compute_audio_core_affinity_mask(24), Some((1usize << 2, 2)));
}

#[test]
fn test_wasapi_deadline_missed_thresholds() {
    // 10ms period = 10,000 us -> 2.5x threshold = 25,000 us
    assert!(!is_wasapi_deadline_missed(10_000, 10_000));
    assert!(!is_wasapi_deadline_missed(25_000, 10_000));
    assert!(is_wasapi_deadline_missed(25_001, 10_000));
    assert!(!is_wasapi_deadline_missed(10_000, 0));
}

#[test]
fn test_dpc_fallback_glitch_counting() {
    let clean = vec![120.0, 240.0, 9999.0];
    assert_eq!(count_dpc_fallback_glitches(&clean), 0);

    let spiked = vec![120.0, 12_500.0, 400.0, 18_000.0];
    assert_eq!(count_dpc_fallback_glitches(&spiked), 2);
}

#[test]
fn test_audio_config_persistence_roundtrip() {
    let tmp = std::env::temp_dir().join(format!("ferrumpulse_audio_cfg_{}.json", std::process::id()));
    let _ = std::fs::remove_file(&tmp);

    let default_cfg = load_audio_config_from(&tmp);
    assert!(!default_cfg.enabled);

    let enabled_cfg = AudioTuningConfig { enabled: true };
    save_audio_config_to(&tmp, &enabled_cfg).expect("Save should succeed");

    let loaded = load_audio_config_from(&tmp);
    assert!(loaded.enabled);

    let _ = std::fs::remove_file(&tmp);
}
