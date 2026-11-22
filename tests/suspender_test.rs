// tests/suspender_test.rs - Unit tests for Intelligent Launcher Webview Suspender

use ferrumpulse::suspender::{
    classify_suspendable_renderer, evaluate_smart_resume, filter_matching_orphans,
    launcher_family_for_foreground, load_recovery_records_from, save_recovery_records_to,
    LauncherFamily, ResumeDecision, SuspendedWebviewRecord,
};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_classify_renderer_accepts_only_storefront_renderers() {
    let renderer_args = vec![
        "steamwebhelper.exe".into(),
        "--type=renderer".into(),
        "--lang=en-US".into(),
    ];
    assert_eq!(
        classify_suspendable_renderer("steamwebhelper.exe", &renderer_args),
        Some(LauncherFamily::Steam)
    );
    assert_eq!(
        classify_suspendable_renderer("EpicWebHelper.exe", &renderer_args),
        Some(LauncherFamily::Epic)
    );
    assert_eq!(
        classify_suspendable_renderer("RiotClientUxRender.exe", &renderer_args),
        Some(LauncherFamily::Riot)
    );
    assert_eq!(
        classify_suspendable_renderer("uplaywebcore.exe", &renderer_args),
        Some(LauncherFamily::Ubisoft)
    );
}

#[test]
fn test_classify_renderer_rejects_gpu_utility_masters_and_protected_apps() {
    let gpu_args = vec!["steamwebhelper.exe".into(), "--type=gpu-process".into()];
    let utility_args = vec!["steamwebhelper.exe".into(), "--type=utility".into()];
    let master_args = vec!["steamwebhelper.exe".into()];
    let renderer_args = vec!["helper.exe".into(), "--type=renderer".into()];

    // Must never freeze GPU/utility helpers or master processes without --type=renderer
    assert_eq!(classify_suspendable_renderer("steamwebhelper.exe", &gpu_args), None);
    assert_eq!(classify_suspendable_renderer("steamwebhelper.exe", &utility_args), None);
    assert_eq!(classify_suspendable_renderer("steamwebhelper.exe", &master_args), None);
    assert_eq!(classify_suspendable_renderer("steam.exe", &renderer_args), None);
    assert_eq!(classify_suspendable_renderer("epicgameslauncher.exe", &renderer_args), None);

    // Must never freeze browsers, Discord, or OBS even with --type=renderer
    assert_eq!(classify_suspendable_renderer("chrome.exe", &renderer_args), None);
    assert_eq!(classify_suspendable_renderer("msedge.exe", &renderer_args), None);
    assert_eq!(classify_suspendable_renderer("discord.exe", &renderer_args), None);
    assert_eq!(classify_suspendable_renderer("obs64.exe", &renderer_args), None);
}

#[test]
fn test_smart_resume_state_transitions() {
    let grace = Duration::from_secs(10);

    // 1. Game alive & focused -> Hold suspended
    assert_eq!(
        evaluate_smart_resume(true, true, None, None, grace),
        ResumeDecision::Hold
    );

    // 2. Short Alt-Tab to browser/Discord (< 10s) -> Hold suspended
    assert_eq!(
        evaluate_smart_resume(true, false, None, Some(Duration::from_secs(4)), grace),
        ResumeDecision::Hold
    );

    // 3. Alt-Tab directly to Steam client -> Immediately resume Steam webviews
    let steam_fg = launcher_family_for_foreground("steam.exe");
    assert_eq!(steam_fg, Some(LauncherFamily::Steam));
    assert_eq!(
        evaluate_smart_resume(true, false, steam_fg, Some(Duration::from_secs(1)), grace),
        ResumeDecision::ResumeFamily(LauncherFamily::Steam)
    );

    // 4. Alt-Tab away for >= 10s -> ResumeAll
    assert_eq!(
        evaluate_smart_resume(true, false, None, Some(Duration::from_secs(10)), grace),
        ResumeDecision::ResumeAll
    );

    // 5. Game exited -> Immediate ResumeAll
    assert_eq!(
        evaluate_smart_resume(false, false, None, None, grace),
        ResumeDecision::ResumeAll
    );
}

#[test]
fn test_orphan_recovery_verifies_start_time_against_pid_reuse() {
    let records = vec![
        SuspendedWebviewRecord {
            pid: 1001,
            exe_name: "steamwebhelper.exe".into(),
            start_time: 1700000100,
            family: LauncherFamily::Steam,
        },
        SuspendedWebviewRecord {
            pid: 2002,
            exe_name: "epicwebhelper.exe".into(),
            start_time: 1700000200,
            family: LauncherFamily::Epic,
        },
    ];

    let mut live = HashMap::new();
    // PID 1001 exact match -> should recover
    live.insert(1001, ("steamwebhelper.exe".into(), 1700000100));
    // PID 2002 reused by notepad.exe with newer start_time -> MUST skip!
    live.insert(2002, ("notepad.exe".into(), 1700099999));

    let matched = filter_matching_orphans(&records, &live);
    assert_eq!(matched, vec![1001]);

    // Persistence roundtrip in temp file
    let tmp = std::env::temp_dir().join(format!("ferrumpulse_recover_test_{}.json", std::process::id()));
    save_recovery_records_to(&tmp, &records);
    let loaded = load_recovery_records_from(&tmp);
    assert_eq!(loaded, records);
    save_recovery_records_to(&tmp, &[]);
    assert!(!tmp.exists());
}
