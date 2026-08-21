use localdock_core::registry::AppEntry;
use localdock_core::spawn::{start_command, ProcessManager};
use localdock_core::LocalDockError;
use std::path::PathBuf;
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
