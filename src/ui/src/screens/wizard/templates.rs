use xray_model::*;

/// 预设模板：一键填好所有字段的常见代理组合
#[derive(Debug, Clone)]
pub enum InboundTemplate {
    VlessWsTls,
    VlessWsReality,
    VlessGrpcReality,
    VlessTcpXtlVision,
    VMessWsTls,
    VMessWsCdn,
    VMessGrpcTls,
    TrojanWsTls,
    TrojanGrpcTls,
    ShadowsocksWsTls,
    VlessHttpUpgradeReality,
    SocksLocal,
    HttpLocal,
    Custom,
}

// ─ 模板参数（数据驱动，替代 apply_template 中的大 match） ─

#[derive(Debug, Clone)]
pub struct TemplateParams {
    pub protocol: InboundProtocol,
    pub port: u16,
    pub listen: &'static str,
    pub tag: &'static str,
    pub transport: TransportNetwork,
    pub security: StreamSecurity,
    pub ws_path: Option<&'static str>,
    pub ws_host: Option<&'static str>,
    pub grpc_service: Option<&'static str>,
    pub grpc_multi: bool,
    pub tls_sni: Option<&'static str>,
    pub tls_cert: Option<&'static str>,
    pub tls_key: Option<&'static str>,
    pub reality_sni: Option<&'static str>,
    pub reality_dest: Option<&'static str>,
    pub reality_sid: Option<&'static str>,
    pub vless_flow: Option<&'static str>,
    pub http_user: Option<&'static str>,
    pub sniff_http: bool,
    pub sniff_tls: bool,
    pub sniff_quic: bool,
    pub sniff_on: bool,
}

impl InboundTemplate {
    pub fn resolve_params(&self) -> TemplateParams {
        use InboundProtocol::VMess;
        let mut p = TemplateParams {
            protocol: VMess,
            port: 443,
            listen: "0.0.0.0",
            tag: "",
            transport: TransportNetwork::Tcp,
            security: StreamSecurity::None,
            ws_path: None,
            ws_host: None,
            grpc_service: None,
            grpc_multi: false,
            tls_sni: None,
            tls_cert: None,
            tls_key: None,
            reality_sni: None,
            reality_dest: None,
            reality_sid: None,
            vless_flow: None,
            http_user: None,
            sniff_http: true,
            sniff_tls: true,
            sniff_quic: false,
            sniff_on: true,
        };
        self.apply_defaults(&mut p);
        p
    }

    fn apply_defaults(&self, p: &mut TemplateParams) {
        use InboundProtocol::*;
        use TransportNetwork::*;
        use StreamSecurity::*;
        match self {
            Self::VlessWsTls => {
                p.protocol = VLess; p.tag = "vless-ws-tls"; p.transport = Ws;
                p.ws_path = Some("/ws"); p.ws_host = Some("your-domain.com");
                p.security = Tls; p.tls_sni = Some("your-domain.com");
                p.tls_cert = Some("/etc/xray/certs/fullchain.pem");
                p.tls_key = Some("/etc/xray/certs/privkey.pem");
            }
            Self::VlessWsReality => {
                p.protocol = VLess; p.tag = "vless-ws-reality"; p.transport = Ws;
                p.ws_path = Some("/ws"); p.security = Reality;
                p.reality_sni = Some("www.microsoft.com");
                p.reality_dest = Some("127.0.0.1:8080"); p.reality_sid = Some("abc123");
                p.vless_flow = Some("xtls-rprx-vision");
            }
            Self::VlessGrpcReality => {
                p.protocol = VLess; p.tag = "vless-grpc-reality"; p.transport = Grpc;
                p.grpc_service = Some("TunService"); p.grpc_multi = true;
                p.security = Reality; p.reality_sni = Some("www.google.com");
                p.reality_dest = Some("127.0.0.1:8080"); p.reality_sid = Some("abc123");
                p.vless_flow = Some("xtls-rprx-vision");
            }
            Self::VlessTcpXtlVision => {
                p.protocol = VLess; p.tag = "vless-tcp-xtls"; p.security = Tls;
                p.tls_sni = Some("your-domain.com");
                p.tls_cert = Some("/etc/xray/certs/fullchain.pem");
                p.tls_key = Some("/etc/xray/certs/privkey.pem");
                p.vless_flow = Some("xtls-rprx-vision");
                p.sniff_on = false; p.sniff_http = false; p.sniff_tls = false; p.sniff_quic = false;
            }
            Self::VMessWsTls => {
                p.protocol = VMess; p.tag = "vmess-ws-tls"; p.transport = Ws;
                p.ws_path = Some("/ws"); p.ws_host = Some("your-domain.com");
                p.security = Tls; p.tls_sni = Some("your-domain.com");
                p.tls_cert = Some("/etc/xray/certs/fullchain.pem");
                p.tls_key = Some("/etc/xray/certs/privkey.pem");
            }
            Self::VMessWsCdn => {
                p.protocol = VMess; p.port = 80; p.tag = "vmess-ws-cdn";
                p.transport = Ws; p.ws_path = Some("/ws");
                p.ws_host = Some("your-cdn-domain.com");
            }
            Self::VMessGrpcTls => {
                p.protocol = VMess; p.tag = "vmess-grpc-tls"; p.transport = Grpc;
                p.grpc_service = Some("TunService"); p.grpc_multi = true;
                p.security = Tls; p.tls_sni = Some("your-domain.com");
                p.tls_cert = Some("/etc/xray/certs/fullchain.pem");
                p.tls_key = Some("/etc/xray/certs/privkey.pem");
            }
            Self::TrojanWsTls => {
                p.protocol = Trojan; p.tag = "trojan-ws-tls"; p.transport = Ws;
                p.ws_path = Some("/trojan"); p.security = Tls;
                p.tls_sni = Some("your-domain.com");
                p.tls_cert = Some("/etc/xray/certs/fullchain.pem");
                p.tls_key = Some("/etc/xray/certs/privkey.pem");
            }
            Self::TrojanGrpcTls => {
                p.protocol = Trojan; p.tag = "trojan-grpc-tls"; p.transport = Grpc;
                p.grpc_service = Some("TunService"); p.grpc_multi = true;
                p.security = Tls; p.tls_sni = Some("your-domain.com");
                p.tls_cert = Some("/etc/xray/certs/fullchain.pem");
                p.tls_key = Some("/etc/xray/certs/privkey.pem");
            }
            Self::ShadowsocksWsTls => {
                p.protocol = Shadowsocks; p.tag = "ss-ws-tls"; p.transport = Ws;
                p.ws_path = Some("/ss"); p.security = Tls;
                p.tls_sni = Some("your-domain.com");
                p.tls_cert = Some("/etc/xray/certs/fullchain.pem");
                p.tls_key = Some("/etc/xray/certs/privkey.pem");
            }
            Self::VlessHttpUpgradeReality => {
                p.protocol = VLess; p.tag = "vless-hup-reality";
                p.transport = HttpUpgrade; p.ws_path = Some("/");
                p.security = Reality; p.reality_sni = Some("www.microsoft.com");
                p.reality_dest = Some("127.0.0.1:8080"); p.reality_sid = Some("abc123");
                p.vless_flow = Some("xtls-rprx-vision");
            }
            Self::SocksLocal => {
                p.protocol = Socks; p.port = 1080; p.listen = "127.0.0.1";
                p.tag = "socks-in"; p.sniff_on = false; p.sniff_http = false;
                p.sniff_tls = false; p.sniff_quic = false;
            }
            Self::HttpLocal => {
                p.protocol = Http; p.port = 8080; p.listen = "127.0.0.1";
                p.tag = "http-in"; p.http_user = Some("admin");
                p.sniff_on = false; p.sniff_http = false; p.sniff_tls = false; p.sniff_quic = false;
            }
            Self::Custom => {}
        }
    }
}

impl InboundTemplate {
    pub fn info(&self) -> (&'static str, &'static str) {
        match self {
            Self::VlessWsTls => ("VLESS + WS + TLS", "最通用：WebSocket 走 CDN，TLS 加密，需域名+证书"),
            Self::VlessWsReality => ("VLESS + WS + Reality", "无需域名/证书：伪装微软等网站，反代特性"),
            Self::VlessGrpcReality => ("VLESS + gRPC + Reality", "gRPC 多路复用 + Reality，适合移动端 and 弱网"),
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
