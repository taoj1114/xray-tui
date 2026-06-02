use super::*;
use xray_model::*;

fn make_vless_inbound() -> InboundConfig {
    InboundConfig {
        tag: Some("test-vless".into()),
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
                public_key: "pub_key".into(),
                private_key: "priv_key".into(),
                short_ids: vec!["abc123".into()],
                fingerprint: "chrome".into(),
                spider_x: None,
                dest: "127.0.0.1:8080".into(),
            }),
            ws_settings: Some(WsSettings { path: "/ws".into(), host: None, headers: None }),
            grpc_settings: None,
            httpupgrade_settings: None,
            tcp_settings: None,
            kcp_settings: None,
            quic_settings: None,
        },
        sniffing: SniffingConfig::default(),
    }
}

#[test]
fn test_vless_share_link() {
    let inbound = make_vless_inbound();
    let link = SubscriptionService::generate_share_link(&inbound, "1.2.3.4", 0).unwrap();
    assert!(link.starts_with("vless://"));
    assert!(link.contains("d290f1ee"));
    assert!(link.contains("1.2.3.4:443"));
    assert!(link.contains("security=reality"));
    assert!(link.contains("flow=xtls-rprx-vision"));
    assert!(link.contains("test-vless"));
}

#[test]
fn test_vmess_share_link() {
    let inbound = InboundConfig {
        tag: Some("test-vmess".into()),
        port: 10084,
        listen: "0.0.0.0".into(),
        protocol: InboundProtocol::VMess,
        settings: ProtocolSettings::VMess(VMessSettings {
            clients: vec![VMessClient {
                id: "abc123-def456-gh789".into(),
                security: "auto".into(),
                email: None,
                level: None,
            }],
        }),
        stream_settings: StreamSettings {
            network: TransportNetwork::Tcp,
            security: StreamSecurity::Tls,
            tls_settings: Some(TlsSettings {
                server_name: Some("vm.example.com".into()),
                certificates: vec![],
                alpn: vec![],
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
    let link = SubscriptionService::generate_share_link(&inbound, "2.3.4.5", 0).unwrap();
    assert!(link.starts_with("vmess://"));
}

#[test]
fn test_export_subscription() {
    let inbounds = vec![make_vless_inbound()];
    let sub = SubscriptionService::export_subscription(&inbounds, "1.2.3.4");
    assert!(!sub.is_empty());
    let decoded = BASE64.decode(&sub).unwrap();
    let text = String::from_utf8(decoded).unwrap();
    assert!(text.contains("vless://"));
}
