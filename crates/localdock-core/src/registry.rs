use crate::path_guard::assert_under_roots;
use crate::LocalDockError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const REGISTRY_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub cwd: PathBuf,
    pub command: String,
    pub args: Vec<String>,
    pub preferred_port: Option<u16>,
    // Default to loopback binding so newly deserialized apps do not expose dev servers.
    #[serde(default = "default_force_loopback")]
    pub force_loopback: bool,
    pub enabled: bool,
}

impl AppEntry {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        cwd: PathBuf,
        command: impl Into<String>,
        args: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            cwd,
            command: command.into(),
            args,
            ..Self::default()
        }
    }
}

impl Default for AppEntry {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            cwd: PathBuf::new(),
            command: String::new(),
            args: Vec::new(),
            preferred_port: None,
            force_loopback: default_force_loopback(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub allowed_roots: Vec<PathBuf>,
    pub apps: Vec<AppEntry>,
}

impl Registry {
    pub fn default_empty() -> Self {
        Self {
            version: REGISTRY_VERSION,
            allowed_roots: Vec::new(),
            apps: Vec::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, LocalDockError> {
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str(&data).map_err(LocalDockError::from)
    }

    pub fn save(&mut self, path: &Path) -> Result<(), LocalDockError> {
        self.version = REGISTRY_VERSION;
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn add_app(&mut self, mut entry: AppEntry) -> Result<(), LocalDockError> {
        validate_command(&entry.command)?;
        let canon = assert_under_roots(&entry.cwd, &self.allowed_roots)?;
        entry.cwd = canon;
        self.apps.push(entry);
        Ok(())
    }

    pub fn remove_app(&mut self, id: &str) -> Result<(), LocalDockError> {
        let pos = self
            .apps
            .iter()
            .position(|app| app.id == id)
            .ok_or_else(|| LocalDockError::AppNotFound(id.to_string()))?;
        self.apps.remove(pos);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&AppEntry> {
        self.apps.iter().find(|app| app.id == id)
    }
}

pub(crate) fn validate_command(command: &str) -> Result<(), LocalDockError> {
    if command.is_empty()
        || command.contains(' ')
        || command.contains('&')
        || command.contains('|')
        || command.contains(';')
        || is_shell_interpreter(command)
    {
        return Err(LocalDockError::InvalidCommand);
    }
    Ok(())
}

fn default_force_loopback() -> bool {
    true
}

fn is_shell_interpreter(command: &str) -> bool {
    let basename = command
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(command)
        .to_ascii_lowercase();

    matches!(
        basename.as_str(),
        "cmd"
            | "cmd.exe"
            | "powershell"
            | "powershell.exe"
            | "pwsh"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
    )
}
