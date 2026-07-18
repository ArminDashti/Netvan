use anyhow::{bail, Context, Result};
use chrono::Utc;
use netvan_core::paths::data_dir;
use netvan_core::types::SpeedtestResult;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn resolve_cli_path(configured: Option<&str>) -> PathBuf {
    if let Some(p) = configured {
        let path = PathBuf::from(p);
        if path.exists() {
            return path;
        }
    }
    // Common install / vendor locations
    let candidates = [
        data_dir().join("speedtest.exe"),
        data_dir().join("ookla").join("speedtest.exe"),
        PathBuf::from("speedtest.exe"),
        PathBuf::from(r"C:\Program Files\Ookla\Speedtest CLI\speedtest.exe"),
    ];
    for c in candidates {
        if c.exists() {
            return c;
        }
    }
    PathBuf::from("speedtest")
}

pub async fn run_speedtest(
    nic_id: Option<String>,
    server_id: Option<String>,
    cli_path: Option<String>,
    accept_eula: bool,
) -> Result<SpeedtestResult> {
    let path = resolve_cli_path(cli_path.as_deref());
    tokio::task::spawn_blocking(move || {
        run_speedtest_blocking(&path, nic_id, server_id, accept_eula)
    })
    .await?
}

fn run_speedtest_blocking(
    cli: &Path,
    nic_id: Option<String>,
    server_id: Option<String>,
    accept_eula: bool,
) -> Result<SpeedtestResult> {
    if which_exists(cli).is_none() && !cli.exists() {
        bail!(
            "Ookla Speedtest CLI not found. Install from https://www.speedtest.net/apps/cli \
             and place speedtest.exe in PATH or Netvan data dir ({}), or set path in Settings.",
            data_dir().display()
        );
    }

    let mut args = vec![
        "--format=json".to_string(),
        "--progress=no".to_string(),
    ];
    if accept_eula {
        args.push("--accept-license".into());
        args.push("--accept-gdpr".into());
    }
    if let Some(id) = server_id {
        args.push("--server-id".into());
        args.push(id);
    }

    let output = Command::new(cli)
        .args(&args)
        .output()
        .with_context(|| format!("spawn {}", cli.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() && stdout.trim().is_empty() {
        bail!("speedtest failed: {stderr}");
    }

    let json: Value = serde_json::from_str(stdout.trim())
        .with_context(|| format!("parse speedtest json: {stdout}"))?;

    let download_bps = json
        .pointer("/download/bandwidth")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let upload_bps = json
        .pointer("/upload/bandwidth")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    // Ookla CLI bandwidth is bytes/sec
    let download_mbps = download_bps * 8.0 / 1_000_000.0;
    let upload_mbps = upload_bps * 8.0 / 1_000_000.0;
    let ping_ms = json
        .pointer("/ping/latency")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let jitter_ms = json.pointer("/ping/jitter").and_then(|v| v.as_f64());
    let packet_loss = json.pointer("/packetLoss").and_then(|v| v.as_f64());
    let server_id = json
        .pointer("/server/id")
        .map(|v| v.to_string().trim_matches('"').to_string());
    let server_name = json
        .pointer("/server/name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(SpeedtestResult {
        id: 0,
        nic_id,
        ts: Utc::now().timestamp(),
        server_id,
        server_name,
        download_mbps,
        upload_mbps,
        ping_ms,
        jitter_ms,
        packet_loss,
        raw_json: Some(stdout.trim().to_string()),
    })
}

fn which_exists(cli: &Path) -> Option<PathBuf> {
    if cli.exists() {
        return Some(cli.to_path_buf());
    }
    // Try PATH lookup via `where` on Windows
    let name = cli.file_name()?.to_string_lossy().to_string();
    let out = Command::new("where").arg(&name).output().ok()?;
    if out.status.success() {
        let line = String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()?
            .trim()
            .to_string();
        if !line.is_empty() {
            return Some(PathBuf::from(line));
        }
    }
    None
}
