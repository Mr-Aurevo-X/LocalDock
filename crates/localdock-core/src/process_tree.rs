use std::collections::{HashMap, HashSet};

/// Build the process tree rooted at `root` from a `pid -> parent pid` map.
/// The set always includes `root`, even if it is missing from the snapshot.
pub fn collect_tree(root: u32, ppid_of: &HashMap<u32, u32>) -> HashSet<u32> {
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (&pid, &ppid) in ppid_of {
        children.entry(ppid).or_default().push(pid);
    }

    let mut tree = HashSet::new();
    let mut stack = vec![root];
    while let Some(pid) = stack.pop() {
        if tree.insert(pid) {
            if let Some(kids) = children.get(&pid) {
                stack.extend(kids.iter().copied());
            }
        }
    }
    tree
}

pub fn process_parent_map() -> HashMap<u32, u32> {
    platform::process_parent_map()
}

pub fn process_exe_names() -> HashMap<u32, String> {
    platform::process_exe_names()
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::mem::size_of;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    pub fn process_parent_map() -> HashMap<u32, u32> {
        let mut map = HashMap::new();
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return map;
            }

            let mut entry = std::mem::zeroed::<PROCESSENTRY32W>();
            entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    map.insert(entry.th32ProcessID, entry.th32ParentProcessID);
                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snapshot);
        }
        map
    }

    pub fn process_exe_names() -> HashMap<u32, String> {
        let mut map = HashMap::new();
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return map;
            }

            let mut entry = std::mem::zeroed::<PROCESSENTRY32W>();
            entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let name = wide_c_string(&entry.szExeFile);
                    if !name.is_empty() {
                        map.insert(entry.th32ProcessID, name);
                    }
                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snapshot);
        }
        map
    }

    fn wide_c_string(buf: &[u16]) -> String {
        let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
        String::from_utf16_lossy(&buf[..len])
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::fs;

    pub fn process_parent_map() -> HashMap<u32, u32> {
        let mut map = HashMap::new();
        let Ok(entries) = fs::read_dir("/proc") else {
            return map;
        };

        for entry in entries.flatten() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
                continue;
            };
            let Ok(status) = fs::read_to_string(entry.path().join("status")) else {
                continue;
            };
            for line in status.lines() {
                let Some(ppid_text) = line.strip_prefix("PPid:") else {
                    continue;
                };
                if let Ok(ppid) = ppid_text.trim().parse::<u32>() {
                    map.insert(pid, ppid);
                }
                break;
            }
        }
        map
    }

    pub fn process_exe_names() -> HashMap<u32, String> {
        let mut map = HashMap::new();
        let Ok(entries) = fs::read_dir("/proc") else {
            return map;
        };
        for entry in entries.flatten() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
                continue;
            };
            let Ok(comm) = fs::read_to_string(entry.path().join("comm")) else {
                continue;
            };
            map.insert(pid, comm.trim().to_string());
        }
        map
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod platform {
    use super::*;

    pub fn process_parent_map() -> HashMap<u32, u32> {
        HashMap::new()
    }

    pub fn process_exe_names() -> HashMap<u32, String> {
        HashMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ppid_map(pairs: &[(u32, u32)]) -> HashMap<u32, u32> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn collect_tree_includes_root_when_snapshot_is_empty() {
        let tree = collect_tree(10, &HashMap::new());
        assert_eq!(tree, HashSet::from([10]));
    }

    #[test]
    fn collect_tree_includes_children_and_grandchildren() {
        // npm 100 → node 200 → listener 300
        let ppid_of = ppid_map(&[(100, 1), (200, 100), (300, 200), (400, 1)]);
        let tree = collect_tree(100, &ppid_of);
        assert_eq!(tree, HashSet::from([100, 200, 300]));
        assert!(!tree.contains(&400));
    }
}
