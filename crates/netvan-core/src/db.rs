use anyhow::{Context, Result};
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Arc;

use crate::history::TimeRange;
use crate::settings::{AppSettings, CaptureMode};
use crate::types::*;

const SCHEMA: &str = r#"
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS nics (
  id TEXT PRIMARY KEY,
  guid TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  interface_index INTEGER,
  mac TEXT,
  mtu INTEGER,
  media_type TEXT,
  oper_status TEXT,
  admin_status TEXT,
  link_speed_bps INTEGER,
  ipv4_json TEXT,
  ipv6_json TEXT,
  gateways_json TEXT,
  dns_json TEXT,
  dhcp_enabled INTEGER,
  wifi_ssid TEXT,
  wifi_bssid TEXT,
  wifi_signal INTEGER,
  driver TEXT,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS nic_bandwidth_samples (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  nic_id TEXT NOT NULL,
  ts INTEGER NOT NULL,
  rx_bytes INTEGER NOT NULL,
  tx_bytes INTEGER NOT NULL,
  rx_bps REAL NOT NULL,
  tx_bps REAL NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_bw_nic_ts ON nic_bandwidth_samples(nic_id, ts);

CREATE TABLE IF NOT EXISTS nic_usage_hourly (
  nic_id TEXT NOT NULL,
  hour_ts INTEGER NOT NULL,
  rx_bytes INTEGER NOT NULL,
  tx_bytes INTEGER NOT NULL,
  PRIMARY KEY (nic_id, hour_ts)
);

CREATE TABLE IF NOT EXISTS ping_samples (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  nic_id TEXT,
  target TEXT NOT NULL,
  ts INTEGER NOT NULL,
  rtt_ms REAL,
  success INTEGER NOT NULL,
  error TEXT
);
CREATE INDEX IF NOT EXISTS idx_ping_ts ON ping_samples(ts);
CREATE INDEX IF NOT EXISTS idx_ping_nic_ts ON ping_samples(nic_id, ts);

CREATE TABLE IF NOT EXISTS http_latency_samples (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  nic_id TEXT,
  url TEXT NOT NULL,
  ts INTEGER NOT NULL,
  dns_ms REAL,
  connect_ms REAL,
  tls_ms REAL,
  ttfb_ms REAL,
  total_ms REAL,
  status_code INTEGER,
  success INTEGER NOT NULL,
  error TEXT
);
CREATE INDEX IF NOT EXISTS idx_http_ts ON http_latency_samples(ts);

CREATE TABLE IF NOT EXISTS nic_link_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  nic_id TEXT NOT NULL,
  ts INTEGER NOT NULL,
  event TEXT NOT NULL,
  detail TEXT
);
CREATE INDEX IF NOT EXISTS idx_link_nic_ts ON nic_link_events(nic_id, ts);

CREATE TABLE IF NOT EXISTS traceroute_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  nic_id TEXT,
  target TEXT NOT NULL,
  ts INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS traceroute_hops (
  run_id INTEGER NOT NULL,
  hop INTEGER NOT NULL,
  address TEXT,
  hostname TEXT,
  rtt_ms REAL,
  FOREIGN KEY(run_id) REFERENCES traceroute_runs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS speedtest_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  nic_id TEXT,
  ts INTEGER NOT NULL,
  server_id TEXT,
  server_name TEXT,
  download_mbps REAL NOT NULL,
  upload_mbps REAL NOT NULL,
  ping_ms REAL NOT NULL,
  jitter_ms REAL,
  packet_loss REAL,
  raw_json TEXT
);

CREATE TABLE IF NOT EXISTS traffic_flows (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts INTEGER NOT NULL,
  process_name TEXT NOT NULL,
  process_path TEXT,
  pid INTEGER,
  local_addr TEXT NOT NULL,
  remote_addr TEXT NOT NULL,
  protocol TEXT NOT NULL,
  bytes_in INTEGER NOT NULL,
  bytes_out INTEGER NOT NULL,
  nic_id TEXT,
  host TEXT
);
CREATE INDEX IF NOT EXISTS idx_flows_ts ON traffic_flows(ts);

CREATE TABLE IF NOT EXISTS dns_queries (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts INTEGER NOT NULL,
  name TEXT NOT NULL,
  resolved_ips TEXT,
  process_name TEXT
);

CREATE TABLE IF NOT EXISTS http_hosts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts INTEGER NOT NULL,
  host TEXT NOT NULL,
  remote_ip TEXT,
  process_name TEXT
);

CREATE TABLE IF NOT EXISTS app_usage_hourly (
  hour_ts INTEGER NOT NULL,
  process_name TEXT NOT NULL,
  remote_ip TEXT NOT NULL DEFAULT '',
  host TEXT NOT NULL DEFAULT '',
  nic_id TEXT NOT NULL DEFAULT '',
  bytes_in INTEGER NOT NULL,
  bytes_out INTEGER NOT NULL,
  PRIMARY KEY (hour_ts, process_name, remote_ip, host, nic_id)
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
"#;

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path).context("open sqlite")?;
        conn.execute_batch(SCHEMA)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.ensure_default_settings()?;
        Ok(db)
    }

    pub fn open_default() -> Result<Self> {
        crate::paths::ensure_data_dir()?;
        Self::open(crate::paths::db_path())
    }

    fn ensure_default_settings(&self) -> Result<()> {
        let settings = self.get_settings()?;
        self.save_settings(&settings)
    }

    pub fn get_settings(&self) -> Result<AppSettings> {
        let conn = self.conn.lock();
        let mut defaults = AppSettings::default();
        if let Some(v) = get_setting(&conn, "capture_mode")? {
            defaults.capture_mode = CaptureMode::parse(&v);
        }
        if let Some(v) = get_setting(&conn, "ping_targets")? {
            if let Ok(list) = serde_json::from_str::<Vec<String>>(&v) {
                defaults.ping_targets = list;
            }
        }
        if let Some(v) = get_setting(&conn, "http_targets")? {
            if let Ok(list) = serde_json::from_str::<Vec<String>>(&v) {
                defaults.http_targets = list;
            }
        }
        if let Some(v) = get_setting(&conn, "ping_interval_secs")? {
            if let Ok(n) = v.parse() {
                defaults.ping_interval_secs = n;
            }
        }
        if let Some(v) = get_setting(&conn, "http_interval_secs")? {
            if let Ok(n) = v.parse() {
                defaults.http_interval_secs = n;
            }
        }
        if let Some(v) = get_setting(&conn, "bandwidth_interval_ms")? {
            if let Ok(n) = v.parse() {
                defaults.bandwidth_interval_ms = n;
            }
        }
        if let Some(v) = get_setting(&conn, "retention_raw_days")? {
            if let Ok(n) = v.parse() {
                defaults.retention_raw_days = n;
            }
        }
        if let Some(v) = get_setting(&conn, "start_ui_with_windows")? {
            defaults.start_ui_with_windows = v == "1" || v == "true";
        }
        if let Some(v) = get_setting(&conn, "speedtest_cli_path")? {
            defaults.speedtest_cli_path = Some(v);
        }
        if let Some(v) = get_setting(&conn, "speedtest_eula_accepted")? {
            defaults.speedtest_eula_accepted = v == "1" || v == "true";
        }
        if let Some(v) = get_setting(&conn, "default_nic_id")? {
            defaults.default_nic_id = Some(v);
        }
        Ok(defaults)
    }

    pub fn save_settings(&self, s: &AppSettings) -> Result<()> {
        let conn = self.conn.lock();
        set_setting(&conn, "capture_mode", s.capture_mode.as_str())?;
        set_setting(
            &conn,
            "ping_targets",
            &serde_json::to_string(&s.ping_targets)?,
        )?;
        set_setting(
            &conn,
            "http_targets",
            &serde_json::to_string(&s.http_targets)?,
        )?;
        set_setting(
            &conn,
            "ping_interval_secs",
            &s.ping_interval_secs.to_string(),
        )?;
        set_setting(
            &conn,
            "http_interval_secs",
            &s.http_interval_secs.to_string(),
        )?;
        set_setting(
            &conn,
            "bandwidth_interval_ms",
            &s.bandwidth_interval_ms.to_string(),
        )?;
        set_setting(
            &conn,
            "retention_raw_days",
            &s.retention_raw_days.to_string(),
        )?;
        set_setting(
            &conn,
            "start_ui_with_windows",
            if s.start_ui_with_windows { "1" } else { "0" },
        )?;
        if let Some(ref p) = s.speedtest_cli_path {
            set_setting(&conn, "speedtest_cli_path", p)?;
        }
        set_setting(
            &conn,
            "speedtest_eula_accepted",
            if s.speedtest_eula_accepted { "1" } else { "0" },
        )?;
        if let Some(ref id) = s.default_nic_id {
            set_setting(&conn, "default_nic_id", id)?;
        }
        Ok(())
    }

    pub fn upsert_nic(&self, nic: &NicInfo) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            r#"INSERT INTO nics (
                id, guid, name, description, interface_index, mac, mtu, media_type,
                oper_status, admin_status, link_speed_bps, ipv4_json, ipv6_json,
                gateways_json, dns_json, dhcp_enabled, wifi_ssid, wifi_bssid,
                wifi_signal, driver, updated_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)
            ON CONFLICT(id) DO UPDATE SET
                name=excluded.name, description=excluded.description,
                interface_index=excluded.interface_index, mac=excluded.mac, mtu=excluded.mtu,
                media_type=excluded.media_type, oper_status=excluded.oper_status,
                admin_status=excluded.admin_status, link_speed_bps=excluded.link_speed_bps,
                ipv4_json=excluded.ipv4_json, ipv6_json=excluded.ipv6_json,
                gateways_json=excluded.gateways_json, dns_json=excluded.dns_json,
                dhcp_enabled=excluded.dhcp_enabled, wifi_ssid=excluded.wifi_ssid,
                wifi_bssid=excluded.wifi_bssid, wifi_signal=excluded.wifi_signal,
                driver=excluded.driver, updated_at=excluded.updated_at
            "#,
            params![
                nic.id,
                nic.guid,
                nic.name,
                nic.description,
                nic.interface_index,
                nic.mac,
                nic.mtu,
                nic.media_type,
                nic.oper_status,
                nic.admin_status,
                nic.link_speed_bps.map(|v| v as i64),
                serde_json::to_string(&nic.ipv4_addresses)?,
                serde_json::to_string(&nic.ipv6_addresses)?,
                serde_json::to_string(&nic.gateways)?,
                serde_json::to_string(&nic.dns_servers)?,
                nic.dhcp_enabled as i32,
                nic.wifi_ssid,
                nic.wifi_bssid,
                nic.wifi_signal,
                nic.driver,
                Utc::now().timestamp(),
            ],
        )?;
        Ok(())
    }

    pub fn list_nics_cached(&self) -> Result<Vec<NicInfo>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, guid, name, description, interface_index, mac, mtu, media_type,
             oper_status, admin_status, link_speed_bps, ipv4_json, ipv6_json, gateways_json,
             dns_json, dhcp_enabled, wifi_ssid, wifi_bssid, wifi_signal, driver
             FROM nics ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(NicInfo {
                id: row.get(0)?,
                guid: row.get(1)?,
                name: row.get(2)?,
                description: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                interface_index: row.get::<_, Option<u32>>(4)?.unwrap_or(0),
                mac: row.get(5)?,
                mtu: row.get(6)?,
                media_type: row.get::<_, Option<String>>(7)?.unwrap_or_else(|| "Unknown".into()),
                oper_status: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "Unknown".into()),
                admin_status: row.get::<_, Option<String>>(9)?.unwrap_or_else(|| "Unknown".into()),
                link_speed_bps: row
                    .get::<_, Option<i64>>(10)?
                    .map(|v| v as u64),
                ipv4_addresses: parse_json_vec(row.get(11)?),
                ipv6_addresses: parse_json_vec(row.get(12)?),
                gateways: parse_json_vec(row.get(13)?),
                dns_servers: parse_json_vec(row.get(14)?),
                dhcp_enabled: row.get::<_, Option<i32>>(15)?.unwrap_or(0) != 0,
                wifi_ssid: row.get(16)?,
                wifi_bssid: row.get(17)?,
                wifi_signal: row.get(18)?,
                driver: row.get(19)?,
                rx_bytes: 0,
                tx_bytes: 0,
                rx_bps: 0.0,
                tx_bps: 0.0,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn insert_bandwidth(&self, s: &BandwidthSample) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO nic_bandwidth_samples (nic_id, ts, rx_bytes, tx_bytes, rx_bps, tx_bps)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![s.nic_id, s.ts, s.rx_bytes as i64, s.tx_bytes as i64, s.rx_bps, s.tx_bps],
        )?;
        let hour = s.ts - (s.ts % 3600);
        conn.execute(
            "INSERT INTO nic_usage_hourly (nic_id, hour_ts, rx_bytes, tx_bytes)
             VALUES (?1,?2,?3,?4)
             ON CONFLICT(nic_id, hour_ts) DO UPDATE SET
               rx_bytes = nic_usage_hourly.rx_bytes + excluded.rx_bytes,
               tx_bytes = nic_usage_hourly.tx_bytes + excluded.tx_bytes",
            params![s.nic_id, hour, (s.rx_bps / 8.0) as i64, (s.tx_bps / 8.0) as i64],
        )?;
        Ok(())
    }

    pub fn insert_ping(&self, s: &PingSample) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO ping_samples (nic_id, target, ts, rtt_ms, success, error)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                s.nic_id,
                s.target,
                s.ts,
                s.rtt_ms,
                s.success as i32,
                s.error
            ],
        )?;
        Ok(())
    }

    pub fn insert_http_latency(&self, s: &HttpLatencySample) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO http_latency_samples
             (nic_id, url, ts, dns_ms, connect_ms, tls_ms, ttfb_ms, total_ms, status_code, success, error)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                s.nic_id,
                s.url,
                s.ts,
                s.dns_ms,
                s.connect_ms,
                s.tls_ms,
                s.ttfb_ms,
                s.total_ms,
                s.status_code.map(|c| c as i64),
                s.success as i32,
                s.error
            ],
        )?;
        Ok(())
    }

    pub fn insert_link_event(&self, e: &LinkEvent) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO nic_link_events (nic_id, ts, event, detail) VALUES (?1,?2,?3,?4)",
            params![e.nic_id, e.ts, e.event, e.detail],
        )?;
        Ok(())
    }

    pub fn insert_traceroute(&self, r: &TracerouteResult) -> Result<i64> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO traceroute_runs (nic_id, target, ts) VALUES (?1,?2,?3)",
            params![r.nic_id, r.target, r.ts],
        )?;
        let id = conn.last_insert_rowid();
        for h in &r.hops {
            conn.execute(
                "INSERT INTO traceroute_hops (run_id, hop, address, hostname, rtt_ms)
                 VALUES (?1,?2,?3,?4,?5)",
                params![id, h.hop, h.address, h.hostname, h.rtt_ms],
            )?;
        }
        Ok(id)
    }

    pub fn insert_speedtest(&self, r: &SpeedtestResult) -> Result<i64> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO speedtest_runs
             (nic_id, ts, server_id, server_name, download_mbps, upload_mbps, ping_ms, jitter_ms, packet_loss, raw_json)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                r.nic_id,
                r.ts,
                r.server_id,
                r.server_name,
                r.download_mbps,
                r.upload_mbps,
                r.ping_ms,
                r.jitter_ms,
                r.packet_loss,
                r.raw_json
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_traffic_flow(&self, f: &TrafficFlow) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO traffic_flows
             (ts, process_name, process_path, pid, local_addr, remote_addr, protocol, bytes_in, bytes_out, nic_id, host)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                f.ts,
                f.process_name,
                f.process_path,
                f.pid.map(|p| p as i64),
                f.local_addr,
                f.remote_addr,
                f.protocol,
                f.bytes_in as i64,
                f.bytes_out as i64,
                f.nic_id,
                f.host
            ],
        )?;
        let hour = f.ts - (f.ts % 3600);
        conn.execute(
            "INSERT INTO app_usage_hourly (hour_ts, process_name, remote_ip, host, nic_id, bytes_in, bytes_out)
             VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(hour_ts, process_name, remote_ip, host, nic_id) DO UPDATE SET
               bytes_in = app_usage_hourly.bytes_in + excluded.bytes_in,
               bytes_out = app_usage_hourly.bytes_out + excluded.bytes_out",
            params![
                hour,
                f.process_name,
                f.remote_addr.split(':').next().unwrap_or("").to_string(),
                f.host.clone().unwrap_or_default(),
                f.nic_id.clone().unwrap_or_default(),
                f.bytes_in as i64,
                f.bytes_out as i64
            ],
        )?;
        Ok(())
    }

    pub fn insert_dns_query(&self, name: &str, ips: &[String], process: Option<&str>) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO dns_queries (ts, name, resolved_ips, process_name) VALUES (?1,?2,?3,?4)",
            params![
                Utc::now().timestamp(),
                name,
                serde_json::to_string(ips)?,
                process
            ],
        )?;
        Ok(())
    }

    pub fn insert_http_host(
        &self,
        host: &str,
        remote_ip: Option<&str>,
        process: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO http_hosts (ts, host, remote_ip, process_name) VALUES (?1,?2,?3,?4)",
            params![Utc::now().timestamp(), host, remote_ip, process],
        )?;
        Ok(())
    }

    pub fn bandwidth_history(
        &self,
        nic_id: Option<&str>,
        range: &TimeRange,
    ) -> Result<Vec<UsagePoint>> {
        let conn = self.conn.lock();
        let mut sql = String::from(
            "SELECT ts, rx_bytes, tx_bytes FROM nic_bandwidth_samples WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(id) = nic_id {
            sql.push_str(" AND nic_id = ?");
            params_vec.push(Box::new(id.to_string()));
        }
        append_range(&mut sql, &mut params_vec, range);
        sql.push_str(" ORDER BY ts ASC LIMIT 5000");
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(UsagePoint {
                ts: row.get(0)?,
                rx_bytes: row.get::<_, i64>(1)? as u64,
                tx_bytes: row.get::<_, i64>(2)? as u64,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn ping_history(
        &self,
        nic_id: Option<&str>,
        range: &TimeRange,
    ) -> Result<Vec<PingSample>> {
        let conn = self.conn.lock();
        let mut sql = String::from(
            "SELECT nic_id, target, ts, rtt_ms, success, error FROM ping_samples WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(id) = nic_id {
            sql.push_str(" AND nic_id = ?");
            params_vec.push(Box::new(id.to_string()));
        }
        append_range(&mut sql, &mut params_vec, range);
        sql.push_str(" ORDER BY ts ASC LIMIT 5000");
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(PingSample {
                nic_id: row.get(0)?,
                target: row.get(1)?,
                ts: row.get(2)?,
                rtt_ms: row.get(3)?,
                success: row.get::<_, i32>(4)? != 0,
                error: row.get(5)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn http_latency_history(
        &self,
        nic_id: Option<&str>,
        range: &TimeRange,
    ) -> Result<Vec<HttpLatencySample>> {
        let conn = self.conn.lock();
        let mut sql = String::from(
            "SELECT nic_id, url, ts, dns_ms, connect_ms, tls_ms, ttfb_ms, total_ms, status_code, success, error
             FROM http_latency_samples WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(id) = nic_id {
            sql.push_str(" AND nic_id = ?");
            params_vec.push(Box::new(id.to_string()));
        }
        append_range(&mut sql, &mut params_vec, range);
        sql.push_str(" ORDER BY ts ASC LIMIT 5000");
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(HttpLatencySample {
                nic_id: row.get(0)?,
                url: row.get(1)?,
                ts: row.get(2)?,
                dns_ms: row.get(3)?,
                connect_ms: row.get(4)?,
                tls_ms: row.get(5)?,
                ttfb_ms: row.get(6)?,
                total_ms: row.get(7)?,
                status_code: row.get::<_, Option<i64>>(8)?.map(|c| c as u16),
                success: row.get::<_, i32>(9)? != 0,
                error: row.get(10)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn link_events(
        &self,
        nic_id: Option<&str>,
        range: &TimeRange,
    ) -> Result<Vec<LinkEvent>> {
        let conn = self.conn.lock();
        let mut sql =
            String::from("SELECT nic_id, ts, event, detail FROM nic_link_events WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(id) = nic_id {
            sql.push_str(" AND nic_id = ?");
            params_vec.push(Box::new(id.to_string()));
        }
        append_range(&mut sql, &mut params_vec, range);
        sql.push_str(" ORDER BY ts DESC LIMIT 2000");
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(LinkEvent {
                nic_id: row.get(0)?,
                ts: row.get(1)?,
                event: row.get(2)?,
                detail: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn app_usage(
        &self,
        range: &TimeRange,
        group_by: &str,
    ) -> Result<Vec<AppUsageRow>> {
        let conn = self.conn.lock();
        let (select, group) = match group_by {
            "ip" => (
                "remote_ip as process_name, NULL as process_path, remote_ip, NULL as host, SUM(bytes_in), SUM(bytes_out), nic_id",
                "remote_ip, nic_id",
            ),
            "url" | "host" => (
                "COALESCE(host,'(unknown)') as process_name, NULL, NULL, host, SUM(bytes_in), SUM(bytes_out), nic_id",
                "host, nic_id",
            ),
            _ => (
                "process_name, NULL as process_path, NULL as remote_ip, NULL as host, SUM(bytes_in), SUM(bytes_out), nic_id",
                "process_name, nic_id",
            ),
        };
        let mut sql = format!(
            "SELECT {select} FROM app_usage_hourly WHERE 1=1"
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(start) = range.start_ts {
            sql.push_str(" AND hour_ts >= ?");
            params_vec.push(Box::new(start));
        }
        if let Some(end) = range.end_ts {
            sql.push_str(" AND hour_ts <= ?");
            params_vec.push(Box::new(end));
        }
        sql.push_str(&format!(
            " GROUP BY {group} ORDER BY SUM(bytes_in)+SUM(bytes_out) DESC LIMIT 200"
        ));
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(AppUsageRow {
                process_name: row.get::<_, Option<String>>(0)?.unwrap_or_else(|| "(unknown)".into()),
                process_path: row.get(1)?,
                remote_ip: row.get(2)?,
                host: row.get(3)?,
                bytes_in: row.get::<_, i64>(4)? as u64,
                bytes_out: row.get::<_, i64>(5)? as u64,
                nic_id: row.get(6)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn speedtest_history(&self, range: &TimeRange) -> Result<Vec<SpeedtestResult>> {
        let conn = self.conn.lock();
        let mut sql = String::from(
            "SELECT id, nic_id, ts, server_id, server_name, download_mbps, upload_mbps, ping_ms, jitter_ms, packet_loss, raw_json
             FROM speedtest_runs WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        append_range(&mut sql, &mut params_vec, range);
        sql.push_str(" ORDER BY ts DESC LIMIT 500");
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(SpeedtestResult {
                id: row.get(0)?,
                nic_id: row.get(1)?,
                ts: row.get(2)?,
                server_id: row.get(3)?,
                server_name: row.get(4)?,
                download_mbps: row.get(5)?,
                upload_mbps: row.get(6)?,
                ping_ms: row.get(7)?,
                jitter_ms: row.get(8)?,
                packet_loss: row.get(9)?,
                raw_json: row.get(10)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn prune_raw(&self, retention_days: u32) -> Result<usize> {
        let cutoff = Utc::now().timestamp() - (retention_days as i64 * 86400);
        let conn = self.conn.lock();
        let mut total = 0usize;
        for table in [
            "nic_bandwidth_samples",
            "ping_samples",
            "http_latency_samples",
            "traffic_flows",
            "dns_queries",
            "http_hosts",
        ] {
            total += conn.execute(
                &format!("DELETE FROM {table} WHERE ts < ?1"),
                params![cutoff],
            )?;
        }
        Ok(total)
    }
}

fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let v = stmt
        .query_row(params![key], |row| row.get(0))
        .optional()?;
    Ok(v)
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1,?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )?;
    Ok(())
}

fn parse_json_vec(s: Option<String>) -> Vec<String> {
    s.and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default()
}

fn append_range(
    sql: &mut String,
    params_vec: &mut Vec<Box<dyn rusqlite::ToSql>>,
    range: &TimeRange,
) {
    if let Some(start) = range.start_ts {
        sql.push_str(" AND ts >= ?");
        params_vec.push(Box::new(start));
    }
    if let Some(end) = range.end_ts {
        sql.push_str(" AND ts <= ?");
        params_vec.push(Box::new(end));
    }
}
