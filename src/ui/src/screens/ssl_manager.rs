use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table, Cell}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action, Screen, ConfirmedAction};

#[derive(Debug, Clone, Default)]
pub struct SslEditState {
    pub domain: String,
    pub method_idx: usize,   // 0=webroot, 1=alpn, 2=dns_cf
    pub webroot: String,
    pub cf_email: String,
    pub cf_key: String,
    pub field: usize,        // 0=domain, 1=method, 2=webroot/cf_email, 3=cf_key
    pub n_fields: usize,
}

const DNS_METHODS: &[(&str, &str, usize)] = &[
    ("webroot", "/var/www/html", 2),  // 2 fields: domain, method, webroot
    ("alpn",    "standalone",    1),  // 1 field:  domain, method
    ("dns_cf",  "Cloudflare API",3),  // 3 fields: domain, method, cf_email, cf_key
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
                // cycle between field and method selector
                let method_name = DNS_METHODS[st.method_idx].0;
                if st.field == 1 {
                    // on method field, cycle method
                    st.method_idx = (st.method_idx + 1) % DNS_METHODS.len();
                    st.n_fields = DNS_METHODS[st.method_idx].2;
                    if st.field + 1 >= st.n_fields { st.field = 0; }
                } else {
                    st.field = (st.field + 1) % st.n_fields;
                }
                return None;
            }
            KeyCode::BackTab => {
                if st.field == 1 {
                    st.method_idx = (st.method_idx + DNS_METHODS.len() - 1) % DNS_METHODS.len();
                    st.n_fields = DNS_METHODS[st.method_idx].2;
                    if st.field >= st.n_fields { st.field = 0; }
                } else {
                    st.field = (st.field + st.n_fields - 1) % st.n_fields;
                }
                return None;
            }
            KeyCode::Enter => {
                let domain = st.domain.clone();
                let method = DNS_METHODS[st.method_idx].0.to_string();
                let webroot = if method == "webroot" { Some(st.webroot.clone()) } else { None };
                let cf_email = if method == "dns_cf" && !st.cf_email.is_empty() { Some(st.cf_email.clone()) } else { None };
                let cf_key = if method == "dns_cf" && !st.cf_key.is_empty() { Some(st.cf_key.clone()) } else { None };
                let action = Action::IssueCertWithMethod { domain, method, webroot, cf_email, cf_key };
                let _ = edit.take();
                return Some(action);
            }
            KeyCode::Up if st.field == 1 => { st.method_idx = (st.method_idx + DNS_METHODS.len() - 1) % DNS_METHODS.len(); st.n_fields = DNS_METHODS[st.method_idx].2; if st.field >= st.n_fields { st.field = 0; } return None; }
            KeyCode::Down if st.field == 1 => { st.method_idx = (st.method_idx + 1) % DNS_METHODS.len(); st.n_fields = DNS_METHODS[st.method_idx].2; if st.field >= st.n_fields { st.field = 0; } return None; }
            KeyCode::Char(c) => {
                match st.field {
                    0 => st.domain.push(c),
                    2 => {
                        if DNS_METHODS[st.method_idx].0 == "webroot" { st.webroot.push(c); }
                        else if DNS_METHODS[st.method_idx].0 == "dns_cf" { st.cf_email.push(c); }
                    }
                    3 => { st.cf_key.push(c); }
                    _ => {}
                }
                return None;
            }
            KeyCode::Backspace => {
                match st.field {
                    0 => { st.domain.pop(); }
                    2 => {
                        if DNS_METHODS[st.method_idx].0 == "webroot" { st.webroot.pop(); }
                        else if DNS_METHODS[st.method_idx].0 == "dns_cf" { st.cf_email.pop(); }
                    }
                    3 => { st.cf_key.pop(); }
                    _ => {}
                }
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
                let installed = std::process::Command::new("which").arg("acme.sh").output().map(|o| o.status.success()).unwrap_or(false);
                if !installed {
                    return Some(Action::ShowMessage("acme.sh not found. Install: curl https://get.acme.sh | sh".into()));
                }
                *edit = Some(SslEditState { domain: String::new(), method_idx: 0, webroot: "/var/www/html".into(), cf_email: String::new(), cf_key: String::new(), field: 0, n_fields: 2 });
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
    let edit_height = if edit.is_some() { 8 } else { 0 };
    let chunks = Layout::vertical([
        Constraint::Length(3 + app.certificates.len().max(1) as u16),
        Constraint::Min(if edit_height > 0 { 6 } else { 4 }),
    ]).split(area);

    let acme_tooltip = if edit.is_some() { "" } else {
        let installed = std::process::Command::new("which").arg("acme.sh").output().map(|o| o.status.success()).unwrap_or(false);
        if !installed { " │  ⚠ acme.sh not installed" } else { "" }
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
        let mut lines: Vec<Line> = vec![
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
        ];

        match method_name {
            "webroot" => {
                let w_focus = st.field == 2;
                lines.push(Line::from(vec![
                    Span::raw(if w_focus { "▶ " } else { "  " }),
                    Span::raw("WebRoot: "),
                    Span::styled(format!("{}_", st.webroot), if w_focus { Style::default().fg(Color::Yellow) } else { Style::default() }),
                ]));
            }
            "dns_cf" => {
                let e_focus = st.field == 2;
                let k_focus = st.field == 3;
                lines.push(Line::from(vec![
                    Span::raw(if e_focus { "▶ " } else { "  " }),
                    Span::raw("CF Email: "),
                    Span::styled(format!("{}_", st.cf_email), if e_focus { Style::default().fg(Color::Yellow) } else { Style::default() }),
                ]));
                lines.push(Line::from(vec![
                    Span::raw(if k_focus { "▶ " } else { "  " }),
                    Span::raw("CF Key:   "),
                    Span::styled(format!("{}_", if st.cf_key.is_empty() { "" } else { "●●●●●●" }), if k_focus { Style::default().fg(Color::Yellow) } else { Style::default() }),
                    Span::styled(" (Global API Key)" , Style::default().fg(Color::DarkGray)),
                ]));
            }
            _ => {}
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
