use std::path::Path;
use std::process::Command;

#[cfg(target_os = "linux")]
use std::path::PathBuf;

#[cfg(target_os = "linux")]
const HOST_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
#[cfg(target_os = "linux")]
const UNSET_HOST_ENV: &[&str] = &[
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "PYTHONPATH",
    "PYTHONHOME",
    "GI_TYPELIB_PATH",
    "GIO_MODULE_DIR",
    "GTK_PATH",
    "GTK_DATA_PREFIX",
];

pub fn running_in_flatpak() -> bool {
    std::env::var_os("FLATPAK_ID").is_some() || Path::new("/.flatpak-info").exists()
}

pub fn host_command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> Command {
    #[cfg(not(target_os = "linux"))]
    {
        let mut cmd = Command::new(program);
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        cmd.envs(env.iter().cloned());
        return cmd;
    }

    #[cfg(target_os = "linux")]
    {
        if !running_in_flatpak() {
            let mut cmd = Command::new(program);
            cmd.args(args);
            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }
            cmd.envs(env.iter().cloned());
            return cmd;
        }

        let dir = host_cwd(cwd);
        let mut cmd = Command::new("flatpak-spawn");
        cmd.arg("--host");
        cmd.arg(format!("--directory={}", dir.display()));
        cmd.arg("--");
        cmd.arg("/usr/bin/env");
        for name in UNSET_HOST_ENV {
            cmd.arg("-u");
            cmd.arg(name);
        }
        cmd.arg(format!("PATH={HOST_PATH}"));
        for (key, value) in env {
            if is_safe_env_key(key) {
                cmd.arg(format!("{key}={value}"));
            }
        }
        cmd.arg(program);
        cmd.args(args);
        cmd
    }
}

#[cfg(target_os = "linux")]
fn host_cwd(cwd: Option<&Path>) -> PathBuf {
    if let Some(dir) = cwd {
        if !is_sandbox_path(dir) {
            return dir.to_path_buf();
        }
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_dir() && !is_sandbox_path(path))
        .unwrap_or_else(|| PathBuf::from("/"))
}

#[cfg(target_os = "linux")]
fn is_sandbox_path(path: &Path) -> bool {
    path == Path::new("/app") || path.starts_with("/app/")
}

fn is_safe_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_keys_must_be_identifiers() {
        assert!(is_safe_env_key("HOST"));
        assert!(is_safe_env_key("PORT"));
        assert!(is_safe_env_key("_A"));
        assert!(!is_safe_env_key(""));
        assert!(!is_safe_env_key("HOST=1"));
        assert!(!is_safe_env_key("PATH;rm"));
    }

    #[test]
    fn native_host_command_runs_program_directly() {
        if running_in_flatpak() {
            return;
        }
        let cmd = host_command("ss", &["-H".into()], None, &[]);
        assert_eq!(cmd.get_program(), "ss");
    }
}
