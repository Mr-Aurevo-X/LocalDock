use localdock_core::win_paths::{
    discover_windows_system_roots, find_windows_localdock_registries, join_on_mount,
    looks_like_windows_path, remap_windows_path, resolve_existing_dir, strip_windows_extended_prefix,
    windows_drive_and_rest, windows_root_from_roaming_registry,
};
use std::path::{Path, PathBuf};

#[test]
fn strips_extended_prefix() {
    assert_eq!(
        strip_windows_extended_prefix(r"\\?\C:\Users\aurel\Documents"),
        r"C:\Users\aurel\Documents"
    );
}

#[test]
fn parses_drive_and_rest() {
    let (drive, rest) =
        windows_drive_and_rest(r"\\?\C:\Users\aurel\Documents\Dev Game Be Like Vercel").unwrap();
    assert_eq!(drive, 'C');
    assert_eq!(rest, "Users/aurel/Documents/Dev Game Be Like Vercel");
    assert!(looks_like_windows_path(r"C:\Projects"));
    assert!(!looks_like_windows_path("/home/mraurevox/projects"));
}

#[test]
fn remaps_onto_mounted_ntfs() {
    let mount = PathBuf::from("/run/media/mraurevox/WIN");
    let mapped = remap_windows_path(
        r"\\?\C:\Users\aurel\Documents\Dev Game Be Like Vercel",
        &mount,
    )
    .unwrap();
    assert_eq!(
        mapped,
        PathBuf::from("/run/media/mraurevox/WIN/Users/aurel/Documents/Dev Game Be Like Vercel")
    );
}

#[test]
fn join_on_mount_ignores_parent_escape() {
    let mapped = join_on_mount(Path::new("/mnt/c"), r"Users\..\Windows");
    assert_eq!(mapped, PathBuf::from("/mnt/c/Users/Windows"));
}

#[test]
fn roaming_registry_walks_up_to_windows_root() {
    let apps = PathBuf::from(
        "/run/media/mraurevox/WIN/Users/aurel/AppData/Roaming/LocalDock/apps.json",
    );
    assert_eq!(
        windows_root_from_roaming_registry(&apps).unwrap(),
        PathBuf::from("/run/media/mraurevox/WIN")
    );
}

#[test]
fn discovers_this_machine_windows_mount_when_present() {
    let system32 = Path::new("/run/media/mraurevox/B88CEE798CEE3194/Windows/System32");
    if !system32.is_dir() {
        return;
    }
    let roots = discover_windows_system_roots();
    assert!(
        roots.iter().any(|root| root.ends_with("B88CEE798CEE3194")),
        "expected NTFS Windows root, got {roots:?}"
    );
    let regs = find_windows_localdock_registries();
    assert!(
        regs.iter().any(|path| path.ends_with("LocalDock/apps.json")),
        "expected Windows LocalDock apps.json, got {regs:?}"
    );
    let mapped = resolve_existing_dir(r"C:\Users\aurel\Documents\Dev Game Be Like Vercel")
        .expect("remap Game Lounge from C:");
    assert!(mapped.join("package.json").is_file(), "{}", mapped.display());
}
