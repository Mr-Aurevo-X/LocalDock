mod github_latest;
mod history;
mod suite_settings;

use localdock_core::host_exec;
use localdock_core::path_guard::{assert_under_roots, display_path};
use localdock_core::ports::{self, AppHint, PortRow};
use localdock_core::registry::{AppEntry, Registry};
use localdock_core::scanner::{scan_root, ProposedApp};
use localdock_core::spawn::ProcessManager;
use localdock_core::win_paths;
use localdock_core::LocalDockError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "linux"))]
use std::process::Command;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

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
    tree_pids: Vec<u32>,
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

#[derive(Debug, Clone, Serialize)]
struct ImportWindowsResult {
    snapshot: RegistrySnapshot,
    source: String,
    roots_added: u32,
    apps_added: u32,
    skipped: u32,
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
    let root = resolve_root_dir(&path)?;
    if !root.is_dir() {
        return Err("root must be an existing directory".to_string());
    }

    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "registry mutex poisoned".to_string())?;
    let display = display_path(&root);
    if !registry
        .allowed_roots
        .iter()
        .any(|existing| existing == &root)
    {
        registry.allowed_roots.push(root);
    }
    persist_registry(&mut registry, &state.registry_path)?;
    let _ = history::append(&state.registry_path, "add_root", &display);

    let processes = state
        .processes
        .lock()
        .map_err(|_| "process manager mutex poisoned".to_string())?;
    Ok(snapshot(&state.registry_path, &registry, &processes))
}

#[tauri::command]
fn import_windows_locals(state: State<'_, AppState>) -> CommandResult<ImportWindowsResult> {
    let sources = win_paths::find_windows_localdock_registries();
    if sources.is_empty() {
        return Err("aucun registre LocalDock Windows trouvé sur un disque monté".to_string());
    }

    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "registry mutex poisoned".to_string())?;

    let mut roots_added = 0u32;
    let mut apps_added = 0u32;
    let mut skipped = 0u32;
    let mut sources_used = Vec::new();

    for source in sources {
        let Ok(data) = std::fs::read_to_string(&source) else {
            skipped += 1;
            continue;
        };
        let Ok(win) = Registry::deserialize_unchecked(&data) else {
            skipped += 1;
            continue;
        };
        let Some(system_root) = win_paths::windows_root_from_roaming_registry(&source) else {
            skipped += 1;
            continue;
        };

        for root in &win.allowed_roots {
            match remap_existing_dir(root, &system_root) {
                Some(mapped) => {
                    if !registry.allowed_roots.iter().any(|existing| existing == &mapped) {
                        registry.allowed_roots.push(mapped);
                        roots_added += 1;
                    }
                }
                None => skipped += 1,
            }
        }

        for mut app in win.apps {
            let Some(mapped) = remap_existing_dir(&app.cwd, &system_root) else {
                skipped += 1;
                continue;
            };
            if registry
                .apps
                .iter()
                .any(|existing| existing.id == app.id || existing.cwd == mapped)
            {
                skipped += 1;
                continue;
            }
            if !registry
                .allowed_roots
                .iter()
                .any(|root| mapped.starts_with(root))
            {
                registry.allowed_roots.push(mapped.clone());
                roots_added += 1;
            }
            app.cwd = mapped;
            match registry.add_app(app) {
                Ok(()) => apps_added += 1,
                Err(_) => skipped += 1,
            }
        }
        sources_used.push(source.display().to_string());
    }

    persist_registry(&mut registry, &state.registry_path)?;
    let source = sources_used.join("; ");
    let _ = history::append(
        &state.registry_path,
        "import_win",
        &format!("{source} roots={roots_added} apps={apps_added}"),
    );
    let processes = state
        .processes
        .lock()
        .map_err(|_| "process manager mutex poisoned".to_string())?;
    Ok(ImportWindowsResult {
        snapshot: snapshot(&state.registry_path, &registry, &processes),
        source,
        roots_added,
        apps_added,
        skipped,
    })
}

fn resolve_root_dir(path: &str) -> CommandResult<PathBuf> {
    win_paths::resolve_existing_dir(path).map_err(|err| err.to_string())
}

fn remap_existing_dir(raw: &Path, system_root: &Path) -> Option<PathBuf> {
    let text = raw.to_string_lossy();
    let mapped = if raw.is_dir() {
        raw.to_path_buf()
    } else {
        win_paths::remap_windows_path(&text, system_root)?
    };
    if !mapped.is_dir() {
        return None;
    }
    std::fs::canonicalize(mapped).ok()
}

#[tauri::command]
fn scan(root: String, state: State<'_, AppState>) -> CommandResult<Vec<ProposedAppView>> {
    let registry = state
        .registry
        .lock()
        .map_err(|_| "registry mutex poisoned".to_string())?;
    let root = assert_under_roots(Path::new(&root), &registry.allowed_roots)
        .map_err(|err| err.to_string())?;
    let shown = display_path(&root);
    let proposals = scan_root(&root, 4)
        .map(|apps| apps.into_iter().map(proposal_to_view).collect::<Vec<_>>())
        .map_err(|err| err.to_string())?;
    let _ = history::append(
        &state.registry_path,
        "scan",
        &format!("{} ({})", shown, proposals.len()),
    );
    Ok(proposals)
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
    let name = entry.name.clone();
    registry.add_app(entry).map_err(|err| err.to_string())?;
    persist_registry(&mut registry, &state.registry_path)?;
    let _ = history::append(&state.registry_path, "register", &name);

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
    processes.start(&app).map_err(|err| err.to_string())?;
    let _ = history::append(&state.registry_path, "start", &app.name);
    Ok(())
}

#[tauri::command]
fn stop_app(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let (preferred_port, name) = {
        let registry = state
            .registry
            .lock()
            .map_err(|_| "registry mutex poisoned".to_string())?;
        let app = registry
            .get(&id)
            .ok_or_else(|| format!("app not found: {id}"))?;
        (app.preferred_port, app.name.clone())
    };
    let stop_result = {
        let processes = state
            .processes
            .lock()
            .map_err(|_| "process manager mutex poisoned".to_string())?;
        processes.stop(&id)
    };
    match stop_result {
        Ok(()) => {
            let _ = history::append(&state.registry_path, "stop", &name);
            Ok(())
        }
        Err(LocalDockError::NotRunning) => {
            terminate_loopback_for_app(preferred_port)?;
            let _ = history::append(&state.registry_path, "stop", &name);
            Ok(())
        }
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
fn remove_app(id: String, state: State<'_, AppState>) -> CommandResult<RegistrySnapshot> {
    let name = {
        let registry = state
            .registry
            .lock()
            .map_err(|_| "registry mutex poisoned".to_string())?;
        registry
            .get(&id)
            .map(|app| app.name.clone())
            .ok_or_else(|| format!("app not found: {id}"))?
    };

    {
        let processes = state
            .processes
            .lock()
            .map_err(|_| "process manager mutex poisoned".to_string())?;
        let _ = processes.stop(&id);
    }

    let mut registry = state
        .registry
        .lock()
        .map_err(|_| "registry mutex poisoned".to_string())?;
    registry.remove_app(&id).map_err(|err| err.to_string())?;
    persist_registry(&mut registry, &state.registry_path)?;
    let _ = history::append(&state.registry_path, "unregister", &name);

    let processes = state
        .processes
        .lock()
        .map_err(|_| "process manager mutex poisoned".to_string())?;
    Ok(snapshot(&state.registry_path, &registry, &processes))
}

#[tauri::command]
fn list_ports(state: State<'_, AppState>) -> CommandResult<Vec<PortRow>> {
    let managed_pids = {
        let processes = state
            .processes
            .lock()
            .map_err(|_| "process manager mutex poisoned".to_string())?;
        processes
            .running_tree_pids()
            .into_iter()
            .collect::<HashSet<_>>()
    };

    let mut rows = ports::list_listeners(false)
        .map(|rows| ports::filter_display_rows(rows, &managed_pids))
        .map_err(|err| err.to_string())?;
    let hints = {
        let registry = state
            .registry
            .lock()
            .map_err(|_| "registry mutex poisoned".to_string())?;
        registry
            .apps
            .iter()
            .map(|app| AppHint {
                name: app.name.clone(),
                cwd: app.cwd.clone(),
                preferred_port: app.preferred_port,
            })
            .collect::<Vec<_>>()
    };
    ports::attach_app_labels(&mut rows, &hints);
    Ok(rows)
}

#[tauri::command]
fn kill_port(port: u16, pid: u32, state: State<'_, AppState>) -> CommandResult<()> {
    if pid == 0 {
        return Err("refusing to kill unknown pid".to_string());
    }

    let listener = ports::list_listeners(true)
        .map_err(|err| err.to_string())?
        .into_iter()
        .find(|row| row.port == port && row.pid == pid && row.is_loopback)
        .ok_or_else(|| "no matching loopback listener for port and pid".to_string())?;

    terminate_pid(listener.pid)?;
    let _ = history::append(
        &state.registry_path,
        "kill",
        &format!("{}:{} pid {}", listener.addr, listener.port, listener.pid),
    );
    Ok(())
}

fn terminate_loopback_for_app(port: Option<u16>) -> CommandResult<()> {
    let Some(port) = port else {
        return Err("not running".to_string());
    };
    let listener = ports::list_listeners(true)
        .map_err(|err| err.to_string())?
        .into_iter()
        .find(|row| row.port == port && row.is_loopback)
        .ok_or_else(|| format!("no loopback listener on port {port}"))?;
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

#[tauri::command]
fn get_suite_settings() -> CommandResult<suite_settings::SuiteSettings> {
    suite_settings::load()
}

#[tauri::command]
fn set_suite_language(lang: String) -> CommandResult<suite_settings::SuiteSettings> {
    suite_settings::set_language(&lang)
}

#[tauri::command]
fn set_check_github_updates(enabled: bool) -> CommandResult<suite_settings::SuiteSettings> {
    suite_settings::set_check_github_updates(enabled)
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(Debug, Clone, Serialize)]
struct AboutPath {
    id: String,
    path: String,
}

#[tauri::command]
fn get_about_local_paths(state: State<'_, AppState>) -> CommandResult<Vec<AboutPath>> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if !suite_settings::looks_like_clone_path(dir) {
                paths.push(AboutPath {
                    id: "app".into(),
                    path: dir.display().to_string(),
                });
            }
        }
    }
    paths.push(AboutPath {
        id: "registry".into(),
        path: display_path(&state.registry_path),
    });
    if let Some(dir) = state.registry_path.parent() {
        paths.push(AboutPath {
            id: "history".into(),
            path: display_path(&dir.join("history.json")),
        });
    }
    paths.push(AboutPath {
        id: "settings".into(),
        path: suite_settings::settings_path()?.display().to_string(),
    });
    Ok(paths)
}

#[tauri::command]
fn check_github_latest() -> CommandResult<github_latest::LatestCheck> {
    let settings = suite_settings::load()?;
    github_latest::check_latest(env!("CARGO_PKG_VERSION"), settings.check_github_updates)
}

#[tauri::command]
fn open_release(url: Option<String>) -> CommandResult<()> {
    let raw = url.unwrap_or_else(|| github_latest::RELEASES_PAGE.to_string());
    let url = github_latest::allowlisted_release_url(&raw)?;
    open_url(&url)
}

#[tauri::command]
fn pick_folder() -> CommandResult<Option<String>> {
    #[cfg(windows)]
    {
        let picked = rfd::FileDialog::new()
            .set_title("LocalDock")
            .pick_folder();
        return Ok(picked.map(|path| display_path(&path)));
    }

    #[cfg(target_os = "linux")]
    {
        return pick_folder_linux();
    }

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let picked = rfd::FileDialog::new()
            .set_title("LocalDock")
            .pick_folder();
        Ok(picked.map(|path| display_path(&path)))
    }
}

/// Sync XDG-portal dialogs (`rfd` + pollster) panic inside Tauri's runtime
/// and kill the process. KDE/GNOME dialogs run as a host subprocess instead.
#[cfg(target_os = "linux")]
fn pick_folder_linux() -> CommandResult<Option<String>> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    let attempts: [(&str, Vec<String>); 2] = [
        (
            "kdialog",
            vec!["--getexistingdirectory".into(), home.clone()],
        ),
        (
            "zenity",
            vec![
                "--file-selection".into(),
                "--directory".into(),
                "--title=LocalDock".into(),
                format!("--filename={home}/"),
            ],
        ),
    ];

    for (bin, args) in attempts {
        let output = match host_exec::host_command(bin, &args, None, &[]).output() {
            Ok(output) => output,
            Err(_) => continue,
        };
        if !output.status.success() {
            return Ok(None);
        }
        return Ok(folder_path_from_dialog_stdout(&output.stdout));
    }
    Err("aucun sélecteur de dossier (installe kdialog ou zenity)".into())
}

fn folder_path_from_dialog_stdout(stdout: &[u8]) -> Option<String> {
    let path = String::from_utf8_lossy(stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

#[tauri::command]
fn list_history(state: State<'_, AppState>) -> CommandResult<Vec<history::HistoryEvent>> {
    history::list(&state.registry_path)
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
            .map(|root| display_path(root))
            .collect(),
        apps: registry
            .apps
            .iter()
            .map(|app| {
                let child_pid = processes.pid(&app.id);
                let tree_pids = processes.tree_pids(&app.id);
                app_to_view(app, child_pid.is_some(), child_pid, tree_pids)
            })
            .collect(),
    }
}

fn app_to_view(
    app: &AppEntry,
    running: bool,
    child_pid: Option<u32>,
    tree_pids: Vec<u32>,
) -> AppView {
    AppView {
        id: app.id.clone(),
        name: app.name.clone(),
        cwd: display_path(&app.cwd),
        command: app.command.clone(),
        args: app.args.clone(),
        preferred_port: app.preferred_port,
        force_loopback: app.force_loopback,
        enabled: app.enabled,
        running,
        child_pid,
        tree_pids,
    }
}

fn proposal_to_view(app: ProposedApp) -> ProposedAppView {
    ProposedAppView {
        name: app.name,
        cwd: display_path(&app.cwd),
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
    let status = host_exec::host_command("xdg-open", &[url.to_string()], None, &[])
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
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    let taskkill = PathBuf::from(system_root)
        .join("System32")
        .join("taskkill.exe");
    let mut cmd = Command::new(taskkill);
    cmd.args(["/PID", pid_text.as_str(), "/T", "/F"]);
    cmd.creation_flags(CREATE_NO_WINDOW);
    let status = cmd
        .status()
        .map_err(|err| format!("kill port process: {err}"))?;
    status_to_result(status, "kill port process")
}

#[cfg(not(windows))]
fn terminate_pid(pid: u32) -> CommandResult<()> {
    let status = host_exec::host_command("kill", &["-TERM".into(), pid.to_string()], None, &[])
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
            import_windows_locals,
            scan,
            register_app,
            remove_app,
            start_app,
            stop_app,
            list_ports,
            kill_port,
            open_loopback,
            open_support,
            get_suite_settings,
            set_suite_language,
            set_check_github_updates,
            get_app_version,
            get_about_local_paths,
            check_github_latest,
            open_release,
            pick_folder,
            list_history,
        ])
        .run(tauri::generate_context!())
        .expect("run LocalDock Tauri application");
}

#[cfg(test)]
mod pick_folder_tests {
    use super::folder_path_from_dialog_stdout;

    #[test]
    fn empty_stdout_is_none() {
        assert_eq!(folder_path_from_dialog_stdout(b""), None);
        assert_eq!(folder_path_from_dialog_stdout(b"  \n"), None);
    }

    #[test]
    fn trims_folder_path() {
        assert_eq!(
            folder_path_from_dialog_stdout(b"/home/mraurevox/Documents\n"),
            Some("/home/mraurevox/Documents".into())
        );
    }
}
