use localdock_core::loopback_env::apply_loopback;

#[test]
fn injects_host_env() {
    let (env, args) = apply_loopback("npm", &["run".into(), "dev".into()], Some(5173));
    assert!(env.iter().any(|(k, v)| k == "HOST" && v == "127.0.0.1"));
    assert!(env.iter().any(|(k, v)| k == "PORT" && v == "5173"));
    assert_eq!(args, vec!["run", "dev"]);
}

#[test]
fn vite_gets_host_flag_when_command_is_vite() {
    let (_env, args) = apply_loopback("vite", &[], Some(5173));
    assert!(args
        .windows(2)
        .any(|w| w == ["--host", "127.0.0.1"]));
}

#[test]
fn next_gets_h_flag_when_command_is_next() {
    let (_env, args) = apply_loopback("next", &["dev".into()], Some(3000));
    assert!(args.windows(2).any(|w| w == ["-H", "127.0.0.1"]));
}

#[test]
fn uvicorn_gets_host_flag() {
    let (_env, args) = apply_loopback(
        "uvicorn",
        &["main:app".into()],
        Some(8000),
    );
    assert!(args
        .windows(2)
        .any(|w| w == ["--host", "127.0.0.1"]));
}

#[test]
fn does_not_duplicate_existing_host_flag() {
    let (_env, args) = apply_loopback(
        "vite",
        &["--host".into(), "127.0.0.1".into()],
        Some(5173),
    );
    assert_eq!(
        args.iter().filter(|a| *a == "--host").count(),
        1
    );
}

#[test]
fn omits_port_env_when_preferred_port_none() {
    let (env, _args) = apply_loopback("npm", &["run".into(), "dev".into()], None);
    assert!(env.iter().any(|(k, v)| k == "HOST" && v == "127.0.0.1"));
    assert!(!env.iter().any(|(k, _)| k == "PORT"));
}
