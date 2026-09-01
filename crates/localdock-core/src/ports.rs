use crate::path_guard::display_path;
use crate::LocalDockError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRow {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub addr: String,
    pub is_loopback: bool,
    #[serde(default)]
    pub image_path: String,
    #[serde(default)]
    pub command_line: String,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub app_name: String,
    #[serde(default)]
    pub started_unix: Option<i64>,
    #[serde(default = "default_killable")]
    pub killable: bool,
}

fn default_killable() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WellKnownListener {
    pub name: &'static str,
    pub image_path: &'static str,
    pub pidof: &'static str,
    pub fill_pid: bool,
    pub killable: bool,
}

pub fn well_known_listener(addr: IpAddr, port: u16) -> Option<WellKnownListener> {
    match port {
        53 if is_resolved_stub(addr) => Some(WellKnownListener {
            name: "systemd-resolved",
            image_path: "/usr/lib/systemd/systemd-resolved",
            pidof: "systemd-resolved",
            fill_pid: true,
            killable: false,
        }),
        631 => Some(WellKnownListener {
            name: "cupsd",
            image_path: "/usr/sbin/cupsd",
            pidof: "cupsd",
            fill_pid: true,
            killable: false,
        }),
        6379 => Some(WellKnownListener {
            name: "redis-server",
            image_path: "/usr/bin/redis-server",
            pidof: "redis-server",
            fill_pid: true,
            killable: true,
        }),
        5432 | 5433 => Some(WellKnownListener {
            name: "postgres",
            image_path: "/usr/bin/postgres",
            pidof: "postgres",
            fill_pid: true,
            killable: true,
        }),
        3306 => Some(WellKnownListener {
            name: "mysqld",
            image_path: "/usr/sbin/mysqld",
            pidof: "mysqld",
            fill_pid: true,
            killable: true,
        }),
        11434 => Some(WellKnownListener {
            name: "ollama",
            image_path: "/usr/bin/ollama",
            pidof: "ollama",
            fill_pid: true,
            killable: true,
        }),
        27017 => Some(WellKnownListener {
            name: "mongod",
            image_path: "/usr/bin/mongod",
            pidof: "mongod",
            fill_pid: true,
            killable: true,
        }),
        _ => None,
    }
}

fn is_resolved_stub(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            octets == [127, 0, 0, 53] || octets == [127, 0, 0, 54]
        }
        IpAddr::V6(_) => false,
    }
}

fn parse_host_proc_details(text: &str) -> (ProcessDetails, Option<u64>, Option<u64>) {
    let mut details = ProcessDetails::default();
    let mut btime = None;
    let mut start_tick = None;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("NAME=") {
            details.name = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("EXE=") {
            details.image_path = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("CWD=") {
            details.cwd = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("CMD=") {
            details.command_line = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("BTIME=") {
            btime = value.trim().parse().ok();
        } else if let Some(value) = line.strip_prefix("STARTTICK=") {
            start_tick = value.trim().parse().ok();
        }
    }
    (details, btime, start_tick)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppHint {
    pub name: String,
    pub cwd: PathBuf,
    pub preferred_port: Option<u16>,
}

#[derive(Debug, Clone, Default)]
struct ProcessDetails {
    name: String,
    image_path: String,
    command_line: String,
    cwd: String,
    started_unix: Option<i64>,
}

pub fn list_listeners(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
    platform::list_listeners(loopback_only)
}

/// Loopback listeners are always shown. Non-loopback rows stay only when the
/// owning PID is in `managed_pids` (spawn PID plus descendants).
pub fn filter_display_rows(rows: Vec<PortRow>, managed_pids: &HashSet<u32>) -> Vec<PortRow> {
    rows.into_iter()
        .filter(|row| row.is_loopback || managed_pids.contains(&row.pid))
        .collect()
}

pub fn attach_app_labels(rows: &mut [PortRow], apps: &[AppHint]) {
    for row in rows {
        let matched = apps.iter().find(|app| {
            app.preferred_port == Some(row.port) || path_mentioned(row, &app.cwd)
        });
        let Some(app) = matched else {
            continue;
        };
        if row.app_name.is_empty() {
            row.app_name = app.name.clone();
        }
        if row.cwd.is_empty() {
            row.cwd = display_path(&app.cwd);
        }
    }
}

pub fn parse_ss_local_address(addr: &str) -> Option<(IpAddr, u16)> {
    let addr = addr.trim();
    if let Some(rest) = addr.strip_prefix('[') {
        let (ip_raw, port) = rest.rsplit_once("]:")?;
        let ip_raw = ip_raw.split('%').next()?;
        let port = port.parse().ok()?;
        let ip = ip_raw.parse().ok()?;
        return Some((ip, port));
    }

    let (ip_raw, port) = addr.rsplit_once(':')?;
    let port = port.parse().ok()?;
    if ip_raw == "*" {
        return Some((IpAddr::V4(Ipv4Addr::UNSPECIFIED), port));
    }
    let ip_raw = ip_raw.split('%').next()?;
    let ip = ip_raw.parse().ok()?;
    Some((ip, port))
}

pub fn parse_ss_users(line: &str) -> (u32, String) {
    let Some(users) = line.split("users:((").nth(1) else {
        return (0, String::new());
    };
    let name = users
        .strip_prefix('"')
        .and_then(|rest| rest.split_once('"'))
        .map(|(name, _)| name.to_string())
        .unwrap_or_default();
    let pid = users
        .split("pid=")
        .nth(1)
        .and_then(|rest| rest.split(|ch: char| !ch.is_ascii_digit()).next())
        .and_then(|pid| pid.parse().ok())
        .unwrap_or(0);
    (pid, name)
}

pub fn parse_ss_listen_line(line: &str) -> Option<(IpAddr, u16, u32, String)> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 4 {
        return None;
    }
    let local = if fields[0] == "LISTEN" {
        fields[3]
    } else if fields.len() >= 5 && fields[1] == "LISTEN" {
        fields[4]
    } else {
        return None;
    };
    let (addr, port) = parse_ss_local_address(local)?;
    let (pid, name) = parse_ss_users(line);
    Some((addr, port, pid, name))
}

fn path_mentioned(row: &PortRow, cwd: &std::path::Path) -> bool {
    let needle = display_path(cwd);
    if needle.is_empty() {
        return false;
    }
    let needle_l = needle.to_ascii_lowercase();
    row.cwd.to_ascii_lowercase().starts_with(&needle_l)
        || row.command_line.to_ascii_lowercase().contains(&needle_l)
        || row.image_path.to_ascii_lowercase().starts_with(&needle_l)
}

fn make_row(port: u16, pid: u32, details: ProcessDetails, addr: IpAddr) -> PortRow {
    PortRow {
        port,
        pid,
        process_name: details.name,
        addr: addr.to_string(),
        is_loopback: addr.is_loopback(),
        image_path: details.image_path,
        command_line: details.command_line,
        cwd: details.cwd,
        app_name: String::new(),
        started_unix: details.started_unix,
        killable: true,
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    const TCP_LISTEN_STATE: &str = "0A";

    pub fn list_listeners(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
        match list_listeners_ss(loopback_only) {
            Ok(rows) => Ok(rows),
            Err(_) => list_listeners_proc(loopback_only),
        }
    }

    fn list_listeners_ss(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
        let output = crate::host_exec::host_command(
            "ss",
            &[
                "-H".into(),
                "-l".into(),
                "-t".into(),
                "-n".into(),
                "-p".into(),
            ],
            None,
            &[],
        )
        .output()?;
        if !output.status.success() {
            return Err(std::io::Error::other("ss failed").into());
        }

        let mut rows = Vec::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let Some((addr, port, pid, name)) = parse_ss_listen_line(line) else {
                continue;
            };
            if loopback_only && !addr.is_loopback() {
                continue;
            }
            rows.push(enrich_linux_row(addr, port, pid, name));
        }
        rows.sort_by_key(|row| (row.port, row.addr.clone(), row.pid));
        Ok(rows)
    }

    fn list_listeners_proc(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
        let inode_owners = inode_owners();
        let mut rows = Vec::new();
        read_tcp_file(
            "/proc/net/tcp",
            false,
            loopback_only,
            &inode_owners,
            &mut rows,
        )?;
        read_tcp_file(
            "/proc/net/tcp6",
            true,
            loopback_only,
            &inode_owners,
            &mut rows,
        )?;
        rows.sort_by_key(|row| (row.port, row.addr.clone(), row.pid));
        Ok(rows)
    }

    fn read_tcp_file(
        path: &str,
        ipv6: bool,
        loopback_only: bool,
        inode_owners: &HashMap<u64, (u32, ProcessDetails)>,
        rows: &mut Vec<PortRow>,
    ) -> Result<(), LocalDockError> {
        let content = fs::read_to_string(path)?;
        for line in content.lines().skip(1) {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.len() < 10 || fields[3] != TCP_LISTEN_STATE {
                continue;
            }

            let Some((addr, port)) = parse_local_address(fields[1], ipv6) else {
                continue;
            };
            if loopback_only && !addr.is_loopback() {
                continue;
            }

            let inode = fields[9].parse::<u64>().unwrap_or_default();
            let (pid, details) = inode_owners
                .get(&inode)
                .cloned()
                .unwrap_or((0, ProcessDetails::default()));
            let mut row = make_row(port, pid, details, addr);
            apply_well_known(&mut row, addr, port, String::new());
            rows.push(row);
        }

        Ok(())
    }

    fn parse_local_address(value: &str, ipv6: bool) -> Option<(IpAddr, u16)> {
        let (addr_hex, port_hex) = value.split_once(':')?;
        let port = u16::from_str_radix(port_hex, 16).ok()?;
        let addr = if ipv6 {
            IpAddr::V6(parse_ipv6_proc_addr(addr_hex)?)
        } else {
            IpAddr::V4(parse_ipv4_proc_addr(addr_hex)?)
        };
        Some((addr, port))
    }

    fn parse_ipv4_proc_addr(hex: &str) -> Option<Ipv4Addr> {
        if hex.len() != 8 {
            return None;
        }

        let raw = u32::from_str_radix(hex, 16).ok()?;
        Some(Ipv4Addr::from(raw.to_le_bytes()))
    }

    fn parse_ipv6_proc_addr(hex: &str) -> Option<Ipv6Addr> {
        if hex.len() != 32 {
            return None;
        }

        let mut bytes = [0u8; 16];
        for word_idx in 0..4 {
            let start = word_idx * 8;
            let word = u32::from_str_radix(&hex[start..start + 8], 16).ok()?;
            bytes[start..start + 4].copy_from_slice(&word.to_le_bytes());
        }
        Some(Ipv6Addr::from(bytes))
    }

    fn inode_owners() -> HashMap<u64, (u32, ProcessDetails)> {
        let mut owners = HashMap::new();
        let Ok(entries) = fs::read_dir("/proc") else {
            return owners;
        };

        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(pid_text) = file_name.to_str() else {
                continue;
            };
            let Ok(pid) = pid_text.parse::<u32>() else {
                continue;
            };

            let details = process_details(pid);
            let fd_dir = entry.path().join("fd");
            let Ok(fds) = fs::read_dir(fd_dir) else {
                continue;
            };

            for fd in fds.flatten() {
                let Ok(target) = fs::read_link(fd.path()) else {
                    continue;
                };
                let Some(inode) = socket_inode(&target) else {
                    continue;
                };
                owners.entry(inode).or_insert((pid, details.clone()));
            }
        }

        owners
    }

    fn socket_inode(path: &Path) -> Option<u64> {
        let target = path.to_string_lossy();
        let inode = target
            .strip_prefix("socket:[")
            .and_then(|rest| rest.strip_suffix(']'))?;
        inode.parse().ok()
    }

    fn enrich_linux_row(addr: IpAddr, port: u16, pid: u32, ss_name: String) -> PortRow {
        let hint = well_known_listener(addr, port);
        let mut pid = pid;
        if pid == 0 {
            if let Some(known) = hint {
                if known.fill_pid {
                    pid = host_pidof(known.pidof).unwrap_or(0);
                }
            }
        }
        let mut details = process_details(pid);
        if details.name.is_empty() {
            details.name = ss_name;
        }
        let mut row = make_row(port, pid, details, addr);
        apply_well_known(&mut row, addr, port, String::new());
        row
    }

    fn apply_well_known(row: &mut PortRow, addr: IpAddr, port: u16, ss_name: String) {
        let Some(hint) = well_known_listener(addr, port) else {
            return;
        };
        if row.process_name.is_empty() {
            row.process_name = if ss_name.is_empty() {
                hint.name.to_string()
            } else {
                ss_name
            };
        } else if hint.name.starts_with(&row.process_name)
            || row.process_name.starts_with(hint.name)
        {
            row.process_name = hint.name.to_string();
        }
        if row.image_path.is_empty() {
            row.image_path = hint.image_path.to_string();
        }
        row.killable = hint.killable;
    }

    fn host_pidof(name: &str) -> Option<u32> {
        let output = crate::host_exec::host_command("pidof", &[name.to_string()], None, &[])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .next()
            .and_then(|pid| pid.parse().ok())
    }

    fn process_details(pid: u32) -> ProcessDetails {
        if pid == 0 {
            return ProcessDetails::default();
        }
        if crate::host_exec::running_in_flatpak() {
            return host_process_details(pid);
        }
        read_local_proc_details(pid)
    }

    fn host_process_details(pid: u32) -> ProcessDetails {
        let script = format!(
            "pid={pid}; \
             printf 'NAME='; cat /proc/$pid/comm 2>/dev/null | tr -d '\\n'; printf '\\n'; \
             printf 'EXE='; readlink /proc/$pid/exe 2>/dev/null; printf '\\n'; \
             printf 'CWD='; readlink /proc/$pid/cwd 2>/dev/null; printf '\\n'; \
             printf 'CMD='; if [ -r /proc/$pid/cmdline ]; then tr '\\0' ' ' < /proc/$pid/cmdline; fi; printf '\\n'; \
             printf 'BTIME='; awk '/^btime /{{print $2}}' /proc/stat 2>/dev/null; \
             printf 'STARTTICK='; if [ -r /proc/$pid/stat ]; then awk -F') ' '{{print $2}}' /proc/$pid/stat | awk '{{print $20}}'; fi; printf '\\n'"
        );
        let Ok(output) =
            crate::host_exec::host_command("sh", &["-c".into(), script], None, &[]).output()
        else {
            return ProcessDetails::default();
        };
        if !output.status.success() {
            return ProcessDetails::default();
        }
        let (mut details, btime, start_tick) =
            parse_host_proc_details(&String::from_utf8_lossy(&output.stdout));
        if let (Some(btime), Some(start_tick)) = (btime, start_tick) {
            details.started_unix = Some((btime + start_tick / 100) as i64);
        }
        details
    }

    fn read_local_proc_details(pid: u32) -> ProcessDetails {
        let image_path = fs::read_link(format!("/proc/{pid}/exe"))
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let name = fs::read_to_string(format!("/proc/{pid}/comm"))
            .map(|name| name.trim().to_string())
            .unwrap_or_else(|_| {
                Path::new(&image_path)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default()
            });
        let command_line = fs::read(format!("/proc/{pid}/cmdline"))
            .map(|raw| {
                String::from_utf8_lossy(&raw)
                    .split('\0')
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let cwd = fs::read_link(format!("/proc/{pid}/cwd"))
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        ProcessDetails {
            name,
            image_path,
            command_line,
            cwd,
            started_unix: linux_start_unix(pid),
        }
    }

    fn linux_start_unix(pid: u32) -> Option<i64> {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let rest = stat.rsplit_once(')')?.1;
        let fields: Vec<&str> = rest.split_whitespace().collect();
        let starttime: u64 = fields.get(19)?.parse().ok()?;
        let btime: u64 = fs::read_to_string("/proc/stat")
            .ok()?
            .lines()
            .find(|line| line.starts_with("btime "))?
            .split_whitespace()
            .nth(1)?
            .parse()
            .ok()?;
        Some((btime + starttime / 100) as i64)
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use crate::process_tree;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{
        CloseHandle, FILETIME, ERROR_INSUFFICIENT_BUFFER, NO_ERROR,
    };
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID,
        TCP_TABLE_OWNER_PID_LISTENER,
    };
    use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    const TCP_TABLE_READ_RETRIES: usize = 3;

    pub fn list_listeners(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
        let names = process_tree::process_exe_names();
        let mut rows = Vec::new();
        rows.extend(list_ipv4(loopback_only, &names)?);
        rows.extend(list_ipv6(loopback_only, &names)?);
        rows.sort_by_key(|row| (row.port, row.addr.clone(), row.pid));
        Ok(rows)
    }

    fn list_ipv4(
        loopback_only: bool,
        names: &HashMap<u32, String>,
    ) -> Result<Vec<PortRow>, LocalDockError> {
        let table = tcp_table(AF_INET as u32)?;
        let count = read_count(&table);
        let mut rows = Vec::new();

        for idx in 0..count {
            let Some(row) = read_row::<MIB_TCPROW_OWNER_PID>(&table, idx) else {
                break;
            };
            let addr = IpAddr::V4(Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes()));
            if loopback_only && !addr.is_loopback() {
                continue;
            }

            rows.push(make_row(
                port_from_windows(row.dwLocalPort),
                row.dwOwningPid,
                process_details(row.dwOwningPid, names.get(&row.dwOwningPid).map(String::as_str)),
                addr,
            ));
        }

        Ok(rows)
    }

    fn list_ipv6(
        loopback_only: bool,
        names: &HashMap<u32, String>,
    ) -> Result<Vec<PortRow>, LocalDockError> {
        let table = tcp_table(AF_INET6 as u32)?;
        let count = read_count(&table);
        let mut rows = Vec::new();

        for idx in 0..count {
            let Some(row) = read_row::<MIB_TCP6ROW_OWNER_PID>(&table, idx) else {
                break;
            };
            let addr = IpAddr::V6(Ipv6Addr::from(row.ucLocalAddr));
            if loopback_only && !addr.is_loopback() {
                continue;
            }

            rows.push(make_row(
                port_from_windows(row.dwLocalPort),
                row.dwOwningPid,
                process_details(row.dwOwningPid, names.get(&row.dwOwningPid).map(String::as_str)),
                addr,
            ));
        }

        Ok(rows)
    }

    fn tcp_table(address_family: u32) -> Result<Vec<u8>, LocalDockError> {
        let mut size = 0u32;
        let first_status = unsafe {
            GetExtendedTcpTable(
                null_mut(),
                &mut size,
                0,
                address_family,
                TCP_TABLE_OWNER_PID_LISTENER,
                0,
            )
        };

        if first_status != ERROR_INSUFFICIENT_BUFFER && first_status != NO_ERROR {
            return Err(std::io::Error::from_raw_os_error(first_status as i32).into());
        }

        if size == 0 {
            return Ok(Vec::new());
        }

        let mut buffer = vec![0u8; size as usize];
        for _ in 0..TCP_TABLE_READ_RETRIES {
            let status = unsafe {
                GetExtendedTcpTable(
                    buffer.as_mut_ptr().cast(),
                    &mut size,
                    0,
                    address_family,
                    TCP_TABLE_OWNER_PID_LISTENER,
                    0,
                )
            };
            match status {
                NO_ERROR => return Ok(buffer),
                ERROR_INSUFFICIENT_BUFFER => buffer.resize(size as usize, 0),
                _ => return Err(std::io::Error::from_raw_os_error(status as i32).into()),
            }
        }

        Err(std::io::Error::from_raw_os_error(ERROR_INSUFFICIENT_BUFFER as i32).into())
    }

    fn read_count(table: &[u8]) -> usize {
        if table.len() < size_of::<u32>() {
            return 0;
        }
        let count = unsafe { table.as_ptr().cast::<u32>().read_unaligned() };
        count as usize
    }

    fn read_row<T: Copy>(table: &[u8], idx: usize) -> Option<T> {
        let offset = size_of::<u32>().checked_add(idx.checked_mul(size_of::<T>())?)?;
        let end = offset.checked_add(size_of::<T>())?;
        if end > table.len() {
            return None;
        }

        Some(unsafe { table.as_ptr().add(offset).cast::<T>().read_unaligned() })
    }

    fn port_from_windows(port: u32) -> u16 {
        u16::from_be(port as u16)
    }

    fn process_details(pid: u32, fallback_name: Option<&str>) -> ProcessDetails {
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle.is_null() {
            return ProcessDetails {
                name: fallback_name.unwrap_or("").to_string(),
                ..ProcessDetails::default()
            };
        }

        let image_path = process_image_path(handle);
        let command_line = process_command_line(handle);
        let started_unix = process_start_unix(handle);
        unsafe {
            CloseHandle(handle);
        }

        let mut name = Path::new(&image_path)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.is_empty() {
            name = fallback_name.unwrap_or("").to_string();
        }

        ProcessDetails {
            name,
            image_path,
            command_line,
            cwd: String::new(),
            started_unix,
        }
    }

    fn process_start_unix(handle: windows_sys::Win32::Foundation::HANDLE) -> Option<i64> {
        let mut creation = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut exit_time = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut kernel = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut user = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let ok = unsafe { GetProcessTimes(handle, &mut creation, &mut exit_time, &mut kernel, &mut user) };
        if ok == 0 {
            return None;
        }
        let ticks = ((creation.dwHighDateTime as u64) << 32) | u64::from(creation.dwLowDateTime);
        let unix = (ticks / 10_000_000) as i64 - 11_644_473_600;
        (unix > 0).then_some(unix)
    }

    fn process_image_path(handle: windows_sys::Win32::Foundation::HANDLE) -> String {
        let mut path = vec![0u16; 32768];
        let mut len = path.len() as u32;
        let ok = unsafe { QueryFullProcessImageNameW(handle, 0, path.as_mut_ptr(), &mut len) };
        if ok == 0 || len == 0 {
            return String::new();
        }
        path.truncate(len as usize);
        OsString::from_wide(&path).to_string_lossy().into_owned()
    }

    fn process_command_line(handle: windows_sys::Win32::Foundation::HANDLE) -> String {
        const PROCESS_COMMAND_LINE_INFORMATION: u32 = 60;
        let mut size = 0u32;
        unsafe {
            NtQueryInformationProcess(
                handle,
                PROCESS_COMMAND_LINE_INFORMATION,
                null_mut(),
                0,
                &mut size,
            );
        }
        if size == 0 {
            size = 4096;
        }
        let mut buf = vec![0u8; size as usize];
        let mut returned = 0u32;
        let status = unsafe {
            NtQueryInformationProcess(
                handle,
                PROCESS_COMMAND_LINE_INFORMATION,
                buf.as_mut_ptr().cast(),
                buf.len() as u32,
                &mut returned,
            )
        };
        if status != 0 {
            return String::new();
        }

        #[repr(C)]
        struct UnicodeString {
            length: u16,
            maximum_length: u16,
            buffer: *const u16,
        }

        if buf.len() < size_of::<UnicodeString>() {
            return String::new();
        }
        let us = unsafe { buf.as_ptr().cast::<UnicodeString>().read_unaligned() };
        if us.buffer.is_null() || us.length == 0 {
            return String::new();
        }
        let nchars = (us.length as usize) / 2;
        let slice = unsafe { std::slice::from_raw_parts(us.buffer, nchars) };
        String::from_utf16_lossy(slice)
    }

    #[link(name = "ntdll")]
    extern "system" {
        fn NtQueryInformationProcess(
            process: windows_sys::Win32::Foundation::HANDLE,
            info_class: u32,
            info: *mut core::ffi::c_void,
            info_len: u32,
            ret_len: *mut u32,
        ) -> i32;
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod platform {
    use super::*;

    pub fn list_listeners(_loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod host_details_tests {
    use super::*;

    #[test]
    fn parses_host_proc_kv() {
        let (details, btime, tick) = parse_host_proc_details(
            "NAME=electron\nEXE=/usr/share/cursor/cursor\nCWD=/home/x\nCMD=cursor --no-sandbox\nBTIME=10\nSTARTTICK=200\n",
        );
        assert_eq!(details.name, "electron");
        assert_eq!(details.image_path, "/usr/share/cursor/cursor");
        assert_eq!(details.cwd, "/home/x");
        assert_eq!(details.command_line, "cursor --no-sandbox");
        assert_eq!(btime, Some(10));
        assert_eq!(tick, Some(200));
    }
}
