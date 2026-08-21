use localdock_core::ports::{attach_app_labels, filter_display_rows, list_listeners, AppHint, PortRow};
use std::collections::HashSet;
use std::net::TcpListener;
use std::path::PathBuf;

fn row(port: u16, pid: u32, addr: &str, is_loopback: bool) -> PortRow {
    PortRow {
        port,
        pid,
        process_name: "node".into(),
        addr: addr.into(),
        is_loopback,
        image_path: String::new(),
        command_line: String::new(),
        cwd: String::new(),
        app_name: String::new(),
        started_unix: None,
    }
}

#[test]
fn loopback_listener_appears_in_port_scan() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let rows = list_listeners(true).unwrap();

    assert!(
        rows.iter()
            .any(|row| row.port == port && row.is_loopback && row.addr == "127.0.0.1"),
        "expected loopback listener on port {port}, got {rows:?}"
    );
}

#[test]
fn lan_row_is_kept_when_listener_pid_is_in_managed_tree() {
    let lan = row(5173, 300, "0.0.0.0", false);
    let managed = HashSet::from([100, 200, 300]);
    let visible = filter_display_rows(vec![lan.clone()], &managed);
    assert_eq!(visible, vec![lan]);
}

#[test]
fn lan_row_is_dropped_when_listener_pid_is_outside_managed_tree() {
    let lan = row(5173, 999, "0.0.0.0", false);
    let managed = HashSet::from([100]);
    let visible = filter_display_rows(vec![lan], &managed);
    assert!(visible.is_empty());
}

#[test]
fn loopback_row_is_kept_even_when_pid_is_unmanaged() {
    let loopback = row(3000, 1, "127.0.0.1", true);
    let visible = filter_display_rows(vec![loopback.clone()], &HashSet::new());
    assert_eq!(visible, vec![loopback]);
}

#[test]
fn attach_app_labels_uses_preferred_port_and_cwd() {
    let mut rows = vec![row(4180, 42, "127.0.0.1", true)];
    let apps = [AppHint {
        name: "game-lounge".into(),
        cwd: PathBuf::from(r"C:\Users\aurel\Documents\Dev Game Be Like Vercel"),
        preferred_port: Some(4180),
    }];
    attach_app_labels(&mut rows, &apps);
    assert_eq!(rows[0].app_name, "game-lounge");
    assert!(
        rows[0].cwd.contains("Dev Game Be Like Vercel"),
        "cwd should come from the registered app: {}",
        rows[0].cwd
    );
}
