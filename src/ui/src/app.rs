use ratatui::{Frame, layout::{Layout, Constraint, Direction, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Paragraph, Tabs}};
use crossterm::event::{Event, KeyEvent, KeyCode, KeyModifiers};
use std::sync::Arc;
use std::time::Instant;

use xray_model::*;
use xray_services::*;

use crate::screens;
use crate::screens::{InboundWizardState, LogViewerState, RoutingEditMode, UserEditMode, SslEditState};

#[derive(Debug, Clone)]
pub enum Screen {
    Dashboard,
    InboundList { selected: usize },
    InboundWizard(InboundWizardState),
    UserManager { inbound_idx: usize, selected: usize, editing: Option<UserEditMode> },
    RoutingEditor { selected: usize, editing: Option<RoutingEditMode> },
    SslManagement { selected: usize, editing: Option<screens::ssl_manager::SslEditState> },
    LogViewer(LogViewerState),
    Settings,
    ConfirmDialog { message: String, on_confirm: ConfirmedAction },
    ShareExport { content: String },
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
    SaveRouting(Vec<RoutingRule>), RestartXray, StartXray, StopXray,
    InstallXray, UninstallXray, InstallSystemd,
    ExportSubscription, UpdateSettings(GlobalSettings), ShowMessage(String),
    IssueCertWithMethod { domain: String, method: String, webroot: Option<String>, cf_email: Option<String>, cf_key: Option<String> },
    ToggleInbound(usize),
    SaveUser { inbound_idx: usize, user_idx: usize, proto: String, labels: Vec<String>, values: Vec<String>, is_new: bool },
    GenerateRealityKeys,
    Quit,
}

#[derive(Debug, Clone)]
pub struct UserData {
    pub uuid: Option<String>, pub password: Option<String>,
    pub flow: Option<String>, pub email: Option<String>, pub method: Option<String>,
}

pub struct App {
    pub command_cursor: usize,
    pub current_screen: Screen,
    screen_history: Vec<Screen>,
    pub mode: InputMode,
    pub terminal_size: (u16, u16),
    pub inbounds: Vec<InboundConfig>,
    pub certificates: Vec<CertInfo>,
    pub routing_rules: Vec<RoutingRule>,
    pub xray_status: XrayStatus,
    pub settings: GlobalSettings,
    pub xray_service: Arc<XrayService>,
    pub systemd_service: Arc<SystemdService>,
    pub storage: Arc<Storage>,
    pub should_quit: bool,
    pub status_message: Option<(String, Instant)>,
    tick_count: u64,
}

impl App {
    pub fn new(xray_service: XrayService, systemd_service: SystemdService, storage: Storage, state: AppState) -> Self {
        Self {
            current_screen: Screen::Dashboard, screen_history: Vec::new(),
            command_cursor: 0,
            mode: InputMode::Normal, terminal_size: (80, 24),
            inbounds: vec![], certificates: state.stored_certs,
            routing_rules: RoutingRule::all_presets(),
            xray_status: XrayStatus::default(), settings: state.settings,
            xray_service: Arc::new(xray_service), systemd_service: Arc::new(systemd_service),
            storage: Arc::new(storage),
            should_quit: false, status_message: None, tick_count: 0,
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
                if !self.handle_escape() { self.pop_screen(); }
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
            Screen::InboundList { .. } if self.screen_history.is_empty() => {
                self.current_screen = Screen::Dashboard; true
            }
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
            Screen::Dashboard => 0, Screen::InboundList { .. } => 1, Screen::RoutingEditor { .. } => 2,
            Screen::SslManagement { .. } => 3, Screen::LogViewer(_) => 4, Screen::Settings => 5,
            _ => return,
        };
        let next = if reverse { (current + 5) % 6 } else { (current + 1) % 6 };
        self.screen_history.clear();
        self.mode = InputMode::Normal;
        self.command_cursor = 0;
        self.current_screen = match next {
            0 => Screen::Dashboard, 1 => Screen::InboundList { selected: 0 },
            2 => Screen::RoutingEditor { selected: 0, editing: None },
            3 => Screen::SslManagement { selected: 0, editing: None },
            4 => Screen::LogViewer(LogViewerState::default()), 5 => Screen::Settings,
            _ => Screen::Dashboard,
        };
    }

    fn handle_screen_key(&mut self, key: KeyEvent) -> Option<Action> {
        // Screens like ConfirmDialog and ShareExport mutate screen_history via pop_screen,
        // so handle them here directly (they don't need mem::replace borrow splitting).
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
                    _ => None,
                };
            }
            _ => {}
        }

        // Remaining screens: extract state, dispatch, restore
        let mut screen = std::mem::replace(&mut self.current_screen, Screen::Dashboard);
        let action = match &mut screen {
            Screen::Dashboard => screens::dashboard::handle_key(key, self),
            Screen::InboundList { selected } => screens::inbound_list::handle_key(key, self, selected),
            Screen::InboundWizard(ref mut wiz) => screens::wizard::handle_key(key, self, wiz),
            Screen::UserManager { selected, inbound_idx, editing } => {
                let mut edit = editing.take();
                let result = screens::user_manager::handle_key(key, self, selected, *inbound_idx, &mut edit);
                if edit.is_some() { *editing = edit; }
                result
            }
            Screen::RoutingEditor { selected, editing } => screens::routing_editor::handle_key(key, self, selected, editing),
            Screen::SslManagement { selected, editing } => {
                let mut edit = editing.take();
                let result = screens::ssl_manager::handle_key(key, self, selected, &mut edit);
                if edit.is_some() { *editing = edit; }
                result
            }
            Screen::LogViewer(ref mut state) => screens::log_viewer::handle_key(key, self, state),
            Screen::Settings => screens::settings_page::handle_key(key, self),
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
                    self.show_msg("Port conflict — another inbound already uses this port");
                    return;
                }
                self.inbounds.push(inbound); self.write_config(); self.show_msg("Inbound saved");
                self.current_screen = Screen::InboundList { selected: self.inbounds.len().saturating_sub(1) };
            }
            Action::UpdateInbound(idx, inbound) => {
                if idx < self.inbounds.len() {
                    if self.detect_port_conflict(&inbound, Some(idx)) {
                        self.show_msg("Port conflict — another inbound already uses this port");
                        return;
                    }
                    self.inbounds[idx] = inbound; self.write_config(); self.show_msg("Updated");
                }
                self.current_screen = Screen::InboundList { selected: idx };
            }
            Action::DeleteInbound(idx) => {
                if idx < self.inbounds.len() { self.inbounds.remove(idx); self.write_config(); self.show_msg("Deleted"); }
                self.current_screen = Screen::InboundList { selected: idx.min(self.inbounds.len().saturating_sub(1)) };
            }
            Action::AddUser(inbound_idx, data) => {
                if inbound_idx < self.inbounds.len() {
                    let inb = &mut self.inbounds[inbound_idx];
                    match &mut inb.settings {
                        ProtocolSettings::VMess(s) => s.clients.push(VMessClient { id: data.uuid.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()), security: "auto".into(), email: data.email, level: None }),
                        ProtocolSettings::VLess(s) => s.clients.push(VLessClient { id: data.uuid.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()), flow: data.flow, email: data.email, level: None }),
                        ProtocolSettings::Trojan(s) => s.clients.push(TrojanClient { password: data.password.unwrap_or_default(), email: data.email, level: None }),
                        _ => {}
                    }
                    self.write_config(); self.show_msg("User added");
                }
            }
            Action::DeleteUser(inbound_idx, user_idx) => {
                if inbound_idx < self.inbounds.len() {
                    let inb = &mut self.inbounds[inbound_idx];
                    match &mut inb.settings {
                        ProtocolSettings::VMess(s) => { if user_idx < s.clients.len() { s.clients.remove(user_idx); } }
                        ProtocolSettings::VLess(s) => { if user_idx < s.clients.len() { s.clients.remove(user_idx); } }
                        ProtocolSettings::Trojan(s) => { if user_idx < s.clients.len() { s.clients.remove(user_idx); } }
                        _ => {}
                    }
                    self.write_config(); self.show_msg("User removed");
                }
            }
            Action::SaveUser { inbound_idx, user_idx, proto, labels, values, is_new } => {
                self.handle_save_user(inbound_idx, user_idx, &proto, &labels, &values, is_new);
            }
            Action::SaveRouting(rules) => {
                self.routing_rules = rules; self.write_config(); self.show_msg("Saved");
                self.current_screen = Screen::RoutingEditor { selected: 0, editing: None };
            }
            Action::RestartXray => { let _ = self.systemd_service.restart(); self.refresh_status(); self.show_msg("Restarted"); }
            Action::StartXray => { let _ = self.systemd_service.start(); self.refresh_status(); self.show_msg("Started"); }
            Action::StopXray => { let _ = self.systemd_service.stop(); self.refresh_status(); self.show_msg("Stopped"); }
            Action::InstallXray => {
                self.show_msg("Installing Xray via official script...");
                match self.xray_service.install_xray() {
                    Ok(_) => { self.refresh_status(); self.show_msg("Xray installed successfully"); }
                    Err(e) => self.show_msg(&format!("Install failed: {}", e)),
                }
            }
            Action::UninstallXray => {
                self.show_msg("Uninstalling Xray...");
                match self.xray_service.uninstall_xray() {
                    Ok(_) => { self.refresh_status(); self.show_msg("Xray uninstalled"); }
                    Err(e) => self.show_msg(&format!("Uninstall failed: {}", e)),
                }
            }
            Action::InstallSystemd => {
                match self.systemd_service.install_unit_file() {
                    Ok(_) => self.show_msg("systemd unit installed & enabled"),
                    Err(e) => self.show_msg(&format!("systemd setup failed: {}", e)),
                }
            }
            Action::ExportSubscription => {
                let ip = self.settings.server_public_ip.clone().unwrap_or_else(|| "your-server-ip".into());
                let sub = SubscriptionService::export_subscription(&self.inbounds, &ip);
                self.current_screen = Screen::ShareExport { content: sub };
            }
            Action::ToggleInbound(_) => { self.show_msg("Toggled"); }
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
            Action::IssueCertWithMethod { domain, method, webroot, cf_email, cf_key } => {
                match xray_services::AcmeService::issue_cert(&domain, &method, webroot.as_deref(), cf_email.as_deref(), cf_key.as_deref()) {
                    Ok(_) => {
                        self.certificates.push(CertInfo {
                            domain: domain.clone(),
                            cert_path: format!("/etc/xray/certs/{}/fullchain.pem", domain),
                            key_path: format!("/etc/xray/certs/{}/privkey.pem", domain),
                            issued_at: chrono::Local::now().date_naive(),
                            expires_at: chrono::Local::now().date_naive() + chrono::Duration::days(90),
                            issuer: "Let's Encrypt".into(),
                            auto_renew: true,
                            renew_command: Some("/root/.acme.sh/acme.sh --renew -d ".to_string() + &domain),
                        });
                        self.show_msg(&format!("Cert issued for {}", domain));
                    }
                    Err(e) => self.show_msg(&format!("Issue failed: {}", e)),
                }
            }
            Action::UpdateSettings(s) => { self.settings = s; self.show_msg("Saved"); self.current_screen = Screen::Dashboard; }
            Action::ShowMessage(msg) => self.show_msg(&msg),
            Action::Quit => self.should_quit = true,
        }
    }

    fn write_config(&self) {
        let routing = RoutingConfig { domain_strategy: "IPIfNonMatch".into(), rules: self.routing_rules.clone() };
        let config = self.xray_service.generate_config(&self.inbounds, &routing);
        if let Err(e) = self.xray_service.write_config(&config) {
            eprintln!("Failed to write config: {}", e);
        }
    }

    pub fn show_msg(&mut self, msg: &str) { self.status_message = Some((msg.to_string(), Instant::now())); }

    fn handle_save_user(&mut self, inbound_idx: usize, user_idx: usize, proto: &str, labels: &[String], values: &[String], is_new: bool) {
        let Some(inb) = self.inbounds.get_mut(inbound_idx) else { return };
        match (&mut inb.settings, proto) {
            (ProtocolSettings::VMess(s), "VMess") => {
                if is_new {
                    s.clients.push(VMessClient { id: values.get(0).cloned().unwrap_or_default(), security: values.get(1).cloned().unwrap_or_else(|| "auto".into()), email: Some(values.get(2).cloned().unwrap_or_default()).filter(|s| !s.is_empty()), level: None });
                } else if let Some(c) = s.clients.get_mut(user_idx) {
                    if let Some(v) = values.get(0) { c.id = v.clone(); }
                    if let Some(v) = values.get(1) { c.security = v.clone(); }
                    if let Some(v) = values.get(2) { c.email = if v.is_empty() { None } else { Some(v.clone()) }; }
                }
            }
            (ProtocolSettings::VLess(s), "VLESS") => {
                if is_new {
                    s.clients.push(VLessClient { id: values.get(0).cloned().unwrap_or_default(), flow: Some(values.get(1).cloned().unwrap_or_default()).filter(|s| !s.is_empty()), email: Some(values.get(2).cloned().unwrap_or_default()).filter(|s| !s.is_empty()), level: None });
                } else if let Some(c) = s.clients.get_mut(user_idx) {
                    if let Some(v) = values.get(0) { c.id = v.clone(); }
                    if let Some(v) = values.get(1) { c.flow = if v.is_empty() { None } else { Some(v.clone()) }; }
                    if let Some(v) = values.get(2) { c.email = if v.is_empty() { None } else { Some(v.clone()) }; }
                }
            }
            (ProtocolSettings::Trojan(s), "Trojan") => {
                if is_new {
                    s.clients.push(TrojanClient { password: values.get(0).cloned().unwrap_or_default(), email: Some(values.get(1).cloned().unwrap_or_default()).filter(|s| !s.is_empty()), level: None });
                } else if let Some(c) = s.clients.get_mut(user_idx) {
                    if let Some(v) = values.get(0) { c.password = v.clone(); }
                    if let Some(v) = values.get(1) { c.email = if v.is_empty() { None } else { Some(v.clone()) }; }
                }
            }
            (ProtocolSettings::Shadowsocks(s), "Shadowsocks") => {
                if let Some(v) = values.get(0) { s.password = v.clone(); }
                if let Some(v) = values.get(1) { s.method = v.clone(); }
                if let Some(v) = values.get(2) { s.email = if v.is_empty() { None } else { Some(v.clone()) }; }
            }
            (ProtocolSettings::Http(s), "HTTP") => {
                if is_new {
                    s.accounts.push(xray_model::HttpAccount { user: values.get(0).cloned().unwrap_or_default(), pass: values.get(1).cloned().unwrap_or_default() });
                } else if let Some(a) = s.accounts.get_mut(user_idx) {
                    if let Some(v) = values.get(0) { a.user = v.clone(); }
                    if let Some(v) = values.get(1) { a.pass = v.clone(); }
                }
            }
            (ProtocolSettings::Socks(s), "SOCKS") => {
                if let SocksAuth::Password { accounts } = &mut s.auth {
                    if is_new {
                        accounts.push(xray_model::SocksAccount { user: values.get(0).cloned().unwrap_or_default(), pass: values.get(1).cloned().unwrap_or_default() });
                    } else if let Some(a) = accounts.get_mut(user_idx) {
                        if let Some(v) = values.get(0) { a.user = v.clone(); }
                        if let Some(v) = values.get(1) { a.pass = v.clone(); }
                    }
                }
            }
            _ => {}
        }
        self.write_config();
        self.show_msg(if is_new { "User added" } else { "User updated" });
        self.current_screen = Screen::UserManager { inbound_idx, selected: user_idx, editing: None };
    }
    pub fn refresh_status(&mut self) { if let Ok(s) = self.systemd_service.get_status() { self.xray_status = s; } }
    pub fn save_and_quit(&self) {
        if let Err(e) = self.storage.save_state(&self.settings, &self.inbounds, &self.certificates) {
            eprintln!("Failed to save state: {}", e);
        }
    }

    fn detect_port_conflict(&self, inbound: &InboundConfig, skip_idx: Option<usize>) -> bool {
        self.inbounds.iter().enumerate().any(|(i, e)| {
            if skip_idx == Some(i) { return false; }
            e.port == inbound.port && e.listen == inbound.listen
        })
    }
}
pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let layout = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(10), Constraint::Length(1)])
        .split(area);
    render_top_bar(f, layout[0], app);
    render_tab_bar(f, layout[1], app);
    render_content(f, layout[2], app);
    render_help_bar(f, layout[3], app);
}

fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let sc = if app.xray_status.is_running { Color::Green } else { Color::Red };
    let si = if app.xray_status.is_running { "●" } else { "○" };
    let ver = app.xray_status.version.as_deref().unwrap_or("---");
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled(si, Style::default().fg(sc)), Span::raw(" xray "), Span::styled(ver, Style::default().fg(Color::Cyan))])).style(Style::default().bg(Color::Rgb(30, 30, 40))), area);
}

fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let tnames = ["Dashboard", "Inbounds", "Routing", "SSL", "Logs", "Settings"];
    let cur = match &app.current_screen { Screen::Dashboard=>0, Screen::InboundList{..}=>1, Screen::RoutingEditor{..}=>2, Screen::SslManagement{..}=>3, Screen::LogViewer(_)=>4, Screen::Settings=>5, _=>0 };
    let tabs: Vec<Span> = tnames.iter().enumerate().map(|(i,n)| if i==cur { Span::styled(format!(" {} ",n), Style::default().fg(Color::Black).bg(Color::Cyan)) } else { Span::styled(format!(" {} ",n), Style::default().fg(Color::Gray)) }).collect();
    f.render_widget(Tabs::new(tabs).block(Block::default().style(Style::default().bg(Color::Rgb(25, 25, 35)))), area);
}

fn render_content(f: &mut Frame, area: Rect, app: &App) {
    match &app.current_screen {
        Screen::Dashboard => screens::dashboard::render(f, area, app),
        Screen::InboundList { selected: s } => screens::inbound_list::render(f, area, app, *s),
        Screen::InboundWizard(ref wiz) => screens::wizard::render(f, area, app, wiz),
        Screen::UserManager { inbound_idx: i, selected: s, .. } => screens::user_manager::render(f, area, app, *i, *s),
        Screen::RoutingEditor { selected: s, editing: e } => screens::routing_editor::render(f, area, app, *s, e.as_ref()),
        Screen::SslManagement { selected: s, editing } => screens::ssl_manager::render(f, area, app, *s, editing),
        Screen::LogViewer(ref st) => screens::log_viewer::render(f, area, st, app.command_cursor),
        Screen::Settings => screens::settings_page::render(f, area, app),
        Screen::ConfirmDialog { message, .. } => screens::confirm::render(f, area, message),
        Screen::ShareExport { content } => screens::share_export::render(f, area, content),
    }
}

fn render_help_bar(f: &mut Frame, area: Rect, app: &App) {
    let help = match &app.current_screen {
        Screen::Dashboard => "q:Quit  Tab:Tab  ↑↓:Select  Enter:Execute",
        Screen::InboundList{..}=>"Esc:Back  ↑↓:Command  ←→:Entry  Enter:Execute  Tab:Tab",
        Screen::InboundWizard(_)=>"Tab:Field  Esc:Close/Back  Enter:Confirm  ←→:Steps",
        Screen::UserManager{..}=>"Esc:Back  ↑↓:Command  ←→:User  Enter:Execute",
        Screen::RoutingEditor{..}=>"Esc:Back  ↑↓:Command  ←→:Rule  Enter:Execute",
        Screen::SslManagement{..}=>"Esc:Back  ↑↓:Command  ←→:Cert  Enter:Execute",
        Screen::LogViewer(_)=>"Esc:Back  ↑↓:Scroll  ←→:Commands  Enter:Execute",
        Screen::Settings=>"Esc:Back  Tab:Field  s:LogLevel",
        Screen::ConfirmDialog{..}=>"y:Yes  n/Esc:No",
        Screen::ShareExport{..}=>"Esc:Back  c:Copy",
    };
    if let Some((msg,_))= &app.status_message {
        f.render_widget(Paragraph::new(Line::from(vec![Span::styled(" ⓘ ", Style::default().fg(Color::Green)), Span::raw(msg)])), area);
    } else { f.render_widget(Paragraph::new(help).style(Style::default().fg(Color::DarkGray)), area); }
}
