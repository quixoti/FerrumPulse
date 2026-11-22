use ferrumpulse::scanner::{
    find_primary_executable_in_dir, parse_epic_manifest_content, parse_steam_library_folders,
    parse_xbox_manifest_xml, DiscoveredGame, InstallSource,
};

#[test]
fn test_parse_steam_library_folders() {
    let sample_vdf = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"apps"
		{
			"730"		"384729103"
		}
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
		"label"		"Games"
		"apps"
		{
			"570"		"294810294"
		}
	}
}
"#;
    let paths = parse_steam_library_folders(sample_vdf);
    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0], "C:\\Program Files (x86)\\Steam");
    assert_eq!(paths[1], "D:\\SteamLibrary");
}

#[test]
fn test_parse_epic_manifest_content() {
    let sample_json = r#"{
        "FormatVersion": 0,
        "AppName": "Fortnite",
        "CatalogItemId": "4fe75bbc5a674f4f9b356b5c90567da5",
        "DisplayName": "Fortnite",
        "InstallLocation": "C:\\Program Files\\Epic Games\\Fortnite",
        "LaunchExecutable": "FortniteClient-Win64-Shipping.exe"
    }"#;

    let game: Option<DiscoveredGame> = parse_epic_manifest_content(sample_json);
    assert!(game.is_some());
    let g = game.unwrap();
    assert_eq!(g.name, "Fortnite");
    assert_eq!(g.executable_name, "FortniteClient-Win64-Shipping.exe");
    assert_eq!(g.source, InstallSource::Epic);
    assert!(g.executable_path.ends_with("FortniteClient-Win64-Shipping.exe"));
}

#[test]
fn test_deep_executable_discovery_prefers_ue5_shipping_and_skips_redist() {
    let root = std::env::temp_dir().join(format!("ferrumpulse_deep_scan_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);

    let shipping_dir = root.join("MyGame").join("Binaries").join("Win64");
    let redist_dir = root.join("_CommonRedist").join("vcredist");
    std::fs::create_dir_all(&shipping_dir).unwrap();
    std::fs::create_dir_all(&redist_dir).unwrap();

    // Decoy files
    std::fs::write(root.join("CrashReportClient.exe"), vec![0u8; 500]).unwrap();
    std::fs::write(redist_dir.join("vc_redist.x64.exe"), vec![0u8; 9000]).unwrap();
    // Real nested Unreal Engine shipping binary
    let real_exe = shipping_dir.join("MyGame-Win64-Shipping.exe");
    std::fs::write(&real_exe, vec![0u8; 2048]).unwrap();

    let found = find_primary_executable_in_dir(&root).expect("Should find nested shipping binary");
    assert_eq!(
        found.file_name().unwrap().to_string_lossy(),
        "MyGame-Win64-Shipping.exe"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_parse_xbox_manifest_xml() {
    let config_xml = r#"
    <Game configVersion="1">
      <ExecutableList>
        <Executable Name="Palworld-WinGDK-Shipping.exe" Id="Game" />
      </ExecutableList>
      <ShellVisuals DefaultDisplayName="Palworld" />
    </Game>
    "#;

    let parsed = parse_xbox_manifest_xml(config_xml).expect("Should parse MicrosoftGame.config");
    assert_eq!(parsed.0, "Palworld");
    assert_eq!(parsed.1, "Palworld-WinGDK-Shipping.exe");
}

