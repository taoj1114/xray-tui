use xray_model::{InboundProtocol, TransportNetwork, StreamSecurity};

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

impl InboundTemplate {
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
