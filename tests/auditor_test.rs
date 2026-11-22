use ferrumpulse::auditor::{
    calculate_readiness_score, ActionType, AuditCheckpoint, CheckpointTier, HagsState,
    Win11AuditorReport,
};

#[test]
fn test_hags_unsupported_does_not_penalize_score() {
    let checkpoints = vec![
        AuditCheckpoint {
            id: "hags".into(),
            title: "Hardware-Accelerated GPU Scheduling (HAGS)".into(),
            tier: CheckpointTier::Advisory,
            status_text: "Unsupported by GPU/Driver".into(),
            is_optimized: false,
            latency_impact: "Low".into(),
            explanation: "GPU does not support WDDM 2.7+".into(),
            action: ActionType::None,
        },
        AuditCheckpoint {
            id: "windowed_opt".into(),
            title: "Optimizations for Windowed Games".into(),
            tier: CheckpointTier::Optimized,
            status_text: "Enabled (Modern Flip Model)".into(),
            is_optimized: true,
            latency_impact: "High".into(),
            explanation: "DXGI flip presentation active".into(),
            action: ActionType::Toggle { enable: false },
        },
    ];

    let score = calculate_readiness_score(&checkpoints);
    // Since HAGS is unsupported, the eligible score should be 100% of applicable items
    assert_eq!(score, 100, "Unsupported hardware should not reduce the readiness score");
}

#[test]
fn test_auditor_report_generation_and_serialization() {
    let report = Win11AuditorReport {
        readiness_score: 85,
        optimized_count: 4,
        actionable_count: 1,
        advisory_count: 1,
        hags_state: HagsState::Enabled,
        checkpoints: vec![
            AuditCheckpoint {
                id: "game_mode".into(),
                title: "Windows Game Mode".into(),
                tier: CheckpointTier::Optimized,
                status_text: "Enabled".into(),
                is_optimized: true,
                latency_impact: "High".into(),
                explanation: "Prioritizes CPU and GPU resources".into(),
                action: ActionType::Toggle { enable: false },
            },
        ],
    };

    let json = serde_json::to_string(&report).expect("Should serialize auditor report");
    assert!(json.contains("readiness_score"));
    assert!(json.contains("game_mode"));
}

