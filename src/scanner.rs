pub use crate::profiles::InstallSource;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A normalized, discovered game instance found on the user's storage drives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredGame {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub executable_name: String,
    pub source: InstallSource,
    pub install_directory: String,
    pub suggested_refresh_rate: Option<u32>,
}

/// Abstract contract for game library scanning (Open/Closed Principle).
pub trait LibraryScanner: Send + Sync {
    fn scan(&self) -> Vec<DiscoveredGame>;
}

// ---------------------------------------------------------------------------
// 1. Steam Library Scanner
// ---------------------------------------------------------------------------

pub struct SteamScanner;

impl LibraryScanner for SteamScanner {
    fn scan(&self) -> Vec<DiscoveredGame> {
        let mut results = Vec::new();
        let steam_paths = get_steam_install_roots();

        for steam_root in steam_paths {
            let vdf_path = steam_root.join("steamapps").join("libraryfolders.vdf");
            if !vdf_path.exists() {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(&vdf_path) {
                let library_paths = parse_steam_library_folders(&content);
                for lib in library_paths {
                    let steamapps_dir = PathBuf::from(lib).join("steamapps");
                    if let Ok(entries) = std::fs::read_dir(&steamapps_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                                    if file_name.starts_with("appmanifest_") && file_name.ends_with(".acf") {
                                        if let Some(game) = parse_steam_manifest(&path, &steamapps_dir) {
                                            results.push(game);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }
}

pub fn parse_steam_library_folders(content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"path\"") {
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 4 {
                let p = parts[3].replace("\\\\", "\\");
                if !p.is_empty() {
                    paths.push(p);
                }
            }
        }
    }
    paths
}

fn parse_steam_manifest(manifest_path: &Path, steamapps_dir: &Path) -> Option<DiscoveredGame> {
    let content = std::fs::read_to_string(manifest_path).ok()?;
    let mut app_id = String::new();
    let mut name = String::new();
    let mut installdir = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"appid\"") {
            if let Some(val) = extract_vdf_value(trimmed) {
                app_id = val;
            }
        } else if trimmed.starts_with("\"name\"") {
            if let Some(val) = extract_vdf_value(trimmed) {
                name = val;
            }
        } else if trimmed.starts_with("\"installdir\"") {
            if let Some(val) = extract_vdf_value(trimmed) {
                installdir = val;
            }
        }
    }

    if app_id.is_empty() || installdir.is_empty() {
        return None;
    }

    let game_dir = steamapps_dir.join("common").join(&installdir);
    let primary_exe = find_primary_executable_in_dir(&game_dir)?;
    let exe_name = primary_exe.file_name()?.to_string_lossy().to_string();

    Some(DiscoveredGame {
        id: format!("steam-{}", app_id),
        name: if name.is_empty() { installdir } else { name },
        executable_path: primary_exe.to_string_lossy().to_string(),
        executable_name: exe_name,
        source: InstallSource::Steam,
        install_directory: game_dir.to_string_lossy().to_string(),
        suggested_refresh_rate: None,
    })
}

fn extract_vdf_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

fn get_steam_install_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    #[cfg(windows)]
    {
        use windows::core::w;
        use windows::Win32::System::Registry::HKEY_CURRENT_USER;
        if let Some(reg_path) = crate::tweaks::read_registry_sz(
            HKEY_CURRENT_USER,
            w!("Software\\Valve\\Steam"),
            w!("SteamPath"),
        ) {
            let p = PathBuf::from(reg_path.replace('/', "\\"));
            if p.exists() && !roots.contains(&p) {
                roots.push(p);
            }
        }
    }
    if let Ok(prog_x86) = std::env::var("ProgramFiles(x86)") {
        let default_steam = PathBuf::from(prog_x86).join("Steam");
        if default_steam.exists() && !roots.contains(&default_steam) {
            roots.push(default_steam);
        }
    }
    roots
}

// ---------------------------------------------------------------------------
// 2. Epic Games Launcher Scanner
// ---------------------------------------------------------------------------

pub struct EpicScanner;

impl LibraryScanner for EpicScanner {
    fn scan(&self) -> Vec<DiscoveredGame> {
        let mut results = Vec::new();
        if let Ok(prog_data) = std::env::var("ProgramData") {
            let manifests_dir = PathBuf::from(prog_data)
                .join("Epic")
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests");

            if manifests_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&manifests_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("item") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Some(game) = parse_epic_manifest_content(&content) {
                                    results.push(game);
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }
}

#[derive(Deserialize)]
struct EpicItemJson {
    #[serde(rename = "CatalogItemId")]
    catalog_item_id: Option<String>,
    #[serde(rename = "DisplayName")]
    display_name: Option<String>,
    #[serde(rename = "InstallLocation")]
    install_location: Option<String>,
    #[serde(rename = "LaunchExecutable")]
    launch_executable: Option<String>,
}

pub fn parse_epic_manifest_content(content: &str) -> Option<DiscoveredGame> {
    let item: EpicItemJson = serde_json::from_str(content).ok()?;
    let install_dir = item.install_location?;
    let launch_exe = item.launch_executable?;
    let name = item.display_name.unwrap_or_else(|| launch_exe.clone());
    let id = item.catalog_item_id.unwrap_or_else(|| name.clone());

    let full_path = PathBuf::from(&install_dir).join(&launch_exe);

    Some(DiscoveredGame {
        id: format!("epic-{}", id),
        name,
        executable_path: full_path.to_string_lossy().to_string(),
        executable_name: launch_exe,
        source: InstallSource::Epic,
        install_directory: install_dir,
        suggested_refresh_rate: None,
    })
}

// ---------------------------------------------------------------------------
// 3. Riot Games Scanner
// ---------------------------------------------------------------------------

pub struct RiotScanner;

impl LibraryScanner for RiotScanner {
    fn scan(&self) -> Vec<DiscoveredGame> {
        let mut results = Vec::new();
        if let Ok(prog_data) = std::env::var("ProgramData") {
            let config_path = PathBuf::from(prog_data)
                .join("Riot Games")
                .join("RiotClientInstalls.json");

            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(obj) = val.as_object() {
                            for (product, path_val) in obj {
                                if let Some(dir_str) = path_val.as_str() {
                                    let dir = PathBuf::from(dir_str);
                                    if let Some(exe_path) = find_primary_executable_in_dir(&dir) {
                                        let exe_name = exe_path
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .to_string();

                                        results.push(DiscoveredGame {
                                            id: format!("riot-{}", product),
                                            name: format!("Riot: {}", product),
                                            executable_path: exe_path.to_string_lossy().to_string(),
                                            executable_name: exe_name,
                                            source: InstallSource::Riot,
                                            install_directory: dir_str.to_string(),
                                            suggested_refresh_rate: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }
}

// ---------------------------------------------------------------------------
// 4. Xbox / PC Game Pass Scanner (Pure-Rust XML, Zero PowerShell)
// ---------------------------------------------------------------------------

pub struct XboxGamePassScanner;

/// Extracts `(display_name, executable_relative_path)` from Xbox `MicrosoftGame.config`
/// or `AppxManifest.xml` content without requiring external XML crates.
pub fn parse_xbox_manifest_xml(content: &str) -> Option<(String, String)> {
    // 1. Display name: prefer <ShellVisuals DefaultDisplayName="..."> or <DisplayName>...</DisplayName>
    let display_name = extract_xml_attr(content, "DefaultDisplayName")
        .or_else(|| extract_xml_tag_text(content, "DisplayName"))
        .filter(|s| !s.is_empty() && !s.starts_with("ms-resource:"));

    // 2. Executable: prefer <Executable Name="..."> (MicrosoftGame.config) or Executable="..." (AppxManifest.xml)
    let exe_rel = extract_xml_attr(content, "Name")
        .filter(|s| s.to_lowercase().ends_with(".exe"))
        .or_else(|| {
            extract_xml_attr(content, "Executable").filter(|s| s.to_lowercase().ends_with(".exe"))
        })?;

    let fallback_name = Path::new(&exe_rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Xbox Game")
        .to_string();

    Some((display_name.unwrap_or(fallback_name), exe_rel))
}

fn extract_xml_attr(xml: &str, attr: &str) -> Option<String> {
    let needle = format!("{}=\"", attr);
    let idx = xml.find(&needle)?;
    let rest = &xml[idx + needle.len()..];
    let end = rest.find('"')?;
    let val = rest[..end].trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

fn extract_xml_tag_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let rest = &xml[start..];
    let end = rest.find(&close)?;
    let val = rest[..end].trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

impl LibraryScanner for XboxGamePassScanner {
    fn scan(&self) -> Vec<DiscoveredGame> {
        let mut results = Vec::new();
        let mut roots: Vec<PathBuf> = Vec::new();

        // Check <Drive>:\XboxGames across mounted drive letters A-Z
        for drive in b'C'..=b'Z' {
            let p = PathBuf::from(format!("{}:\\XboxGames", drive as char));
            if p.exists() && p.is_dir() {
                roots.push(p);
            }
        }
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            let mod_apps = PathBuf::from(prog_files).join("ModifiableWindowsApps");
            if mod_apps.exists() && mod_apps.is_dir() {
                roots.push(mod_apps);
            }
        }

        for root in roots {
            if let Ok(entries) = std::fs::read_dir(&root) {
                for entry in entries.flatten() {
                    let game_root = entry.path();
                    if !game_root.is_dir() {
                        continue;
                    }
                    let content_dir = if game_root.join("Content").is_dir() {
                        game_root.join("Content")
                    } else {
                        game_root.clone()
                    };

                    let manifest_candidates = [
                        content_dir.join("MicrosoftGame.config"),
                        content_dir.join("AppxManifest.xml"),
                    ];

                    let mut matched = None;
                    for mf in &manifest_candidates {
                        if mf.exists() {
                            if let Ok(xml) = std::fs::read_to_string(mf) {
                                if let Some((name, exe_rel)) = parse_xbox_manifest_xml(&xml) {
                                    let exe_path = content_dir.join(&exe_rel);
                                    let exe_name = exe_path
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or(&exe_rel)
                                        .to_string();
                                    let folder_id = game_root
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or(&name)
                                        .to_lowercase()
                                        .replace(' ', "-");
                                    matched = Some(DiscoveredGame {
                                        id: format!("xbox-{}", folder_id),
                                        name,
                                        executable_path: exe_path.to_string_lossy().to_string(),
                                        executable_name: exe_name,
                                        source: InstallSource::XboxGamePass,
                                        install_directory: content_dir.to_string_lossy().to_string(),
                                        suggested_refresh_rate: None,
                                    });
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(g) = matched {
                        results.push(g);
                    }
                }
            }
        }
        results
    }
}

// ---------------------------------------------------------------------------
// 5. Composite Coordinator Scanner
// ---------------------------------------------------------------------------

pub struct CompositeLibraryScanner {
    scanners: Vec<Box<dyn LibraryScanner>>,
}

impl CompositeLibraryScanner {
    pub fn default_suite() -> Self {
        Self {
            scanners: vec![
                Box::new(SteamScanner),
                Box::new(EpicScanner),
                Box::new(RiotScanner),
                Box::new(XboxGamePassScanner),
            ],
        }
    }

    pub fn scan_all(&self) -> Vec<DiscoveredGame> {
        let mut all_games = Vec::new();
        for scanner in &self.scanners {
            all_games.extend(scanner.scan());
        }
        all_games.sort_by(|a, b| a.name.cmp(&b.name));
        all_games.dedup_by(|a, b| a.executable_path == b.executable_path);
        all_games
    }
}

/// Bounded recursive search (max depth 4) that discovers real game binaries nested inside
/// modern engines (e.g. `game/bin/win64/cs2.exe`, `*/Binaries/Win64/*-Win64-Shipping.exe`,
/// `bin/x64/Cyberpunk2077.exe`) while ignoring crash reporters, installers, and redists.
pub fn find_primary_executable_in_dir(dir: &Path) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }

    let root_hint = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase()
        .replace([' ', '_', '-'], "");

    let mut candidates: Vec<(i32, u64, PathBuf)> = Vec::new();
    collect_exe_candidates(dir, 0, 4, &root_hint, &mut candidates);

    candidates.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    candidates.into_iter().next().map(|(_, _, p)| p)
}

fn collect_exe_candidates(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    root_hint: &str,
    out: &mut Vec<(i32, u64, PathBuf)>,
) {
    if depth > max_depth {
        return;
    }

    const SKIP_DIRS: &[&str] = &[
        "_commonredist",
        "redist",
        "directx",
        "vcredist",
        "easyanticheat",
        "battleye",
        "prerequisites",
        "extras",
        "support",
        "__installer",
    ];

    const SKIP_EXE_TOKENS: &[&str] = &[
        "uninstall",
        "unins000",
        "crash",
        "setup",
        "helper",
        "redist",
        "vcredist",
        "dxsetup",
        "unitycrashhandler",
        "ue4prereq",
        "ueprereq",
        "battleye",
        "easyanticheat",
        "eac_",
        "report",
        "updater",
    ];

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if let Some(dname) = p.file_name().and_then(|s| s.to_str()) {
                    let lower_dir = dname.to_lowercase();
                    if !SKIP_DIRS.iter().any(|&s| lower_dir == s) {
                        collect_exe_candidates(&p, depth + 1, max_depth, root_hint, out);
                    }
                }
            } else if p.is_file() {
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("exe") {
                        let name = p
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_lowercase();
                        if SKIP_EXE_TOKENS.iter().any(|&t| name.contains(t)) {
                            continue;
                        }

                        let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                        let mut score: i32 = 0;

                        // Strongly prefer Unreal Engine shipping binaries
                        if name.ends_with("-win64-shipping.exe") || name.ends_with("-shipping.exe") {
                            score += 100;
                        }
                        // Prefer executables inside win64 / x64 directories
                        let path_lower = p.to_string_lossy().to_lowercase();
                        if path_lower.contains("\\win64\\") || path_lower.contains("\\x64\\") {
                            score += 30;
                        }
                        // Prefer executables whose stem resembles the game folder name
                        let stem_clean = p
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase()
                            .replace([' ', '_', '-'], "");
                        if !root_hint.is_empty()
                            && !stem_clean.is_empty()
                            && (root_hint.contains(&stem_clean) || stem_clean.contains(root_hint))
                        {
                            score += 40;
                        }
                        // Mild penalty for generic launcher stubs if a real binary exists
                        if name.contains("launcher") {
                            score -= 25;
                        }

                        out.push((score, size, p));
                    }
                }
            }
        }
    }
}

