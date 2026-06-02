use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table, Cell}};
use crossterm::event::{KeyEvent, KeyCode};
use xray_model::{InboundConfig, ProtocolSettings, SocksAuth};
use crate::{App, Action, Screen, ConfirmedAction};

#[derive(Debug, Clone)]
pub struct UserEditMode {
    pub is_new: bool,
    pub user_idx: usize,
    pub field: usize,
    pub values: Vec<String>,
    pub field_labels: Vec<String>,
}

impl UserEditMode {
    fn for_protocol(protocol: &str, existing: &[&str]) -> Self {
        let (labels, defaults): (Vec<String>, Vec<String>) = match protocol {
            "VMess" => (vec!["UUID".into(), "Security".into(), "Email".into()],
                        vec![existing.get(0).unwrap_or(&"").to_string(), existing.get(1).unwrap_or(&"auto").to_string(), existing.get(2).unwrap_or(&"").to_string()]),
            "VLESS" => (vec!["UUID".into(), "Flow".into(), "Email".into()],
                        vec![existing.get(0).unwrap_or(&"").to_string(), existing.get(1).unwrap_or(&"").to_string(), existing.get(2).unwrap_or(&"").to_string()]),
            "Trojan" => (vec!["Password".into(), "Email".into()],
                         vec![existing.get(0).unwrap_or(&"").to_string(), existing.get(1).unwrap_or(&"").to_string()]),
            "Shadowsocks" => (vec!["Password".into(), "Method".into(), "Email".into()],
                              vec![existing.get(0).unwrap_or(&"").to_string(), existing.get(1).unwrap_or(&"aes-256-gcm").to_string(), existing.get(2).unwrap_or(&"").to_string()]),
            "HTTP" => (vec!["Username".into(), "Password".into()],
                       vec![existing.get(0).unwrap_or(&"").to_string(), existing.get(1).unwrap_or(&"").to_string()]),
            "SOCKS" => (vec!["Username".into(), "Password".into()],
                        vec![existing.get(0).unwrap_or(&"").to_string(), existing.get(1).unwrap_or(&"").to_string()]),
            _ => (vec!["UUID".into()], vec![uuid::Uuid::new_v4().to_string()]),
        };
        Self { is_new: true, user_idx: 0, field: 0, values: defaults, field_labels: labels }
    }

    fn n_fields(&self) -> usize { self.values.len() }
}

const COMMANDS: &[(&str, &str)] = &[
    ("Add User",    "Add a new user to this inbound"),
    ("Edit User",   "Edit the selected user's credentials"),
    ("Delete User", "Remove the selected user"),
    ("Copy Link",   "Copy subscription link for the selected user"),
    ("Export All",  "Export links for all users in this inbound"),
];

pub fn handle_key(key: KeyEvent, app: &mut App, selected: &mut usize, inbound_idx: usize, editing: &mut Option<UserEditMode>) -> Option<Action> {
    let inb = match app.inbounds.get(inbound_idx) { Some(i) => i, None => return None };
    let count = inb.user_count();
    let proto = inb.protocol.to_string();

    // If in edit mode, handle inline editing keys
    if let Some(ref mut edit) = editing {
        match key.code {
            KeyCode::Esc => { *editing = None; return None; }
            KeyCode::Tab => { edit.field = (edit.field + 1) % edit.n_fields(); return None; }
            KeyCode::BackTab => { edit.field = (edit.field + edit.n_fields() - 1) % edit.n_fields(); return None; }
            KeyCode::Enter if edit.field == edit.n_fields() - 1 => {
                let mut values = edit.values.clone();
                let labels = edit.field_labels.clone();
                let is_new = edit.is_new;
                let user_idx = edit.user_idx;
                *editing = None;

                if is_new && labels.first().map(|l| l.as_str()) == Some("UUID") && values[0].is_empty() {
                    values[0] = uuid::Uuid::new_v4().to_string();
                }
                return Some(Action::SaveUser { inbound_idx, user_idx, proto, labels, values, is_new });
            }
            KeyCode::Char(c) => {
                edit.values[edit.field].push(c);
                return None;
            }
            KeyCode::Backspace => { edit.values[edit.field].pop(); return None; }
            _ => return None,
        }
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => { let c = &mut app.command_cursor; if *c + 1 < COMMANDS.len() { *c += 1; } None }
        KeyCode::Left  => { if *selected > 0 { *selected -= 1; } None }
        KeyCode::Right => { if *selected + 1 < count { *selected += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => {
                *editing = Some(UserEditMode::for_protocol(&proto, &[]));
                None
            }
            1 if count > 0 => {
                let existing = get_user_values(inb, *selected);
                let refs: Vec<&str> = existing.iter().map(|s| s.as_str()).collect();
                let mut edit = UserEditMode::for_protocol(&proto, &refs);
                edit.is_new = false;
                edit.user_idx = *selected;
                *editing = Some(edit);
                None
            }
            2 if count > 0 => Some(Action::PushScreen(Screen::ConfirmDialog {
                message: format!("Delete user #{}?", *selected+1),
                on_confirm: ConfirmedAction::DeleteUser { inbound_idx, user_idx: *selected }
            })),
            3 if count > 0 => {
                let ip = app.settings.server_public_ip.clone().unwrap_or("your-server-ip".into());
                xray_services::SubscriptionService::generate_share_link(&app.inbounds[inbound_idx], &ip, *selected)
                    .map(|l| Action::PushScreen(Screen::ShareExport { content: l }))
            }
            4 => {
                let ip = app.settings.server_public_ip.clone().unwrap_or("your-server-ip".into());
                let mut links = Vec::new();
                for i in 0..count {
                    if let Some(l) = xray_services::SubscriptionService::generate_share_link(&app.inbounds[inbound_idx], &ip, i) {
                        links.push(l);
                    }
                }
                Some(Action::PushScreen(Screen::ShareExport { content: links.join("\n") }))
            }
            _ => None,
        },
        _ => None,
    }
}

fn get_user_values(inb: &InboundConfig, idx: usize) -> Vec<String> {
    match &inb.settings {
        ProtocolSettings::VMess(s) => s.clients.get(idx).map(|c| vec![c.id.clone(), c.security.clone(), c.email.clone().unwrap_or_default()]).unwrap_or_default(),
        ProtocolSettings::VLess(s) => s.clients.get(idx).map(|c| vec![c.id.clone(), c.flow.clone().unwrap_or_default(), c.email.clone().unwrap_or_default()]).unwrap_or_default(),
        ProtocolSettings::Trojan(s) => s.clients.get(idx).map(|c| vec![c.password.clone(), c.email.clone().unwrap_or_default()]).unwrap_or_default(),
        ProtocolSettings::Shadowsocks(s) => vec![s.password.clone(), s.method.clone(), s.email.clone().unwrap_or_default()],
        ProtocolSettings::Http(s) => s.accounts.get(idx).map(|a| vec![a.user.clone(), a.pass.clone()]).unwrap_or_default(),
        ProtocolSettings::Socks(s) => match &s.auth {
            SocksAuth::Password { accounts } => accounts.get(idx).map(|a| vec![a.user.clone(), a.pass.clone()]).unwrap_or_default(),
            SocksAuth::NoAuth {} => vec!["(no auth)".into()],
        },
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App, inbound_idx: usize, selected: usize) {
    let inb = match app.inbounds.get(inbound_idx) { Some(i) => i, None => { f.render_widget(Paragraph::new("none").block(Block::default().borders(Borders::ALL)), area); return; } };
    let editing = match &app.current_screen { Screen::UserManager { editing, .. } => editing.as_ref(), _ => None };

    let edit_height = if editing.is_some() { 4 } else { 0 };
    let chunks = Layout::vertical([
        Constraint::Length(3 + inb.user_count().max(1) as u16),
        Constraint::Min(if edit_height > 0 { 8 } else { 4 }),
    ]).split(area);

    let header = Row::new(["#", "Credential", "Email", "Option 1"]).style(Style::default().fg(Color::Cyan));
    let rows: Vec<Row> = match &inb.settings {
        ProtocolSettings::VMess(s) => s.clients.iter().enumerate().map(|(i,c)| { let hl=i==selected; let st=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; Row::new(vec![Cell::from((i+1).to_string()).style(st),Cell::from(c.id.as_str()).style(st),Cell::from(c.email.as_deref().unwrap_or("")).style(st),Cell::from(c.security.as_str()).style(st)]) }).collect(),
        ProtocolSettings::VLess(s) => s.clients.iter().enumerate().map(|(i,c)| { let hl=i==selected; let st=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; Row::new(vec![Cell::from((i+1).to_string()).style(st),Cell::from(c.id.as_str()).style(st),Cell::from(c.email.as_deref().unwrap_or("")).style(st),Cell::from(c.flow.as_deref().unwrap_or("")).style(st)]) }).collect(),
        ProtocolSettings::Trojan(s) => s.clients.iter().enumerate().map(|(i,c)| { let hl=i==selected; let st=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; Row::new(vec![Cell::from((i+1).to_string()).style(st),Cell::from(c.password.as_str()).style(st),Cell::from(c.email.as_deref().unwrap_or("")).style(st),Cell::from("")]) }).collect(),
        ProtocolSettings::Shadowsocks(s) => { let hl=0==selected; let st=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; vec![Row::new(vec![Cell::from("1").style(st),Cell::from(s.password.as_str()).style(st),Cell::from(s.email.as_deref().unwrap_or("")).style(st),Cell::from(s.method.as_str()).style(st)])] },
        ProtocolSettings::Http(s) => s.accounts.iter().enumerate().map(|(i,a)| { let hl=i==selected; let st=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; Row::new(vec![Cell::from((i+1).to_string()).style(st),Cell::from(a.user.as_str()).style(st),Cell::from(a.pass.as_str()).style(st),Cell::from("")]) }).collect(),
        ProtocolSettings::Socks(s) => match &s.auth { SocksAuth::Password{ accounts } => accounts.iter().enumerate().map(|(i,a)| { let hl=i==selected; let st=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; Row::new(vec![Cell::from((i+1).to_string()).style(st),Cell::from(a.user.as_str()).style(st),Cell::from(a.pass.as_str()).style(st),Cell::from("")]) }).collect(), SocksAuth::NoAuth{} => vec![Row::new(vec!["","(no auth required)","",""])] },
        _ => vec![Row::new(["","no users","",""])],
    };
    f.render_widget(Table::new(rows,[Constraint::Length(3),Constraint::Length(32),Constraint::Length(16),Constraint::Length(18)]).header(header).block(Block::default().borders(Borders::ALL).title(format!("Users — {}", inb.tag.as_deref().unwrap_or("-")))), chunks[0]);

    // Command menu or edit form
    let bottom_lines: Vec<Line> = if let Some(edit) = editing {
        let header_line = Line::from(Span::styled(if edit.is_new { "  Adding new user — type values, Tab:next field, Enter on last:save, Esc:cancel" } else { "  Editing user — Tab:next field, Enter on last:save, Esc:cancel" }, Style::default().fg(Color::Yellow)));
        let field_lines: Vec<Line> = edit.field_labels.iter().enumerate().map(|(i, label)| {
            let val = edit.values.get(i).cloned().unwrap_or_default();
            let is_focused = i == edit.field;
            let (prefix, suffix) = if is_focused { (">>> ", "_") } else { ("    ", "") };
            Line::from(vec![
                Span::raw(format!("{}{}: ", prefix, label)),
                Span::styled(format!("{}{}", val, suffix), if is_focused { Style::default().fg(Color::Cyan).bg(Color::DarkGray) } else { Style::default() }),
            ])
        }).collect();
        std::iter::once(header_line).chain(field_lines).collect()
    } else {
        COMMANDS.iter().enumerate().map(|(i,(l,d))| {
            let hl = i == app.command_cursor;
            let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
            Line::from(vec![Span::styled(if hl { format!(" ▶ {}", l) } else { format!("   {}", l) }, s), Span::styled(format!("  — {}", d), Style::default().fg(Color::DarkGray))])
        }).collect()
    };

    let title = if editing.is_some() { "User Editor" } else { "Commands — ↑↓ select  ←→ switch user  Enter execute" };
    f.render_widget(Paragraph::new(bottom_lines).block(Block::default().borders(Borders::ALL).title(title)), chunks[1]);
}
