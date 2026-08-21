use crate::loopback_env;
use crate::registry::{validate_command, AppEntry};
use crate::LocalDockError;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

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

        let (env_pairs, final_args) = if app.force_loopback {
            loopback_env::apply_loopback(&app.command, &app.args, app.preferred_port)
        } else {
            (Vec::new(), app.args.clone())
        };

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

        let child = start_command(&app.command, &final_args, &app.cwd, &env_pairs)?;
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

    pub fn owned_pids(&self, app_id: &str) -> Vec<u32> {
        match self.pid(app_id) {
            Some(pid) => process_tree_pids(pid),
            None => Vec::new(),
        }
    }

    pub fn running_pids(&self) -> Vec<u32> {
        let roots = {
            let mut children = self
                .children
                .lock()
                .expect("process manager mutex poisoned");
            children.retain(|_, child| matches!(child.try_wait(), Ok(None)));
            children
                .values()
                .map(|child| child.id())
                .collect::<Vec<_>>()
        };

        let mut pids = roots
            .into_iter()
            .flat_map(process_tree_pids)
            .collect::<Vec<_>>();
        pids.sort_unstable();
        pids.dedup();
        pids
    }
}

pub fn process_tree_pids(root: u32) -> Vec<u32> {
    collect_process_tree(root)
}

fn collect_from_parent_map(root: u32, children: &HashMap<u32, Vec<u32>>) -> Vec<u32> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    let mut seen = HashSet::new();
    while let Some(pid) = stack.pop() {
        if !seen.insert(pid) {
            continue;
        }
        out.push(pid);
        if let Some(kids) = children.get(&pid) {
            stack.extend(kids.iter().copied());
        }
    }
    out.sort_unstable();
    out
}

#[cfg(target_os = "linux")]
fn collect_process_tree(root: u32) -> Vec<u32> {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return vec![root];
    };

    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        if let Some(ppid) = read_ppid(pid) {
            children.entry(ppid).or_default().push(pid);
        }
    }

    collect_from_parent_map(root, &children)
}

#[cfg(target_os = "linux")]
fn read_ppid(pid: u32) -> Option<u32> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("PPid:")
            .and_then(|rest| rest.trim().parse().ok())
    })
}

#[cfg(windows)]
fn collect_process_tree(root: u32) -> Vec<u32> {
    use std::mem::size_of;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return vec![root];
    }

    let mut entry = unsafe { std::mem::zeroed::<PROCESSENTRY32W>() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut ok = unsafe { Process32FirstW(snapshot, &mut entry) };
    while ok != 0 {
        children
            .entry(entry.th32ParentProcessID)
            .or_default()
            .push(entry.th32ProcessID);
        ok = unsafe { Process32NextW(snapshot, &mut entry) };
    }
    unsafe {
        CloseHandle(snapshot);
    }

    collect_from_parent_map(root, &children)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn collect_process_tree(root: u32) -> Vec<u32> {
    vec![root]
}

pub fn start_command(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &[(String, String)],
) -> Result<Child, LocalDockError> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(cwd)
        .envs(env.iter().map(|(key, value)| (key, value)))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd.spawn().map_err(LocalDockError::from)
}
