use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub xray_binary_path: String,
    pub config_path: String,
    pub log_level: String,
    pub server_public_ip: Option<String>,
    pub state_dir: String,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            xray_binary_path: "/usr/local/bin/xray".into(),
            config_path: "/usr/local/etc/xray/config.json".into(),
            log_level: "warning".into(),
            server_public_ip: None,
            state_dir: dirs_like_path(),
        }
    }
}

fn dirs_like_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    format!("{}/.config/xray-tui", home)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Runtime status of Xray (not persisted — refreshed each tick)
#[derive(Debug, Clone)]
pub struct XrayStatus {
    pub is_installed: bool,
    pub is_running: bool,
    pub version: Option<String>,
    pub pid: Option<u32>,
    pub cpu_percent: Option<f64>,
    pub memory_bytes: Option<u64>,
    pub uptime_seconds: Option<u64>,
}

impl Default for XrayStatus {
    fn default() -> Self {
        Self {
            is_installed: false,
            is_running: false,
            version: None,
            pid: None,
            cpu_percent: None,
            memory_bytes: None,
            uptime_seconds: None,
        }
    }
}

/// Persisted inbound metadata without sensitive info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredInbound {
    pub tag: Option<String>,
    pub port: u16,
    pub listen: String,
    pub protocol: String,
    pub transport: String,
    pub security: String,
    pub enabled: bool,
    pub user_count: usize,
}

use crate::config::InboundConfig;

impl From<&InboundConfig> for StoredInbound {
    fn from(inb: &InboundConfig) -> Self {
        Self {
            tag: inb.tag.clone(),
            port: inb.port,
            listen: inb.listen.clone(),
            protocol: inb.protocol.to_string(),
            transport: inb.stream_settings.network.to_string(),
            security: inb.stream_settings.security.to_string(),
            enabled: true,
            user_count: inb.user_count(),
        }
    }
}

/// App state snapshot for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub settings: GlobalSettings,
    pub stored_inbounds: Vec<StoredInbound>,
    pub stored_certs: Vec<crate::ssl::CertInfo>,
}
