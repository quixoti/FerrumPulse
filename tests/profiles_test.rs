use ferrumpulse::battery::{get_supported_refresh_rates, normalize_supported_refresh_rates};
use ferrumpulse::profiles::{
    export_profiles_json, import_profiles_from_json, GameProfile, InMemoryProfileRepository,
    InstallSource, ProfileRepository,
};

#[test]
fn test_default_profiles_store_has_desktop_fallback() {
    let repo = InMemoryProfileRepository::new();
    let store = repo.load().expect("Should load initial store");
    assert_eq!(store.default_desktop.power_scheme, "balanced");
    assert_eq!(store.default_desktop.core_unparking_percent, 50);
    assert!(!store.default_desktop.lock_timer_resolution);
}

#[test]
fn test_profile_creation_and_matching() {
    let repo = InMemoryProfileRepository::new();
    let mut store = repo.load().unwrap();

    let cs2_profile = GameProfile::new(
        "cs2-profile-id".into(),
        "Counter-Strike 2".into(),
        vec!["cs2.exe".into()],
        InstallSource::Steam,
    )
    .with_power_scheme("high".into())
    .with_core_unparking(100)
    .with_timer_lock(true)
    .with_refresh_rate(240);

    store.insert(cs2_profile);
    repo.save(&store).expect("Should save store");

    let loaded = repo.load().unwrap();
    let matched = loaded.find_by_executable("CS2.EXE");
    assert!(matched.is_some(), "Matching should be case-insensitive");
    let p = matched.unwrap();
    assert_eq!(p.display_name, "Counter-Strike 2");
    assert_eq!(p.power_scheme, "high");
    assert_eq!(p.core_unparking_percent, 100);
    assert!(p.lock_timer_resolution);
    assert_eq!(p.target_refresh_rate, Some(240));
}

#[test]
fn test_profile_deletion() {
    let repo = InMemoryProfileRepository::new();
    let mut store = repo.load().unwrap();

    let valorant = GameProfile::new(
        "val-id".into(),
        "Valorant".into(),
        vec!["valorant.exe".into()],
        InstallSource::Riot,
    );
    store.insert(valorant);
    assert!(store.find_by_executable("valorant.exe").is_some());

    assert!(store.remove("val-id"));
    assert!(store.find_by_executable("valorant.exe").is_none());
}

#[test]
fn test_export_and_import_profiles_roundtrip() {
    let repo = InMemoryProfileRepository::new();
    let mut store = repo.load().unwrap();

    let cs2 = GameProfile::new(
        "cs2-id".into(),
        "Counter-Strike 2".into(),
        vec!["cs2.exe".into()],
        InstallSource::Steam,
    )
    .with_refresh_rate(240);
    store.insert(cs2);

    let json = export_profiles_json(&store).expect("Export should succeed");
    let mut target = repo.load().unwrap();
    let imported = import_profiles_from_json(&json, &mut target).expect("Import should succeed");
    assert_eq!(imported, 1);
    assert!(target.find_by_executable("cs2.exe").is_some());
}

#[test]
fn test_supported_refresh_rates_normalization() {
    let rates = normalize_supported_refresh_rates(vec![60, 240, 144, 60, 15, 240, 120]);
    assert_eq!(rates, vec![240, 144, 120, 60]);

    let live = get_supported_refresh_rates();
    assert!(!live.is_empty());
}

