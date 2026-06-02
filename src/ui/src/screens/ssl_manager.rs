use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table, Cell}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action, Screen, ConfirmedAction};

#[derive(Debug, Clone, Default)]
pub struct SslEditState {
    pub domain: String,
    pub show_prompt: bool,
}

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
            KeyCode::Esc => { *edit = None; return None; }
            KeyCode::Enter => {
                let domain = st.domain.clone();
                *edit = None;
                return Some(Action::IssueCert(domain));
            }
            KeyCode::Char(c) => { st.domain.push(c); return None; }
            KeyCode::Backspace => { st.domain.pop(); return None; }
            _ => return None,
        }
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => { let c = &mut app.command_cursor; if *c + 1 < COMMANDS.len() { *c += 1; } None }
        KeyCode::Left  => { if *selected > 0 { *selected -= 1; } None }
        KeyCode::Right => { if *selected + 1 < len { *selected += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => { *edit = Some(SslEditState { domain: String::new(), show_prompt: true }); None }
            1 if len > 0 => { let d = app.certificates[*selected].domain.clone(); match xray_services::AcmeService::renew_cert(&d) { Ok(_) => Some(Action::ShowMessage("Renewed".into())), Err(e) => Some(Action::ShowMessage(format!("Err:{}",e))) } }
            2 if len > 0 => Some(Action::PushScreen(Screen::ConfirmDialog { message: format!("Delete {} cert?", app.certificates[*selected].domain), on_confirm: ConfirmedAction::DeleteCert(*selected) })),
            3 if len > 0 => { let c = &mut app.certificates[*selected]; c.auto_renew = !c.auto_renew; Some(Action::ShowMessage(format!("Auto:{}", if c.auto_renew {"ON"} else {"OFF"}))) }
            _ => None,
        },
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App, selected: usize, edit: &Option<SslEditState>) {
    let edit_height = if edit.is_some() { 3 } else { 0 };
    let chunks = Layout::vertical([
        Constraint::Length(3 + app.certificates.len().max(1) as u16),
        Constraint::Min(if edit_height > 0 { 6 } else { 4 }),
    ]).split(area);

    let header = Row::new(["#","Domain","Expires","Status","Auto"]).style(Style::default().fg(Color::Cyan));
    let rows: Vec<Row> = if app.certificates.is_empty() { vec![Row::new(["","(none)","","",""])] } else { app.certificates.iter().enumerate().map(|(i,c)| { let hl=i==selected; let s=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; let days=(c.expires_at-chrono::Local::now().date_naive()).num_days(); let st=if days>30{"OK"}else if days>0{"soon"}else{"expired"}; Row::new(vec![Cell::from((i+1).to_string()).style(s),Cell::from(c.domain.clone()).style(s),Cell::from(c.expires_at.to_string()).style(s),Cell::from(st),Cell::from(if c.auto_renew{"Y"}else{"N"}).style(s)]) }).collect() };
    f.render_widget(Table::new(rows,[Constraint::Length(3),Constraint::Length(20),Constraint::Length(10),Constraint::Length(8),Constraint::Length(4)]).header(header).block(Block::default().borders(Borders::ALL).title("SSL")), chunks[0]);

    let title;
    let bottom_lines: Vec<Line> = if let Some(st) = edit {
        title = "Issue Certificate";
        let prompt = Line::from(Span::styled(format!("  Domain: {}_", st.domain), Style::default().fg(Color::Yellow)));
        let help = Line::from(Span::styled("  Type domain, Enter:issue, Esc:cancel", Style::default().fg(Color::DarkGray)));
        vec![prompt, help]
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
