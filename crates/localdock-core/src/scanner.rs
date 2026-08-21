use crate::LocalDockError;
use serde_json::Value;
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &["node_modules", ".git", "dist", "target", "venv"];

const VITE_CONFIG_NAMES: &[&str] = &[
    "vite.config.js",
    "vite.config.ts",
    "vite.config.mjs",
    "vite.config.cjs",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedApp {
    pub name: String,
    pub cwd: PathBuf,
    pub command: String,
    pub args: Vec<String>,
    pub preferred_port: Option<u16>,
}

pub fn scan_root(root: &Path, max_depth: u32) -> Result<Vec<ProposedApp>, LocalDockError> {
    let mut proposals = Vec::new();
    scan_dir(root, 0, max_depth, &mut proposals)?;
    Ok(proposals)
}

fn scan_dir(
    dir: &Path,
    depth: u32,
    max_depth: u32,
    proposals: &mut Vec<ProposedApp>,
) -> Result<(), LocalDockError> {
    if depth > max_depth {
        return Ok(());
    }

    if let Some(proposal) = detect_in_dir(dir)? {
        proposals.push(proposal);
    }

    if depth == max_depth {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if SKIP_DIRS.iter().any(|skip| name_str == *skip) {
            continue;
        }
        scan_dir(&entry.path(), depth + 1, max_depth, proposals)?;
    }

    Ok(())
}

fn detect_in_dir(dir: &Path) -> Result<Option<ProposedApp>, LocalDockError> {
    if dir.join("manage.py").is_file() {
        return Ok(Some(ProposedApp {
            name: dir_name(dir),
            cwd: dir.to_path_buf(),
            command: "python".into(),
            args: vec![
                "manage.py".into(),
                "runserver".into(),
                "127.0.0.1:8000".into(),
            ],
            preferred_port: Some(8000),
        }));
    }

    let package_json_path = dir.join("package.json");
    if package_json_path.is_file() {
        return detect_npm_dev(dir, &package_json_path);
    }

    Ok(None)
}

fn detect_npm_dev(dir: &Path, package_json_path: &Path) -> Result<Option<ProposedApp>, LocalDockError> {
    let content = std::fs::read_to_string(package_json_path)?;
    let json: Value = serde_json::from_str(&content)?;

    let has_dev = json
        .get("scripts")
        .and_then(|scripts| scripts.get("dev"))
        .and_then(Value::as_str)
        .is_some();

    if !has_dev {
        return Ok(None);
    }

    let name = json
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| dir_name(dir));

    let preferred_port = guess_vite_port(dir).or_else(|| guess_env_port(dir));

    Ok(Some(ProposedApp {
        name,
        cwd: dir.to_path_buf(),
        command: "npm".into(),
        args: vec!["run".into(), "dev".into()],
        preferred_port,
    }))
}

fn guess_vite_port(dir: &Path) -> Option<u16> {
    for name in VITE_CONFIG_NAMES {
        let path = dir.join(name);
        if !path.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        if let Some(port) = parse_port_from_vite_config(&content) {
            return Some(port);
        }
    }
    None
}

fn parse_port_from_vite_config(content: &str) -> Option<u16> {
    for line in content.lines() {
        let Some(port_idx) = line.find("port:") else {
            continue;
        };
        let after = &line[port_idx + 5..];
        let digits: String = after
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(port) = digits.parse::<u16>() {
            if port > 0 {
                return Some(port);
            }
        }
    }
    None
}

fn guess_env_port(dir: &Path) -> Option<u16> {
    let env_path = dir.join(".env");
    if !env_path.is_file() {
        return None;
    }
    let content = std::fs::read_to_string(&env_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(rest) = line.strip_prefix("PORT=") else {
            continue;
        };
        let val = rest.trim().trim_matches('"').trim_matches('\'');
        if let Ok(port) = val.parse::<u16>() {
            if port > 0 {
                return Some(port);
            }
        }
    }
    None
}

fn dir_name(dir: &Path) -> String {
    dir.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "app".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ld-scan-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parse_vite_port_from_config_line() {
        let content = "export default {\n  server: { port: 4173 },\n}\n";
        assert_eq!(parse_port_from_vite_config(content), Some(4173));
    }

    #[test]
    fn guess_env_port_reads_dotenv() {
        let dir = temp_dir();
        fs::write(dir.join(".env"), "PORT=9001\n").unwrap();
        assert_eq!(guess_env_port(&dir), Some(9001));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_listed_directories() {
        assert!(SKIP_DIRS.contains(&"node_modules"));
    }
}
