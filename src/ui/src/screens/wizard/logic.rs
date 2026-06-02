use xray_model::*;
use super::state::{WizardStep, InboundWizardState};

pub fn apply_fields(state: &mut InboundWizardState) {
    match state.current_step {
        WizardStep::Template => {},
        WizardStep::Basic => {
            if let Some(f) = state.fields.get(0) { state.builder.protocol = parse_protocol(&f.value); }
            if let Some(f) = state.fields.get(1) { state.builder.port = f.value.parse().unwrap_or(443); }
            if let Some(f) = state.fields.get(2) { state.builder.listen = f.value.clone(); }
            if let Some(f) = state.fields.get(3) { state.builder.tag = if f.value.is_empty() { None } else { Some(f.value.clone()) }; }
        }
        WizardStep::Transport => {
            if let Some(f) = state.fields.get(0) {
                state.builder.transport = match f.value.as_str() { "TCP"=>TransportNetwork::Tcp, "WebSocket"|"ws"=>TransportNetwork::Ws, "gRPC"|"grpc"=>TransportNetwork::Grpc, "HTTPUpgrade"=>TransportNetwork::HttpUpgrade, _=>TransportNetwork::Tcp };
            }
            if let Some(f) = state.fields.get(1) { state.builder.ws_path = Some(f.value.clone()); }
            if let Some(f) = state.fields.get(2) { state.builder.ws_host = Some(f.value.clone()); }
        }
        WizardStep::Sniffing => {
            if let Some(f) = state.fields.get(0) { state.builder.sniffing_enabled = f.value == "true"; }
            if let Some(f) = state.fields.get(1) { state.builder.sniffing_http = f.value == "true"; }
            if let Some(f) = state.fields.get(2) { state.builder.sniffing_tls = f.value == "true"; }
            if let Some(f) = state.fields.get(3) { state.builder.sniffing_quic = f.value == "true"; }
            if let Some(f) = state.fields.get(4) { state.builder.sniffing_route_only = f.value == "true"; }
        }
        WizardStep::Security => {
            if let Some(f) = state.fields.get(0) { state.builder.security = match f.value.as_str() { "TLS"|"tls"=>StreamSecurity::Tls,"Reality"|"reality"=>StreamSecurity::Reality,_=>StreamSecurity::None }; }
            if let Some(f) = state.fields.get(1) { state.builder.tls_server_name = Some(f.value.clone()); }
            if let Some(f) = state.fields.get(2) { state.builder.tls_cert_path = Some(f.value.clone()); }
            if let Some(f) = state.fields.get(3) { state.builder.tls_key_path = Some(f.value.clone()); }
        }
        WizardStep::Users => {
            if let Some(f) = state.fields.get(0) { state.builder.uuid = Some(f.value.clone()); state.builder.password = Some(f.value.clone()); }
            if let Some(f) = state.fields.get(1) { state.builder.email = Some(f.value.clone()); }
        }
        _ => {}
    }
}

pub fn validate(state: &InboundWizardState) -> Option<String> {
    match state.current_step {
        WizardStep::Basic => {
            let b = &state.builder;
            if b.port == 0 { return Some("Port must be 1–65535".into()); }
            if b.listen.is_empty() { return Some("Listen address is required".into()); }
        }
        WizardStep::Transport => {
            let b = &state.builder;
            if b.transport == TransportNetwork::Ws {
                if let Some(ref p) = b.ws_path { if !p.starts_with('/') { return Some("WS Path must start with /".into()); } }
            }
        }
        WizardStep::Users => {
            let b = &state.builder;
            match b.protocol {
                InboundProtocol::VMess | InboundProtocol::VLess => {
                    if let Some(ref id) = b.uuid {
                        if uuid::Uuid::parse_str(id).is_err() { return Some("UUID format is invalid".into()); }
                    }
                }
                InboundProtocol::Trojan | InboundProtocol::Shadowsocks => {
                    if b.password.as_ref().map_or(false, |p| p.is_empty()) { return Some("Password is required".into()); }
                }
                _ => {}
            }
        }
        _ => {}
    }
    None
}

fn parse_protocol(s: &str) -> InboundProtocol {
    match s { "VMess"=>InboundProtocol::VMess,"VLESS"=>InboundProtocol::VLess,"Trojan"=>InboundProtocol::Trojan,"Shadowsocks"=>InboundProtocol::Shadowsocks,"HTTP"=>InboundProtocol::Http,"SOCKS"=>InboundProtocol::Socks,_=>InboundProtocol::VMess }
}
