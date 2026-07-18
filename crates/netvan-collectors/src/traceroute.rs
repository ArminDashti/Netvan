use anyhow::Result;
use chrono::Utc;
use netvan_core::types::{HttpLatencySample, TracerouteHop, TracerouteResult};
use std::process::Command;

pub async fn traceroute(
    target: &str,
    nic_id: Option<String>,
    max_hops: u8,
) -> Result<TracerouteResult> {
    let target = target.to_string();
    let nic_id_c = nic_id.clone();
    tokio::task::spawn_blocking(move || traceroute_blocking(&target, nic_id_c, max_hops)).await?
}

fn traceroute_blocking(
    target: &str,
    nic_id: Option<String>,
    max_hops: u8,
) -> Result<TracerouteResult> {
    let ts = Utc::now().timestamp();
    let output = Command::new("tracert")
        .args(["-d", "-h", &max_hops.to_string(), "-w", "2000", target])
        .output();

    let hops = match output {
        Ok(out) => parse_tracert(&String::from_utf8_lossy(&out.stdout)),
        Err(_) => Vec::new(),
    };

    Ok(TracerouteResult {
        id: 0,
        nic_id,
        target: target.to_string(),
        ts,
        hops,
    })
}

fn parse_tracert(text: &str) -> Vec<TracerouteHop> {
    let mut hops = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        if let Ok(hop) = parts[0].parse::<u8>() {
            let mut rtt = None;
            let mut address = None;
            for p in &parts[1..] {
                if *p == "*" || *p == "<1" || p.ends_with("ms") {
                    if p.ends_with("ms") {
                        let num = p.trim_end_matches("ms").replace('<', "");
                        if let Ok(v) = num.parse::<f64>() {
                            rtt = Some(v);
                        }
                    }
                } else if p.contains('.') || p.contains(':') {
                    address = Some((*p).to_string());
                }
            }
            hops.push(TracerouteHop {
                hop,
                address,
                hostname: None,
                rtt_ms: rtt,
            });
        }
    }
    hops
}

/// Re-export alias used by tools page.
pub type OneShotHttp = HttpLatencySample;
