use anyhow::Result;
use chrono::Utc;
use netvan_core::types::TrafficFlow;
use std::collections::HashMap;
use sysinfo::{ProcessesToUpdate, System};

#[cfg(windows)]
mod win {
    use super::*;
    use windows::Win32::Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
    use windows::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
    };
    use windows::Win32::Networking::WinSock::AF_INET;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::core::PWSTR;

    #[repr(C)]
    struct TcpTableOwnerPid {
        dw_num_entries: u32,
        table: [MIB_TCPROW_OWNER_PID; 1],
    }

    pub fn sample_connections(sys: &mut System) -> Result<Vec<TrafficFlow>> {
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let ts = Utc::now().timestamp();
        let mut flows = Vec::new();

        unsafe {
            let mut size: u32 = 0;
            let _ = GetExtendedTcpTable(
                None,
                &mut size,
                false,
                AF_INET.0 as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            );
            if size == 0 {
                return Ok(flows);
            }
            let mut buf = vec![0u8; size as usize];
            let status = GetExtendedTcpTable(
                Some(buf.as_mut_ptr() as *mut _),
                &mut size,
                false,
                AF_INET.0 as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            );
            if status != NO_ERROR.0 && status != ERROR_INSUFFICIENT_BUFFER.0 {
                // still try to parse if buffer filled
            }
            if size as usize > buf.len() {
                return Ok(flows);
            }
            let table = &*(buf.as_ptr() as *const TcpTableOwnerPid);
            let count = table.dw_num_entries as usize;
            let row_size = std::mem::size_of::<MIB_TCPROW_OWNER_PID>();
            let base = buf.as_ptr().add(4) as *const MIB_TCPROW_OWNER_PID;
            for i in 0..count {
                let row = &*base.add(i);
                let pid = row.dwOwningPid;
                if pid == 0 {
                    continue;
                }
                let local = format_ipv4_port(row.dwLocalAddr, row.dwLocalPort);
                let remote = format_ipv4_port(row.dwRemoteAddr, row.dwRemotePort);
                if remote.starts_with("0.0.0.0") || remote.starts_with("127.") {
                    continue;
                }
                let (name, path) = process_info(sys, pid);
                flows.push(TrafficFlow {
                    ts,
                    process_name: name,
                    process_path: path,
                    pid: Some(pid),
                    local_addr: local,
                    remote_addr: remote,
                    protocol: "TCP".into(),
                    bytes_in: 0,
                    bytes_out: estimate_bytes(row_size as u64),
                    nic_id: None,
                    host: None,
                });
            }
        }
        Ok(flows)
    }

    fn estimate_bytes(_hint: u64) -> u64 {
        // Without ESTATS we attribute a nominal sample weight per open connection.
        1024
    }

    fn format_ipv4_port(addr: u32, port: u32) -> String {
        let a = u32::from_be(addr).to_be_bytes();
        let p = u16::from_be((port & 0xFFFF) as u16);
        format!("{}.{}.{}.{}:{}", a[0], a[1], a[2], a[3], p)
    }

    fn process_info(sys: &System, pid: u32) -> (String, Option<String>) {
        use sysinfo::Pid;
        if let Some(p) = sys.process(Pid::from_u32(pid)) {
            let name = p.name().to_string_lossy().to_string();
            let path = p.exe().map(|p| p.to_string_lossy().to_string());
            return (name, path);
        }
        // Fallback OpenProcess
        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut buf = [0u16; 260];
                let mut size = buf.len() as u32;
                let ok = windows::Win32::System::Threading::QueryFullProcessImageNameW(
                    handle,
                    windows::Win32::System::Threading::PROCESS_NAME_FORMAT(0),
                    PWSTR(buf.as_mut_ptr()),
                    &mut size,
                );
                let _ = CloseHandle(handle);
                if ok.is_ok() {
                    let path = String::from_utf16_lossy(&buf[..size as usize]);
                    let name = std::path::Path::new(&path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| format!("pid-{pid}"));
                    return (name, Some(path));
                }
            }
        }
        (format!("pid-{pid}"), None)
    }
}

#[cfg(not(windows))]
mod win {
    use super::*;
    pub fn sample_connections(_sys: &mut System) -> Result<Vec<TrafficFlow>> {
        Ok(Vec::new())
    }
}

pub struct ProcessTrafficCollector {
    sys: System,
    last: HashMap<(u32, String), u64>,
}

impl ProcessTrafficCollector {
    pub fn new() -> Self {
        Self {
            sys: System::new(),
            last: HashMap::new(),
        }
    }

    pub fn sample(&mut self) -> Result<Vec<TrafficFlow>> {
        let flows = win::sample_connections(&mut self.sys)?;
        // Deduplicate by remote for rollup friendliness
        let mut by_key: HashMap<(String, String), TrafficFlow> = HashMap::new();
        for f in flows {
            let key = (f.process_name.clone(), f.remote_addr.clone());
            by_key
                .entry(key)
                .and_modify(|e| {
                    e.bytes_out = e.bytes_out.saturating_add(f.bytes_out);
                    e.bytes_in = e.bytes_in.saturating_add(f.bytes_in);
                })
                .or_insert(f);
        }
        let _ = &self.last;
        Ok(by_key.into_values().collect())
    }
}

impl Default for ProcessTrafficCollector {
    fn default() -> Self {
        Self::new()
    }
}
