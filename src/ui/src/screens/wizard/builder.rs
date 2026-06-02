use xray_model::*;
use super::templates::InboundTemplate;

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
            port: 443, listen: "0.0.0.0".into(), tag: None,
            transport: TransportNetwork::Tcp, security: StreamSecurity::None,
            ws_path: None, ws_host: None,
            grpc_service_name: None, grpc_multi_mode: false,
            tls_server_name: None, tls_cert_path: None, tls_key_path: None,
            reality_server_name: None, reality_dest: None,
            reality_public_key: None, reality_private_key: None,
            reality_short_id: None, reality_fingerprint: "chrome".into(),
            sniffing_enabled: true, sniffing_http: true, sniffing_tls: true,
            sniffing_quic: false, sniffing_route_only: false,
            uuid: Some(uuid::Uuid::new_v4().to_string()),
            vless_flow: None, password: None, email: None,
            ss_method: "aes-256-gcm".into(),
            http_user: None, http_pass: None,
        }
    }
}

impl InboundConfigBuilder {
    pub fn apply_template(&mut self, template: &InboundTemplate) {
        let uuid = uuid::Uuid::new_v4().to_string();
        match template {
            InboundTemplate::VlessWsTls => {
                self.protocol = InboundProtocol::VLess; self.port = 443;
                self.tag = Some("vless-ws-tls".into());
                self.transport = TransportNetwork::Ws;
                self.ws_path = Some("/ws".into()); self.ws_host = Some("your-domain.com".into());
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.uuid = Some(uuid);
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::VlessWsReality => {
                self.protocol = InboundProtocol::VLess; self.port = 443;
                self.tag = Some("vless-ws-reality".into());
                self.transport = TransportNetwork::Ws; self.ws_path = Some("/ws".into());
                self.security = StreamSecurity::Reality;
                self.reality_server_name = Some("www.microsoft.com".into());
                self.reality_dest = Some("127.0.0.1:8080".into());
                self.reality_short_id = Some("abc123".into());
                self.uuid = Some(uuid);
                self.vless_flow = Some("xtls-rprx-vision".into());
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::VlessGrpcReality => {
                self.protocol = InboundProtocol::VLess; self.port = 443;
                self.tag = Some("vless-grpc-reality".into());
                self.transport = TransportNetwork::Grpc;
                self.grpc_service_name = Some("TunService".into()); self.grpc_multi_mode = true;
                self.security = StreamSecurity::Reality;
                self.reality_server_name = Some("www.google.com".into());
                self.reality_dest = Some("127.0.0.1:8080".into());
                self.reality_short_id = Some("abc123".into());
                self.uuid = Some(uuid);
                self.vless_flow = Some("xtls-rprx-vision".into());
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::VlessTcpXtlVision => {
                self.protocol = InboundProtocol::VLess; self.port = 443;
                self.tag = Some("vless-tcp-xtls".into());
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.uuid = Some(uuid);
                self.vless_flow = Some("xtls-rprx-vision".into());
                self.sniffing_enabled = false; self.sniffing_http = false;
                self.sniffing_tls = false; self.sniffing_quic = false;
            }
            InboundTemplate::VMessWsTls => {
                self.protocol = InboundProtocol::VMess; self.port = 443;
                self.tag = Some("vmess-ws-tls".into());
                self.transport = TransportNetwork::Ws;
                self.ws_path = Some("/ws".into()); self.ws_host = Some("your-domain.com".into());
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.uuid = Some(uuid);
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::VMessWsCdn => {
                self.protocol = InboundProtocol::VMess; self.port = 80;
                self.tag = Some("vmess-ws-cdn".into());
                self.transport = TransportNetwork::Ws;
                self.ws_path = Some("/ws".into()); self.ws_host = Some("your-cdn-domain.com".into());
                self.uuid = Some(uuid);
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::VMessGrpcTls => {
                self.protocol = InboundProtocol::VMess; self.port = 443;
                self.tag = Some("vmess-grpc-tls".into());
                self.transport = TransportNetwork::Grpc;
                self.grpc_service_name = Some("TunService".into()); self.grpc_multi_mode = true;
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.uuid = Some(uuid);
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::TrojanWsTls => {
                self.protocol = InboundProtocol::Trojan; self.port = 443;
                self.tag = Some("trojan-ws-tls".into());
                self.transport = TransportNetwork::Ws; self.ws_path = Some("/trojan".into());
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.password = Some(Self::gen_password());
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::TrojanGrpcTls => {
                self.protocol = InboundProtocol::Trojan; self.port = 443;
                self.tag = Some("trojan-grpc-tls".into());
                self.transport = TransportNetwork::Grpc;
                self.grpc_service_name = Some("TunService".into()); self.grpc_multi_mode = true;
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.password = Some(Self::gen_password());
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::ShadowsocksWsTls => {
                self.protocol = InboundProtocol::Shadowsocks; self.port = 443;
                self.tag = Some("ss-ws-tls".into());
                self.transport = TransportNetwork::Ws; self.ws_path = Some("/ss".into());
                self.security = StreamSecurity::Tls;
                self.tls_server_name = Some("your-domain.com".into());
                self.tls_cert_path = Some("/etc/xray/certs/fullchain.pem".into());
                self.tls_key_path = Some("/etc/xray/certs/privkey.pem".into());
                self.password = Some(Self::gen_password());
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::VlessHttpUpgradeReality => {
                self.protocol = InboundProtocol::VLess; self.port = 443;
                self.tag = Some("vless-hup-reality".into());
                self.transport = TransportNetwork::HttpUpgrade; self.ws_path = Some("/".into());
                self.security = StreamSecurity::Reality;
                self.reality_server_name = Some("www.microsoft.com".into());
                self.reality_dest = Some("127.0.0.1:8080".into());
                self.reality_short_id = Some("abc123".into());
                self.uuid = Some(uuid);
                self.vless_flow = Some("xtls-rprx-vision".into());
                self.sniffing_http = true; self.sniffing_tls = true;
            }
            InboundTemplate::SocksLocal => {
                self.protocol = InboundProtocol::Socks; self.port = 1080;
                self.listen = "127.0.0.1".into();
                self.tag = Some("socks-in".into());
                self.sniffing_enabled = false; self.sniffing_http = false;
                self.sniffing_tls = false; self.sniffing_quic = false;
            }
            InboundTemplate::HttpLocal => {
                self.protocol = InboundProtocol::Http; self.port = 8080;
                self.listen = "127.0.0.1".into();
                self.tag = Some("http-in".into());
                self.http_user = Some("admin".into());
                self.http_pass = Some(Self::gen_password());
                self.sniffing_enabled = false; self.sniffing_http = false;
                self.sniffing_tls = false; self.sniffing_quic = false;
            }
            InboundTemplate::Custom => {}
        }
    }

    pub fn build(&self) -> InboundConfig {
        let transport = self.transport.clone();
        let security = self.security.clone();
        let stream_settings = StreamSettings {
            network: transport.clone(), security: security.clone(),
            tls_settings: if security == StreamSecurity::Tls {
                Some(TlsSettings { server_name: self.tls_server_name.clone(),
                    certificates: vec![TlsCertificate { certificate_file: self.tls_cert_path.clone().unwrap_or_default(), key_file: self.tls_key_path.clone().unwrap_or_default() }],
                    alpn: vec![], min_version: None })
            } else { None },
            reality_settings: if security == StreamSecurity::Reality {
                Some(RealitySettings { server_name: self.reality_server_name.clone().unwrap_or_default(),
                    public_key: self.reality_public_key.clone().unwrap_or_default(), private_key: self.reality_private_key.clone().unwrap_or_default(),
                    short_ids: self.reality_short_id.clone().map(|s| vec![s]).unwrap_or_default(), fingerprint: self.reality_fingerprint.clone(),
                    spider_x: None, dest: self.reality_dest.clone().unwrap_or_default() })
            } else { None },
            ws_settings: if transport == TransportNetwork::Ws {
                Some(WsSettings { path: self.ws_path.clone().unwrap_or_else(|| "/ws".into()), host: self.ws_host.clone(), headers: None })
            } else { None },
            grpc_settings: if transport == TransportNetwork::Grpc {
                Some(GrpcSettings { service_name: self.grpc_service_name.clone().unwrap_or_else(|| "TunService".into()), multi_mode: self.grpc_multi_mode, authority: None })
            } else { None },
            httpupgrade_settings: None, tcp_settings: None, kcp_settings: None, quic_settings: None,
        };
        let mut sniff_overrides = Vec::new();
        if self.sniffing_http { sniff_overrides.push("http".into()); }
        if self.sniffing_tls { sniff_overrides.push("tls".into()); }
        if self.sniffing_quic { sniff_overrides.push("quic".into()); }
        let settings = match self.protocol {
            InboundProtocol::VMess => ProtocolSettings::VMess(VMessSettings {
                clients: vec![VMessClient { id: self.uuid.clone().unwrap_or_default(), security: "auto".into(), email: self.email.clone(), level: None }],
            }),
            InboundProtocol::VLess => ProtocolSettings::VLess(VLessSettings {
                clients: vec![VLessClient { id: self.uuid.clone().unwrap_or_default(), flow: self.vless_flow.clone(), email: self.email.clone(), level: None }],
                decryption: "none".into(),
            }),
            InboundProtocol::Trojan => ProtocolSettings::Trojan(TrojanSettings {
                clients: vec![TrojanClient { password: self.password.clone().unwrap_or_default(), email: self.email.clone(), level: None }],
                fallbacks: vec![],
            }),
            InboundProtocol::Shadowsocks => ProtocolSettings::Shadowsocks(ShadowsocksSettings {
                method: self.ss_method.clone(), password: self.password.clone().unwrap_or_default(),
                network: "tcp,udp".into(), email: self.email.clone(), level: None,
            }),
            InboundProtocol::Http => ProtocolSettings::Http(HttpSettings {
                accounts: vec![HttpAccount { user: self.http_user.clone().unwrap_or_default(), pass: self.http_pass.clone().unwrap_or_default() }],
                timeout: 300, allow_transparent: false,
            }),
            InboundProtocol::Socks => ProtocolSettings::Socks(SocksSettings {
                auth: SocksAuth::NoAuth {}, udp: true, ip: None, user_level: 0,
            }),
        };
        InboundConfig { tag: self.tag.clone(), port: self.port, listen: self.listen.clone(), protocol: self.protocol.clone(), settings, stream_settings,
            sniffing: SniffingConfig { enabled: self.sniffing_enabled, dest_override: sniff_overrides, route_only: self.sniffing_route_only, metadata_only: None },
        }
    }

    fn gen_password() -> String {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        uuid::Uuid::new_v4().to_string().hash(&mut h);
        format!("{:x}", h.finish())[..16].to_string()
    }
}
