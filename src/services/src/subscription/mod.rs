use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use xray_model::{InboundConfig, ProtocolSettings, TransportNetwork, StreamSecurity};

pub struct SubscriptionService;

impl SubscriptionService {
    pub fn generate_share_link(
        inbound: &InboundConfig,
        server_ip: &str,
        user_index: usize,
    ) -> Option<String> {
        match &inbound.settings {
            ProtocolSettings::VMess(settings) => {
                let client = settings.clients.get(user_index)?;
                Some(Self::vmess_link(inbound, server_ip, client.id.as_str(), client.security.as_str()))
            }
            ProtocolSettings::VLess(settings) => {
                let client = settings.clients.get(user_index)?;
                Some(Self::vless_link(inbound, server_ip, client.id.as_str(), client.flow.as_deref()))
            }
            ProtocolSettings::Trojan(settings) => {
                let client = settings.clients.get(user_index)?;
                Some(Self::trojan_link(inbound, server_ip, &client.password))
            }
            ProtocolSettings::Shadowsocks(settings) => {
                Some(Self::ss_link(inbound, server_ip, &settings.method, &settings.password))
            }
            _ => None,
        }
    }

    fn transport_params(inbound: &InboundConfig) -> Vec<(String, String)> {
        let mut params = vec![("type".into(), match inbound.stream_settings.network {
            TransportNetwork::Tcp => "tcp",
            TransportNetwork::Ws => "ws",
            TransportNetwork::Grpc => "grpc",
            TransportNetwork::HttpUpgrade => "httpupgrade",
            TransportNetwork::Quic => "quic",
            TransportNetwork::Kcp => "kcp",
            TransportNetwork::DomainSocket => "ds",
        }.into())];

        match &inbound.stream_settings.security {
            StreamSecurity::None => params.push(("security".into(), "none".into())),
            StreamSecurity::Tls => params.push(("security".into(), "tls".into())),
            StreamSecurity::Reality => params.push(("security".into(), "reality".into())),
        }

        if let Some(tls) = &inbound.stream_settings.tls_settings {
            if let Some(sni) = &tls.server_name {
                params.push(("sni".into(), sni.clone()));
            }
        }
        if let Some(reality) = &inbound.stream_settings.reality_settings {
            params.push(("sni".into(), reality.server_name.clone()));
            params.push(("fp".into(), reality.fingerprint.clone()));
            params.push(("pbk".into(), reality.public_key.clone()));
            if !reality.short_ids.is_empty() {
                params.push(("sid".into(), reality.short_ids[0].clone()));
            }
            params.push(("spx".into(), reality.spider_x.clone().unwrap_or_else(|| "/".into())));
        }
        if let Some(ws) = &inbound.stream_settings.ws_settings {
            params.push(("path".into(), ws.path.clone()));
            if let Some(host) = &ws.host {
                params.push(("host".into(), host.clone()));
            }
        }
        if let Some(grpc) = &inbound.stream_settings.grpc_settings {
            params.push(("serviceName".into(), grpc.service_name.clone()));
            if grpc.multi_mode {
                params.push(("mode".into(), "multi".into()));
            }
        }

        params
    }

    fn make_params(params: &[(String, String)]) -> String {
        params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }

    fn vmess_link(inbound: &InboundConfig, server: &str, uuid: &str, security: &str) -> String {
        let tls = match inbound.stream_settings.security {
            StreamSecurity::Tls => "tls",
            StreamSecurity::Reality => "reality",
            StreamSecurity::None => "",
        };
        let net = match inbound.stream_settings.network {
            TransportNetwork::Tcp => "tcp",
            TransportNetwork::Ws => "ws",
            TransportNetwork::Grpc => "grpc",
            TransportNetwork::HttpUpgrade => "h2",
            TransportNetwork::Quic => "quic",
            TransportNetwork::Kcp => "kcp",
            TransportNetwork::DomainSocket => "domainsocket",
        };
        let (path, host, sni) = Self::extract_transport_info(inbound);

        let vmess = serde_json::json!({
            "v": "2",
            "ps": inbound.tag.as_deref().unwrap_or(""),
            "add": server,
            "port": inbound.port.to_string(),
            "id": uuid,
            "aid": "0",
            "scy": security,
            "net": net,
            "type": "none",
            "host": host,
            "path": path,
            "tls": tls,
            "sni": sni,
            "alpn": "",
        });

        format!("vmess://{}", BASE64.encode(vmess.to_string()))
    }

    fn vless_link(inbound: &InboundConfig, server: &str, uuid: &str, flow: Option<&str>) -> String {
        let params = Self::transport_params(inbound);
        let mut param_parts: Vec<String> = params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        if let Some(flow) = flow {
            if !flow.is_empty() {
                param_parts.push(format!("flow={}", flow));
            }
        }
        let param_str = param_parts.join("&");
        let tag = urlencoding(inbound.tag.as_deref().unwrap_or(""));
        format!("vless://{}@{}:{}?{}#{}", uuid, server, inbound.port, param_str, tag)
    }

    fn trojan_link(inbound: &InboundConfig, server: &str, password: &str) -> String {
        let params = Self::transport_params(inbound);
        let tag = urlencoding(inbound.tag.as_deref().unwrap_or(""));
        format!("trojan://{}@{}:{}?{}#{}",
            urlencoding(password), server, inbound.port,
            Self::make_params(&params), tag)
    }

    fn ss_link(inbound: &InboundConfig, server: &str, method: &str, password: &str) -> String {
        let userinfo = BASE64.encode(format!("{}:{}", method, password));
        let tag = urlencoding(inbound.tag.as_deref().unwrap_or(""));
        format!("ss://{}@{}:{}#{}", userinfo, server, inbound.port, tag)
    }

    fn extract_transport_info(inbound: &InboundConfig) -> (String, String, String) {
        let path = inbound.stream_settings.ws_settings.as_ref()
            .map(|w| w.path.as_str()).unwrap_or("/");
        let host = inbound.stream_settings.ws_settings.as_ref()
            .and_then(|w| w.host.as_deref()).unwrap_or("");
        let sni = inbound.stream_settings.tls_settings.as_ref()
            .and_then(|t| t.server_name.as_deref()).unwrap_or("");
        (path.into(), host.into(), sni.into())
    }

    pub fn export_subscription(inbounds: &[InboundConfig], server_ip: &str) -> String {
        let links: Vec<String> = inbounds.iter()
            .flat_map(|inb| {
                let count = inb.user_count();
                (0..count).filter_map(move |i| Self::generate_share_link(inb, server_ip, i))
            })
            .collect();
        BASE64.encode(links.join("\n"))
    }

    pub fn detect_public_ip() -> Option<String> {
        duct::cmd!("curl", "-s", "-4", "ifconfig.me")
            .read()
            .ok()
            .map(|s| s.trim().to_string())
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '#' => "%23".into(),
            '&' => "%26".into(),
            '=' => "%3D".into(),
            '?' => "%3F".into(),
            '@' => "%40".into(),
            ':' => "%3A".into(),
            '/' => "%2F".into(),
            ' ' => "%20".into(),
            _ => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests;
