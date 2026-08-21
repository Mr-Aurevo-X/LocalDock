use localdock_core::path_guard::assert_under_roots;
use std::path::{Path, PathBuf};

#[test]
fn accepts_path_inside_root() {
    let root = std::env::temp_dir().join("ld-root-ok");
    let child = root.join("proj");
    std::fs::create_dir_all(&child).unwrap();
    let got = assert_under_roots(&child, &[root.clone()]).unwrap();
    assert!(got.ends_with("proj"));
}

#[test]
fn rejects_path_outside_root() {
    let root = std::env::temp_dir().join("ld-root-a");
    let other = std::env::temp_dir().join("ld-root-b").join("proj");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    let err = assert_under_roots(&other, &[root]).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("escapes") || msg.contains("not allowed"));
}
