use super::*;

#[test]
fn test_vmess_tls_serialization() {
    let inbound = InboundConfig {
        tag: Some("vmess-tls".into()),
        port: 443,
        listen: "0.0.0.0".into(),
        protocol: InboundProtocol::VMess,
        settings: ProtocolSettings::VMess(VMessSettings {
            clients: vec![VMessClient { id: "uuid".into(), security: "auto".into(), email: None, level: None }],
        }),
        stream_settings: StreamSettings {
            network: TransportNetwork::Tcp,
            security: StreamSecurity::Tls,
            tls_settings: Some(TlsSettings {
                server_name: Some("example.com".into()),
                certificates: vec![TlsCertificate { certificate_file: "/path/to/cert".into(), key_file: "/path/to/key".into() }],
                alpn: vec!["h2".into(), "http/1.1".into()],
                min_version: None,
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

    let json = serde_json::to_string(&inbound).unwrap();
    assert!(json.contains("\"protocol\":\"vmess\""));
    assert!(json.contains("\"streamSettings\""));
}

#[test]
fn test_vless_reality_serialization() {
    let inbound = InboundConfig {
        tag: Some("vless-reality".into()),
        port: 443,
        listen: "0.0.0.0".into(),
        protocol: InboundProtocol::VLess,
        settings: ProtocolSettings::VLess(VLessSettings {
            clients: vec![VLessClient { id: "uuid".into(), flow: Some("xtls-rprx-vision".into()), email: None, level: None }],
            decryption: "none".into(),
        }),
        stream_settings: StreamSettings {
            network: TransportNetwork::Tcp,
            security: StreamSecurity::Reality,
            reality_settings: Some(RealitySettings {
                server_name: "example.com".into(),
                public_key: "pubkey".into(),
                private_key: "privkey".into(),
                short_ids: vec!["sid".into()],
                fingerprint: "chrome".into(),
                spider_x: None,
                dest: "1.1.1.1:443".into(),
            }),
            tls_settings: None,
            ws_settings: None,
            grpc_settings: None,
            httpupgrade_settings: None,
            tcp_settings: None,
            kcp_settings: None,
            quic_settings: None,
        },
        sniffing: SniffingConfig::default(),
    };

    let json = serde_json::to_string(&inbound).unwrap();
    assert!(json.contains("\"protocol\":\"vless\""));
    assert!(json.contains("\"realitySettings\""));
}

#[test]
fn test_trojan_ws_serialization() {
    let inbound = InboundConfig {
        tag: Some("trojan-ws".into()),
        port: 8080,
        listen: "0.0.0.0".into(),
        protocol: InboundProtocol::Trojan,
        settings: ProtocolSettings::Trojan(TrojanSettings {
            clients: vec![TrojanClient { password: "pass".into(), email: None, level: None }],
            fallbacks: vec![],
        }),
        stream_settings: StreamSettings {
            network: TransportNetwork::Ws,
            security: StreamSecurity::None,
            ws_settings: Some(WsSettings { path: "/ws".into(), host: None, headers: None }),
            tls_settings: None,
            reality_settings: None,
            grpc_settings: None,
            httpupgrade_settings: None,
            tcp_settings: None,
            kcp_settings: None,
            quic_settings: None,
        },
        sniffing: SniffingConfig::default(),
    };

    let json = serde_json::to_string(&inbound).unwrap();
    assert!(json.contains("\"protocol\":\"trojan\""));
    assert!(json.contains("\"wsSettings\""));
}

#[test]
fn test_vmess_grpc_serialization() {
    let inbound = InboundConfig {
        tag: Some("vmess-grpc".into()),
        port: 443,
        listen: "0.0.0.0".into(),
        protocol: InboundProtocol::VMess,
        settings: ProtocolSettings::VMess(VMessSettings {
            clients: vec![VMessClient { id: "uuid".into(), security: "auto".into(), email: None, level: None }],
        }),
        stream_settings: StreamSettings {
            network: TransportNetwork::Grpc,
            security: StreamSecurity::Tls,
            grpc_settings: Some(GrpcSettings { service_name: "grpc".into(), multi_mode: false, authority: None }),
            tls_settings: Some(TlsSettings {
                server_name: Some("example.com".into()),
                certificates: vec![],
                alpn: vec![],
                min_version: None,
            }),
            reality_settings: None,
            ws_settings: None,
            httpupgrade_settings: None,
            tcp_settings: None,
            kcp_settings: None,
            quic_settings: None,
        },
        sniffing: SniffingConfig::default(),
    };

    let json = serde_json::to_string(&inbound).unwrap();
    assert!(json.contains("\"grpcSettings\""));
}

#[test]
fn test_socks_serialization() {
    let inbound = InboundConfig {
        tag: Some("socks".into()),
        port: 1080,
        listen: "127.0.0.1".into(),
        protocol: InboundProtocol::Socks,
        settings: ProtocolSettings::Socks(SocksSettings {
            auth: SocksAuth::NoAuth {},
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
        sniffing: SniffingConfig::default(),
    };

    let json = serde_json::to_string(&inbound).unwrap();
    assert!(json.contains("\"protocol\":\"socks\""));
}

#[test]
fn test_full_xray_config() {
    let empty: Vec<InboundConfig> = vec![];
    let routing = RoutingConfig::default();
    let config = XrayConfig::from_inbounds(&empty, &routing);
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("\"log\""));
    assert!(json.contains("\"outbounds\""));
    assert!(json.contains("\"freedom\""));
}
