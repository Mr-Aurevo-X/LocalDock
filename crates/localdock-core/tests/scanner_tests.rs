use localdock_core::{scan_root, ProposedApp};
use std::path::{Path, PathBuf};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scanner")
}

fn find_proposal<'a>(proposals: &'a [ProposedApp], name: &str) -> &'a ProposedApp {
    proposals
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("missing proposal named {name:?}; got {proposals:?}"))
}

#[test]
fn scan_root_finds_vite_and_django_fixtures() {
    let root = fixtures_root();
    let proposals = scan_root(&root, 3).expect("scan should succeed");

    let vite = find_proposal(&proposals, "fixture-vite-app");
    assert_eq!(vite.command, "npm");
    assert_eq!(vite.args, vec!["run", "dev"]);
    assert_eq!(vite.preferred_port, Some(3001));
    assert!(vite.cwd.ends_with("vite-app"));

    let django = find_proposal(&proposals, "django-app");
    assert_eq!(django.command, "python");
    assert_eq!(
        django.args,
        vec!["manage.py", "runserver", "127.0.0.1:8000"]
    );
    assert_eq!(django.preferred_port, Some(8000));
}

#[test]
fn scan_root_skips_plain_node_and_ambiguous_python() {
    let root = fixtures_root();
    let proposals = scan_root(&root, 3).expect("scan should succeed");

    assert!(
        !proposals.iter().any(|p| p.name == "fixture-plain-node"),
        "package.json without dev script must not be proposed"
    );
    assert!(
        !proposals
            .iter()
            .any(|p| p.cwd.ends_with("python-ambiguous")),
        "ambiguous python layout must not be auto-proposed"
    );
}

#[test]
fn scan_root_reads_env_port_when_no_vite_config() {
    let root = fixtures_root();
    let proposals = scan_root(&root, 3).expect("scan should succeed");

    let env_app = find_proposal(&proposals, "fixture-env-port");
    assert_eq!(env_app.preferred_port, Some(4321));
}

#[test]
fn scan_root_skips_node_modules_tree() {
    let root = std::env::temp_dir().join(format!("ld-scan-skip-{}", uuid::Uuid::new_v4()));
    let hidden = root.join("node_modules").join("hidden-package");
    std::fs::create_dir_all(&hidden).unwrap();
    std::fs::write(
        hidden.join("package.json"),
        r#"{"name":"hidden","scripts":{"dev":"vite"}}"#,
    )
    .unwrap();

    let proposals = scan_root(&root, 3).expect("scan should succeed");
    assert!(
        proposals.is_empty(),
        "node_modules contents must be skipped: {proposals:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn scan_root_finds_nested_project() {
    let root = fixtures_root().join("nested");
    let proposals = scan_root(&root, 4).expect("scan should succeed");

    let nested = find_proposal(&proposals, "nested-web");
    assert_eq!(nested.command, "npm");
    assert!(nested.cwd.ends_with(Path::new("apps").join("web")));
}

#[test]
fn scan_root_respects_max_depth() {
    let root = fixtures_root();
    let shallow = scan_root(&root, 0).expect("scan should succeed");
    assert!(
        shallow.is_empty(),
        "depth 0 should only inspect root dir, which has no markers"
    );

    let limited = scan_root(&root.join("nested"), 1).expect("scan should succeed");
    assert!(
        limited.is_empty(),
        "depth 1 should not reach nested/apps/web: {limited:?}"
    );
}

#[test]
fn scan_root_finds_dev_local_script_and_js_port() {
    let root = fixtures_root();
    let proposals = scan_root(&root, 3).expect("scan should succeed");

    let lounge = find_proposal(&proposals, "fixture-lounge");
    assert_eq!(lounge.command, "pnpm");
    assert_eq!(lounge.args, vec!["run", "dev:local"]);
    assert_eq!(lounge.preferred_port, Some(4180));
    assert!(lounge.cwd.ends_with("dev-local-app"));
}

#[test]
fn scan_root_returns_io_error_for_missing_root() {
    let missing = fixtures_root().join("does-not-exist");
    let err = scan_root(&missing, 1).unwrap_err();
    assert!(err.to_string().contains("io"));
}
