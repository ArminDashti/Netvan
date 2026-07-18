use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode {
    #[default]
    Process,
    Full,
}

impl CaptureMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CaptureMode::Process => "process",
            CaptureMode::Full => "full",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "full" => CaptureMode::Full,
            _ => CaptureMode::Process,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub capture_mode: CaptureMode,
    pub ping_targets: Vec<String>,
    pub http_targets: Vec<String>,
    pub ping_interval_secs: u64,
    pub http_interval_secs: u64,
    pub bandwidth_interval_ms: u64,
    pub retention_raw_days: u32,
    pub start_ui_with_windows: bool,
    pub speedtest_cli_path: Option<String>,
    pub speedtest_eula_accepted: bool,
    pub default_nic_id: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            capture_mode: CaptureMode::Process,
            ping_targets: vec![
                "8.8.8.8".into(),
                "1.1.1.1".into(),
            ],
            http_targets: vec![
                "https://www.cloudflare.com/cdn-cgi/trace".into(),
                "https://www.google.com/generate_204".into(),
            ],
            ping_interval_secs: 5,
            http_interval_secs: 30,
            bandwidth_interval_ms: 1000,
            retention_raw_days: 14,
            start_ui_with_windows: false,
            speedtest_cli_path: None,
            speedtest_eula_accepted: false,
            default_nic_id: None,
        }
    }
}
