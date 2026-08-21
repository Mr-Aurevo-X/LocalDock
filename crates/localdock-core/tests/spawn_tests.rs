use localdock_core::registry::AppEntry;
use localdock_core::spawn::{start_command, ProcessManager};
use localdock_core::LocalDockError;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[cfg(unix)]
fn sleeper_command() -> (String, Vec<String>) {
    ("/bin/sleep".into(), vec!["5".into()])
}

#[cfg(windows)]
fn sleeper_command() -> (String, Vec<String>) {
    (
        "ping".into(),
        vec!["-n".into(), "6".into(), "127.0.0.1".into()],
    )
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup_dir(dir: &PathBuf) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn start_command_spawns_process_with_argv() {
    let cwd = temp_test_dir("ld-spawn-primitive");
    let (program, args) = sleeper_command();
    let mut child = start_command(
        &program,
        &args,
        &cwd,
        &[("LOCALDOCK_TEST_ENV".into(), "1".into())],
    )
    .unwrap();

    assert!(child.try_wait().unwrap().is_none());

    let _ = child.kill();
    let _ = child.wait();
    cleanup_dir(&cwd);
}

#[test]
fn process_manager_tracks_and_stops_app() {
    let cwd = temp_test_dir("ld-spawn-manager");
    let (command, args) = sleeper_command();
    let app_id = Uuid::new_v4().to_string();
    let app = AppEntry {
        id: app_id.clone(),
        name: "sleepy".into(),
        cwd: cwd.clone(),
        command,
        args,
        preferred_port: None,
        force_loopback: false,
        enabled: true,
    };
    let manager = ProcessManager::default();

    manager.start(&app).unwrap();
    assert!(manager.is_running(&app_id));
    assert_eq!(manager.running_ids(), vec![app_id.clone()]);
    let pid = manager.pid(&app_id).expect("running app should have a pid");
    assert!(pid > 0);
    assert_eq!(manager.running_pids(), vec![pid]);

    manager.stop(&app_id).unwrap();
    assert!(!manager.is_running(&app_id));
    assert!(manager.running_ids().is_empty());
    cleanup_dir(&cwd);
}

#[test]
fn process_manager_rejects_duplicate_running_app() {
    let cwd = temp_test_dir("ld-spawn-duplicate");
    let (command, args) = sleeper_command();
    let app = AppEntry {
        id: Uuid::new_v4().to_string(),
        name: "sleepy".into(),
        cwd: cwd.clone(),
        command,
        args,
        preferred_port: None,
        force_loopback: false,
        enabled: true,
    };
    let manager = ProcessManager::default();

    manager.start(&app).unwrap();
    let err = manager.start(&app).unwrap_err();
    assert!(matches!(err, LocalDockError::AlreadyRunning));

    manager.stop(&app.id).unwrap();
    cleanup_dir(&cwd);
}

#[test]
fn process_manager_rejects_shell_interpreters_before_spawn() {
    let cwd = temp_test_dir("ld-spawn-shell-reject");
    let manager = ProcessManager::default();

    for cmd in [
        "cmd",
        "cmd.exe",
        "powershell",
        "powershell.exe",
        "pwsh",
        "sh",
        "bash",
        "zsh",
        "fish",
        "C:\\Windows\\System32\\cmd.exe",
        "/usr/bin/bash",
        "PwSh",
    ] {
        let app = AppEntry {
            id: Uuid::new_v4().to_string(),
            name: "shell".into(),
            cwd: cwd.clone(),
            command: cmd.into(),
            args: vec![],
            preferred_port: None,
            force_loopback: false,
            enabled: true,
        };
        let err = manager.start(&app).unwrap_err();
        assert!(matches!(err, LocalDockError::InvalidCommand), "cmd={cmd:?}");
        assert!(!manager.is_running(&app.id));
    }

    cleanup_dir(&cwd);
}

#[test]
fn start_applies_loopback_env_before_spawn() {
    let cwd = temp_test_dir("ld-spawn-loopback");
    std::fs::write(cwd.join("env-probe.marker"), "").unwrap();
    let probe_exe = cwd.join("env-probe-child.exe");
    std::fs::copy(std::env::current_exe().unwrap(), &probe_exe).unwrap();
    let app_id = Uuid::new_v4().to_string();
    let app = AppEntry {
        id: app_id.clone(),
        name: "probe".into(),
        cwd: cwd.clone(),
        command: probe_exe.display().to_string(),
        args: vec![
            "--ignored".into(),
            "--exact".into(),
            "localdock_env_probe_child".into(),
        ],
        preferred_port: Some(49231),
        force_loopback: true,
        enabled: true,
    };
    let manager = ProcessManager::default();

    manager.start(&app).unwrap();

    let out_path = cwd.join("env-probe.txt");
    let deadline = Instant::now() + Duration::from_secs(5);
    while !out_path.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }

    let observed = std::fs::read_to_string(&out_path).unwrap();
    assert!(observed.contains("HOST=127.0.0.1"));
    assert!(observed.contains("PORT=49231"));

    manager.stop(&app_id).unwrap();
    cleanup_dir(&cwd);
}

#[test]
#[ignore]
fn localdock_env_probe_child() {
    if !std::path::Path::new("env-probe.marker").exists() {
        return;
    }

    let host = std::env::var("HOST").unwrap_or_default();
    let port = std::env::var("PORT").unwrap_or_default();
    std::fs::write("env-probe.txt", format!("HOST={host}\nPORT={port}\n")).unwrap();
    std::thread::sleep(Duration::from_secs(30));
}

#[test]
fn start_command_does_not_block_verbose_child() {
    let cwd = temp_test_dir("ld-spawn-verbose");
    std::fs::write(cwd.join("verbose-probe.marker"), "").unwrap();
    let probe_exe = cwd.join("verbose-probe-child.exe");
    std::fs::copy(std::env::current_exe().unwrap(), &probe_exe).unwrap();

    let mut child = start_command(
        &probe_exe.display().to_string(),
        &[
            "--ignored".into(),
            "--exact".into(),
            "localdock_verbose_probe_child".into(),
        ],
        &cwd,
        &[],
    )
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
    }
    let status = child.wait().expect("wait for verbose child");
    cleanup_dir(&cwd);

    assert!(
        status.success(),
        "verbose child should exit instead of blocking on a full pipe: {status}"
    );
}

#[test]
#[ignore]
fn localdock_verbose_probe_child() {
    if !std::path::Path::new("verbose-probe.marker").exists() {
        return;
    }

    use std::io::Write;
    let chunk = vec![b'x'; 64 * 1024];
    for _ in 0..16 {
        let _ = std::io::stdout().write_all(&chunk);
        let _ = std::io::stderr().write_all(&chunk);
    }
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
}

#[test]
fn running_tree_pids_includes_spawned_descendants() {
    let cwd = temp_test_dir("ld-spawn-tree");
    std::fs::write(cwd.join("tree-parent.marker"), "").unwrap();
    let probe_exe = cwd.join("tree-probe-child.exe");
    std::fs::copy(std::env::current_exe().unwrap(), &probe_exe).unwrap();
    let app_id = Uuid::new_v4().to_string();
    let app = AppEntry {
        id: app_id.clone(),
        name: "tree-probe".into(),
        cwd: cwd.clone(),
        command: probe_exe.display().to_string(),
        args: vec![
            "--ignored".into(),
            "--exact".into(),
            "localdock_tree_probe_parent".into(),
        ],
        preferred_port: None,
        force_loopback: false,
        enabled: true,
    };
    let manager = ProcessManager::default();
    manager.start(&app).unwrap();

    let leaf_path = cwd.join("tree-leaf.pid");
    let deadline = Instant::now() + Duration::from_secs(8);
    while !leaf_path.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }

    let leaf_pid = std::fs::read_to_string(&leaf_path)
        .unwrap_or_default()
        .trim()
        .parse::<u32>()
        .unwrap_or(0);
    let root = manager
        .pid(&app_id)
        .expect("parent should still be running");
    let tree = manager.running_tree_pids();

    let _ = manager.stop(&app_id);
    if leaf_pid > 0 {
        kill_pid(leaf_pid);
    }
    cleanup_dir(&cwd);

    assert!(
        leaf_pid > 0,
        "tree parent should have recorded a descendant pid"
    );
    assert_ne!(leaf_pid, root);
    assert!(
        tree.contains(&leaf_pid),
        "expected descendant {leaf_pid} in tree {tree:?} (root {root})"
    );
}

#[test]
#[ignore]
fn localdock_tree_probe_parent() {
    if !std::path::Path::new("tree-parent.marker").exists() {
        return;
    }

    let exe = std::env::current_exe().unwrap();
    let mut child = Command::new(exe);
    child
        .args(["--ignored", "--exact", "localdock_tree_probe_leaf"])
        .current_dir(std::env::current_dir().unwrap())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        child.creation_flags(0x0800_0000);
    }

    let mut spawned = child.spawn().expect("spawn tree leaf");
    std::fs::write("tree-leaf.pid", spawned.id().to_string()).unwrap();
    std::thread::sleep(Duration::from_secs(30));
    let _ = spawned.kill();
    let _ = spawned.wait();
}

#[test]
#[ignore]
fn localdock_tree_probe_leaf() {
    if !std::path::Path::new("tree-parent.marker").exists() {
        return;
    }
    std::thread::sleep(Duration::from_secs(30));
}

fn kill_pid(pid: u32) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill.exe")
            .args(["/PID", &pid.to_string(), "/F"])
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
    }
}

#[cfg(windows)]
#[test]
fn resolve_program_finds_cmd_shim_on_path() {
    use localdock_core::spawn::resolve_program;

    let dir = temp_test_dir("ld-resolve");
    let shim = dir.join("ldshim.cmd");
    std::fs::write(&shim, "@echo off\r\nexit /b 0\r\n").unwrap();

    let old_path = std::env::var_os("PATH");
    let mut path = dir.display().to_string();
    if let Some(old) = &old_path {
        path.push(';');
        path.push_str(&old.to_string_lossy());
    }
    std::env::set_var("PATH", &path);
    let resolved = resolve_program("ldshim");
    match old_path {
        Some(old) => std::env::set_var("PATH", old),
        None => std::env::remove_var("PATH"),
    }

    assert_eq!(resolved, shim);
    cleanup_dir(&dir);
}
