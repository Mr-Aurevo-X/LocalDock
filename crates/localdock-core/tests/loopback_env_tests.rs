use localdock_core::loopback_env::apply_loopback;

#[test]
fn injects_host_env() {
    let (env, args) = apply_loopback("npm", &["run".into(), "dev".into()], Some(5173));
    assert!(env.iter().any(|(k, v)| k == "HOST" && v == "127.0.0.1"));
    assert!(env.iter().any(|(k, v)| k == "HOSTNAME" && v == "127.0.0.1"));
    assert!(env.iter().any(|(k, v)| k == "PORT" && v == "5173"));
    assert_eq!(args, vec!["run", "dev"]);
}

#[test]
fn vite_gets_host_flag_when_command_is_vite() {
    let (_env, args) = apply_loopback("vite", &[], Some(5173));
    assert!(args.windows(2).any(|w| w == ["--host", "127.0.0.1"]));
}

#[test]
fn vite_windows_shim_gets_host_flag() {
    let (_env, args) = apply_loopback(
        "C:\\Projects\\node_modules\\.bin\\vite.cmd",
        &[],
        Some(5173),
    );
    assert!(args.windows(2).any(|w| w == ["--host", "127.0.0.1"]));
}

#[test]
fn next_gets_h_flag_when_command_is_next() {
    let (_env, args) = apply_loopback("next", &["dev".into()], Some(3000));
    assert!(args.windows(2).any(|w| w == ["-H", "127.0.0.1"]));
}

#[test]
fn next_windows_exe_arg_gets_h_flag() {
    let (_env, args) = apply_loopback("npx", &["next.exe".into(), "dev".into()], Some(3000));
    assert!(args.windows(2).any(|w| w == ["-H", "127.0.0.1"]));
}

#[test]
fn uvicorn_gets_host_flag() {
    let (_env, args) = apply_loopback("uvicorn", &["main:app".into()], Some(8000));
    assert!(args.windows(2).any(|w| w == ["--host", "127.0.0.1"]));
}

#[test]
fn does_not_duplicate_existing_host_flag() {
    let (_env, args) = apply_loopback("vite", &["--host".into(), "127.0.0.1".into()], Some(5173));
    assert_eq!(args.iter().filter(|a| *a == "--host").count(), 1);
}

#[test]
fn rewrites_explicit_lan_host_flag_to_loopback() {
    let (_env, args) = apply_loopback("vite", &["--host".into(), "0.0.0.0".into()], Some(5173));
    assert_eq!(args, vec!["--host", "127.0.0.1"]);
}

#[test]
fn rewrites_inline_lan_host_flag_to_loopback() {
    let (_env, args) = apply_loopback("vite", &["--host=0.0.0.0".into()], Some(5173));
    assert_eq!(args, vec!["--host=127.0.0.1"]);
}

#[test]
fn rewrites_next_h_lan_host_to_loopback() {
    let (_env, args) = apply_loopback(
        "next",
        &["dev".into(), "-H".into(), "0.0.0.0".into()],
        Some(3000),
    );
    assert_eq!(args, vec!["dev", "-H", "127.0.0.1"]);
}

#[test]
fn bare_vite_host_flag_does_not_block_loopback_injection() {
    let (_env, args) = apply_loopback("vite", &["--host".into()], Some(5173));
    assert_eq!(args, vec!["--host", "--host", "127.0.0.1"]);
}

#[test]
fn bare_next_h_flag_does_not_block_loopback_injection() {
    let (_env, args) = apply_loopback("next", &["dev".into(), "-H".into()], Some(3000));
    assert_eq!(args, vec!["dev", "-H", "-H", "127.0.0.1"]);
}

#[test]
fn omits_port_env_when_preferred_port_none() {
    let (env, _args) = apply_loopback("npm", &["run".into(), "dev".into()], None);
    assert!(env.iter().any(|(k, v)| k == "HOST" && v == "127.0.0.1"));
    assert!(!env.iter().any(|(k, _)| k == "PORT"));
}
