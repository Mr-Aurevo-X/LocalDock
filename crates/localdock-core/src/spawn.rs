use crate::loopback_env;
use crate::registry::{validate_command, AppEntry};
use crate::LocalDockError;
use std::collections::HashMap;
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
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd.spawn().map_err(LocalDockError::from)
}
