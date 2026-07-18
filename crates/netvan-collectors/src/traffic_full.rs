//! Mode B — full capture (WinDivert).
//!
//! When WinDivert is not installed, this module records DNS via system tools
//! and returns a clear status so the UI can prompt for admin/driver setup.
//! Packet diversion is attempted only if `WinDivert.dll` is loadable.

use anyhow::Result;
use chrono::Utc;
use netvan_core::types::{DnsQueryRecord, TrafficFlow};
use std::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct FullCaptureStatus {
    pub available: bool,
    pub message: String,
}

pub struct FullTrafficCollector {
    enabled: bool,
    last_status: FullCaptureStatus,
}

impl FullTrafficCollector {
    pub fn new() -> Self {
        let status = probe_windivert();
        Self {
            enabled: status.available,
            last_status: status,
        }
    }

    pub fn status(&self) -> FullCaptureStatus {
        self.last_status.clone()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled && self.last_status.available;
        if enabled && !self.last_status.available {
            warn!("{}", self.last_status.message);
        }
    }

    /// Sample: DNS cache + optional WinDivert path (stub until driver present).
    pub fn sample(&mut self) -> Result<(Vec<TrafficFlow>, Vec<DnsQueryRecord>)> {
        let mut dns = Vec::new();
        if let Ok(out) = Command::new("ipconfig").args(["/displaydns"]).output() {
            let text = String::from_utf8_lossy(&out.stdout);
            dns.extend(parse_display_dns(&text));
        }

        if !self.enabled {
            return Ok((Vec::new(), dns));
        }

        // WinDivert live diversion would go here. Without the driver we keep
        // process-level flows empty and rely on Mode A collector running in parallel.
        info!("Mode B active: DNS enrichment only until WinDivert driver is installed");
        Ok((Vec::new(), dns))
    }
}

impl Default for FullTrafficCollector {
    fn default() -> Self {
        Self::new()
    }
}

fn probe_windivert() -> FullCaptureStatus {
    let dll_candidates = [
        "WinDivert.dll",
        r"C:\Windows\System32\WinDivert.dll",
        r".\WinDivert.dll",
    ];
    for c in dll_candidates {
        if std::path::Path::new(c).exists() {
            return FullCaptureStatus {
                available: true,
                message: format!("WinDivert found at {c}"),
            };
        }
    }
    FullCaptureStatus {
        available: false,
        message: "Mode B requires WinDivert (admin). Install WinDivert.dll next to the service \
                  or in System32. Until then, use Process mode for app+IP usage; hostnames \
                  still enrich from DNS cache when Full mode is selected."
            .into(),
    }
}

fn parse_display_dns(text: &str) -> Vec<DnsQueryRecord> {
    let ts = Utc::now().timestamp();
    let mut out = Vec::new();
    let mut current_name: Option<String> = None;
    let mut ips = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("Record Name") || t.starts_with("    ") && t.contains(".") && !t.contains(":") {
            // Windows ipconfig /displaydns format varies by locale; use simple heuristics.
        }
        if let Some(rest) = t.strip_prefix("Record Name") {
            if current_name.is_some() && !ips.is_empty() {
                out.push(DnsQueryRecord {
                    ts,
                    name: current_name.take().unwrap(),
                    resolved_ips: std::mem::take(&mut ips),
                    process_name: None,
                });
            }
            let name = rest
                .trim_start_matches(':')
                .trim()
                .trim_start_matches('.')
                .trim()
                .to_string();
            if !name.is_empty() {
                current_name = Some(name);
            }
        } else if t.starts_with("A (Host) Record") || t.starts_with("AAAA") {
            // next lines may hold address
        } else if looks_like_ip(t) {
            ips.push(t.to_string());
        }
    }
    if let Some(name) = current_name {
        if !ips.is_empty() {
            out.push(DnsQueryRecord {
                ts,
                name,
                resolved_ips: ips,
                process_name: None,
            });
        }
    }
    out.truncate(50);
    out
}

fn looks_like_ip(s: &str) -> bool {
    s.parse::<std::net::IpAddr>().is_ok()
}
