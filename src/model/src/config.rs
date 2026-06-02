use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct LogConfig {
    #[serde(default = "default_loglevel")]
    pub loglevel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
fn default_loglevel() -> String { "warning".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub tag: String,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<HashMap<String, PolicyLevel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<PolicySystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyLevel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conn_idle: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uplink_only: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downlink_only: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_user_uplink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_user_downlink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySystem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_inbound_uplink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_inbound_downlink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_outbound_uplink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_outbound_downlink: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniffingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_sniff_dest_override")]
    pub dest_override: Vec<String>,
    #[serde(default)]
    pub route_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_only: Option<bool>,
}
fn default_true() -> bool { true }
fn default_sniff_dest_override() -> Vec<String> { vec!["http".into(), "tls".into()] }

impl Default for SniffingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dest_override: default_sniff_dest_override(),
            route_only: false,
            metadata_only: None,
        }
    }
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
#[serde(untagged)]
pub enum ProtocolSettings {
    VMess(VMessSettings),
    VLess(VLessSettings),
    Trojan(TrojanSettings),
    Shadowsocks(ShadowsocksSettings),
    Http(HttpSettings),
    Socks(SocksSettings),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMessSettings {
    pub clients: Vec<VMessClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMessClient {
    pub id: String,
    #[serde(default = "default_vmess_security")]
    pub security: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
}
fn default_vmess_security() -> String { "auto".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLessSettings {
    pub clients: Vec<VLessClient>,
    #[serde(default = "default_decryption")]
    pub decryption: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLessClient {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
}
fn default_decryption() -> String { "none".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrojanSettings {
    pub clients: Vec<TrojanClient>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub fallbacks: Vec<FallbackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrojanClient {
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
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
pub struct ShadowsocksSettings {
    pub method: String,
    pub password: String,
    #[serde(default = "default_ss_network")]
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
}
fn default_ss_network() -> String { "tcp,udp".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSettings {
    #[serde(default)]
    pub accounts: Vec<HttpAccount>,
    #[serde(default = "default_http_timeout")]
    pub timeout: u32,
    #[serde(default)]
    pub allow_transparent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAccount {
    pub user: String,
    pub pass: String,
}
fn default_http_timeout() -> u32 { 300 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocksSettings {
    pub auth: SocksAuth,
    #[serde(default)]
    pub udp: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(default)]
    pub user_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SocksAuth {
    NoAuth {},
    Password { accounts: Vec<SocksAccount> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocksAccount {
    pub user: String,
    pub pass: String,
}

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
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TcpHeaderConfig {
    None,
    Http {
        #[serde(skip_serializing_if = "Option::is_none")]
        request: Option<TcpHttpRequest>,
        #[serde(skip_serializing_if = "Option::is_none")]
        response: Option<TcpHttpResponse>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpHttpRequest {
    pub version: String,
    pub method: String,
    pub path: Vec<String>,
    pub headers: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpHttpResponse {
    pub version: String,
    pub status: String,
    pub reason: String,
    pub headers: HashMap<String, Vec<String>>,
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
    pub header: KcpHeaderConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KcpHeaderConfig {
    None { #[serde(rename = "type")] type_: String },
    Other(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicSettings {
    pub security: String,
    pub key: String,
    pub header: QuicHeaderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QuicHeaderConfig {
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<DnsServer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_fallback: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServer {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expect_ips: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_fallback: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<StreamSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_settings: Option<ProxySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub tag: String,
}

impl InboundConfig {
    pub fn user_count(&self) -> usize {
        match &self.settings {
            ProtocolSettings::VMess(s) => s.clients.len(),
            ProtocolSettings::VLess(s) => s.clients.len(),
            ProtocolSettings::Trojan(s) => s.clients.len(),
            ProtocolSettings::Http(s) => s.accounts.len(),
            ProtocolSettings::Socks(s) => match &s.auth {
                SocksAuth::NoAuth {} => 0,
                SocksAuth::Password { accounts } => accounts.len(),
            },
            ProtocolSettings::Shadowsocks(_) => 1,
        }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

impl XrayConfig {
    pub fn from_inbounds(
        inbounds: &[InboundConfig],
        routing: &RoutingConfig,
    ) -> Self {
        Self {
            log: LogConfig {
                loglevel: "warning".into(),
                access: None,
                error: None,
            },
            api: None,
            routing: routing.clone(),
            policy: PolicyConfig::default(),
            inbounds: inbounds.to_vec(),
            outbounds: vec![
                OutboundConfig {
                    tag: Some("direct".into()),
                    protocol: "freedom".into(),
                    settings: None,
                    stream_settings: None,
                    proxy_settings: None,
                },
                OutboundConfig {
                    tag: Some("block".into()),
                    protocol: "blackhole".into(),
                    settings: None,
                    stream_settings: None,
                    proxy_settings: None,
                },
            ],
            dns: None,
        }
    }
}

use crate::routing::RoutingConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vless_reality_serialization() {
        let inbound = InboundConfig {
            tag: Some("web-vless".into()),
            port: 443,
            listen: "0.0.0.0".into(),
            protocol: InboundProtocol::VLess,
            settings: ProtocolSettings::VLess(VLessSettings {
                clients: vec![VLessClient {
                    id: "d290f1ee-6c54-4b01-90e6-d701748f0851".into(),
                    flow: Some("xtls-rprx-vision".into()),
                    email: None,
                    level: None,
                }],
                decryption: "none".into(),
            }),
            stream_settings: StreamSettings {
                network: TransportNetwork::Ws,
                security: StreamSecurity::Reality,
                tls_settings: None,
                reality_settings: Some(RealitySettings {
                    server_name: "www.microsoft.com".into(),
                    public_key: "test_pub_key".into(),
                    private_key: "test_priv_key".into(),
                    short_ids: vec!["abc123".into()],
                    fingerprint: "chrome".into(),
                    spider_x: None,
                    dest: "127.0.0.1:8080".into(),
                }),
                ws_settings: Some(WsSettings {
                    path: "/ws".into(),
                    host: None,
                    headers: None,
                }),
                grpc_settings: None,
                httpupgrade_settings: None,
                tcp_settings: None,
                kcp_settings: None,
                quic_settings: None,
            },
            sniffing: SniffingConfig::default(),
        };

        let json = serde_json::to_string_pretty(&inbound).unwrap();
        assert!(json.contains("\"protocol\": \"vless\""));
        assert!(json.contains("\"network\": \"ws\""));
        assert!(json.contains("\"security\": \"reality\""));
        assert!(json.contains("\"serverName\": \"www.microsoft.com\""));
    }

    #[test]
    fn test_vmess_tls_serialization() {
        let inbound = InboundConfig {
            tag: Some("api-vmess".into()),
            port: 10084,
            listen: "127.0.0.1".into(),
            protocol: InboundProtocol::VMess,
            settings: ProtocolSettings::VMess(VMessSettings {
                clients: vec![VMessClient {
                    id: "c1a2b3c4-d5e6-f7a8-b9c0-d1e2f3a4b5c6".into(),
                    security: "auto".into(),
                    email: None,
                    level: None,
                }],
            }),
            stream_settings: StreamSettings {
                network: TransportNetwork::Tcp,
                security: StreamSecurity::Tls,
                tls_settings: Some(TlsSettings {
                    server_name: Some("example.com".into()),
                    certificates: vec![TlsCertificate {
                        certificate_file: "/etc/xray/certs/fullchain.pem".into(),
                        key_file: "/etc/xray/certs/privkey.pem".into(),
                    }],
                    alpn: vec!["h2".into(), "http/1.1".into()],
                    min_version: Some("1.2".into()),
                }),
                reality_settings: None,
                ws_settings: None,
                grpc_settings: None,
                httpupgrade_settings: None,
                tcp_settings: None,
                kcp_settings: None,
                quic_settings: None,
            },
            sniffing: SniffingConfig::default(),
        };

        let json = serde_json::to_string_pretty(&inbound).unwrap();
        assert!(json.contains("\"protocol\": \"vmess\""));
        assert!(json.contains("\"security\": \"tls\""));
        assert!(json.contains("\"certificateFile\""));
    }

    #[test]
    fn test_trojan_ws_serialization() {
        let inbound = InboundConfig {
            tag: Some("tg-bridge".into()),
            port: 8443,
            listen: "0.0.0.0".into(),
            protocol: InboundProtocol::Trojan,
            settings: ProtocolSettings::Trojan(TrojanSettings {
                clients: vec![TrojanClient {
                    password: "testpass123".into(),
                    email: None,
                    level: None,
                }],
                fallbacks: vec![],
            }),
            stream_settings: StreamSettings {
                network: TransportNetwork::Ws,
                security: StreamSecurity::Tls,
                tls_settings: Some(TlsSettings {
                    server_name: Some("tg.example.com".into()),
                    certificates: vec![TlsCertificate {
                        certificate_file: "/etc/xray/certs/fullchain.pem".into(),
                        key_file: "/etc/xray/certs/privkey.pem".into(),
                    }],
                    alpn: vec![],
                    min_version: None,
                }),
                reality_settings: None,
                ws_settings: Some(WsSettings {
                    path: "/trojan".into(),
                    host: Some("tg.example.com".into()),
                    headers: None,
                }),
                grpc_settings: None,
                httpupgrade_settings: None,
                tcp_settings: None,
                kcp_settings: None,
                quic_settings: None,
            },
            sniffing: SniffingConfig::default(),
        };

        let json = serde_json::to_string_pretty(&inbound).unwrap();
        assert!(json.contains("\"protocol\": \"trojan\""));
        assert!(json.contains("\"network\": \"ws\""));
        assert!(json.contains("\"password\": \"testpass123\""));
    }

    #[test]
    fn test_vmess_grpc_serialization() {
        let inbound = InboundConfig {
            tag: Some("grpc-vmess".into()),
            port: 12345,
            listen: "0.0.0.0".into(),
            protocol: InboundProtocol::VMess,
            settings: ProtocolSettings::VMess(VMessSettings {
                clients: vec![VMessClient {
                    id: "abc123-def456-ghi789-jkl012-mno345".into(),
                    security: "chacha20-poly1305".into(),
                    email: None,
                    level: None,
                }],
            }),
            stream_settings: StreamSettings {
                network: TransportNetwork::Grpc,
                security: StreamSecurity::Tls,
                tls_settings: Some(TlsSettings {
                    server_name: Some("grpc.example.com".into()),
                    certificates: vec![TlsCertificate {
                        certificate_file: "/etc/xray/certs/fullchain.pem".into(),
                        key_file: "/etc/xray/certs/privkey.pem".into(),
                    }],
                    alpn: vec!["h2".into()],
                    min_version: None,
                }),
                reality_settings: None,
                ws_settings: None,
                grpc_settings: Some(GrpcSettings {
                    service_name: "TunService".into(),
                    multi_mode: true,
                    authority: None,
                }),
                httpupgrade_settings: None,
                tcp_settings: None,
                kcp_settings: None,
                quic_settings: None,
            },
            sniffing: SniffingConfig {
                enabled: true,
                dest_override: vec!["http".into(), "tls".into()],
                route_only: false,
                metadata_only: None,
            },
        };

        let json = serde_json::to_string_pretty(&inbound).unwrap();
        assert!(json.contains("\"network\": \"grpc\""));
        assert!(json.contains("\"serviceName\": \"TunService\""));
        assert!(json.contains("\"multiMode\": true"));
    }

    #[test]
    fn test_full_xray_config() {
        let inbounds = vec![
            InboundConfig {
                tag: Some("in-1".into()),
                port: 443,
                listen: "0.0.0.0".into(),
                protocol: InboundProtocol::VLess,
                settings: ProtocolSettings::VLess(VLessSettings {
                    clients: vec![VLessClient {
                        id: uuid::Uuid::new_v4().to_string(),
                        flow: Some("xtls-rprx-vision".into()),
                        email: None,
                        level: None,
                    }],
                    decryption: "none".into(),
                }),
                stream_settings: StreamSettings {
                    network: TransportNetwork::Ws,
                    security: StreamSecurity::None,
                    tls_settings: None,
                    reality_settings: None,
                    ws_settings: Some(WsSettings { path: "/ws".into(), host: None, headers: None }),
                    grpc_settings: None,
                    httpupgrade_settings: None,
                    tcp_settings: None,
                    kcp_settings: None,
                    quic_settings: None,
                },
                sniffing: SniffingConfig::default(),
            },
        ];
        let routing = RoutingConfig::default();
        let config = XrayConfig::from_inbounds(&inbounds, &routing);
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"inbounds\""));
        assert!(json.contains("\"outbounds\""));
        assert!(json.contains("\"tag\": \"direct\""));
        assert!(json.contains("\"tag\": \"block\""));
    }

    #[test]
    fn test_socks_serialization() {
        let inbound = InboundConfig {
            tag: Some("socks-in".into()),
            port: 1080,
            listen: "127.0.0.1".into(),
            protocol: InboundProtocol::Socks,
            settings: ProtocolSettings::Socks(SocksSettings {
                auth: SocksAuth::Password {
                    accounts: vec![SocksAccount {
                        user: "admin".into(),
                        pass: "pass123".into(),
                    }],
                },
                udp: true,
                ip: None,
                user_level: 0,
            }),
            stream_settings: StreamSettings {
                network: TransportNetwork::Tcp,
                security: StreamSecurity::None,
                tls_settings: None,
                reality_settings: None,
                ws_settings: None,
                grpc_settings: None,
                httpupgrade_settings: None,
                tcp_settings: None,
                kcp_settings: None,
                quic_settings: None,
            },
            sniffing: SniffingConfig { enabled: false, dest_override: vec![], route_only: false, metadata_only: None },
        };
        let json = serde_json::to_string_pretty(&inbound).unwrap();
        assert!(json.contains("\"user\": \"admin\""));
        assert!(json.contains("\"protocol\": \"socks\""));
    }
}
