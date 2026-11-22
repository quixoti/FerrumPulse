use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

/// Source where the game was discovered or registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallSource {
    Steam,
    Epic,
    XboxGamePass,
    Riot,
    Manual,
}

impl Default for InstallSource {
    fn default() -> Self {
        Self::Manual
    }
}

fn default_true() -> bool {
    true
}

/// A domain model representing performance policies for a specific game executable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProfile {
    pub id: String,
    pub display_name: String,
    pub executables: Vec<String>,
    pub source: InstallSource,
    pub icon_path: Option<String>,

    // Performance Tuning Parameters
    pub power_scheme: String,             // e.g. "high", "ultimate", "balanced"
    pub core_unparking_percent: u32,      // 50..100
    pub lock_timer_resolution: bool,      // 1.0ms high-precision lock
    pub target_refresh_rate: Option<u32>, // e.g. 240, 144, 60

    // Network & Companion Optimizations
    #[serde(default = "default_true")]
    pub dscp_qos_enabled: bool,           // DSCP 46 Expedited Forwarding packet tagging
    pub suspend_launcher_webviews: bool,  // Suspend background Chromium renderers
    pub boost_priority: bool,             // HIGH_PRIORITY_CLASS ceiling
    pub post_game_ram_sweep: bool,        // Gentle memory purge 5s after game exit
}

impl GameProfile {
    pub fn new(id: String, display_name: String, executables: Vec<String>, source: InstallSource) -> Self {
        Self {
            id,
            display_name,
            executables,
            source,
            icon_path: None,
            power_scheme: "high".into(),
            core_unparking_percent: 100,
            lock_timer_resolution: true,
            target_refresh_rate: None,
            dscp_qos_enabled: true,
            suspend_launcher_webviews: true,
            boost_priority: true,
            post_game_ram_sweep: true,
        }
    }

    pub fn default_desktop() -> Self {
        Self {
            id: "default-desktop".into(),
            display_name: "Desktop Baseline".into(),
            executables: vec![],
            source: InstallSource::Manual,
            icon_path: None,
            power_scheme: "balanced".into(),
            core_unparking_percent: 50,
            lock_timer_resolution: false,
            target_refresh_rate: None,
            dscp_qos_enabled: false,
            suspend_launcher_webviews: false,
            boost_priority: false,
            post_game_ram_sweep: false,
        }
    }

    pub fn with_power_scheme(mut self, scheme: String) -> Self {
        self.power_scheme = scheme;
        self
    }

    pub fn with_core_unparking(mut self, percent: u32) -> Self {
        self.core_unparking_percent = percent.clamp(50, 100);
        self
    }

    pub fn with_timer_lock(mut self, lock: bool) -> Self {
        self.lock_timer_resolution = lock;
        self
    }

    pub fn with_refresh_rate(mut self, hz: u32) -> Self {
        self.target_refresh_rate = Some(hz);
        self
    }

    /// Matches against an incoming foreground binary filename (case-insensitive).
    pub fn matches(&self, binary_name: &str) -> bool {
        let clean_target = Path::new(binary_name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(binary_name)
            .to_lowercase();

        self.executables.iter().any(|exe| {
            let clean_exe = Path::new(exe)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(exe)
                .to_lowercase();
            clean_target == clean_exe
        })
    }
}

/// Thread-safe shared profiles store eliminating redundant disk reads in background loops.
pub type SharedProfilesStore = Arc<RwLock<ProfilesStore>>;

/// Extracts clean executable filenames (preventing path traversal or empty entries).
pub fn sanitize_executables(exes: Vec<String>) -> Vec<String> {
    exes.into_iter()
        .filter_map(|raw| {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }
            Path::new(trimmed)
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .collect()
}

/// Serializes profiles store into pretty JSON for backup/sharing.
pub fn export_profiles_json(store: &ProfilesStore) -> Result<String, String> {
    serde_json::to_string_pretty(store)
        .map_err(|e| format!("Failed to export profiles JSON: {}", e))
}

/// Parses either a full `ProfilesStore` JSON document or a JSON array of `GameProfile` objects
/// and merges them into `target`, returning the count of imported profiles.
pub fn import_profiles_from_json(json: &str, target: &mut ProfilesStore) -> Result<usize, String> {
    if let Ok(parsed_store) = serde_json::from_str::<ProfilesStore>(json) {
        let count = parsed_store.profiles.len();
        for (id, mut prof) in parsed_store.profiles {
            prof.executables = sanitize_executables(prof.executables);
            target.profiles.insert(id, prof);
        }
        return Ok(count);
    }

    if let Ok(parsed_vec) = serde_json::from_str::<Vec<GameProfile>>(json) {
        let count = parsed_vec.len();
        for mut prof in parsed_vec {
            prof.executables = sanitize_executables(prof.executables);
            target.insert(prof);
        }
        return Ok(count);
    }

    if let Ok(mut single) = serde_json::from_str::<GameProfile>(json) {
        single.executables = sanitize_executables(single.executables);
        target.insert(single);
        return Ok(1);
    }

    Err("Unrecognized profiles JSON format".into())
}

/// Collection of all registered game profiles and the default desktop state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilesStore {
    pub default_desktop: GameProfile,
    pub profiles: HashMap<String, GameProfile>, // Keyed by profile id
}

impl Default for ProfilesStore {
    fn default() -> Self {
        Self {
            default_desktop: GameProfile::default_desktop(),
            profiles: HashMap::new(),
        }
    }
}

impl ProfilesStore {
    pub fn insert(&mut self, profile: GameProfile) {
        self.profiles.insert(profile.id.clone(), profile);
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.profiles.remove(id).is_some()
    }

    pub fn get(&self, id: &str) -> Option<&GameProfile> {
        self.profiles.get(id)
    }

    pub fn find_by_executable(&self, binary_name: &str) -> Option<&GameProfile> {
        self.profiles.values().find(|p| p.matches(binary_name))
    }
}

/// Abstraction for storing and retrieving profiles (Dependency Inversion Principle).
pub trait ProfileRepository: Send + Sync {
    fn load(&self) -> Result<ProfilesStore, String>;
    fn save(&self, store: &ProfilesStore) -> Result<(), String>;
}

/// In-memory repository used for fast, deterministic unit testing.
pub struct InMemoryProfileRepository {
    store: Arc<Mutex<ProfilesStore>>,
}

impl InMemoryProfileRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(ProfilesStore::default())),
        }
    }
}

impl Default for InMemoryProfileRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileRepository for InMemoryProfileRepository {
    fn load(&self) -> Result<ProfilesStore, String> {
        Ok(self.store.lock().unwrap().clone())
    }

    fn save(&self, store: &ProfilesStore) -> Result<(), String> {
        *self.store.lock().unwrap() = store.clone();
        Ok(())
    }
}

/// Persistent JSON file repository stored in `%APPDATA%\FerrumPulse\profiles.json`.
pub struct JsonFileProfileRepository {
    file_path: PathBuf,
}

impl JsonFileProfileRepository {
    pub fn new() -> Self {
        let base = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        let path = base.join("FerrumPulse").join("profiles.json");
        Self { file_path: path }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { file_path: path }
    }
}

impl Default for JsonFileProfileRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileRepository for JsonFileProfileRepository {
    fn load(&self) -> Result<ProfilesStore, String> {
        if !self.file_path.exists() {
            return Ok(ProfilesStore::default());
        }

        let content = std::fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read profiles store: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Malformed profiles store format: {}", e))
    }

    fn save(&self, store: &ProfilesStore) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(store)
            .map_err(|e| format!("Failed to serialize profiles: {}", e))?;

        std::fs::write(&self.file_path, json)
            .map_err(|e| format!("Failed to write profiles to disk: {}", e))
    }
}

