use xray_model::*;
use super::builder::InboundConfigBuilder;
use super::templates::InboundTemplate;

#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep { Template = 0, Basic = 1, Transport = 2, Sniffing = 3, Security = 4, Users = 5, Confirm = 6 }

#[derive(Debug, Clone, PartialEq)]
pub enum WizardFieldType { Dropdown, TextInput, Toggle, Button }

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
    pub fn dropdown(label: &str, value: &str, options: Vec<String>) -> Self {
        let selected = options.iter().position(|o| o == value).unwrap_or(0);
        Self { label: label.into(), field_type: WizardFieldType::Dropdown, value: value.into(), options, selected_option: selected, is_open: false }
    }
    pub fn text(label: &str, value: &str) -> Self {
        Self { label: label.into(), field_type: WizardFieldType::TextInput, value: value.into(), options: vec![], selected_option: 0, is_open: false }
    }
    pub fn toggle(label: &str, value: bool) -> Self {
        Self { label: label.into(), field_type: WizardFieldType::Toggle, value: if value { "true" } else { "false" }.into(), options: vec![], selected_option: 0, is_open: false }
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
    pub error_msg: Option<String>,
}

impl InboundWizardState {
    pub fn new() -> Self {
        Self { current_step: WizardStep::Template, edit_index: None, builder: InboundConfigBuilder::default(),
            fields: Self::build_template_fields(), focused: 0, auto_restart: false, json_preview: String::new(), selected_template: 0, error_msg: None }
    }

    pub fn edit(index: usize, inbound: InboundConfig) -> Self {
        let mut wiz = Self::new();
        wiz.edit_index = Some(index);
        wiz.current_step = WizardStep::Basic;
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
            WizardField { label: name.into(), field_type: WizardFieldType::Button,
                value: format!("  {}  │  {}", name, desc), options: vec![], selected_option: i, is_open: false }
        }).collect()
    }

    pub fn build_step_fields(step: WizardStep, builder: &InboundConfigBuilder) -> Vec<WizardField> {
        match step {
            WizardStep::Template => Self::build_template_fields(),
            WizardStep::Basic => vec![
                WizardField::dropdown("Protocol", &builder.protocol.to_string(),
                    vec!["VMess".into(),"VLESS".into(),"Trojan".into(),"Shadowsocks".into(),"HTTP".into(),"SOCKS".into()]),
                WizardField::text("Port", &builder.port.to_string()),
                WizardField::text("Listen", &builder.listen),
                WizardField::text("Tag", builder.tag.as_deref().unwrap_or("")),
            ],
            WizardStep::Transport => {
                let mut fields = vec![WizardField::dropdown("Network", &builder.transport.to_string(),
                    vec!["TCP".into(), "WebSocket".into(), "gRPC".into()])];
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
                WizardField::toggle("Sniffing", builder.sniffing_enabled),
                WizardField::toggle("HTTP", builder.sniffing_http),
                WizardField::toggle("TLS", builder.sniffing_tls),
                WizardField::toggle("QUIC", builder.sniffing_quic),
                WizardField::toggle("Route Only", builder.sniffing_route_only),
            ],
            WizardStep::Security => {
                let mut fields = vec![WizardField::dropdown("Security", &builder.security.to_string(),
                    vec!["none".into(), "TLS".into(), "Reality".into()])];
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
                    WizardField::dropdown("Method", &builder.ss_method, vec!["aes-256-gcm".into(), "chacha20-ietf-poly1305".into()]),
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
        let mut any = false;
        for f in &mut self.fields { if f.is_open { f.is_open = false; any = true; } }
        any
    }

    pub fn next_step(&mut self) -> Option<String> {
        self.apply_fields();
        if let Some(err) = self.validate() { self.error_msg = Some(err); return self.error_msg.clone(); }
        self.error_msg = None;
        match self.current_step {
            WizardStep::Template => self.current_step = WizardStep::Basic,
            WizardStep::Basic => self.current_step = WizardStep::Transport,
            WizardStep::Transport => self.current_step = WizardStep::Sniffing,
            WizardStep::Sniffing => self.current_step = WizardStep::Security,
            WizardStep::Security => self.current_step = WizardStep::Users,
            WizardStep::Users => { self.json_preview = serde_json::to_string_pretty(&self.builder.build()).unwrap_or_default(); self.current_step = WizardStep::Confirm; }
            WizardStep::Confirm => {}
        }
        if self.current_step != WizardStep::Confirm { self.fields = Self::build_step_fields(self.current_step.clone(), &self.builder); }
        self.focused = 0;
        None
    }

    fn validate(&self) -> Option<String> {
        match self.current_step {
            WizardStep::Basic => {
                let b = &self.builder;
                if b.port == 0 || b.port < 1 || b.port > 65535 { return Some("Port must be 1–65535".into()); }
                if b.listen.is_empty() { return Some("Listen address is required".into()); }
            }
            WizardStep::Transport => {
                let b = &self.builder;
                if b.transport == TransportNetwork::Ws {
                    if let Some(ref p) = b.ws_path { if !p.starts_with('/') { return Some("WS Path must start with /".into()); } }
                }
            }
            WizardStep::Users => {
                let b = &self.builder;
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
        if self.current_step != WizardStep::Confirm { self.fields = Self::build_step_fields(self.current_step.clone(), &self.builder); }
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
                    self.builder.transport = match f.value.as_str() { "TCP"=>TransportNetwork::Tcp, "WebSocket"|"ws"=>TransportNetwork::Ws, "gRPC"|"grpc"=>TransportNetwork::Grpc, "HTTPUpgrade"=>TransportNetwork::HttpUpgrade, _=>TransportNetwork::Tcp };
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
                if let Some(f) = self.fields.get(0) { self.builder.security = match f.value.as_str() { "TLS"|"tls"=>StreamSecurity::Tls,"Reality"|"reality"=>StreamSecurity::Reality,_=>StreamSecurity::None }; }
                if let Some(f) = self.fields.get(1) { self.builder.tls_server_name = Some(f.value.clone()); }
                if let Some(f) = self.fields.get(2) { self.builder.tls_cert_path = Some(f.value.clone()); }
                if let Some(f) = self.fields.get(3) { self.builder.tls_key_path = Some(f.value.clone()); }
            }
            WizardStep::Users => {
                if let Some(f) = self.fields.get(0) { self.builder.uuid = Some(f.value.clone()); self.builder.password = Some(f.value.clone()); }
                if let Some(f) = self.fields.get(1) { self.builder.email = Some(f.value.clone()); }
            }
            _ => {}
        }
    }

    fn parse_protocol(s: &str) -> InboundProtocol {
        match s { "VMess"=>InboundProtocol::VMess,"VLESS"=>InboundProtocol::VLess,"Trojan"=>InboundProtocol::Trojan,"Shadowsocks"=>InboundProtocol::Shadowsocks,"HTTP"=>InboundProtocol::Http,"SOCKS"=>InboundProtocol::Socks,_=>InboundProtocol::VMess }
    }
}
