use ratatui::{
    Frame,
    layout::{Layout, Constraint, Direction, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use crossterm::event::{KeyEvent, KeyCode};
use xray_model::*;

use crate::{App, Action, Screen, InputMode};

// ─── Wizard State ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep {
    Template = 0,
    Basic = 1,
    Transport = 2,
    Sniffing = 3,
    Security = 4,
    Users = 5,
    Confirm = 6,
}

/// 预设模板：一键填好所有字段的常见代理组合
#[derive(Debug, Clone)]
pub enum InboundTemplate {
    /// VLESS + WebSocket + TLS — 最通用的 CDN 友好配置
    VlessWsTls,
    /// VLESS + WebSocket + Reality — 无需域名/证书，反代伪装
    VlessWsReality,
    /// VLESS + gRPC + Reality — gRPC 多路复用 + 反代
    VlessGrpcReality,
    /// VLESS + TCP + XTLS Vision — 无额外开销，适合直连
    VlessTcpXtlVision,
    /// VMess + WebSocket + TLS — 兼容旧客户端的经典组合
    VMessWsTls,
    /// VMess + WebSocket + CDN — 无证书，靠 CDN 提供 TLS
    VMessWsCdn,
    /// VMess + gRPC + TLS — 适合移动端弱网
    VMessGrpcTls,
    /// Trojan + WebSocket + TLS — Trojan 协议 + WS
    TrojanWsTls,
    /// Trojan + gRPC + TLS
    TrojanGrpcTls,
    /// Shadowsocks + WebSocket + TLS — SS 的 WebSocket 隧道
    ShadowsocksWsTls,
    /// VLESS + HTTPUpgrade + Reality — 新协议，伪装 HTTP
    VlessHttpUpgradeReality,
    /// SOCKS5 — 本地代理
    SocksLocal,
    /// HTTP 代理 — 本地
    HttpLocal,
    /// 空白模板：从头开始配置（自定义）
    Custom,
}

impl InboundTemplate {
    /// 模板名称和简短描述
    pub fn info(&self) -> (&'static str, &'static str) {
        match self {
            Self::VlessWsTls => ("VLESS + WS + TLS", "最通用：WebSocket 走 CDN，TLS 加密，需域名+证书"),
            Self::VlessWsReality => ("VLESS + WS + Reality", "无需域名/证书：伪装微软等网站，反代特性"),
            Self::VlessGrpcReality => ("VLESS + gRPC + Reality", "gRPC 多路复用 + Reality，适合移动端和弱网"),
            Self::VlessTcpXtlVision => ("VLESS + TCP + XTLS Vision", "直连最优：无额外封装开销，需 443 端口"),
            Self::VMessWsTls => ("VMess + WS + TLS", "经典组合：兼容老客户端，WebSocket + TLS"),
            Self::VMessWsCdn => ("VMess + WS (CDN)", "无证书：WebSocket 裸奔，靠 CDN 提供 SSL"),
            Self::VMessGrpcTls => ("VMess + gRPC + TLS", "gRPC 高效传输 + TLS，适合多用户在同一个端口"),
            Self::TrojanWsTls => ("Trojan + WS + TLS", "Trojan over WebSocket，伪装 HTTP 流量"),
            Self::TrojanGrpcTls => ("Trojan + gRPC + TLS", "Trojan over gRPC，复用连接"),
            Self::ShadowsocksWsTls => ("Shadowsocks + WS + TLS", "SS over WebSocket，兼容 SIP008"),
            Self::VlessHttpUpgradeReality => ("VLESS + HTTPUpgrade + Reality", "新版 HTTP 升级协议 + Reality 伪装"),
            Self::SocksLocal => ("SOCKS5 本地", "本地 SOCKS5 代理，通常监听 127.0.0.1:1080"),
            Self::HttpLocal => ("HTTP 本地代理", "本地 HTTP 代理，通常监听 127.0.0.1:8080"),
            Self::Custom => ("自定义（从零开始）", "自由选择协议、传输、安全等所有参数"),
        }
    }

    /// 所有非 Custom 模板的列表
    pub fn all() -> Vec<Self> {
        vec![
            Self::VlessWsTls, Self::VlessWsReality, Self::VlessGrpcReality,
            Self::VlessTcpXtlVision, Self::VMessWsTls, Self::VMessWsCdn,
            Self::VMessGrpcTls, Self::TrojanWsTls, Self::TrojanGrpcTls,
            Self::ShadowsocksWsTls, Self::VlessHttpUpgradeReality,
            Self::SocksLocal, Self::HttpLocal, Self::Custom,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WizardFieldType {
    Dropdown,
    TextInput,
    Toggle,
    Button,
}

#[derive(Debug, Clone)]
pub struct WizardField {
    pub label: String,
    pub field_type: WizardFieldType,
    pub value: String,
    pub options: Vec<String>,
    pub selected_option: usize,
    pub is_open: bool,
}

impl WizardField {
    fn dropdown(label: &str, value: &str, options: Vec<String>) -> Self {
        let selected = options.iter().position(|o| o == value).unwrap_or(0);
        Self { label: label.into(), field_type: WizardFieldType::Dropdown, value: value.into(), options, selected_option: selected, is_open: false }
    }
    fn text(label: &str, value: &str) -> Self {
        Self { label: label.into(), field_type: WizardFieldType::TextInput, value: value.into(), options: vec![], selected_option: 0, is_open: false }
    }
    fn toggle(label: &str, value: bool) -> Self {
        Self { label: label.into(), field_type: WizardFieldType::Toggle, value: if value { "true" } else { "false" }.into(), options: vec![], selected_option: 0, is_open: false }
    }
}

#[derive(Debug, Clone)]
pub struct InboundConfigBuilder {
    pub protocol: InboundProtocol,
    pub port: u16,
    pub listen: String,
    pub tag: Option<String>,
    pub transport: TransportNetwork,
    pub security: StreamSecurity,
    pub ws_path: Option<String>,
    pub ws_host: Option<String>,
    pub grpc_service_name: Option<String>,
    pub grpc_multi_mode: bool,
    pub tls_server_name: Option<String>,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub reality_server_name: Option<String>,
    pub reality_dest: Option<String>,
    pub reality_public_key: Option<String>,
    pub reality_private_key: Option<String>,
    pub reality_short_id: Option<String>,
    pub reality_fingerprint: String,
    pub sniffing_enabled: bool,
    pub sniffing_http: bool,
    pub sniffing_tls: bool,
    pub sniffing_quic: bool,
    pub sniffing_route_only: bool,
    pub uuid: Option<String>,
    pub vless_flow: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub ss_method: String,
    pub http_user: Option<String>,
    pub http_pass: Option<String>,
}

impl Default for InboundConfigBuilder {
    fn default() -> Self {
        Self {
            protocol: InboundProtocol::VMess,
            port: 443,
            listen: "0.0.0.0".into(),
            tag: None,
            transport: TransportNetwork::Tcp,
            security: StreamSecurity::None,
            ws_path: None, ws_host: None,
            grpc_service_name: None, grpc_multi_mode: false,
            tls_server_name: None, tls_cert_path: None, tls_key_path: None,
            reality_server_name: None, reality_dest: None,
            reality_public_key: None, reality_private_key: None,
            reality_short_id: None,
            reality_fingerprint: "chrome".into(),
            sniffing_enabled: true, sniffing_http: true, sniffing_tls: true, sniffing_quic: false,
            sniffing_route_only: false,
            uuid: Some(uuid::Uuid::new_v4().to_string()),
            vless_flow: None,
            password: None, email: None,
            ss_method: "aes-256-gcm".into(),
            http_user: None, http_pass: None,
        }
    }
}

impl InboundConfigBuilder {
    /// 应用预设模板，一键填好所有字段
    pub fn apply_template(&mut self, template: &InboundTemplate) {
        let uuid = uuid::Uuid::new_v4().to_string();
        match template {
            InboundTemplate::VlessWsTls => {
                self.protocol = InboundProtocol::VLess;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("vless-ws-tls".into());
                self.transport = TransportNetwork::Ws;
                self.ws_path = Some("/ws".into());
                self.ws_host = Some("your-domain.com".into());
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.uuid = Some(uuid);
                self.vless_flow = None;
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::VlessWsReality => {
                self.protocol = InboundProtocol::VLess;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("vless-ws-reality".into());
                self.transport = TransportNetwork::Ws;
                self.ws_path = Some("/ws".into());
                self.security = StreamSecurity::Reality;
                self.reality_server_name = Some("www.microsoft.com".into());
                self.reality_dest = Some("127.0.0.1:8080".into());
                self.reality_fingerprint = "chrome".into();
                self.reality_short_id = Some("abc123".into());
                self.uuid = Some(uuid);
                self.vless_flow = Some("xtls-rprx-vision".into());
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::VlessGrpcReality => {
                self.protocol = InboundProtocol::VLess;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("vless-grpc-reality".into());
                self.transport = TransportNetwork::Grpc;
                self.grpc_service_name = Some("TunService".into());
                self.grpc_multi_mode = true;
                self.security = StreamSecurity::Reality;
                self.reality_server_name = Some("www.google.com".into());
                self.reality_dest = Some("127.0.0.1:8080".into());
                self.reality_fingerprint = "chrome".into();
                self.reality_short_id = Some("abc123".into());
                self.uuid = Some(uuid);
                self.vless_flow = Some("xtls-rprx-vision".into());
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::VlessTcpXtlVision => {
                self.protocol = InboundProtocol::VLess;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("vless-tcp-xtls".into());
                self.transport = TransportNetwork::Tcp;
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.uuid = Some(uuid);
                self.vless_flow = Some("xtls-rprx-vision".into());
                self.sniffing_enabled = false;
                self.sniffing_http = false; self.sniffing_tls = false; self.sniffing_quic = false;
            }
            InboundTemplate::VMessWsTls => {
                self.protocol = InboundProtocol::VMess;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("vmess-ws-tls".into());
                self.transport = TransportNetwork::Ws;
                self.ws_path = Some("/ws".into());
                self.ws_host = Some("your-domain.com".into());
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.uuid = Some(uuid);
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::VMessWsCdn => {
                self.protocol = InboundProtocol::VMess;
                self.port = 80;
                self.listen = "0.0.0.0".into();
                self.tag = Some("vmess-ws-cdn".into());
                self.transport = TransportNetwork::Ws;
                self.ws_path = Some("/ws".into());
                self.ws_host = Some("your-cdn-domain.com".into());
                self.security = StreamSecurity::None;
                self.uuid = Some(uuid);
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::VMessGrpcTls => {
                self.protocol = InboundProtocol::VMess;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("vmess-grpc-tls".into());
                self.transport = TransportNetwork::Grpc;
                self.grpc_service_name = Some("TunService".into());
                self.grpc_multi_mode = true;
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.uuid = Some(uuid);
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::TrojanWsTls => {
                self.protocol = InboundProtocol::Trojan;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("trojan-ws-tls".into());
                self.transport = TransportNetwork::Ws;
                self.ws_path = Some("/trojan".into());
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.password = Some(Self::gen_password());
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::TrojanGrpcTls => {
                self.protocol = InboundProtocol::Trojan;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("trojan-grpc-tls".into());
                self.transport = TransportNetwork::Grpc;
                self.grpc_service_name = Some("TunService".into());
                self.grpc_multi_mode = true;
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.password = Some(Self::gen_password());
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::ShadowsocksWsTls => {
                self.protocol = InboundProtocol::Shadowsocks;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("ss-ws-tls".into());
                self.transport = TransportNetwork::Ws;
                self.ws_path = Some("/ss".into());
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.ss_method = "aes-256-gcm".into();
                self.password = Some(Self::gen_password());
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::VlessHttpUpgradeReality => {
                self.protocol = InboundProtocol::VLess;
                self.port = 443;
                self.listen = "0.0.0.0".into();
                self.tag = Some("vless-hup-reality".into());
                self.transport = TransportNetwork::HttpUpgrade;
                self.ws_path = Some("/".into());
                self.security = StreamSecurity::Reality;
                self.reality_server_name = Some("www.microsoft.com".into());
                self.reality_dest = Some("127.0.0.1:8080".into());
                self.reality_fingerprint = "chrome".into();
                self.reality_short_id = Some("abc123".into());
                self.uuid = Some(uuid);
                self.vless_flow = Some("xtls-rprx-vision".into());
                self.sniffing_enabled = true;
                self.sniffing_http = true; self.sniffing_tls = true; self.sniffing_quic = false;
            }
            InboundTemplate::SocksLocal => {
                self.protocol = InboundProtocol::Socks;
                self.port = 1080;
                self.listen = "127.0.0.1".into();
                self.tag = Some("socks-in".into());
                self.transport = TransportNetwork::Tcp;
                self.security = StreamSecurity::None;
                self.sniffing_enabled = false;
                self.sniffing_http = false; self.sniffing_tls = false; self.sniffing_quic = false;
            }
            InboundTemplate::HttpLocal => {
                self.protocol = InboundProtocol::Http;
                self.port = 8080;
                self.listen = "127.0.0.1".into();
                self.tag = Some("http-in".into());
                self.transport = TransportNetwork::Tcp;
                self.security = StreamSecurity::None;
                self.http_user = Some("admin".into());
                self.http_pass = Some(Self::gen_password());
                self.sniffing_enabled = false;
                self.sniffing_http = false; self.sniffing_tls = false; self.sniffing_quic = false;
            }
            InboundTemplate::Custom => {}
        }
    }

    fn gen_password() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        uuid::Uuid::new_v4().to_string().hash(&mut h);
        format!("{:x}", h.finish())[..16].to_string()
    }
}

impl InboundConfigBuilder {
    pub fn build(&self) -> InboundConfig {
        let transport = self.transport.clone();
        let security = self.security.clone();

        let stream_settings = StreamSettings {
            network: transport.clone(),
            security: security.clone(),
            tls_settings: if security == StreamSecurity::Tls {
                Some(TlsSettings {
                    server_name: self.tls_server_name.clone(),
                    certificates: vec![TlsCertificate {
                        certificate_file: self.tls_cert_path.clone().unwrap_or_default(),
                        key_file: self.tls_key_path.clone().unwrap_or_default(),
                    }],
                    alpn: vec![],
                    min_version: None,
                })
            } else { None },
            reality_settings: if security == StreamSecurity::Reality {
                Some(RealitySettings {
                    server_name: self.reality_server_name.clone().unwrap_or_default(),
                    public_key: self.reality_public_key.clone().unwrap_or_default(),
                    private_key: self.reality_private_key.clone().unwrap_or_default(),
                    short_ids: self.reality_short_id.clone().map(|s| vec![s]).unwrap_or_default(),
                    fingerprint: self.reality_fingerprint.clone(),
                    spider_x: None,
                    dest: self.reality_dest.clone().unwrap_or_default(),
                })
            } else { None },
            ws_settings: if transport == TransportNetwork::Ws {
                Some(WsSettings {
                    path: self.ws_path.clone().unwrap_or_else(|| "/ws".into()),
                    host: self.ws_host.clone(),
                    headers: None,
                })
            } else { None },
            grpc_settings: if transport == TransportNetwork::Grpc {
                Some(GrpcSettings {
                    service_name: self.grpc_service_name.clone().unwrap_or_else(|| "TunService".into()),
                    multi_mode: self.grpc_multi_mode,
                    authority: None,
                })
            } else { None },
            httpupgrade_settings: None, tcp_settings: None, kcp_settings: None, quic_settings: None,
        };

        let mut sniff_overrides = Vec::new();
        if self.sniffing_http { sniff_overrides.push("http".into()); }
        if self.sniffing_tls { sniff_overrides.push("tls".into()); }
        if self.sniffing_quic { sniff_overrides.push("quic".into()); }

        let settings = match self.protocol {
            InboundProtocol::VMess => ProtocolSettings::VMess(VMessSettings {
                clients: vec![VMessClient {
                    id: self.uuid.clone().unwrap_or_default(),
                    security: "auto".into(),
                    email: self.email.clone(),
                    level: None,
                }],
            }),
            InboundProtocol::VLess => ProtocolSettings::VLess(VLessSettings {
                clients: vec![VLessClient {
                    id: self.uuid.clone().unwrap_or_default(),
                    flow: self.vless_flow.clone(),
                    email: self.email.clone(),
                    level: None,
                }],
                decryption: "none".into(),
            }),
            InboundProtocol::Trojan => ProtocolSettings::Trojan(TrojanSettings {
                clients: vec![TrojanClient {
                    password: self.password.clone().unwrap_or_default(),
                    email: self.email.clone(),
                    level: None,
                }],
                fallbacks: vec![],
            }),
            InboundProtocol::Shadowsocks => ProtocolSettings::Shadowsocks(ShadowsocksSettings {
                method: self.ss_method.clone(),
                password: self.password.clone().unwrap_or_default(),
                network: "tcp,udp".into(),
                email: self.email.clone(),
                level: None,
            }),
            InboundProtocol::Http => ProtocolSettings::Http(HttpSettings {
                accounts: vec![HttpAccount {
                    user: self.http_user.clone().unwrap_or_default(),
                    pass: self.http_pass.clone().unwrap_or_default(),
                }],
                timeout: 300,
                allow_transparent: false,
            }),
            InboundProtocol::Socks => ProtocolSettings::Socks(SocksSettings {
                auth: SocksAuth::NoAuth {},
                udp: true,
                ip: None,
                user_level: 0,
            }),
        };

        InboundConfig {
            tag: self.tag.clone(),
            port: self.port,
            listen: self.listen.clone(),
            protocol: self.protocol.clone(),
            settings,
            stream_settings,
            sniffing: SniffingConfig {
                enabled: self.sniffing_enabled,
                dest_override: sniff_overrides,
                route_only: self.sniffing_route_only,
                metadata_only: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct InboundWizardState {
    pub current_step: WizardStep,
    pub edit_index: Option<usize>,
    pub builder: InboundConfigBuilder,
    pub fields: Vec<WizardField>,
    pub focused: usize,
    pub auto_restart: bool,
    pub json_preview: String,
    pub selected_template: usize,
}

impl InboundWizardState {
    pub fn new() -> Self {
        Self {
            current_step: WizardStep::Template,
            edit_index: None,
            builder: InboundConfigBuilder::default(),
            fields: Self::build_template_fields(),
            focused: 0,
            auto_restart: false,
            json_preview: String::new(),
            selected_template: 0,
        }
    }

    pub fn edit(index: usize, inbound: InboundConfig) -> Self {
        let mut wiz = Self::new();
        wiz.edit_index = Some(index);
        wiz.current_step = WizardStep::Basic; // skip template for edit
        let inb = inbound;
        wiz.builder.protocol = inb.protocol.clone();
        wiz.builder.port = inb.port;
        wiz.builder.listen = inb.listen.clone();
        wiz.builder.tag = inb.tag.clone();
        wiz.builder.transport = inb.stream_settings.network.clone();
        wiz.builder.security = inb.stream_settings.security.clone();
        wiz.builder.sniffing_enabled = inb.sniffing.enabled;
        wiz.builder.sniffing_http = inb.sniffing.dest_override.contains(&"http".to_string());
        wiz.builder.sniffing_tls = inb.sniffing.dest_override.contains(&"tls".to_string());
        wiz.fields = Self::build_step_fields(WizardStep::Basic, &wiz.builder);
        wiz
    }

    fn build_template_fields() -> Vec<WizardField> {
        InboundTemplate::all().iter().enumerate().map(|(i, t)| {
            let (name, desc) = t.info();
            WizardField {
                label: name.into(),
                field_type: WizardFieldType::Button,
                value: format!("  {}  │  {}", name, desc),
                options: vec![],
                selected_option: i,
                is_open: false,
            }
        }).collect()
    }

    fn build_step_fields(step: WizardStep, builder: &InboundConfigBuilder) -> Vec<WizardField> {
        match step {
            WizardStep::Template => Self::build_template_fields(),
            WizardStep::Basic => vec![
                WizardField::dropdown("Protocol", &builder.protocol.to_string(),
                    vec!["VMess".into(), "VLESS".into(), "Trojan".into(), "Shadowsocks".into(), "HTTP".into(), "SOCKS".into()]),
                WizardField::text("Port", &builder.port.to_string()),
                WizardField::text("Listen", &builder.listen),
                WizardField::text("Tag", builder.tag.as_deref().unwrap_or("")),
            ],
            WizardStep::Transport => {
                let mut fields = vec![
                    WizardField::dropdown("Network", &builder.transport.to_string(),
                        vec!["TCP".into(), "WebSocket".into(), "gRPC".into()]),
                ];
                match builder.transport {
                    TransportNetwork::Ws => {
                        fields.push(WizardField::text("WS Path", builder.ws_path.as_deref().unwrap_or("/ws")));
                        fields.push(WizardField::text("WS Host", builder.ws_host.as_deref().unwrap_or("")));
                    }
                    TransportNetwork::Grpc => {
                        fields.push(WizardField::text("gRPC Service", builder.grpc_service_name.as_deref().unwrap_or("TunService")));
                        fields.push(WizardField::toggle("Multi Mode", builder.grpc_multi_mode));
                    }
                    _ => {}
                }
                fields
            }
            WizardStep::Sniffing => vec![
                WizardField::toggle("Enable Sniffing", builder.sniffing_enabled),
                WizardField::toggle("Sniff HTTP", builder.sniffing_http),
                WizardField::toggle("Sniff TLS", builder.sniffing_tls),
                WizardField::toggle("Sniff QUIC", builder.sniffing_quic),
                WizardField::toggle("Route Only", builder.sniffing_route_only),
            ],
            WizardStep::Security => {
                let mut fields = vec![
                    WizardField::dropdown("Security", &builder.security.to_string(),
                        vec!["none".into(), "TLS".into(), "Reality".into()]),
                ];
                match builder.security {
                    StreamSecurity::Tls => {
                        fields.push(WizardField::text("TLS ServerName", builder.tls_server_name.as_deref().unwrap_or("")));
                        fields.push(WizardField::text("Cert Path", builder.tls_cert_path.as_deref().unwrap_or("")));
                        fields.push(WizardField::text("Key Path", builder.tls_key_path.as_deref().unwrap_or("")));
                    }
                    StreamSecurity::Reality => {
                        fields.push(WizardField::text("ServerName", builder.reality_server_name.as_deref().unwrap_or("www.microsoft.com")));
                        fields.push(WizardField::text("Dest", builder.reality_dest.as_deref().unwrap_or("127.0.0.1:8080")));
                        fields.push(WizardField::text("Fingerprint", &builder.reality_fingerprint));
                        fields.push(WizardField::text("ShortID", builder.reality_short_id.as_deref().unwrap_or("")));
                        fields.push(WizardField::text("Public Key", builder.reality_public_key.as_deref().unwrap_or("")));
                        fields.push(WizardField::text("Private Key", builder.reality_private_key.as_deref().unwrap_or("")));
                    }
                    _ => {}
                }
                fields
            }
            WizardStep::Users => match builder.protocol {
                InboundProtocol::VMess | InboundProtocol::VLess => vec![
                    WizardField::text("UUID", builder.uuid.as_deref().unwrap_or("")),
                    WizardField::text("Email", builder.email.as_deref().unwrap_or("")),
                ],
                InboundProtocol::Trojan => vec![
                    WizardField::text("Password", builder.password.as_deref().unwrap_or("")),
                    WizardField::text("Email", builder.email.as_deref().unwrap_or("")),
                ],
                InboundProtocol::Shadowsocks => vec![
                    WizardField::text("Password", builder.password.as_deref().unwrap_or("")),
                    WizardField::dropdown("Method", &builder.ss_method,
                        vec!["aes-256-gcm".into(), "chacha20-ietf-poly1305".into()]),
                ],
                InboundProtocol::Http => vec![
                    WizardField::text("Username", builder.http_user.as_deref().unwrap_or("")),
                    WizardField::text("Password", builder.http_pass.as_deref().unwrap_or("")),
                ],
                InboundProtocol::Socks => vec![],
            },
            WizardStep::Confirm => vec![],
        }
    }

    pub fn close_dropdowns(&mut self) -> bool {
        let mut any_open = false;
        for field in &mut self.fields {
            if field.is_open { field.is_open = false; any_open = true; }
        }
        any_open
    }

    pub fn next_step(&mut self) {
        self.apply_fields();
        match self.current_step {
            WizardStep::Template => self.current_step = WizardStep::Basic,
            WizardStep::Basic => self.current_step = WizardStep::Transport,
            WizardStep::Transport => self.current_step = WizardStep::Sniffing,
            WizardStep::Sniffing => self.current_step = WizardStep::Security,
            WizardStep::Security => self.current_step = WizardStep::Users,
            WizardStep::Users => {
                let config = self.builder.build();
                self.json_preview = serde_json::to_string_pretty(&config).unwrap_or_default();
                self.current_step = WizardStep::Confirm;
            }
            WizardStep::Confirm => {}
        }
        if self.current_step != WizardStep::Confirm {
            self.fields = Self::build_step_fields(self.current_step.clone(), &self.builder);
        }
        self.focused = 0;
    }

    pub fn prev_step(&mut self) {
        self.apply_fields();
        self.current_step = match self.current_step {
            WizardStep::Transport => WizardStep::Basic,
            WizardStep::Sniffing => WizardStep::Transport,
            WizardStep::Security => WizardStep::Sniffing,
            WizardStep::Users => WizardStep::Security,
            WizardStep::Confirm => WizardStep::Users,
            WizardStep::Basic => WizardStep::Template,
            WizardStep::Template => WizardStep::Template,
        };
        if self.current_step != WizardStep::Confirm {
            self.fields = Self::build_step_fields(self.current_step.clone(), &self.builder);
        }
        self.focused = 0;
    }

    fn apply_fields(&mut self) {
        match self.current_step {
            WizardStep::Template => {},
            WizardStep::Basic => {
                if let Some(f) = self.fields.get(0) { self.builder.protocol = Self::parse_protocol(&f.value); }
                if let Some(f) = self.fields.get(1) { self.builder.port = f.value.parse().unwrap_or(443); }
                if let Some(f) = self.fields.get(2) { self.builder.listen = f.value.clone(); }
                if let Some(f) = self.fields.get(3) { self.builder.tag = if f.value.is_empty() { None } else { Some(f.value.clone()) }; }
            }
            WizardStep::Transport => {
                if let Some(f) = self.fields.get(0) {
                    self.builder.transport = match f.value.as_str() {
                        "TCP" => TransportNetwork::Tcp,
                        "WebSocket" | "ws" => TransportNetwork::Ws,
                        "gRPC" | "grpc" => TransportNetwork::Grpc,
                        "HTTPUpgrade" => TransportNetwork::HttpUpgrade,
                        _ => TransportNetwork::Tcp,
                    };
                }
                if let Some(f) = self.fields.get(1) { self.builder.ws_path = Some(f.value.clone()); }
                if let Some(f) = self.fields.get(2) { self.builder.ws_host = Some(f.value.clone()); }
            }
            WizardStep::Sniffing => {
                if let Some(f) = self.fields.get(0) { self.builder.sniffing_enabled = f.value == "true"; }
                if let Some(f) = self.fields.get(1) { self.builder.sniffing_http = f.value == "true"; }
                if let Some(f) = self.fields.get(2) { self.builder.sniffing_tls = f.value == "true"; }
                if let Some(f) = self.fields.get(3) { self.builder.sniffing_quic = f.value == "true"; }
                if let Some(f) = self.fields.get(4) { self.builder.sniffing_route_only = f.value == "true"; }
            }
            WizardStep::Security => {
                if let Some(f) = self.fields.get(0) {
                    self.builder.security = match f.value.as_str() {
                        "TLS" | "tls" => StreamSecurity::Tls,
                        "Reality" | "reality" => StreamSecurity::Reality,
                        _ => StreamSecurity::None,
                    };
                }
                if let Some(f) = self.fields.get(1) { self.builder.tls_server_name = Some(f.value.clone()); }
                if let Some(f) = self.fields.get(2) { self.builder.tls_cert_path = Some(f.value.clone()); }
                if let Some(f) = self.fields.get(3) { self.builder.tls_key_path = Some(f.value.clone()); }
            }
            WizardStep::Users => {
                if let Some(f) = self.fields.get(0) {
                    self.builder.uuid = Some(f.value.clone());
                    self.builder.password = Some(f.value.clone());
                }
                if let Some(f) = self.fields.get(1) { self.builder.email = Some(f.value.clone()); }
            }
            _ => {}
        }
    }

    fn parse_protocol(s: &str) -> InboundProtocol {
        match s {
            "VMess" => InboundProtocol::VMess,
            "VLESS" => InboundProtocol::VLess,
            "Trojan" => InboundProtocol::Trojan,
            "Shadowsocks" => InboundProtocol::Shadowsocks,
            "HTTP" => InboundProtocol::Http,
            "SOCKS" => InboundProtocol::Socks,
            _ => InboundProtocol::VMess,
        }
    }
}

// ─── Key Handler ──────────────────────────────────────────────────

pub fn handle_key(key: KeyEvent, app: &mut App, wiz: &mut InboundWizardState) -> Option<Action> {
    // Template selection step — handled separately
    if wiz.current_step == WizardStep::Template {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if wiz.focused > 0 { wiz.focused -= 1; wiz.selected_template = wiz.focused; }
                return None;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = InboundTemplate::all().len().saturating_sub(1);
                if wiz.focused < max { wiz.focused += 1; wiz.selected_template = wiz.focused; }
                return None;
            }
            KeyCode::Enter => {
                let templates = InboundTemplate::all();
                if let Some(t) = templates.get(wiz.selected_template) {
                    wiz.builder.apply_template(t);
                }
                wiz.next_step();
                return None;
            }
            KeyCode::Esc => return Some(Action::PopScreen),
            _ => return None,
        }
    }

    match key.code {
        KeyCode::Tab => {
            let count = wiz.fields.len();
            if count > 0 { wiz.focused = (wiz.focused + 1) % count; }
            return None;
        }
        KeyCode::BackTab => {
            let count = wiz.fields.len();
            if count > 0 { wiz.focused = (wiz.focused + count - 1) % count; }
            return None;
        }
        KeyCode::Enter => {
            if let Some(field) = wiz.fields.get_mut(wiz.focused) {
                match field.field_type {
                    WizardFieldType::Dropdown => {
                        field.is_open = !field.is_open;
                        if field.is_open { app.mode = InputMode::Selecting; }
                        return None;
                    }
                    WizardFieldType::Toggle => {
                        field.value = if field.value == "true" { "false".into() } else { "true".into() };
                        return None;
                    }
                    _ => {}
                }
            }
            if wiz.current_step == WizardStep::Confirm {
                let config = wiz.builder.build();
                return if let Some(idx) = wiz.edit_index {
                    Some(Action::UpdateInbound(idx, config))
                } else {
                    Some(Action::SaveInbound(config))
                };
            }
            wiz.next_step();
            return None;
        }
        KeyCode::Esc => {
            if wiz.close_dropdowns() {
                app.mode = InputMode::Normal;
                return None;
            }
            if wiz.current_step == WizardStep::Basic {
                return Some(Action::PopScreen);
            }
            wiz.prev_step();
            return None;
        }
        KeyCode::Right | KeyCode::Left if key.modifiers.is_empty() => {
            if key.code == KeyCode::Right { wiz.next_step(); } else { wiz.prev_step(); }
            return None;
        }
        KeyCode::Up => {
            if let Some(field) = wiz.fields.get_mut(wiz.focused) {
                if field.is_open && field.field_type == WizardFieldType::Dropdown {
                    if field.selected_option > 0 { field.selected_option -= 1; }
                    field.value = field.options.get(field.selected_option).cloned().unwrap_or_default();
                }
            }
            return None;
        }
        KeyCode::Down => {
            if let Some(field) = wiz.fields.get_mut(wiz.focused) {
                if field.is_open && field.field_type == WizardFieldType::Dropdown {
                    if field.selected_option + 1 < field.options.len() { field.selected_option += 1; }
                    field.value = field.options.get(field.selected_option).cloned().unwrap_or_default();
                }
            }
            return None;
        }
        KeyCode::Char('g') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
            // Generate UUID
            if let Some(f) = wiz.fields.iter_mut().find(|f| f.label == "UUID") {
                f.value = uuid::Uuid::new_v4().to_string();
            }
            return None;
        }
        KeyCode::Char(c) => {
            if let Some(field) = wiz.fields.get_mut(wiz.focused) {
                if field.field_type == WizardFieldType::TextInput {
                    field.value.push(c);
                    app.mode = InputMode::Editing;
                }
            }
            return None;
        }
        KeyCode::Backspace => {
            if let Some(field) = wiz.fields.get_mut(wiz.focused) {
                if field.field_type == WizardFieldType::TextInput {
                    field.value.pop();
                }
            }
            return None;
        }
        _ => None,
    }
}

// ─── Render ───────────────────────────────────────────────────────

pub fn render(f: &mut Frame, area: Rect, _app: &App, wiz: &InboundWizardState) {
    let step_names = ["Template", "Basic", "Transport", "Sniffing", "Security", "Users", "Confirm"];
    let step_idx = match wiz.current_step {
        WizardStep::Template => 0, WizardStep::Basic => 1, WizardStep::Transport => 2,
        WizardStep::Sniffing => 3, WizardStep::Security => 4, WizardStep::Users => 5, WizardStep::Confirm => 6,
    };

    let title = format!("{} Inbound — Step {}/6: {}",
        if wiz.edit_index.is_some() { "Edit" } else { "New" },
        step_idx, step_names[step_idx]);

    if wiz.current_step == WizardStep::Template {
        render_template_selector(f, area, wiz);
        return;
    }

    if wiz.current_step == WizardStep::Confirm {
        render_confirm(f, area, &title, wiz);
        return;
    }

    let form_height = (wiz.fields.len() + 4) as u16;
    let chunks = Layout::vertical([Constraint::Length(form_height), Constraint::Min(1)]).split(area);

    let field_lines: Vec<Line> = wiz.fields.iter().enumerate().map(|(i, fld)| {
        let is_focused = i == wiz.focused;
        let (indicator, val_display) = match fld.field_type {
            WizardFieldType::Dropdown => {
                let arrow = if fld.is_open { "▾" } else { "▸" };
                if is_focused {
                    (format!(" ▶{}", arrow), Span::styled(format!(" {} ", fld.value), Style::default().bg(Color::Cyan).fg(Color::Black)))
                } else {
                    (format!("  {}", arrow), Span::raw(format!(" {} ", fld.value)))
                }
            }
            WizardFieldType::TextInput => {
                if is_focused {
                    ("> ".into(), Span::styled(format!(" {}_", fld.value), Style::default().fg(Color::Yellow)))
                } else {
                    ("  ".into(), Span::raw(format!(" {}", fld.value)))
                }
            }
            WizardFieldType::Toggle => {
                let toggle = if fld.value == "true" { "[X]" } else { "[ ]" };
                if is_focused {
                    ("> ".into(), Span::styled(format!("{} {}", toggle, fld.label), Style::default().fg(Color::Green)))
                } else {
                    ("  ".into(), Span::raw(format!("{} {}", toggle, fld.label)))
                }
            }
            _ => ("> ".into(), Span::raw(&fld.value)),
        };
        let label_span = if fld.field_type != WizardFieldType::Toggle {
            Span::raw(format!("{}:  ", fld.label))
        } else { Span::raw("") };

        Line::from(vec![Span::raw(indicator), label_span, val_display])
    }).collect();

    f.render_widget(Paragraph::new(field_lines).block(Block::default().borders(Borders::ALL).title(title)), chunks[0]);

    // Dropdown popup
    for (i, fld) in wiz.fields.iter().enumerate() {
        if fld.is_open && fld.field_type == WizardFieldType::Dropdown {
            let y_offset = (i + 1) as u16;
            let popup_area = Rect::new(
                chunks[0].x + 18,
                chunks[0].y + y_offset,
                20,
                fld.options.len() as u16 + 2,
            );
            // Simple list rendering
            let items: Vec<Line> = fld.options.iter().enumerate().map(|(oi, opt)| {
                if oi == fld.selected_option {
                    Line::from(Span::styled(format!(" ▶ {}", opt), Style::default().fg(Color::Black).bg(Color::Cyan)))
                } else {
                    Line::from(Span::raw(format!("   {}", opt)))
                }
            }).collect();
            f.render_widget(
                Paragraph::new(items).block(Block::default().borders(Borders::ALL).style(Style::default().bg(Color::Rgb(20, 20, 30)))),
                popup_area,
            );
        }
    }

    // Help text
    let help = vec![
        Line::from(Span::styled("←→ step  Tab field  Enter confirm/open  Esc back/close  ^G new UUID", Style::default().fg(Color::DarkGray))),
    ];
    f.render_widget(Paragraph::new(help), chunks[1]);
}

fn render_confirm(f: &mut Frame, area: Rect, title: &str, wiz: &InboundWizardState) {
    let preview: Vec<Line> = wiz.json_preview.lines().take(area.height as usize - 5).map(Line::from).collect();
    let block = Block::default().borders(Borders::ALL).title(title).style(Style::default().fg(Color::Green));
    f.render_widget(Paragraph::new(preview).block(block), area);
}

fn render_template_selector(f: &mut Frame, area: Rect, wiz: &InboundWizardState) {
    let templates = InboundTemplate::all();
    let title = "New Inbound — Step 0/6: Choose Template";

    let header: Vec<Line> = vec![
        Line::from(Span::styled(" Select a preset template to quickly configure your inbound:", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(" ─── Real-world configs (修改参数后即可使用) ───", Style::default().fg(Color::DarkGray))),
        Line::from(""),
    ];

    let content: Vec<Line> = templates.iter().enumerate().map(|(i, t)| {
        let (name, desc) = t.info();
        let suffix = if matches!(t, InboundTemplate::Custom) { "" } else { "  ← 预设值已填好" };
        if i == wiz.selected_template {
            Line::from(vec![
                Span::styled(format!(" ▶ {}  ", name), Style::default().fg(Color::Black).bg(Color::Cyan)),
                Span::styled(format!("{} {}", desc, suffix), Style::default().fg(Color::Cyan)),
            ])
        } else {
            Line::from(vec![
                Span::raw(format!("   {}  ", name)),
                Span::raw(desc),
            ])
        }
    }).collect();

    let help: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(" ↑↓:选择模板  Enter:确认并继续  Esc:返回", Style::default().fg(Color::DarkGray))),
    ];

    let all_lines: Vec<Line> = header.into_iter().chain(content).chain(help).collect();

    f.render_widget(
        Paragraph::new(all_lines).block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
}
