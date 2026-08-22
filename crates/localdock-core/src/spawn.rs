use crate::host_exec;
use crate::launch_resolve;
use crate::loopback_env;
use crate::process_tree;
use crate::registry::{validate_command, AppEntry};
use crate::LocalDockError;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::Mutex;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Default)]
pub struct ProcessManager {
    children: Mutex<HashMap<String, Child>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&self, app: &AppEntry) -> Result<(), LocalDockError> {
        validate_command(&app.command)?;
        let (command, args) = launch_resolve::resolve_launch(&app.command, &app.args, &app.cwd)?;
        validate_command(&command)?;

        let (env_pairs, final_args) = if app.force_loopback {
            loopback_env::apply_loopback(&command, &args, app.preferred_port)
        } else {
            (Vec::new(), args)
        };

        {
            let mut children = self
                .children
                .lock()
                .expect("process manager mutex poisoned");
            if let Some(child) = children.get_mut(&app.id) {
                if child.try_wait()?.is_none() {
                    return Err(LocalDockError::AlreadyRunning);
                }
                children.remove(&app.id);
            }
        }

        let log = std::env::temp_dir().join(format!("localdock-{}.log", app.id));
        let mut child = start_command_logged(&command, &final_args, &app.cwd, &env_pairs, Some(&log))?;
        std::thread::sleep(Duration::from_millis(200));
        if let Some(status) = child.try_wait()? {
            let tail = std::fs::read_to_string(&log)
                .unwrap_or_default()
                .trim()
                .to_string();
            return Err(LocalDockError::StartFailed(if tail.is_empty() {
                format!("le process s’est arrêté ({status})")
            } else {
                format!("le process s’est arrêté ({status}): {tail}")
            }));
        }

        let mut children = self
            .children
            .lock()
            .expect("process manager mutex poisoned");
        children.insert(app.id.clone(), child);
        Ok(())
    }

    pub fn stop(&self, app_id: &str) -> Result<(), LocalDockError> {
        let mut child = {
            let mut children = self
                .children
                .lock()
                .expect("process manager mutex poisoned");
            children.remove(app_id).ok_or(LocalDockError::NotRunning)?
        };

        if child.try_wait()?.is_none() {
            child.kill()?;
        }
        child.wait()?;
        Ok(())
    }

    pub fn is_running(&self, app_id: &str) -> bool {
        let mut children = self
            .children
            .lock()
            .expect("process manager mutex poisoned");
        match children.get_mut(app_id) {
            Some(child) => match child.try_wait() {
                Ok(None) => true,
                Ok(Some(_)) | Err(_) => {
                    children.remove(app_id);
                    false
                }
            },
            None => false,
        }
    }

    pub fn running_ids(&self) -> Vec<String> {
        let mut children = self
            .children
            .lock()
            .expect("process manager mutex poisoned");
        children.retain(|_, child| matches!(child.try_wait(), Ok(None)));

        let mut ids = children.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        ids
    }

    pub fn pid(&self, app_id: &str) -> Option<u32> {
        let mut children = self
            .children
            .lock()
            .expect("process manager mutex poisoned");
        match children.get_mut(app_id) {
            Some(child) => match child.try_wait() {
                Ok(None) => Some(child.id()),
                Ok(Some(_)) | Err(_) => {
                    children.remove(app_id);
                    None
                }
            },
            None => None,
        }
    }

    pub fn running_pids(&self) -> Vec<u32> {
        let mut children = self
            .children
            .lock()
            .expect("process manager mutex poisoned");
        children.retain(|_, child| matches!(child.try_wait(), Ok(None)));

        let mut pids = children
            .values()
            .map(|child| child.id())
            .collect::<Vec<_>>();
        pids.sort_unstable();
        pids
    }

    pub fn tree_pids(&self, app_id: &str) -> Vec<u32> {
        match self.pid(app_id) {
            Some(root) => sorted_tree(root),
            None => Vec::new(),
        }
    }

    pub fn running_tree_pids(&self) -> Vec<u32> {
        let roots = self.running_pids();
        let ppid_of = process_tree::process_parent_map();
        let mut all = HashSet::new();
        for root in roots {
            all.extend(process_tree::collect_tree(root, &ppid_of));
        }
        let mut pids = all.into_iter().collect::<Vec<_>>();
        pids.sort_unstable();
        pids
    }
}

fn sorted_tree(root: u32) -> Vec<u32> {
    let mut pids = process_tree::collect_tree(root, &process_tree::process_parent_map())
        .into_iter()
        .collect::<Vec<_>>();
    pids.sort_unstable();
    pids
}

pub fn start_command(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &[(String, String)],
) -> Result<Child, LocalDockError> {
    start_command_logged(program, args, cwd, env, None)
}

pub fn start_command_logged(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &[(String, String)],
    log: Option<&Path>,
) -> Result<Child, LocalDockError> {
    let resolved = resolve_program(program);
    let resolved_text = resolved.to_string_lossy();
    let mut cmd = host_exec::host_command(&resolved_text, args, Some(cwd), env);
    cmd.stdin(Stdio::null());
    if let Some(log) = log {
        let file = File::create(log)?;
        let err = file.try_clone()?;
        cmd.stdout(Stdio::from(file)).stderr(Stdio::from(err));
    } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd.spawn().map_err(LocalDockError::from)
}

pub fn resolve_program(program: &str) -> PathBuf {
    let given = PathBuf::from(program);
    if given.components().count() > 1 && given.is_file() {
        return given;
    }

    #[cfg(windows)]
    {
        resolve_windows_program(program)
    }

    #[cfg(not(windows))]
    {
        given
    }
}

#[cfg(windows)]
fn resolve_windows_program(program: &str) -> PathBuf {
    let names = if program.contains('.') {
        vec![program.to_string()]
    } else {
        vec![
            format!("{program}.exe"),
            format!("{program}.cmd"),
            format!("{program}.bat"),
            program.to_string(),
        ]
    };

    let mut dirs: Vec<PathBuf> = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .collect();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        dirs.push(PathBuf::from(appdata).join("npm"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local);
        dirs.push(local.join("pnpm"));
        dirs.push(local.join("Programs").join("pnpm"));
    }

    for dir in dirs {
        for name in &names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return candidate;
            }
        }
    }

    PathBuf::from(program)
}
