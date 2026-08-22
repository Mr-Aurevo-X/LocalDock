use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const SETTINGS_DIR: &str = "Mr-Aurevo-X";
const SETTINGS_FILE: &str = "user-settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiteSettings {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_check_github_updates")]
    pub check_github_updates: bool,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn default_language() -> String {
    "fr".into()
}

fn default_check_github_updates() -> bool {
    true
}

impl Default for SuiteSettings {
    fn default() -> Self {
        Self {
            language: default_language(),
            check_github_updates: default_check_github_updates(),
            extra: serde_json::Map::new(),
        }
    }
}

pub fn settings_path() -> Result<PathBuf, String> {
    Ok(settings_dir()?.join(SETTINGS_FILE))
}

pub fn settings_dir() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let local = std::env::var_os("LOCALAPPDATA")
            .ok_or_else(|| "%LOCALAPPDATA% is not set".to_string())?;
        Ok(PathBuf::from(local).join(SETTINGS_DIR))
    }

    #[cfg(not(windows))]
    {
        if let Some(home) = std::env::var_os("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(home).join(SETTINGS_DIR));
        }
        let home = std::env::var_os("HOME").ok_or_else(|| "$HOME is not set".to_string())?;
        Ok(PathBuf::from(home).join(".config").join(SETTINGS_DIR))
    }
}

pub fn load() -> Result<SuiteSettings, String> {
    let path = settings_path()?;
    if !path.is_file() {
        return Ok(SuiteSettings::default());
    }
    let raw = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    serde_json::from_str(&raw).map_err(|err| err.to_string())
}

pub fn save(settings: &SuiteSettings) -> Result<(), String> {
    let dir = settings_dir()?;
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    let path = dir.join(SETTINGS_FILE);
    let json = serde_json::to_string_pretty(settings).map_err(|err| err.to_string())?;
    fs::write(&path, json).map_err(|err| err.to_string())
}

pub fn set_language(lang: &str) -> Result<SuiteSettings, String> {
    let language = match lang.trim().to_ascii_lowercase().as_str() {
        "en" => "en",
        _ => "fr",
    };
    let mut settings = load()?;
    settings.language = language.into();
    save(&settings)?;
    Ok(settings)
}

pub fn set_check_github_updates(enabled: bool) -> Result<SuiteSettings, String> {
    let mut settings = load()?;
    settings.check_github_updates = enabled;
    save(&settings)?;
    Ok(settings)
}

pub fn looks_like_clone_path(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.contains("Dev Central Tree")
        || text.contains("Dev Tree Linux")
        || text.contains("target\\debug")
        || text.contains("target/debug")
        || text.contains("target\\release")
        || text.contains("target/release")
}
