use anyhow::Result;
use chrono::Utc;
use netvan_core::types::PingSample;
use std::process::Command;
use std::time::Instant;

/// ICMP ping via Windows `ping.exe`, optionally binding source address for a NIC.
pub async fn ping_once(
    target: &str,
    nic_id: Option<String>,
    source_ip: Option<String>,
) -> Result<PingSample> {
    let target = target.to_string();
    let nic_id_c = nic_id.clone();
    let source = source_ip.clone();
    tokio::task::spawn_blocking(move || ping_blocking(&target, nic_id_c, source)).await?
}

fn ping_blocking(
    target: &str,
    nic_id: Option<String>,
    source_ip: Option<String>,
) -> Result<PingSample> {
    let ts = Utc::now().timestamp();
    let mut cmd = Command::new("ping");
    cmd.args(["-n", "1", "-w", "3000"]);
    if let Some(src) = source_ip {
        cmd.args(["-S", &src]);
    }
    cmd.arg(target);
    let start = Instant::now();
    let output = cmd.output();
    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            let rtt = parse_rtt_ms(&text).or_else(|| {
                if out.status.success() {
                    Some(start.elapsed().as_secs_f64() * 1000.0)
                } else {
                    None
                }
            });
            let success = rtt.is_some() && out.status.success();
            Ok(PingSample {
                nic_id,
                target: target.to_string(),
                ts,
                rtt_ms: rtt,
                success,
                error: if success {
                    None
                } else {
                    Some(String::from_utf8_lossy(&out.stderr).trim().to_string())
                },
            })
        }
        Err(e) => Ok(PingSample {
            nic_id,
            target: target.to_string(),
            ts,
            rtt_ms: None,
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

fn parse_rtt_ms(text: &str) -> Option<f64> {
    // English: time=12ms or time<1ms; also Average = 12ms
    for line in text.lines() {
        let lower = line.to_lowercase();
        if let Some(idx) = lower.find("time=") {
            let rest = &lower[idx + 5..];
            let num: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(v) = num.parse::<f64>() {
                return Some(v);
            }
            if rest.starts_with('<') {
                return Some(1.0);
            }
        }
        if let Some(idx) = lower.find("time<") {
            let _ = idx;
            return Some(1.0);
        }
    }
    None
}
