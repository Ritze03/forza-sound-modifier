use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Deserializes a `HashMap<String, String>` while silently dropping entries whose
/// value is JSON `null` (the Python app writes null for disabled override fields).
fn deser_string_map<'de, D>(de: D) -> Result<HashMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<String, Option<String>> = HashMap::deserialize(de)?;
    Ok(raw.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect())
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ProfileData {
    #[serde(default)]
    pub car_overrides: HashMap<String, HashMap<String, String>>,
    #[serde(default, deserialize_with = "deser_string_map")]
    pub global_overrides: HashMap<String, String>,
    #[serde(default)]
    pub synth_overrides: HashMap<String, HashMap<String, HashMap<String, String>>>,
    #[serde(default, deserialize_with = "deser_string_map")]
    pub misc_overrides: HashMap<String, String>,
    #[serde(default)]
    pub reverb_overrides: HashMap<String, f64>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub created: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ConfigData {
    #[serde(default = "default_game_path")]
    pub game_path: String,
    #[serde(default, deserialize_with = "deser_string_map")]
    pub global_overrides: HashMap<String, String>,
    #[serde(default)]
    pub car_overrides: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub synth_overrides: HashMap<String, HashMap<String, HashMap<String, String>>>,
    #[serde(default, deserialize_with = "deser_string_map")]
    pub misc_overrides: HashMap<String, String>,
    #[serde(default)]
    pub reverb_overrides: HashMap<String, f64>,
    #[serde(default)]
    pub active_profile: String,
}

fn default_game_path() -> String {
    String::new()
}

pub struct Config {
    pub data: ConfigData,
    pub profiles: HashMap<String, ProfileData>,
    config_dir: PathBuf,
    profiles_dir: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let config_dir = config_base_dir();
        std::fs::create_dir_all(&config_dir).ok();
        let profiles_dir = config_dir.join("profiles");
        std::fs::create_dir_all(&profiles_dir).ok();

        let config_file = config_dir.join("config.json");
        let data = Self::load_config_file(&config_file);
        let profiles = Self::load_profiles_from_dir(&profiles_dir);

        let mut cfg = Config {
            data,
            profiles,
            config_dir,
            profiles_dir,
        };

        if cfg.profiles.is_empty() {
            cfg.create_default_profile();
        }
        if cfg.data.active_profile.is_empty() {
            cfg.data.active_profile = cfg.profiles.keys().next().cloned().unwrap_or_default();
        }

        cfg
    }

    fn load_config_file(path: &PathBuf) -> ConfigData {
        if path.exists() {
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(d) = serde_json::from_str::<ConfigData>(&text) {
                    return d;
                }
            }
        }
        ConfigData::default()
    }

    fn load_profiles_from_dir(dir: &PathBuf) -> HashMap<String, ProfileData> {
        let mut profiles = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(text) = std::fs::read_to_string(&path) {
                            if let Ok(p) = serde_json::from_str::<ProfileData>(&text) {
                                profiles.insert(stem.to_string(), p);
                            }
                        }
                    }
                }
            }
        }
        profiles
    }

    fn create_default_profile(&mut self) {
        let profile = ProfileData {
            description: "Default profile".to_string(),
            created: now_iso(),
            ..Default::default()
        };
        self.save_profile("Default", &profile);
        self.profiles.insert("Default".to_string(), profile);
        self.data.active_profile = "Default".to_string();
        self.save();
    }

    pub fn save(&self) {
        let path = self.config_dir.join("config.json");
        if let Ok(text) = serde_json::to_string_pretty(&self.data) {
            std::fs::write(path, text).ok();
        }
    }

    pub fn save_profile(&mut self, name: &str, profile: &ProfileData) {
        let path = self.profiles_dir.join(format!("{}.json", name));
        if let Ok(text) = serde_json::to_string_pretty(profile) {
            std::fs::write(path, text).ok();
        }
        self.profiles.insert(name.to_string(), profile.clone());
    }

    pub fn delete_profile(&mut self, name: &str) {
        let path = self.profiles_dir.join(format!("{}.json", name));
        std::fs::remove_file(path).ok();
        self.profiles.remove(name);
    }

    pub fn snapshot(&self) -> ProfileData {
        ProfileData {
            car_overrides: self.data.car_overrides.clone(),
            global_overrides: self.data.global_overrides.clone(),
            synth_overrides: self.data.synth_overrides.clone(),
            misc_overrides: self.data.misc_overrides.clone(),
            reverb_overrides: self.data.reverb_overrides.clone(),
            description: String::new(),
            created: now_iso(),
        }
    }

    pub fn apply_snapshot(&mut self, snapshot: &ProfileData) {
        self.data.car_overrides = snapshot.car_overrides.clone();
        self.data.global_overrides = snapshot.global_overrides.clone();
        self.data.synth_overrides = snapshot.synth_overrides.clone();
        self.data.misc_overrides = snapshot.misc_overrides.clone();
        self.data.reverb_overrides = snapshot.reverb_overrides.clone();
        self.save();
    }
}

fn config_base_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ForzaHorizon6SoundMod")
}

pub fn now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}
