use std::path::{Path, PathBuf};

/// Strip `\\?\` / `//?/` extended prefixes. UNC stays unmapped.
pub fn strip_windows_extended_prefix(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{rest}");
    }
    if let Some(rest) = trimmed.strip_prefix(r"\\?\") {
        return rest.to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("//?/") {
        return rest.to_string();
    }
    trimmed.to_string()
}

pub fn looks_like_windows_path(raw: &str) -> bool {
    windows_drive_and_rest(raw).is_some()
}

/// `C:\Users\aurel\Documents` → `('C', "Users/aurel/Documents")`
pub fn windows_drive_and_rest(raw: &str) -> Option<(char, String)> {
    let stripped = strip_windows_extended_prefix(raw);
    let mut chars = stripped.chars();
    let drive = chars.next()?;
    if !drive.is_ascii_alphabetic() {
        return None;
    }
    if chars.next() != Some(':') {
        return None;
    }
    match chars.next() {
        Some('\\' | '/') => {}
        _ => return None,
    }
    let rest: String = chars.collect::<String>().replace('\\', "/");
    Some((drive, rest))
}

pub fn join_on_mount(mount: &Path, rest: &str) -> PathBuf {
    let rest = rest.trim_matches(|ch| ch == '/' || ch == '\\');
    let mut path = mount.to_path_buf();
    if rest.is_empty() {
        return path;
    }
    for part in rest.split(['/', '\\']) {
        if part.is_empty() || part == "." || part == ".." {
            continue;
        }
        path.push(part);
    }
    path
}

pub fn remap_windows_path(raw: &str, system_root: &Path) -> Option<PathBuf> {
    let (_, rest) = windows_drive_and_rest(raw)?;
    Some(join_on_mount(system_root, &rest))
}

pub fn is_windows_system_root(path: &Path) -> bool {
    path.join("Windows").join("System32").is_dir()
        || path.join("Windows").join("system32").is_dir()
}

pub fn mount_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut users = Vec::new();
    if let Ok(user) = std::env::var("USER") {
        if !user.is_empty() {
            users.push(user);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if let Some(name) = Path::new(&home).file_name() {
            let name = name.to_string_lossy().into_owned();
            if !name.is_empty() && !users.iter().any(|user| user == &name) {
                users.push(name);
            }
        }
    }

    for user in users {
        push_dir_children(&mut out, &PathBuf::from("/run/media").join(&user));
        push_dir_children(&mut out, &PathBuf::from("/media").join(&user));
    }
    out.push(PathBuf::from("/mnt/c"));
    out.push(PathBuf::from("/mnt/C"));
    out
}

fn push_dir_children(out: &mut Vec<PathBuf>, dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            out.push(entry.path());
        }
    }
}

pub fn discover_windows_system_roots() -> Vec<PathBuf> {
    mount_candidates()
        .into_iter()
        .filter(|path| is_windows_system_root(path))
        .collect()
}

/// Resolve a typed or pasted path: Linux path as-is, or `C:\…` onto a mounted NTFS.
pub fn resolve_existing_dir(raw: &str) -> std::io::Result<PathBuf> {
    let trimmed = raw.trim();
    let direct = Path::new(trimmed);
    if direct.is_dir() {
        return std::fs::canonicalize(direct);
    }

    if let Some((_, rest)) = windows_drive_and_rest(trimmed) {
        let mut mounts = discover_windows_system_roots();
        for extra in mount_candidates() {
            if !mounts.iter().any(|known| known == &extra) {
                mounts.push(extra);
            }
        }
        for mount in mounts {
            let mapped = join_on_mount(&mount, &rest);
            if mapped.is_dir() {
                return std::fs::canonicalize(mapped);
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("directory not found: {trimmed}"),
    ))
}

pub fn windows_root_from_roaming_registry(apps_json: &Path) -> Option<PathBuf> {
    let localdock = apps_json.parent()?;
    let roaming = localdock.parent()?;
    let appdata = roaming.parent()?;
    let user_dir = appdata.parent()?;
    let users = user_dir.parent()?;
    Some(users.parent()?.to_path_buf())
}

const SKIP_WINDOWS_USERS: &[&str] = &[
    "Default",
    "Default User",
    "Public",
    "All Users",
    "WsiAccount",
];

pub fn find_windows_localdock_registries() -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in discover_windows_system_roots() {
        let users = root.join("Users");
        let Ok(entries) = std::fs::read_dir(&users) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if SKIP_WINDOWS_USERS.iter().any(|skip| name.eq_ignore_ascii_case(skip)) {
                continue;
            }
            let apps = entry
                .path()
                .join("AppData")
                .join("Roaming")
                .join("LocalDock")
                .join("apps.json");
            if apps.is_file() {
                found.push(apps);
            }
        }
    }
    found
}
