use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table, Cell}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action, Screen, ConfirmedAction};

#[derive(Debug, Clone, Default)]
pub struct SslEditState {
    pub domain: String,
    pub method_idx: usize,
    pub webroot: String,
    pub field: usize, // 0=domain, 1=method, 2=webroot
}

const VALIDATION_METHODS: &[(&str, &str)] = &[
    ("webroot", "/var/www/html"),
    ("alpn",    "standalone"),
    ("dns",     "dns_cf"),
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
            KeyCode::Tab => { st.field = (st.field + 1) % 3; return None; }
            KeyCode::BackTab => { st.field = (st.field + 2) % 3; return None; }
            KeyCode::Enter => {
                let domain = st.domain.clone();
                let method = VALIDATION_METHODS[st.method_idx].0;
                let webroot = if method == "webroot" { Some(st.webroot.clone()) } else { None };
                let action = Action::IssueCertWithMethod { domain, method: method.to_string(), webroot };
                let _ = edit.take();
                return Some(action);
            }
            KeyCode::Up if st.field == 1 => { st.method_idx = (st.method_idx + 2) % 3; return None; }
            KeyCode::Down if st.field == 1 => { st.method_idx = (st.method_idx + 1) % 3; return None; }
            KeyCode::Char(c) => {
                match st.field {
                    0 => st.domain.push(c),
                    2 => st.webroot.push(c),
                    _ => {}
                }
                return None;
            }
            KeyCode::Backspace => {
                match st.field {
                    0 => { st.domain.pop(); }
                    2 => { st.webroot.pop(); }
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
                // check acme.sh installed
                let installed = std::process::Command::new("which").arg("acme.sh").output().map(|o| o.status.success()).unwrap_or(false);
                if !installed {
                    return Some(Action::ShowMessage("acme.sh not found. Install: curl https://get.acme.sh | sh".into()));
                }
                *edit = Some(SslEditState { domain: String::new(), method_idx: 0, webroot: "/var/www/html".into(), field: 0 });
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
    let edit_height = if edit.is_some() { 6 } else { 0 };
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
        let method_label = VALIDATION_METHODS[st.method_idx].0;
        let d_focus = st.field == 0; let m_focus = st.field == 1; let w_focus = st.field == 2;
        let w_visible = method_label == "webroot";
        let mut lines = vec![
            Line::from(vec![
                Span::raw(if d_focus { "▶" } else { " " }),
                Span::raw("Domain:  "),
                Span::styled(format!("{}_", st.domain), if d_focus { Style::default().fg(Color::Yellow) } else { Style::default() }),
            ]),
            Line::from(vec![
                Span::raw(if m_focus { "▶" } else { " " }),
                Span::raw("Method:  "),
                Span::styled(format!(" {} ", method_label), if m_focus { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() }),
                Span::styled(" (↑↓ to cycle)", Style::default().fg(Color::DarkGray)),
            ]),
        ];
        if w_visible {
            lines.push(Line::from(vec![
                Span::raw(if w_focus { "▶" } else { " " }),
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
