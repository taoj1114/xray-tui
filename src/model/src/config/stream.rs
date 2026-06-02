use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSettings {
    #[serde(default = "default_network")]
    pub network: TransportNetwork,
    #[serde(default = "default_stream_security")]
    pub security: StreamSecurity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_settings: Option<TlsSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reality_settings: Option<RealitySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_settings: Option<WsSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_settings: Option<GrpcSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub httpupgrade_settings: Option<HttpUpgradeSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_settings: Option<TcpSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kcp_settings: Option<KcpSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quic_settings: Option<QuicSettings>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportNetwork {
    Tcp,
    Ws,
    Kcp,
    Grpc,
    HttpUpgrade,
    Quic,
    DomainSocket,
}
fn default_network() -> TransportNetwork { TransportNetwork::Tcp }

impl std::fmt::Display for TransportNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tcp => write!(f, "TCP"),
            Self::Ws => write!(f, "WebSocket"),
            Self::Kcp => write!(f, "KCP"),
            Self::Grpc => write!(f, "gRPC"),
            Self::HttpUpgrade => write!(f, "HTTPUpgrade"),
            Self::Quic => write!(f, "QUIC"),
            Self::DomainSocket => write!(f, "DomainSocket"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamSecurity {
    None,
    Tls,
    Reality,
}
fn default_stream_security() -> StreamSecurity { StreamSecurity::None }

impl std::fmt::Display for StreamSecurity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Tls => write!(f, "TLS"),
            Self::Reality => write!(f, "Reality"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    pub certificates: Vec<TlsCertificate>,
    #[serde(default)]
    pub alpn: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsCertificate {
    #[serde(rename = "certificateFile")]
    pub certificate_file: String,
    #[serde(rename = "keyFile")]
    pub key_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealitySettings {
    #[serde(rename = "serverName")]
    pub server_name: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
    #[serde(rename = "shortIds")]
    pub short_ids: Vec<String>,
    #[serde(default = "default_fingerprint")]
    pub fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "spiderX")]
    pub spider_x: Option<String>,
    pub dest: String,
}
fn default_fingerprint() -> String { "chrome".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSettings {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcSettings {
    #[serde(rename = "serviceName")]
    pub service_name: String,
    #[serde(default)]
    #[serde(rename = "multiMode")]
    pub multi_mode: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpUpgradeSettings {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<TcpHeaderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpHeaderConfig {
    #[serde(rename = "type")]
    pub header_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KcpSettings {
    pub mtu: u32,
    pub tti: u32,
    #[serde(rename = "uplinkCapacity")]
    pub uplink_capacity: u32,
    #[serde(rename = "downlinkCapacity")]
    pub downlink_capacity: u32,
    pub congestion: bool,
    #[serde(rename = "readBufferSize")]
    pub read_buffer_size: u32,
    #[serde(rename = "writeBufferSize")]
    pub write_buffer_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<KcpHeaderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KcpHeaderConfig {
    #[serde(rename = "type")]
    pub header_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicSettings {
    pub security: String,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<QuicHeaderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicHeaderConfig {
    #[serde(rename = "type")]
    pub header_type: String,
}
