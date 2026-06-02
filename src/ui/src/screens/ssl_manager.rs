use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table, Cell}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action, Screen, ConfirmedAction};

#[derive(Debug, Clone, Default)]
pub struct SslEditState {
    pub domain: String,
    pub method_idx: usize,   // 0=webroot, 1=alpn, 2=dns_cf
    pub webroot: String,
    pub field: usize,        // 0=domain, 1=method, 2=webroot (only for webroot)
}

const DNS_METHODS: &[(&str, &str)] = &[
    ("webroot", "Local webroot"),
    ("alpn",    "Standalone ALPN"),
    ("dns_cf",  "Cloudflare DNS (uses Settings)"),
];

const COMMANDS: &[(&str, &str)] = &[
    ("Issue Cert",   "Issue a new Let's Encrypt certificate via acme.sh"),
    ("Renew Cert",   "Renew the selected certificate"),
    ("Delete Cert",  "Remove the selected certificate record"),
    ("Auto Renew",   "Toggle auto-renew for selected domain"),
];

pub fn handle_key(key: KeyEvent, app: &mut App, selected: &mut usize, edit: &mut Option<SslEditState>) -> Option<Action> {
    let len = app.certificates.len();

    if let Some(ref mut st) = edit {
        match key.code {
            KeyCode::Esc => { let _ = edit.take(); return None; }
            KeyCode::Tab => {
                let method_name = DNS_METHODS[st.method_idx].0;
                let n_fields = if method_name == "webroot" { 3 } else { 2 };
                if st.field == 1 {
                    st.method_idx = (st.method_idx + 1) % DNS_METHODS.len();
                    if st.field >= n_fields { st.field = 0; }
                } else {
                    st.field = (st.field + 1) % n_fields;
                }
                return None;
            }
            KeyCode::BackTab => {
                st.method_idx = (st.method_idx + DNS_METHODS.len() - 1) % DNS_METHODS.len();
                if st.field >= if DNS_METHODS[st.method_idx].0 == "webroot" { 3 } else { 2 } { st.field = 0; }
                return None;
            }
            KeyCode::Enter => {
                let method = DNS_METHODS[st.method_idx].0.to_string();
                // Check CF credentials pre-configured
                if method == "dns_cf" {
                    if app.settings.cf_email.is_none() || app.settings.cf_token.is_none() {
                        let _ = edit.take();
                        return Some(Action::ShowMessage("Cloudflare credentials not set. Go to Settings → Edit CF Email / Token first.".into()));
                    }
                }
                let domain = st.domain.clone();
                let webroot = if method == "webroot" { Some(st.webroot.clone()) } else { None };
                let cf_email = if method == "dns_cf" { app.settings.cf_email.clone() } else { None };
                let cf_token = if method == "dns_cf" { app.settings.cf_token.clone() } else { None };
                let action = Action::IssueCertWithMethod { domain, method, webroot, cf_email, cf_key: cf_token };
                let _ = edit.take();
                return Some(action);
            }
            KeyCode::Up if st.field == 1 => { st.method_idx = (st.method_idx + DNS_METHODS.len() - 1) % DNS_METHODS.len(); return None; }
            KeyCode::Down if st.field == 1 => { st.method_idx = (st.method_idx + 1) % DNS_METHODS.len(); return None; }
            KeyCode::Char(c) => {
                if st.field == 0 { st.domain.push(c); }
                else if st.field == 2 && DNS_METHODS[st.method_idx].0 == "webroot" { st.webroot.push(c); }
                return None;
            }
            KeyCode::Backspace => {
                if st.field == 0 { st.domain.pop(); }
                else if st.field == 2 && DNS_METHODS[st.method_idx].0 == "webroot" { st.webroot.pop(); }
                return None;
            }
            _ => return None,
        }
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => { let c = &mut app.command_cursor; if *c + 1 < COMMANDS.len() { *c += 1; } None }
        KeyCode::Left  => { if *selected > 0 { *selected -= 1; } None }
        KeyCode::Right => { if *selected + 1 < len { *selected += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => {
                // Auto-detect & install acme.sh if missing
                if !xray_services::AcmeService::is_installed() {
                    match xray_services::AcmeService::install_acme(app.settings.cf_email.as_deref()) {
                        Ok(_) => { /* success, continue */ }
                        Err(e) => return Some(Action::ShowMessage(format!("acme.sh install failed: {}", e))),
                    }
                }
                let cf_ok = app.settings.cf_email.is_some() && app.settings.cf_token.is_some();
                *edit = Some(SslEditState { domain: String::new(), method_idx: if cf_ok { 2 } else { 0 }, webroot: "/var/www/html".into(), field: 0 });
                None
            }
            1 if len > 0 => { let d = app.certificates[*selected].domain.clone(); match xray_services::AcmeService::renew_cert(&d) { Ok(_) => Some(Action::ShowMessage("Renewed".into())), Err(e) => Some(Action::ShowMessage(format!("Err:{}",e))) } }
            2 if len > 0 => Some(Action::PushScreen(Screen::ConfirmDialog { message: format!("Delete {} cert?", app.certificates[*selected].domain), on_confirm: ConfirmedAction::DeleteCert(*selected) })),
            3 if len > 0 => { let c = &mut app.certificates[*selected]; c.auto_renew = !c.auto_renew; Some(Action::ShowMessage(format!("Auto:{}", if c.auto_renew {"ON"} else {"OFF"}))) }
            _ => None,
        },
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App, selected: usize, edit: &Option<SslEditState>) {
    let edit_height = if edit.is_some() { 7 } else { 0 };
    let chunks = Layout::vertical([
        Constraint::Length(3 + app.certificates.len().max(1) as u16),
        Constraint::Min(if edit_height > 0 { 6 } else { 4 }),
    ]).split(area);

    let acme_tooltip = if edit.is_some() { "" } else {
        if !xray_services::AcmeService::is_installed() { " │  ⚠ acme.sh not installed" } else { "" }
    };

    let header = Row::new(["#","Domain","Expires","Status","Auto"]).style(Style::default().fg(Color::Cyan));
    let rows: Vec<Row> = if app.certificates.is_empty() { vec![Row::new(["","(none)","","",""])] } else { app.certificates.iter().enumerate().map(|(i,c)| { let hl=i==selected; let s=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; let days=(c.expires_at-chrono::Local::now().date_naive()).num_days(); let st=if days>30{"OK"}else if days>0{"soon"}else{"expired"}; Row::new(vec![Cell::from((i+1).to_string()).style(s),Cell::from(c.domain.clone()).style(s),Cell::from(c.expires_at.to_string()).style(s),Cell::from(st),Cell::from(if c.auto_renew{"Y"}else{"N"}).style(s)]) }).collect() };
    f.render_widget(Table::new(rows,[Constraint::Length(3),Constraint::Length(20),Constraint::Length(10),Constraint::Length(8),Constraint::Length(4)]).header(header).block(Block::default().borders(Borders::ALL).title(format!("SSL{}", acme_tooltip))), chunks[0]);

    let title;
    let bottom_lines: Vec<Line> = if let Some(st) = edit {
        title = "Issue Certificate — Tab:switch field  Enter:issue  Esc:cancel";
        let method_name = DNS_METHODS[st.method_idx].0;
        let d_focus = st.field == 0;
        let m_focus = st.field == 1;

        let cf_configured = app.settings.cf_email.is_some() && app.settings.cf_token.is_some();
        let cf_hint = if method_name == "dns_cf" && !cf_configured {
            Line::from(Span::styled("  ⚠ CF credentials not set — go to Settings first", Style::default().fg(Color::Red)))
        } else if method_name == "dns_cf" {
            Line::from(Span::styled(format!("  CF configured: {} / {}", app.settings.cf_email.as_deref().unwrap_or("?"), if app.settings.cf_token.is_some() { "●●●●" } else { "?" }), Style::default().fg(Color::Green)))
        } else {
            Line::from("")
        };

        let mut lines = vec![
            Line::from(vec![
                Span::raw(if d_focus { "▶ " } else { "  " }),
                Span::raw("Domain: "),
                Span::styled(format!("{}_", st.domain), if d_focus { Style::default().fg(Color::Yellow) } else { Style::default() }),
            ]),
            Line::from(vec![
                Span::raw(if m_focus { "▶ " } else { "  " }),
                Span::raw("Method: "),
                Span::styled(format!(" {} ", method_name), if m_focus { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() }),
                Span::styled(" (↑↓ cycle)", Style::default().fg(Color::DarkGray)),
            ]),
            cf_hint,
        ];

        if method_name == "webroot" {
            let w_focus = st.field == 2;
            lines.push(Line::from(vec![
                Span::raw(if w_focus { "▶ " } else { "  " }),
                Span::raw("WebRoot: "),
                Span::styled(format!("{}_", st.webroot), if w_focus { Style::default().fg(Color::Yellow) } else { Style::default() }),
            ]));
        }
        lines
    } else {
        title = "Commands — ↑↓ select  ←→ switch cert  Enter execute";
        COMMANDS.iter().enumerate().map(|(i,(l,d))| {
            let hl = i == app.command_cursor;
            let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
            Line::from(vec![Span::styled(if hl { format!(" ▶ {}", l) } else { format!("   {}", l) }, s), Span::styled(format!("  — {}", d), Style::default().fg(Color::DarkGray))])
        }).collect()
    };
    f.render_widget(Paragraph::new(bottom_lines).block(Block::default().borders(Borders::ALL).title(title)), chunks[1]);
}
