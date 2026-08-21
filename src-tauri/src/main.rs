use localdock_core::path_guard::assert_under_roots;
use localdock_core::ports::{self, PortRow};
use localdock_core::registry::{AppEntry, Registry};
use localdock_core::scanner::{scan_root, ProposedApp};
use localdock_core::spawn::ProcessManager;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

type CommandResult<T> = Result<T, String>;

struct AppState {
    registry_path: PathBuf,
    registry: Mutex<Registry>,
    processes: Mutex<ProcessManager>,
}

#[derive(Debug, Clone, Serialize)]
struct AppView {
    id: String,
    name: String,
    cwd: String,
    command: String,
    args: Vec<String>,
    preferred_port: Option<u16>,
    force_loopback: bool,
    enabled: bool,
    running: bool,
    child_pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
struct RegistrySnapshot {
    registry_path: String,
    allowed_roots: Vec<String>,
    apps: Vec<AppView>,
}

#[derive(Debug, Clone, Serialize)]
struct ProposedAppView {
    name: String,
    cwd: String,
    command: String,
    args: Vec<String>,
    preferred_port: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
struct RegisterAppInput {
    name: String,
    cwd: String,
    command: String,
    args: Vec<String>,
    preferred_port: Option<u16>,
    force_loopback: Option<bool>,
}

#[tauri::command]
fn list_apps(state: State<'_, AppState>) -> CommandResult<RegistrySnapshot> {
    let registry = state
        .registry
        .lock()
        .map_err(|_| "registry mutex poisoned".to_string())?;
    let processes = state
        .processes
        .lock()
        .map_err(|_| "process manager mutex poisoned".to_string())?;
    Ok(snapshot(&state.registry_path, &registry, &processes))
}

#[tauri::command]
fn add_root(path: String, state: State<'_, AppState>) -> CommandResult<RegistrySnapshot> {
    let root = std::fs::canonicalize(Path::new(&path)).map_err(|err| err.to_string())?;
    if !root.is_dir() {
        return Err("root must be an existing directory".to_string());
    }

    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "registry mutex poisoned".to_string())?;
    if !registry
        .allowed_roots
        .iter()
        .any(|existing| existing == &root)
    {
        registry.allowed_roots.push(root);
    }
    persist_registry(&mut registry, &state.registry_path)?;

    let processes = state
        .processes
        .lock()
        .map_err(|_| "process manager mutex poisoned".to_string())?;
    Ok(snapshot(&state.registry_path, &registry, &processes))
}

#[tauri::command]
fn scan(root: String, state: State<'_, AppState>) -> CommandResult<Vec<ProposedAppView>> {
    let registry = state
        .registry
        .lock()
        .map_err(|_| "registry mutex poisoned".to_string())?;
    let root = assert_under_roots(Path::new(&root), &registry.allowed_roots)
        .map_err(|err| err.to_string())?;
    scan_root(&root, 4)
        .map(|apps| apps.into_iter().map(proposal_to_view).collect())
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn register_app(
    input: RegisterAppInput,
    state: State<'_, AppState>,
) -> CommandResult<RegistrySnapshot> {
    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "registry mutex poisoned".to_string())?;

    let mut entry = AppEntry::new(
        Uuid::new_v4().to_string(),
        input.name,
        PathBuf::from(input.cwd),
        input.command,
        input.args,
    );
    entry.preferred_port = input.preferred_port;
    entry.force_loopback = input.force_loopback.unwrap_or(true);
    entry.enabled = true;
    registry.add_app(entry).map_err(|err| err.to_string())?;
    persist_registry(&mut registry, &state.registry_path)?;

    let processes = state
        .processes
        .lock()
        .map_err(|_| "process manager mutex poisoned".to_string())?;
    Ok(snapshot(&state.registry_path, &registry, &processes))
}

#[tauri::command]
fn start_app(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let app = {
        let registry = state
            .registry
            .lock()
            .map_err(|_| "registry mutex poisoned".to_string())?;
        registry
            .get(&id)
            .cloned()
            .ok_or_else(|| format!("app not found: {id}"))?
    };
    if !app.enabled {
        return Err("app is disabled".to_string());
    }

    let processes = state
        .processes
        .lock()
        .map_err(|_| "process manager mutex poisoned".to_string())?;
    processes.start(&app).map_err(|err| err.to_string())
}

#[tauri::command]
fn stop_app(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let processes = state
        .processes
        .lock()
        .map_err(|_| "process manager mutex poisoned".to_string())?;
    processes.stop(&id).map_err(|err| err.to_string())
}

#[tauri::command]
fn list_ports(state: State<'_, AppState>) -> CommandResult<Vec<PortRow>> {
    let child_pids = {
        let processes = state
            .processes
            .lock()
            .map_err(|_| "process manager mutex poisoned".to_string())?;
        processes.running_pids().into_iter().collect::<HashSet<_>>()
    };

    ports::list_listeners(false)
        .map(|rows| {
            rows.into_iter()
                .filter(|row| row.is_loopback || child_pids.contains(&row.pid))
                .collect()
        })
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn kill_port(port: u16, pid: u32) -> CommandResult<()> {
    if pid == 0 {
        return Err("refusing to kill unknown pid".to_string());
    }

    let listener = ports::list_listeners(true)
        .map_err(|err| err.to_string())?
        .into_iter()
        .find(|row| row.port == port && row.pid == pid && row.is_loopback)
        .ok_or_else(|| "no matching loopback listener for port and pid".to_string())?;

    terminate_pid(listener.pid)
}

#[tauri::command]
fn open_loopback(url: String) -> CommandResult<()> {
    let url = validate_loopback_url(&url)?;
    open_url(&url)
}

/// Voluntary support / contact links (Discord, PayPal, Revolut, GitHub).
/// Not a license fee — allowlisted HTTPS only, opened in the system browser.
#[tauri::command]
fn open_support(kind: String) -> CommandResult<()> {
    let url = support_url(&kind)?;
    open_url(url)
}

fn support_url(kind: &str) -> CommandResult<&'static str> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "discord" => Ok("https://discord.com/users/406891052516114442"),
        "paypal" => Ok("https://www.paypal.com/paypalme/aurevo1"),
        "revolut" => Ok("https://revolut.me/mr_aurevo_x"),
        "github" => Ok("https://github.com/Mr-Aurevo-X"),
        _ => Err("unsupported support link".to_string()),
    }
}

fn config_path() -> CommandResult<PathBuf> {
    #[cfg(windows)]
    {
        let appdata =
            std::env::var_os("APPDATA").ok_or_else(|| "%APPDATA% is not set".to_string())?;
        return Ok(PathBuf::from(appdata).join("LocalDock").join("apps.json"));
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(config_home)
                .join("LocalDock")
                .join("apps.json"));
        }
        let home = std::env::var_os("HOME").ok_or_else(|| "$HOME is not set".to_string())?;
        return Ok(PathBuf::from(home)
            .join(".config")
            .join("LocalDock")
            .join("apps.json"));
    }

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let home = std::env::var_os("HOME").ok_or_else(|| "$HOME is not set".to_string())?;
        Ok(PathBuf::from(home)
            .join(".config")
            .join("LocalDock")
            .join("apps.json"))
    }
}

fn load_or_create_registry(path: &Path) -> CommandResult<Registry> {
    ensure_registry_parent(path)?;
    if path.is_file() {
        return Registry::load(path).map_err(|err| err.to_string());
    }

    let mut registry = Registry::default_empty();
    persist_registry(&mut registry, path)?;
    Ok(registry)
}

fn ensure_registry_parent(path: &Path) -> CommandResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| "registry path has no parent directory".to_string())?;
    std::fs::create_dir_all(parent).map_err(|err| err.to_string())
}

fn persist_registry(registry: &mut Registry, path: &Path) -> CommandResult<()> {
    ensure_registry_parent(path)?;
    registry.save(path).map_err(|err| err.to_string())?;
    restrict_registry_permissions(path)
}

fn restrict_registry_permissions(path: &Path) -> CommandResult<()> {
    #[cfg(unix)]
    {
        let permissions = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, permissions).map_err(|err| err.to_string())?;
    }

    #[cfg(not(unix))]
    {
        let _ = path;
    }

    Ok(())
}

fn snapshot(
    registry_path: &Path,
    registry: &Registry,
    processes: &ProcessManager,
) -> RegistrySnapshot {
    RegistrySnapshot {
        registry_path: registry_path.display().to_string(),
        allowed_roots: registry
            .allowed_roots
            .iter()
            .map(|root| root.display().to_string())
            .collect(),
        apps: registry
            .apps
            .iter()
            .map(|app| {
                let child_pid = processes.pid(&app.id);
                app_to_view(app, child_pid.is_some(), child_pid)
            })
            .collect(),
    }
}

fn app_to_view(app: &AppEntry, running: bool, child_pid: Option<u32>) -> AppView {
    AppView {
        id: app.id.clone(),
        name: app.name.clone(),
        cwd: app.cwd.display().to_string(),
        command: app.command.clone(),
        args: app.args.clone(),
        preferred_port: app.preferred_port,
        force_loopback: app.force_loopback,
        enabled: app.enabled,
        running,
        child_pid,
    }
}

fn proposal_to_view(app: ProposedApp) -> ProposedAppView {
    ProposedAppView {
        name: app.name,
        cwd: app.cwd.display().to_string(),
        command: app.command,
        args: app.args,
        preferred_port: app.preferred_port,
    }
}

fn validate_loopback_url(raw: &str) -> CommandResult<String> {
    let rest = if let Some(rest) = raw.strip_prefix("http://127.0.0.1:") {
        rest
    } else if let Some(rest) = raw.strip_prefix("http://localhost:") {
        rest
    } else {
        return Err("only http://127.0.0.1:* or http://localhost:* is allowed".to_string());
    };

    let port_end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    if port_end == 0 {
        return Err("loopback URL must include a numeric port".to_string());
    }

    let port = rest[..port_end]
        .parse::<u16>()
        .map_err(|_| "loopback URL port is invalid".to_string())?;
    if port == 0 {
        return Err("loopback URL port must be greater than zero".to_string());
    }

    let suffix = &rest[port_end..];
    if !suffix.is_empty()
        && !suffix.starts_with('/')
        && !suffix.starts_with('?')
        && !suffix.starts_with('#')
    {
        return Err("loopback URL contains an invalid port suffix".to_string());
    }

    Ok(raw.to_string())
}

#[cfg(windows)]
fn open_url(url: &str) -> CommandResult<()> {
    let status = Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .status()
        .map_err(|err| err.to_string())?;
    status_to_result(status, "open loopback URL")
}

#[cfg(target_os = "linux")]
fn open_url(url: &str) -> CommandResult<()> {
    let status = Command::new("xdg-open")
        .arg(url)
        .status()
        .map_err(|err| err.to_string())?;
    status_to_result(status, "open loopback URL")
}

#[cfg(not(any(windows, target_os = "linux")))]
fn open_url(url: &str) -> CommandResult<()> {
    let status = Command::new("open")
        .arg(url)
        .status()
        .map_err(|err| err.to_string())?;
    status_to_result(status, "open loopback URL")
}

#[cfg(windows)]
fn terminate_pid(pid: u32) -> CommandResult<()> {
    let pid_text = pid.to_string();
    let status = Command::new("taskkill.exe")
        .args(["/PID", pid_text.as_str(), "/T", "/F"])
        .status()
        .map_err(|err| err.to_string())?;
    status_to_result(status, "kill port process")
}

#[cfg(not(windows))]
fn terminate_pid(pid: u32) -> CommandResult<()> {
    let pid_text = pid.to_string();
    let status = Command::new("kill")
        .args(["-TERM", pid_text.as_str()])
        .status()
        .map_err(|err| err.to_string())?;
    status_to_result(status, "kill port process")
}

fn status_to_result(status: std::process::ExitStatus, action: &str) -> CommandResult<()> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("{action} failed with status {status}"))
    }
}

fn main() {
    let registry_path = config_path().expect("resolve LocalDock registry path");
    let registry = load_or_create_registry(&registry_path).expect("load LocalDock registry");

    tauri::Builder::default()
        .manage(AppState {
            registry_path,
            registry: Mutex::new(registry),
            processes: Mutex::new(ProcessManager::new()),
        })
        .invoke_handler(tauri::generate_handler![
            list_apps,
            add_root,
            scan,
            register_app,
            start_app,
            stop_app,
            list_ports,
            kill_port,
            open_loopback,
            open_support
        ])
        .run(tauri::generate_context!())
        .expect("run LocalDock Tauri application");
}
