use anyhow::Result;
use chrono::Utc;
use netvan_core::db::Database;
use netvan_core::history::{HistoryRange, TimeRange};
use netvan_core::ipc::{RpcRequest, RpcResponse};
use netvan_core::settings::{AppSettings, CaptureMode};
use netvan_core::types::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::http_latency;
use crate::nic::NicCollector;
use crate::nslookup;
use crate::ping;
use crate::speedtest;
use crate::traceroute;
use crate::traffic_full::FullTrafficCollector;
use crate::traffic_process::ProcessTrafficCollector;

pub struct CollectorEngine {
    db: Database,
    nic: NicCollector,
    process_traffic: Mutex<ProcessTrafficCollector>,
    full_traffic: Mutex<FullTrafficCollector>,
    last_oper: RwLock<HashMap<String, String>>,
    settings: RwLock<AppSettings>,
    running: RwLock<bool>,
}

impl CollectorEngine {
    pub fn new(db: Database) -> Result<Arc<Self>> {
        let settings = db.get_settings()?;
        Ok(Arc::new(Self {
            db,
            nic: NicCollector::new(),
            process_traffic: Mutex::new(ProcessTrafficCollector::new()),
            full_traffic: Mutex::new(FullTrafficCollector::new()),
            last_oper: RwLock::new(HashMap::new()),
            settings: RwLock::new(settings),
            running: RwLock::new(true),
        }))
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub async fn start_background(self: Arc<Self>) {
        let eng = self.clone();
        tokio::spawn(async move {
            loop {
                if !*eng.running.read() {
                    break;
                }
                if let Err(e) = eng.tick_bandwidth().await {
                    error!("bandwidth tick: {e:#}");
                }
                let ms = eng.settings.read().bandwidth_interval_ms;
                tokio::time::sleep(Duration::from_millis(ms.max(500))).await;
            }
        });

        let eng = self.clone();
        tokio::spawn(async move {
            loop {
                if !*eng.running.read() {
                    break;
                }
                if let Err(e) = eng.tick_ping().await {
                    error!("ping tick: {e:#}");
                }
                let secs = eng.settings.read().ping_interval_secs;
                tokio::time::sleep(Duration::from_secs(secs.max(2))).await;
            }
        });

        let eng = self.clone();
        tokio::spawn(async move {
            loop {
                if !*eng.running.read() {
                    break;
                }
                if let Err(e) = eng.tick_http().await {
                    error!("http tick: {e:#}");
                }
                let secs = eng.settings.read().http_interval_secs;
                tokio::time::sleep(Duration::from_secs(secs.max(5))).await;
            }
        });

        let eng = self.clone();
        tokio::spawn(async move {
            loop {
                if !*eng.running.read() {
                    break;
                }
                if let Err(e) = eng.tick_traffic().await {
                    error!("traffic tick: {e:#}");
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        let eng = self.clone();
        tokio::spawn(async move {
            loop {
                if !*eng.running.read() {
                    break;
                }
                let days = eng.settings.read().retention_raw_days;
                if let Err(e) = eng.db.prune_raw(days) {
                    error!("prune: {e:#}");
                }
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        });

        info!("collector background tasks started");
    }

    pub fn stop(&self) {
        *self.running.write() = false;
    }

    async fn tick_bandwidth(&self) -> Result<()> {
        let nics = self.nic.list()?;
        let mut last = self.last_oper.write();
        for n in &nics {
            self.db.upsert_nic(n)?;
            if let Some(prev) = last.get(&n.id) {
                if prev != &n.oper_status {
                    let event = if n.oper_status == "Up" {
                        "connected"
                    } else {
                        "disconnected"
                    };
                    self.db.insert_link_event(&LinkEvent {
                        nic_id: n.id.clone(),
                        ts: Utc::now().timestamp(),
                        event: event.into(),
                        detail: Some(format!("{prev} -> {}", n.oper_status)),
                    })?;
                }
            }
            last.insert(n.id.clone(), n.oper_status.clone());
        }
        drop(last);
        for s in NicCollector::sample_bandwidth(&nics) {
            self.db.insert_bandwidth(&s)?;
        }
        Ok(())
    }

    async fn tick_ping(&self) -> Result<()> {
        let settings = self.settings.read().clone();
        let nics = self.nic.list().unwrap_or_default();
        let up_nics: Vec<_> = nics
            .into_iter()
            .filter(|n| n.oper_status == "Up" && n.media_type != "Loopback")
            .collect();
        for target in &settings.ping_targets {
            if up_nics.is_empty() {
                let sample = ping::ping_once(target, None, None).await?;
                self.db.insert_ping(&sample)?;
            } else {
                for nic in &up_nics {
                    let src = nic.ipv4_addresses.first().cloned();
                    let sample = ping::ping_once(target, Some(nic.id.clone()), src).await?;
                    self.db.insert_ping(&sample)?;
                }
            }
        }
        Ok(())
    }

    async fn tick_http(&self) -> Result<()> {
        let settings = self.settings.read().clone();
        let default_nic = settings.default_nic_id.clone();
        for url in &settings.http_targets {
            let sample = http_latency::measure_http(url, default_nic.clone(), None).await?;
            self.db.insert_http_latency(&sample)?;
        }
        Ok(())
    }

    async fn tick_traffic(&self) -> Result<()> {
        let mode = self.settings.read().capture_mode;
        {
            let mut pt = self.process_traffic.lock().await;
            for f in pt.sample()? {
                self.db.insert_traffic_flow(&f)?;
            }
        }
        if mode == CaptureMode::Full {
            let mut ft = self.full_traffic.lock().await;
            ft.set_enabled(true);
            let (flows, dns) = ft.sample()?;
            for f in flows {
                self.db.insert_traffic_flow(&f)?;
            }
            for d in dns {
                self.db
                    .insert_dns_query(&d.name, &d.resolved_ips, d.process_name.as_deref())?;
                if let Some(ip) = d.resolved_ips.first() {
                    self.db
                        .insert_http_host(&d.name, Some(ip), d.process_name.as_deref())?;
                }
            }
        }
        Ok(())
    }

    pub async fn handle(&self, req: RpcRequest) -> RpcResponse {
        match self.handle_inner(req).await {
            Ok(r) => r,
            Err(e) => RpcResponse::Error {
                message: format!("{e:#}"),
            },
        }
    }

    async fn handle_inner(&self, req: RpcRequest) -> Result<RpcResponse> {
        Ok(match req {
            RpcRequest::Ping => RpcResponse::Pong,
            RpcRequest::GetStatus => {
                let mode = self.settings.read().capture_mode;
                let full = self.full_traffic.lock().await.status();
                let msg = if mode == CaptureMode::Full {
                    full.message
                } else {
                    "Process capture mode".into()
                };
                RpcResponse::Status(ServiceStatus {
                    running: *self.running.read(),
                    pipe_connected: true,
                    capture_mode: mode.as_str().into(),
                    message: msg,
                })
            }
            RpcRequest::GetSettings => RpcResponse::Settings(self.settings.read().clone()),
            RpcRequest::SetSettings { settings } => {
                self.db.save_settings(&settings)?;
                *self.settings.write() = settings;
                RpcResponse::Ok
            }
            RpcRequest::SetCaptureMode { mode } => {
                let mut s = self.settings.read().clone();
                s.capture_mode = mode;
                self.db.save_settings(&s)?;
                *self.settings.write() = s;
                RpcResponse::Ok
            }
            RpcRequest::ListNics => {
                let nics = self.nic.list()?;
                for n in &nics {
                    let _ = self.db.upsert_nic(n);
                }
                RpcResponse::Nics(nics)
            }
            RpcRequest::GetNic { nic_id } => match self.nic.find(&nic_id)? {
                Some(n) => RpcResponse::Nic(n),
                None => RpcResponse::Error {
                    message: format!("NIC not found: {nic_id}"),
                },
            },
            RpcRequest::GetBandwidthHistory {
                nic_id,
                range,
                start_ts,
                end_ts,
            } => {
                let tr = TimeRange::from_history(range, start_ts, end_ts);
                RpcResponse::BandwidthHistory(
                    self.db.bandwidth_history(nic_id.as_deref(), &tr)?,
                )
            }
            RpcRequest::GetPingHistory {
                nic_id,
                range,
                start_ts,
                end_ts,
            } => {
                let tr = TimeRange::from_history(range, start_ts, end_ts);
                RpcResponse::PingHistory(self.db.ping_history(nic_id.as_deref(), &tr)?)
            }
            RpcRequest::GetHttpLatencyHistory {
                nic_id,
                range,
                start_ts,
                end_ts,
            } => {
                let tr = TimeRange::from_history(range, start_ts, end_ts);
                RpcResponse::HttpLatencyHistory(
                    self.db.http_latency_history(nic_id.as_deref(), &tr)?,
                )
            }
            RpcRequest::GetLinkEvents {
                nic_id,
                range,
                start_ts,
                end_ts,
            } => {
                let tr = TimeRange::from_history(range, start_ts, end_ts);
                RpcResponse::LinkEvents(self.db.link_events(nic_id.as_deref(), &tr)?)
            }
            RpcRequest::GetAppUsage {
                range,
                start_ts,
                end_ts,
                group_by,
            } => {
                let tr = TimeRange::from_history(range, start_ts, end_ts);
                RpcResponse::AppUsage(self.db.app_usage(&tr, &group_by)?)
            }
            RpcRequest::RunPing { target, nic_id } => {
                let src = if let Some(ref id) = nic_id {
                    self.nic
                        .find(id)?
                        .and_then(|n| n.ipv4_addresses.first().cloned())
                } else {
                    None
                };
                let sample = ping::ping_once(&target, nic_id, src).await?;
                self.db.insert_ping(&sample)?;
                RpcResponse::PingResult(sample)
            }
            RpcRequest::RunHttpLatency { url, nic_id } => {
                let sample = http_latency::measure_http(&url, nic_id, None).await?;
                self.db.insert_http_latency(&sample)?;
                RpcResponse::HttpLatencyResult(sample)
            }
            RpcRequest::RunTraceroute {
                target,
                nic_id,
                max_hops,
            } => {
                let mut result =
                    traceroute::traceroute(&target, nic_id, max_hops.unwrap_or(30)).await?;
                let id = self.db.insert_traceroute(&result)?;
                result.id = id;
                RpcResponse::Traceroute(result)
            }
            RpcRequest::RunNslookup { query } => {
                RpcResponse::Nslookup(nslookup::nslookup(&query).await?)
            }
            RpcRequest::RunSpeedtest {
                nic_id,
                server_id,
                accept_eula,
            } => {
                let mut settings = self.settings.read().clone();
                if accept_eula {
                    settings.speedtest_eula_accepted = true;
                    self.db.save_settings(&settings)?;
                    *self.settings.write() = settings.clone();
                }
                if !settings.speedtest_eula_accepted && !accept_eula {
                    return Ok(RpcResponse::Error {
                        message: "Accept Ookla Speedtest EULA/GDPR first".into(),
                    });
                }
                let mut result = speedtest::run_speedtest(
                    nic_id,
                    server_id,
                    settings.speedtest_cli_path.clone(),
                    true,
                )
                .await?;
                let id = self.db.insert_speedtest(&result)?;
                result.id = id;
                RpcResponse::Speedtest(result)
            }
            RpcRequest::GetSpeedtestHistory {
                range,
                start_ts,
                end_ts,
            } => {
                let tr = TimeRange::from_history(range, start_ts, end_ts);
                RpcResponse::SpeedtestHistory(self.db.speedtest_history(&tr)?)
            }
            RpcRequest::AcceptSpeedtestEula => {
                let mut s = self.settings.read().clone();
                s.speedtest_eula_accepted = true;
                self.db.save_settings(&s)?;
                *self.settings.write() = s;
                RpcResponse::Ok
            }
        })
    }

    #[allow(dead_code)]
    pub fn history_range_demo() -> HistoryRange {
        HistoryRange::Today
    }
}
