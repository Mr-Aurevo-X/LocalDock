use crate::LocalDockError;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRow {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub addr: String,
    pub is_loopback: bool,
}

pub fn list_listeners(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
    platform::list_listeners(loopback_only)
}

fn make_row(port: u16, pid: u32, process_name: String, addr: IpAddr) -> PortRow {
    PortRow {
        port,
        pid,
        process_name,
        addr: addr.to_string(),
        is_loopback: addr.is_loopback(),
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
        inode_owners: &HashMap<u64, (u32, String)>,
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
            let (pid, process_name) = inode_owners
                .get(&inode)
                .cloned()
                .unwrap_or((0, String::new()));
            rows.push(make_row(port, pid, process_name, addr));
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

    fn inode_owners() -> HashMap<u64, (u32, String)> {
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

            let process_name = process_name(pid);
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
                owners.entry(inode).or_insert((pid, process_name.clone()));
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

    fn process_name(pid: u32) -> String {
        fs::read_to_string(format!("/proc/{pid}/comm"))
            .map(|name| name.trim().to_string())
            .unwrap_or_default()
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::ffi::OsString;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID,
        TCP_TABLE_OWNER_PID_LISTENER,
    };
    use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    const TCP_TABLE_READ_RETRIES: usize = 3;

    pub fn list_listeners(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
        let mut rows = Vec::new();
        rows.extend(list_ipv4(loopback_only)?);
        rows.extend(list_ipv6(loopback_only)?);
        rows.sort_by_key(|row| (row.port, row.addr.clone(), row.pid));
        Ok(rows)
    }

    fn list_ipv4(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
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
                process_name(row.dwOwningPid),
                addr,
            ));
        }

        Ok(rows)
    }

    fn list_ipv6(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
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
                process_name(row.dwOwningPid),
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

    fn process_name(pid: u32) -> String {
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle.is_null() {
            return String::new();
        }

        let mut path = vec![0u16; 32768];
        let mut len = path.len() as u32;
        let ok = unsafe { QueryFullProcessImageNameW(handle, 0, path.as_mut_ptr(), &mut len) };
        unsafe {
            CloseHandle(handle);
        }
        if ok == 0 || len == 0 {
            return String::new();
        }

        path.truncate(len as usize);
        let full_path = OsString::from_wide(&path);
        Path::new(&full_path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod platform {
    use super::*;

    pub fn list_listeners(_loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError> {
        Ok(Vec::new())
    }
}
