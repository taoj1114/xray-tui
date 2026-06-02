use serde::{Deserialize, Serialize};

pub mod common;
pub mod protocol;
pub mod stream;

pub use common::*;
pub use protocol::*;
pub use stream::*;

use super::routing::RoutingConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrayConfig {
    pub log: LogConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ApiConfig>,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
    pub inbounds: Vec<InboundConfig>,
    pub outbounds: Vec<OutboundConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<DnsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub port: u16,
    pub listen: String,
    pub protocol: InboundProtocol,
    pub settings: ProtocolSettings,
    #[serde(rename = "streamSettings")]
    pub stream_settings: StreamSettings,
    #[serde(default)]
    pub sniffing: SniffingConfig,
}

impl InboundConfig {
    pub fn user_count(&self) -> usize {
        match &self.settings {
            ProtocolSettings::VMess(s) => s.clients.len(),
            ProtocolSettings::VLess(s) => s.clients.len(),
            ProtocolSettings::Trojan(s) => s.clients.len(),
            ProtocolSettings::Shadowsocks(_) => 1,
            ProtocolSettings::Http(s) => s.accounts.len(),
            ProtocolSettings::Socks(s) => match &s.auth {
                SocksAuth::Password { accounts } => accounts.len(),
                SocksAuth::NoAuth {} => 0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InboundProtocol {
    VMess,
    VLess,
    Trojan,
    Shadowsocks,
    Http,
    Socks,
}

impl std::fmt::Display for InboundProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VMess => write!(f, "VMess"),
            Self::VLess => write!(f, "VLESS"),
            Self::Trojan => write!(f, "Trojan"),
            Self::Shadowsocks => write!(f, "Shadowsocks"),
            Self::Http => write!(f, "HTTP"),
            Self::Socks => write!(f, "SOCKS"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub dest: u16,
    pub xver: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundConfig {
    pub protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(rename = "streamSettings", skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<StreamSettings>,
}

impl XrayConfig {
    pub fn from_inbounds(inbounds: &[InboundConfig], routing: &RoutingConfig) -> Self {
        Self {
            log: LogConfig { loglevel: "warning".into(), access: None, error: None },
            api: None,
            routing: routing.clone(),
            policy: PolicyConfig::default(),
            inbounds: inbounds.to_vec(),
            outbounds: vec![
                OutboundConfig {
                    protocol: "freedom".into(),
                    settings: None,
                    tag: Some("direct".into()),
                    stream_settings: None,
                },
                OutboundConfig {
                    protocol: "blackhole".into(),
                    settings: None,
                    tag: Some("block".into()),
                    stream_settings: None,
                }
            ],
            dns: None,
        }
    }
}

#[cfg(test)]
mod tests;
