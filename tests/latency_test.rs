// tests/latency_test.rs - Unit tests for Latency & Stutter Telemetry Engine

use ferrumpulse::latency::{
    calculate_jitter_us, get_timer_resolutions, synthesize_report, LatencyGrade,
};

#[test]
fn test_calculate_jitter_zero_and_variance() {
    assert_eq!(calculate_jitter_us(&[]), 0.0);
    assert_eq!(calculate_jitter_us(&[1000.0]), 0.0);

    // Identical intervals = 0 jitter
    let identical = vec![1000.0, 1000.0, 1000.0, 1000.0];
    assert_eq!(calculate_jitter_us(&identical), 0.0);

    // Known variance
    let varying = vec![100.0, 200.0]; // mean = 150, diffs = -50, +50, variance = 2500, sqrt = 50
    assert!((calculate_jitter_us(&varying) - 50.0).abs() < 1e-6);
}

#[test]
fn test_synthesize_report_pristine_grade_a_plus() {
    let report = synthesize_report(
        120.0,  // peak dpc = 120µs (<250)
        0.4,    // avg dpc % = 0.4%
        12.0,   // jitter = 12µs (<30)
        1.0,    // timer = 1.0ms
        0.5,    // min timer = 0.5ms
        0,      // glitches = 0
        150,    // samples
        15.0,   // elapsed
    );

    assert_eq!(report.grade, LatencyGrade::GradeAPlus);
    assert_eq!(report.grade_label, "A+");
    assert!(report.score >= 95);
    assert!(report.primary_culprit_hint.is_none());
    assert!(report.recommendations.iter().any(|r| r.contains("pristine")));
}

#[test]
fn test_synthesize_report_high_dpc_penalty_grade_c() {
    let report = synthesize_report(
        1450.0, // critical peak dpc (>1000µs)
        4.8,    // high dpc % (>4%)
        95.0,   // high jitter (>80µs)
        15.6,   // stock resolution (>2.0ms)
        0.5,
        2,      // 2 glitches
        150,
        15.0,
    );

    assert_eq!(report.grade, LatencyGrade::GradeC);
    assert_eq!(report.grade_label, "C");
    assert!(report.score < 70);
    assert!(report.primary_culprit_hint.is_some());
    assert!(report.recommendations.iter().any(|r| r.contains("exceeded 1000µs")));
    assert!(report.recommendations.iter().any(|r| r.contains("Dynamic 1.0ms Timer")));
}

#[test]
fn test_get_timer_resolutions_returns_valid_floats() {
    let (cur, min) = get_timer_resolutions();
    assert!(cur > 0.0);
    assert!(min > 0.0);
    assert!(min <= cur);
}

