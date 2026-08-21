use localdock_core::ports::list_listeners;
use std::net::TcpListener;

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
