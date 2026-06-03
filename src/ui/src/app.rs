use crossterm::event::{Event, KeyEvent, KeyCode, KeyModifiers};
use std::sync::Arc;
use std::time::Instant;

use xray_model::*;
use xray_services::*;
use xray_services::config_manager::ConfigEntry;

use crate::screens;
use crate::screens::{InboundWizardState, LogViewerState, UserEditMode, SettingsEditState};

#[derive(Debug, Clone)]
pub enum PickerAction {
    EditConfig,
    DeleteConfig,
    ToggleConfig,
    ManageUsers,
    CopyLink,
}

#[derive(Debug, Clone)]
pub enum Screen {
    Dashboard,
    InboundList,
    InboundWizard(InboundWizardState),
    UserManager { inbound_idx: usize, selected: usize, editing: Option<UserEditMode> },
    SslManagement { selected: usize, editing: Option<screens::ssl_manager::SslEditState> },
    LogViewer(LogViewerState),
    Settings(Option<SettingsEditState>),
    ConfirmDialog { message: String, on_confirm: ConfirmedAction },
    ConfigPicker { selected: usize, action: PickerAction },
    ShareExport { content: String },
    Others,
}

#[derive(Debug, Clone)]
pub enum ConfirmedAction {
    DeleteInbound(usize),
    DeleteUser { inbound_idx: usize, user_idx: usize },
    DeleteCert(usize),
    RestartXray,
    StopXray,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal, Editing, Selecting,
}

#[derive(Debug, Clone)]
pub enum Action {
    Navigate(Screen), PushScreen(Screen), PopScreen,
    SaveInbound(InboundConfig), UpdateInbound(usize, InboundConfig),
    DeleteInbound(usize), AddUser(usize, UserData), DeleteUser(usize, usize),
    RestartXray, StartXray, StopXray,
    InstallXray, UninstallXray, InstallSystemd,
    ExportSubscription, UpdateSettings(GlobalSettings), ShowMessage(String),
    IssueCertWithMethod { domain: String, method: String, webroot: Option<String>, cf_email: Option<String>, cf_key: Option<String> },
    ToggleInbound(usize),
    SaveUser { inbound_idx: usize, user_idx: usize, proto: String, labels: Vec<String>, values: Vec<String>, is_new: bool },
    GenerateRealityKeys,
    RefreshInbounds,
    GenerateDemo,
    EnableBBR, DisableBBR, CheckBBR,
    UpdateGeoFiles, SyncNTP, CheckNTP,
    Quit,
}

#[derive(Debug, Clone)]
pub struct UserData {
    pub uuid: Option<String>, pub password: Option<String>,
    pub flow: Option<String>, pub email: Option<String>, pub method: Option<String>,
}

use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub enum TaskResult {
    Message(String),
    CertIssued(CertInfo),
    Success,
    Error(String),
}

pub struct App {
    pub command_cursor: usize,
    pub current_screen: Screen,
    screen_history: Vec<Screen>,
    pub mode: InputMode,
    pub terminal_size: (u16, u16),
    pub inbounds: Vec<ConfigEntry>,
    pub certificates: Vec<CertInfo>,
    pub routing_rules: Vec<RoutingRule>,
    pub xray_status: XrayStatus,
    pub settings: GlobalSettings,
    pub xray_service: Arc<XrayService>,
    pub systemd_service: Arc<SystemdService>,
    pub storage: Arc<Storage>,
    pub config_manager: Arc<ConfigManager>,
    pub should_quit: bool,
    pub status_message: Option<(String, Instant)>,
    pub tick_count: u64,
    pub is_busy: bool,
    pub busy_msg: String,
    task_receiver: Receiver<TaskResult>,
    task_sender: Sender<TaskResult>,
}

impl App {
    pub fn new(xray_service: XrayService, systemd_service: SystemdService, storage: Storage, config_manager: ConfigManager, state: AppState) -> Self {
        let (tx, rx) = mpsc::channel();
        let inbounds = config_manager.load_configs().unwrap_or_default();
        Self {
            current_screen: Screen::Dashboard, screen_history: Vec::new(),
            command_cursor: 0,
            mode: InputMode::Normal, terminal_size: (80, 24),
            inbounds, certificates: state.stored_certs,
            routing_rules: RoutingRule::all_presets(),
            xray_status: XrayStatus::default(), settings: state.settings,
            xray_service: Arc::new(xray_service), systemd_service: Arc::new(systemd_service),
            storage: Arc::new(storage),
            config_manager: Arc::new(config_manager),
            should_quit: false, status_message: None, tick_count: 0,
            is_busy: false, busy_msg: String::new(),
            task_receiver: rx, task_sender: tx,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Resize(w, h) => { self.terminal_size = (w, h); }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        self.tick_count += 1;
        if self.tick_count % 30 == 0 { self.refresh_status(); }
        if let Some((_, ts)) = &self.status_message {
            if ts.elapsed().as_secs() > 3 { self.status_message = None; }
        }

        while let Ok(result) = self.task_receiver.try_recv() {
            self.is_busy = false;
            match result {
                TaskResult::Message(m) => self.show_msg(&m),
                TaskResult::CertIssued(c) => {
                    self.certificates.push(c.clone());
                    // Auto-update inbound configs that use TLS for this domain
                    let mut changed = false;
                    for entry in &mut self.inbounds {
                        if let Some(ref tls) = &entry.config.stream_settings.tls_settings {
                            if tls.server_name.as_deref() == Some(&c.domain) {
                                entry.config.stream_settings.tls_settings = Some(TlsSettings {
                                    server_name: tls.server_name.clone(),
                                    certificates: vec![TlsCertificate {
                                        certificate_file: c.cert_path.clone(),
                                        key_file: c.key_path.clone(),
                                    }],
                                    alpn: tls.alpn.clone(),
                                    min_version: tls.min_version.clone(),
                                });
                                let _ = self.config_manager.save_inbound(&entry.config, entry.enabled);
                                changed = true;
                            }
                        }
                    }
                    if changed {
                        self.inbounds = self.config_manager.load_configs().unwrap_or_default();
                        self.write_config();
                    }
                    let _ = self.storage.save_state(&self.settings, &self.certificates);
                    self.show_msg("Certificate issued + saved");
                }
                TaskResult::Error(e) => self.show_msg(&format!("Error: {}", e)),
                TaskResult::Success => self.show_msg("Success"),
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let action = match self.mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Editing => {
                if key.code == KeyCode::Esc { self.mode = InputMode::Normal; return; }
                self.handle_screen_key(key)
            }
            InputMode::Selecting => {
                if key.code == KeyCode::Esc {
                    if let Screen::InboundWizard(ref mut wiz) = self.current_screen { wiz.close_dropdowns(); }
                    self.mode = InputMode::Normal; return;
                }
                self.handle_screen_key(key)
            }
        };
        if let Some(action) = action { self.handle_action(action); }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('q') => { self.should_quit = true; return None; }
            KeyCode::Esc => {
                let handled = self.handle_escape();
                if !handled { self.pop_screen(); }
                // For Settings/Cert sub-menus, pass Esc through to screen handler for multi-level back
                if matches!(&self.current_screen, Screen::Settings(Some(_)) | Screen::SslManagement { editing: Some(_), .. }) {
                    return self.handle_screen_key(key);
                }
                return None;
            }
            KeyCode::Tab => {
                self.switch_tab(key.modifiers.contains(KeyModifiers::SHIFT));
                return None;
            }
            _ => {}
        }
        self.handle_screen_key(key)
    }

    fn handle_escape(&mut self) -> bool {
        match &mut self.current_screen {
            Screen::InboundWizard(ref mut wiz) => wiz.close_dropdowns(),
            Screen::SslManagement { editing, .. } => match editing {
                Some(_) => { *editing = None; true }
                None => false,
            },
            Screen::Settings(ref mut edit) => match edit {
                Some(ref mut ed) if ed.editing || ed.cf_sub.is_some() => true,  // let screen handler handle it
                Some(_) => { *edit = None; true }
                None => false,
            },
            _ => false,
        }
    }

    pub fn pop_screen(&mut self) {
        if self.mode != InputMode::Normal { self.mode = InputMode::Normal; return; }
        if let Some(prev) = self.screen_history.pop() { self.current_screen = prev; }
        else { self.current_screen = Screen::Dashboard; }
    }

    fn push_screen(&mut self, screen: Screen) {
        let old = std::mem::replace(&mut self.current_screen, screen);
        self.screen_history.push(old);
    }

    fn switch_tab(&mut self, reverse: bool) {
        let current = match &self.current_screen {
            Screen::Dashboard => 0, Screen::InboundList => 1,
            Screen::SslManagement { .. } => 2, Screen::LogViewer(_) => 3,
            Screen::Settings(_) => 4, Screen::Others => 5,
            _ => return,
        };
        let next = if reverse { (current + 5) % 6 } else { (current + 1) % 6 };
        self.screen_history.clear();
        self.mode = InputMode::Normal;
        self.command_cursor = 0;
        self.current_screen = match next {
            0 => Screen::Dashboard, 1 => Screen::InboundList,
            2 => Screen::SslManagement { selected: 0, editing: None },
            3 => Screen::LogViewer(LogViewerState::default()), 4 => Screen::Settings(None),
            5 => Screen::Others,
            _ => Screen::Dashboard,
        };
    }

    fn handle_screen_key(&mut self, key: KeyEvent) -> Option<Action> {
        match &mut self.current_screen {
            Screen::ConfirmDialog { on_confirm, .. } => {
                let confirmed = on_confirm.clone();
                return match key.code {
                    KeyCode::Enter => {
                        self.pop_screen();
                        match confirmed {
                            ConfirmedAction::DeleteInbound(idx) => Some(Action::DeleteInbound(idx)),
                            ConfirmedAction::DeleteUser { inbound_idx, user_idx } => Some(Action::DeleteUser(inbound_idx, user_idx)),
                            ConfirmedAction::DeleteCert(idx) => { self.certificates.remove(idx); Some(Action::ShowMessage("Deleted".into())) }
                            ConfirmedAction::RestartXray => Some(Action::RestartXray),
                            ConfirmedAction::StopXray => Some(Action::StopXray),
                        }
                    }
                    KeyCode::Esc => { self.pop_screen(); None }
                    _ => None,
                };
            }
            Screen::ShareExport { content } => {
                let c = content.clone();
                return match key.code {
                    KeyCode::Esc => { self.pop_screen(); None }
                    KeyCode::Char('y') => {
                        let pipe = std::process::Command::new("xclip").arg("-sel").arg("clip")
                            .stdin(std::process::Stdio::piped()).spawn();
                        if let Ok(mut child) = pipe {
                            if let Some(mut stdin) = child.stdin.take() {
                                use std::io::Write; let _ = stdin.write_all(c.as_bytes());
                            }
                            let _ = child.wait();
                        }
                        Some(Action::ShowMessage("Copied".into()))
                    }
                    KeyCode::Char('o') => {
                        let svg = crate::screens::qr_svg_data(&c);
                        let html = format!("<!DOCTYPE html><html><head><meta charset=utf-8><title>Xray QR</title></head><body style='margin:0;display:flex;justify-content:center;align-items:center;height:100vh;background:#fff'><img src='data:image/svg+xml;charset=utf-8,{}'></body></html>", svg);
                        let html = Arc::new(html);
                        let ip = self.settings.server_public_ip.clone().unwrap_or_else(|| "0.0.0.0".into());
                        let tx = self.task_sender.clone();
                        std::thread::spawn(move || {
                            let listener = match std::net::TcpListener::bind("0.0.0.0:0") {
                                Ok(l) => l,
                                Err(e) => { let _ = tx.send(TaskResult::Error(format!("Bind failed: {}", e))); return; }
                            };
                            let port = match listener.local_addr() {
                                Ok(a) => a.port(),
                                Err(_) => { let _ = tx.send(TaskResult::Error("No port".into())); return; }
                            };
                            let url = format!("http://{}:{}", ip, port);
                            let _ = tx.send(TaskResult::Message(format!("{} (closes in 5s)", url)));
                            let _ = listener.set_nonblocking(true);
                            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                            loop {
                                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                                if remaining.is_zero() { break; }
                                match listener.accept() {
                                    Ok((mut stream, _)) => {
                                        use std::io::Write;
                                        let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", html.len(), html);
                                        let _ = stream.write_all(resp.as_bytes());
                                        break;
                                    }
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                        std::thread::sleep(std::cmp::min(remaining, std::time::Duration::from_millis(100)));
                                    }
                                    Err(_) => break,
                                }
                            }
                        });
                        None
                    }
                    _ => None,
                };
            }
            Screen::ConfigPicker { selected, action: picker_action } => {
                let len = self.inbounds.len();
                return match key.code {
                    KeyCode::Up | KeyCode::Char('k') => { if *selected > 0 { *selected -= 1; } None }
                    KeyCode::Down | KeyCode::Char('j') => { if *selected + 1 < len { *selected += 1; } None }
                    KeyCode::Esc => { self.pop_screen(); None }
                    KeyCode::Enter if *selected < len => {
                        let idx = *selected;
                        let action = picker_action.clone();
                        self.pop_screen(); // back to InboundList
                        match action {
                            PickerAction::EditConfig => {
                                Some(Action::PushScreen(Screen::InboundWizard(InboundWizardState::edit(idx, self.inbounds[idx].config.clone()))))
                            }
                            PickerAction::DeleteConfig => {
                                Some(Action::PushScreen(Screen::ConfirmDialog {
                                    message: format!("Delete '{}'?", self.inbounds[idx].filename),
                                    on_confirm: ConfirmedAction::DeleteInbound(idx),
                                }))
                            }
                            PickerAction::ToggleConfig => Some(Action::ToggleInbound(idx)),
                            PickerAction::ManageUsers => {
                                Some(Action::PushScreen(Screen::UserManager { inbound_idx: idx, selected: 0, editing: None }))
                            }
                            PickerAction::CopyLink => {
                                let ip = self.settings.server_public_ip.clone().unwrap_or_else(|| "your-server-ip".into());
                                let server = xray_services::SubscriptionService::resolve_server_addr(&self.inbounds[idx].config, &ip).to_string();
                                match xray_services::SubscriptionService::generate_share_link(&self.inbounds[idx].config, &server, 0) {
                                    Some(l) => Some(Action::PushScreen(Screen::ShareExport { content: l })),
                                    None => Some(Action::ShowMessage("Sharing not supported for this protocol".into())),
                                }
                            }
                        }
                    }
                    _ => None,
                };
            }
            _ => {}
        }

        let mut screen = std::mem::replace(&mut self.current_screen, Screen::Dashboard);
        let action = match &mut screen {
            Screen::Dashboard => screens::dashboard::handle_key(key, self),
            Screen::InboundList => screens::inbound_list::handle_key(key, self),
            Screen::InboundWizard(ref mut wiz) => screens::wizard::handle_key(key, self, wiz),
            Screen::UserManager { selected, inbound_idx, editing } => {
                let mut edit = editing.take();
                let result = screens::user_manager::handle_key(key, self, selected, *inbound_idx, &mut edit);
                if edit.is_some() { *editing = edit; }
                result
            }
            Screen::SslManagement { selected, editing } => {
                let mut edit = editing.take();
                let result = screens::ssl_manager::handle_key(key, self, selected, &mut edit);
                if edit.is_some() { *editing = edit; }
                result
            }
            Screen::LogViewer(ref mut state) => screens::log_viewer::handle_key(key, self, state),
            Screen::Settings(editing) => {
                let mut edit = editing.take();
                let result = screens::settings_page::handle_key(key, self, &mut edit);
                if edit.is_some() { *editing = edit; }
                result
            }
            Screen::Others => screens::others::handle_key(key, self),
            _ => None,
        };
        self.current_screen = screen;
        action
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Navigate(s) => { self.screen_history.clear(); self.mode = InputMode::Normal; self.current_screen = s; }
            Action::PushScreen(s) => self.push_screen(s),
            Action::PopScreen => self.pop_screen(),
            Action::SaveInbound(inbound) => {
                if self.detect_port_conflict(&inbound, None) {
                    self.show_msg("Port conflict detected");
                    return;
                }
                if let Err(e) = self.config_manager.save_inbound(&inbound, true) {
                    self.show_msg(&format!("Failed to save: {}", e));
                    return;
                }
                match self.config_manager.load_configs() {
                    Ok(entries) => { self.inbounds = entries; self.show_msg("Inbound saved"); }
                    Err(e) => { self.show_msg(&format!("Reload failed: {}", e)); }
                }
                self.write_config();
                self.current_screen = Screen::InboundList;
            }
            Action::UpdateInbound(idx, inbound) => {
                if let Some(entry) = self.inbounds.get(idx) {
                    if self.detect_port_conflict(&inbound, Some(idx)) {
                        self.show_msg("Port conflict detected");
                        return;
                    }
                    let _ = self.config_manager.delete_config(&entry.filename);
                    if let Err(e) = self.config_manager.save_inbound(&inbound, entry.enabled) {
                        self.show_msg(&format!("Failed to save: {}", e));
                        return;
                    }
                    
                    match self.config_manager.load_configs() {
                        Ok(entries) => { self.inbounds = entries; self.show_msg("Updated"); }
                        Err(e) => { self.show_msg(&format!("Reload failed: {}", e)); }
                    }
                    self.write_config(); 
                }
                self.current_screen = Screen::InboundList;
            }
            Action::DeleteInbound(idx) => {
                if let Some(entry) = self.inbounds.get(idx) {
                    if let Err(e) = self.config_manager.delete_config(&entry.filename) {
                        self.show_msg(&format!("Failed to delete: {}", e));
                        return;
                    }
                    match self.config_manager.load_configs() {
                        Ok(entries) => { self.inbounds = entries; self.show_msg("Deleted"); }
                        Err(e) => { self.show_msg(&format!("Reload failed: {}", e)); }
                    }
                    self.write_config();
                }
                self.current_screen = Screen::InboundList;
            }
            Action::AddUser(inbound_idx, data) => {
                if let Some(entry) = self.inbounds.get_mut(inbound_idx) {
                    let inb = &mut entry.config;
                    match &mut inb.settings {
                        ProtocolSettings::VMess(s) => s.clients.push(VMessClient { id: data.uuid.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()), security: "auto".into(), email: data.email, level: None }),
                        ProtocolSettings::VLess(s) => s.clients.push(VLessClient { id: data.uuid.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()), flow: data.flow, email: data.email, level: None }),
                        ProtocolSettings::Trojan(s) => s.clients.push(TrojanClient { password: data.password.unwrap_or_default(), email: data.email, level: None }),
                        _ => {}
                    }
                    let _ = self.config_manager.save_inbound(inb, entry.enabled);
                    self.write_config(); self.show_msg("User added");
                }
            }
            Action::DeleteUser(inbound_idx, user_idx) => {
                if let Some(entry) = self.inbounds.get_mut(inbound_idx) {
                    let inb = &mut entry.config;
                    match &mut inb.settings {
                        ProtocolSettings::VMess(s) => { if user_idx < s.clients.len() { s.clients.remove(user_idx); } }
                        ProtocolSettings::VLess(s) => { if user_idx < s.clients.len() { s.clients.remove(user_idx); } }
                        ProtocolSettings::Trojan(s) => { if user_idx < s.clients.len() { s.clients.remove(user_idx); } }
                        _ => {}
                    }
                    let _ = self.config_manager.save_inbound(inb, entry.enabled);
                    self.write_config(); self.show_msg("User removed");
                }
            }
            Action::SaveUser { inbound_idx, user_idx, proto, labels, values, is_new } => {
                self.handle_save_user(inbound_idx, user_idx, &proto, &labels, &values, is_new);
                if let Some(entry) = self.inbounds.get(inbound_idx) {
                    let _ = self.config_manager.save_inbound(&entry.config, entry.enabled);
                }
            }
            Action::RestartXray => { let _ = self.systemd_service.restart(); self.refresh_status(); self.show_msg("Restarted"); }
            Action::StartXray => { let _ = self.systemd_service.start(); self.refresh_status(); self.show_msg("Started"); }
            Action::StopXray => { let _ = self.systemd_service.stop(); self.refresh_status(); self.show_msg("Stopped"); }
            Action::InstallXray => {
                self.is_busy = true; self.busy_msg = "Installing Xray...".into();
                let svc = self.xray_service.clone();
                let sysd = self.systemd_service.clone();
                let tx = self.task_sender.clone();
                std::thread::spawn(move || {
                    match svc.install_xray() {
                        Ok(_) => {
                            // Auto-generate systemd unit with correct binary/config paths
                            match sysd.install_unit_file() {
                                Ok(_) => { let _ = tx.send(TaskResult::Message("Xray installed + systemd unit created".into())); }
                                Err(e) => { let _ = tx.send(TaskResult::Message(format!("Xray installed but systemd unit failed: {}", e))); }
                            }
                        }
                        Err(e) => { let _ = tx.send(TaskResult::Error(format!("Install failed: {}", e))); }
                    }
                });
            }
            Action::UninstallXray => {
                self.is_busy = true; self.busy_msg = "Uninstalling Xray...".into();
                let svc = self.xray_service.clone();
                let sysd = self.systemd_service.clone();
                let tx = self.task_sender.clone();
                std::thread::spawn(move || {
                    match svc.uninstall_xray(sysd.unit_path()) {
                        Ok(_) => { let _ = tx.send(TaskResult::Message("Xray uninstalled".into())); }
                        Err(e) => { let _ = tx.send(TaskResult::Error(format!("Uninstall failed: {}", e))); }
                    }
                });
            }
            Action::InstallSystemd => {
                match self.systemd_service.install_unit_file() {
                    Ok(_) => self.show_msg("systemd unit installed"),
                    Err(e) => self.show_msg(&format!("Failed: {}", e)),
                }
            }
            Action::ExportSubscription => {
                let ip = self.settings.server_public_ip.clone().unwrap_or_else(|| "your-server-ip".into());
                let sub_inbounds: Vec<InboundConfig> = self.inbounds.iter().map(|e| e.config.clone()).collect();
                let sub = SubscriptionService::export_subscription(&sub_inbounds, &ip);
                self.current_screen = Screen::ShareExport { content: sub };
            }
            Action::ToggleInbound(idx) => {
                if let Some(entry) = self.inbounds.get(idx) {
                    match self.config_manager.toggle_enabled(&entry.filename) {
                        Ok(_) => {
                            self.inbounds = self.config_manager.load_configs().unwrap_or_default();
                            self.write_config(); self.show_msg("Toggled");
                        }
                        Err(e) => self.show_msg(&format!("Toggle failed: {}", e)),
                    }
                }
            }
            Action::GenerateRealityKeys => {
                match self.xray_service.generate_reality_keys() {
                    Ok((priv_key, pub_key)) => {
                        if let Screen::InboundWizard(ref mut wiz) = &mut self.current_screen {
                            wiz.builder.reality_public_key = Some(pub_key);
                            wiz.builder.reality_private_key = Some(priv_key);
                            for f in &mut wiz.fields {
                                if f.label == "Public Key" { f.value = wiz.builder.reality_public_key.clone().unwrap_or_default(); }
                                if f.label == "Private Key" { f.value = wiz.builder.reality_private_key.clone().unwrap_or_default(); }
                            }
                        }
                        self.show_msg("Reality keys generated");
                    }
                    Err(e) => self.show_msg(&format!("Key gen failed: {}", e)),
                }
            }
            Action::RefreshInbounds => {
                self.inbounds = self.config_manager.load_configs().unwrap_or_default();
                self.show_msg("Inbounds reloaded");
            }
            Action::IssueCertWithMethod { domain, method, webroot, cf_email, cf_key } => {
                self.is_busy = true; self.busy_msg = format!("Issuing cert for {}...", domain);
                let tx = self.task_sender.clone();
                std::thread::spawn(move || {
                    match xray_services::AcmeService::issue_cert(&domain, &method, webroot.as_deref(), cf_email.as_deref(), cf_key.as_deref()) {
                        Ok(_) => {
                            let cert = CertInfo {
                                domain: domain.clone(), cert_path: format!("/etc/xray/certs/{}/fullchain.pem", domain), key_path: format!("/etc/xray/certs/{}/privkey.pem", domain),
                                issued_at: chrono::Local::now().date_naive(), expires_at: chrono::Local::now().date_naive() + chrono::Duration::days(90), issuer: "Let's Encrypt".into(),
                                auto_renew: true, renew_command: Some("/root/.acme.sh/acme.sh --renew -d ".to_string() + &domain),
                            };
                            let _ = tx.send(TaskResult::CertIssued(cert));
                        }
                        Err(e) => { let _ = tx.send(TaskResult::Error(format!("Issue failed: {}", e))); }
                    }
                });
            }
            Action::UpdateSettings(s) => { self.settings = s; self.show_msg("Saved"); }
            Action::GenerateDemo => {
                self.show_msg("not implemented");
            }
            Action::ShowMessage(msg) => self.show_msg(&msg),
            Action::EnableBBR => {
                let msg = run_script("modprobe tcp_bbr && sysctl -w net.core.default_qdisc=fq && sysctl -w net.ipv4.tcp_congestion_control=bbr && echo 'net.core.default_qdisc=fq' >> /etc/sysctl.conf && echo 'net.ipv4.tcp_congestion_control=bbr' >> /etc/sysctl.conf");
                self.show_msg(&msg);
            }
            Action::DisableBBR => {
                let msg = run_script("sysctl -w net.ipv4.tcp_congestion_control=cubic && sysctl -w net.core.default_qdisc=fq_codel && sed -i '/tcp_congestion_control=/d;/default_qdisc=/d' /etc/sysctl.conf");
                self.show_msg(&msg);
            }
            Action::CheckBBR => {
                let msg = run_script("echo 'Congestion:'; sysctl net.ipv4.tcp_congestion_control; echo 'Qdisc:'; sysctl net.core.default_qdisc; echo 'Modules:'; lsmod | grep bbr");
                self.show_msg(&msg);
            }
            Action::UpdateGeoFiles => {
                self.is_busy = true; self.busy_msg = "Updating geo files...".into();
                let tx = self.task_sender.clone();
                std::thread::spawn(move || {
                    let result = run_script_default("curl -sL -o /usr/local/share/xray/geoip.dat https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geoip.dat && curl -sL -o /usr/local/share/xray/geosite.dat https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geosite.dat && echo 'Geo files updated' || echo 'FAILED'");
                    let _ = tx.send(TaskResult::Message(result));
                });
            }
            Action::SyncNTP => {
                let msg = run_script("ntpdate -s ntp.aliyun.com 2>/dev/null || chronyd -q 'server ntp.aliyun.com iburst' 2>/dev/null || timedatectl set-ntp true 2>/dev/null && echo 'NTP synced' || echo 'NTP sync failed'");
                self.show_msg(&msg);
            }
            Action::CheckNTP => {
                let msg = run_script("timedatectl show-timesync --property=NTPSynchronized 2>/dev/null || chronyc tracking 2>/dev/null || echo 'check timedatectl status'");
                self.show_msg(&msg);
            }
            Action::Quit => self.should_quit = true,
        }
    }

    fn write_config(&mut self) {
        let active_inbounds: Vec<InboundConfig> = self.inbounds.iter().filter(|e| e.enabled).map(|e| e.config.clone()).collect();
        let routing = RoutingConfig { domain_strategy: "IPIfNonMatch".into(), rules: self.routing_rules.clone() };
        let config = self.xray_service.generate_config(&active_inbounds, &routing);
        if let Err(e) = self.xray_service.write_config(&config) { self.show_msg(&format!("Failed to write: {}", e)); }
    }

    pub fn show_msg(&mut self, msg: &str) { self.status_message = Some((msg.to_string(), Instant::now())); }

    fn handle_save_user(&mut self, inbound_idx: usize, user_idx: usize, proto: &str, _labels: &[String], values: &[String], is_new: bool) {
        let Some(entry) = self.inbounds.get_mut(inbound_idx) else { return };
        let inb = &mut entry.config;
        match (&mut inb.settings, proto) {
            (ProtocolSettings::VMess(s), "VMess") => {
                if is_new { s.clients.push(VMessClient { id: values.get(0).cloned().unwrap_or_default(), security: values.get(1).cloned().unwrap_or_else(|| "auto".into()), email: Some(values.get(2).cloned().unwrap_or_default()).filter(|s| !s.is_empty()), level: None }); }
                else if let Some(c) = s.clients.get_mut(user_idx) { if let Some(v) = values.get(0) { c.id = v.clone(); } if let Some(v) = values.get(1) { c.security = v.clone(); } if let Some(v) = values.get(2) { c.email = if v.is_empty() { None } else { Some(v.clone()) }; } }
            }
            (ProtocolSettings::VLess(s), "VLESS") => {
                if is_new { s.clients.push(VLessClient { id: values.get(0).cloned().unwrap_or_default(), flow: Some(values.get(1).cloned().unwrap_or_default()).filter(|s| !s.is_empty()), email: Some(values.get(2).cloned().unwrap_or_default()).filter(|s| !s.is_empty()), level: None }); }
                else if let Some(c) = s.clients.get_mut(user_idx) { if let Some(v) = values.get(0) { c.id = v.clone(); } if let Some(v) = values.get(1) { c.flow = if v.is_empty() { None } else { Some(v.clone()) }; } if let Some(v) = values.get(2) { c.email = if v.is_empty() { None } else { Some(v.clone()) }; } }
            }
            (ProtocolSettings::Trojan(s), "Trojan") => {
                if is_new { s.clients.push(TrojanClient { password: values.get(0).cloned().unwrap_or_default(), email: Some(values.get(1).cloned().unwrap_or_default()).filter(|s| !s.is_empty()), level: None }); }
                else if let Some(c) = s.clients.get_mut(user_idx) { if let Some(v) = values.get(0) { c.password = v.clone(); } if let Some(v) = values.get(1) { c.email = if v.is_empty() { None } else { Some(v.clone()) }; } }
            }
            (ProtocolSettings::Shadowsocks(s), "Shadowsocks") => { if let Some(v) = values.get(0) { s.password = v.clone(); } if let Some(v) = values.get(1) { s.method = v.clone(); } if let Some(v) = values.get(2) { s.email = if v.is_empty() { None } else { Some(v.clone()) }; } }
            (ProtocolSettings::Http(s), "HTTP") => { if is_new { s.accounts.push(xray_model::HttpAccount { user: values.get(0).cloned().unwrap_or_default(), pass: values.get(1).cloned().unwrap_or_default() }); } else if let Some(a) = s.accounts.get_mut(user_idx) { if let Some(v) = values.get(0) { a.user = v.clone(); } if let Some(v) = values.get(1) { a.pass = v.clone(); } } }
            (ProtocolSettings::Socks(s), "SOCKS") => { if let SocksAuth::Password { accounts } = &mut s.auth { if is_new { accounts.push(xray_model::SocksAccount { user: values.get(0).cloned().unwrap_or_default(), pass: values.get(1).cloned().unwrap_or_default() }); } else if let Some(a) = accounts.get_mut(user_idx) { if let Some(v) = values.get(0) { a.user = v.clone(); } if let Some(v) = values.get(1) { a.pass = v.clone(); } } } }
            _ => {}
        }
        self.show_msg(if is_new { "User added" } else { "User updated" });
        self.current_screen = Screen::UserManager { inbound_idx, selected: user_idx, editing: None };
    }

    pub fn refresh_status(&mut self) { if let Ok(s) = self.systemd_service.get_status() { self.xray_status = s; } }
    pub fn save_and_quit(&self) { if let Err(e) = self.storage.save_state(&self.settings, &self.certificates) { eprintln!("Failed to save state: {}", e); } }
    fn detect_port_conflict(&self, inbound: &InboundConfig, skip_idx: Option<usize>) -> bool { self.inbounds.iter().enumerate().any(|(i, e)| { if skip_idx == Some(i) { return false; } e.config.port == inbound.port && e.config.listen == inbound.listen }) }
}

fn run_script(cmd: &str) -> String {
    use std::process::Command;
    let output = Command::new("sudo").args(["bash", "-c", cmd]).output();
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let trimmed = stdout.trim();
            if trimmed.is_empty() && !o.status.success() {
                String::from_utf8_lossy(&o.stderr).trim().to_string()
            } else {
                trimmed.to_string()
            }
        }
        Err(e) => format!("script error: {}", e),
    }
}

fn run_script_default(cmd: &str) -> String {
    run_script(cmd).trim().lines().last().unwrap_or("done").to_string()
}

