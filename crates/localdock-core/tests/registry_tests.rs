use localdock_core::registry::{AppEntry, Registry};
use localdock_core::LocalDockError;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ld-reg-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup_dir(dir: &PathBuf) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn registry_round_trip() {
    let dir = temp_test_dir();
    let path = dir.join("apps.json");
    let mut reg = Registry::default_empty();
    reg.allowed_roots.push(dir.clone());
    reg.save(&path).unwrap();
    let loaded = Registry::load(&path).unwrap();
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.allowed_roots.len(), 1);
    cleanup_dir(&dir);
}

#[test]
fn add_app_validates_cwd_under_roots() {
    let root = temp_test_dir();
    let child = root.join("proj");
    std::fs::create_dir_all(&child).unwrap();
    let other = std::env::temp_dir().join(format!("ld-reg-other-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&other).unwrap();

    let mut reg = Registry::default_empty();
    reg.allowed_roots.push(root.clone());

    let entry = AppEntry {
        id: Uuid::new_v4().to_string(),
        name: "test".into(),
        cwd: child.clone(),
        command: "npm".into(),
        args: vec!["run".into(), "dev".into()],
        preferred_port: Some(5173),
        force_loopback: true,
        enabled: true,
    };
    reg.add_app(entry).unwrap();
    assert_eq!(reg.apps.len(), 1);

    let bad = AppEntry {
        id: Uuid::new_v4().to_string(),
        name: "bad".into(),
        cwd: other.clone(),
        command: "npm".into(),
        args: vec![],
        preferred_port: None,
        force_loopback: true,
        enabled: true,
    };
    let err = reg.add_app(bad).unwrap_err();
    assert!(matches!(err, LocalDockError::PathNotAllowed(_)));

    cleanup_dir(&root);
    cleanup_dir(&other);
}

#[test]
fn add_app_rejects_invalid_command() {
    let root = temp_test_dir();
    let child = root.join("proj");
    std::fs::create_dir_all(&child).unwrap();

    let mut reg = Registry::default_empty();
    reg.allowed_roots.push(root.clone());

    for cmd in ["", "npm run", "cmd&", "a|b", "a;b"] {
        let entry = AppEntry {
            id: Uuid::new_v4().to_string(),
            name: "x".into(),
            cwd: child.clone(),
            command: cmd.into(),
            args: vec![],
            preferred_port: None,
            force_loopback: true,
            enabled: true,
        };
        let err = reg.add_app(entry).unwrap_err();
        assert!(matches!(err, LocalDockError::InvalidCommand), "cmd={cmd:?}");
    }

    cleanup_dir(&root);
}

#[test]
fn add_app_rejects_shell_interpreters() {
    let root = temp_test_dir();
    let child = root.join("proj");
    std::fs::create_dir_all(&child).unwrap();

    let mut reg = Registry::default_empty();
    reg.allowed_roots.push(root.clone());

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
        "PoWeRsHeLl.ExE",
    ] {
        let entry = AppEntry {
            id: Uuid::new_v4().to_string(),
            name: "shell".into(),
            cwd: child.clone(),
            command: cmd.into(),
            args: vec![],
            preferred_port: None,
            force_loopback: true,
            enabled: true,
        };
        let err = reg.add_app(entry).unwrap_err();
        assert!(matches!(err, LocalDockError::InvalidCommand), "cmd={cmd:?}");
    }

    cleanup_dir(&root);
}

#[test]
fn app_entry_defaults_force_loopback_true() {
    let default_entry = AppEntry::default();
    assert!(default_entry.force_loopback);

    let constructed = AppEntry::new(
        Uuid::new_v4().to_string(),
        "constructed",
        PathBuf::from("."),
        "npm",
        vec!["run".into(), "dev".into()],
    );
    assert!(constructed.force_loopback);

    let json = r#"{
        "id": "app",
        "name": "App",
        "cwd": ".",
        "command": "npm",
        "args": [],
        "preferred_port": null,
        "enabled": true
    }"#;
    let from_json: AppEntry = serde_json::from_str(json).unwrap();
    assert!(from_json.force_loopback);
}

#[test]
fn remove_app_and_get() {
    let root = temp_test_dir();
    let child = root.join("proj");
    std::fs::create_dir_all(&child).unwrap();

    let mut reg = Registry::default_empty();
    reg.allowed_roots.push(root.clone());

    let id = Uuid::new_v4().to_string();
    let entry = AppEntry {
        id: id.clone(),
        name: "app".into(),
        cwd: child,
        command: "npm".into(),
        args: vec![],
        preferred_port: None,
        force_loopback: true,
        enabled: true,
    };
    reg.add_app(entry).unwrap();

    assert!(reg.get(&id).is_some());
    reg.remove_app(&id).unwrap();
    assert!(reg.get(&id).is_none());

    let err = reg.remove_app("missing").unwrap_err();
    assert!(matches!(err, LocalDockError::AppNotFound(_)));

    cleanup_dir(&root);
}
