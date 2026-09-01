use localdock_core::ports::{
    attach_app_labels, filter_display_rows, list_listeners, parse_ss_listen_line,
    parse_ss_local_address, well_known_listener, AppHint, PortRow,
};
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
        killable: true,
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
fn parse_ss_handles_loopback_iface_and_users() {
    let (addr, port, pid, name) = parse_ss_listen_line(
        r#"LISTEN 0 511 127.0.0.1:7879 0.0.0.0:* users:(("electron",pid=3268,fd=89))"#,
    )
    .expect("ss line");
    assert!(addr.is_loopback());
    assert_eq!(port, 7879);
    assert_eq!(pid, 3268);
    assert_eq!(name, "electron");

    let (scoped, scoped_port) = parse_ss_local_address("127.0.0.53%lo:53").expect("scoped");
    assert_eq!(scoped.to_string(), "127.0.0.53");
    assert_eq!(scoped_port, 53);

    let (v6, v6_port) = parse_ss_local_address("[::1]:631").expect("ipv6");
    assert!(v6.is_loopback());
    assert_eq!(v6_port, 631);

    let (tcp_addr, tcp_port, tcp_pid, tcp_name) = parse_ss_listen_line(
        r#"tcp LISTEN 0 4096 127.0.0.1:7879 0.0.0.0:* users:(("electron",pid=3268,fd=89))"#,
    )
    .expect("tcp ss line");
    assert!(tcp_addr.is_loopback());
    assert_eq!(tcp_port, 7879);
    assert_eq!(tcp_pid, 3268);
    assert_eq!(tcp_name, "electron");
}

#[test]
fn well_known_labels_resolved_and_cups() {
    let dns = well_known_listener("127.0.0.53".parse().unwrap(), 53).expect("resolved stub");
    assert_eq!(dns.name, "systemd-resolved");
    assert!(!dns.killable);

    assert!(well_known_listener("127.0.0.1".parse().unwrap(), 53).is_none());

    let cups = well_known_listener("127.0.0.1".parse().unwrap(), 631).expect("cups");
    assert_eq!(cups.name, "cupsd");
    assert!(!cups.killable);
}

#[test]
fn linux_known_loopback_rows_have_names() {
    let rows = list_listeners(true).unwrap();
    let dns = rows
        .iter()
        .find(|row| row.port == 53 && row.addr == "127.0.0.53");
    if let Some(row) = dns {
        assert_eq!(row.process_name, "systemd-resolved");
        assert!(!row.image_path.is_empty());
        assert!(!row.killable);
    }
    let cups = rows.iter().find(|row| row.port == 631);
    if let Some(row) = cups {
        assert_eq!(row.process_name, "cupsd");
        assert!(!row.killable);
    }
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
