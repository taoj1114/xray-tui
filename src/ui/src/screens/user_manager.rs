use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table, Cell}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action, Screen, ConfirmedAction};

const COMMANDS: &[(&str, &str)] = &[
    ("Add User",    "Add a new user"),
    ("Delete User", "Remove the selected user"),
    ("Copy Link",   "Copy link for selected user"),
];

pub fn handle_key(key: KeyEvent, app: &mut App, selected: &mut usize, inbound_idx: usize) -> Option<Action> {
    let count = app.inbounds.get(inbound_idx).map(|i| i.user_count()).unwrap_or(0);
    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => { let c = &mut app.command_cursor; if *c + 1 < COMMANDS.len() { *c += 1; } None }
        KeyCode::Left  => { if *selected > 0 { *selected -= 1; } None }
        KeyCode::Right => { if *selected + 1 < count { *selected += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => Some(Action::ShowMessage("Add User: coming".into())),
            1 if count > 0 => Some(Action::PushScreen(Screen::ConfirmDialog { message: format!("Delete user #{}?", *selected+1), on_confirm: ConfirmedAction::DeleteUser { inbound_idx, user_idx: *selected } })),
            2 if count > 0 => { let ip = app.settings.server_public_ip.clone().unwrap_or("your-server-ip".into()); xray_services::SubscriptionService::generate_share_link(&app.inbounds[inbound_idx], &ip, *selected).map(|l| Action::PushScreen(Screen::ShareExport { content: l })) }
            _ => None,
        },
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App, inbound_idx: usize, selected: usize) {
    let inb = match app.inbounds.get(inbound_idx) { Some(i) => i, None => { f.render_widget(Paragraph::new("none").block(Block::default().borders(Borders::ALL)), area); return; } };
    let chunks = Layout::vertical([Constraint::Length(3+inb.user_count().max(1) as u16), Constraint::Min(1)]).split(area);
    let header = Row::new(["#", "ID/Password", "Email", "Option"]).style(Style::default().fg(Color::Cyan));
    let rows: Vec<Row> = match &inb.settings {
        xray_model::ProtocolSettings::VMess(s) => s.clients.iter().enumerate().map(|(i,c)| { let hl=i==selected; let st=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; Row::new(vec![Cell::from((i+1).to_string()).style(st),Cell::from(c.id.as_str()).style(st),Cell::from(c.email.as_deref().unwrap_or("")).style(st),Cell::from(c.security.as_str()).style(st)]) }).collect(),
        xray_model::ProtocolSettings::VLess(s) => s.clients.iter().enumerate().map(|(i,c)| { let hl=i==selected; let st=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; Row::new(vec![Cell::from((i+1).to_string()).style(st),Cell::from(c.id.as_str()).style(st),Cell::from(c.email.as_deref().unwrap_or("")).style(st),Cell::from(c.flow.as_deref().unwrap_or("")).style(st)]) }).collect(),
        _ => vec![Row::new(["","no users","",""])],
    };
    f.render_widget(Table::new(rows,[Constraint::Length(3),Constraint::Length(38),Constraint::Length(16),Constraint::Length(14)]).header(header).block(Block::default().borders(Borders::ALL).title("Users")), chunks[0]);
    let items: Vec<Line> = COMMANDS.iter().enumerate().map(|(i,(l,d))| { let hl=i==app.command_cursor; let s=if hl{Style::default().fg(Color::Black).bg(Color::Cyan)}else{Style::default()}; Line::from(vec![Span::styled(if hl{format!(" ▶ {}",l)}else{format!("   {}",l)},s),Span::styled(format!("  — {}",d),Style::default().fg(Color::DarkGray))]) }).collect();
    f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).title("Commands — ↑↓ select  ←→ switch user  Enter execute")), chunks[1]);
}
