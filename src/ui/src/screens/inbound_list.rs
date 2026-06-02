use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table, Cell}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action, Screen, ConfirmedAction, UserData};
use crate::screens::InboundWizardState;

const COMMANDS: &[(&str, &str)] = &[
    ("New Config",        "Create a new inbound via template wizard"),
    ("Edit Config",       "Modify the selected inbound"),
    ("Clone Config",      "Duplicate the selected inbound as a new entry"),
    ("Delete Config",     "Remove the selected inbound (with confirmation)"),
    ("Manage Users",      "Add / remove / edit users for the selected inbound"),
    ("Copy Share Link",   "Copy subscription link for the selected inbound"),
    ("Export All Links",  "Export subscription links for ALL inbounds"),
    ("View Config JSON",  "Display generated Xray JSON for this inbound"),
    ("Toggle Enable",     "Enable / disable this inbound (adds/removes tag prefix)"),
];

pub fn handle_key(key: KeyEvent, app: &mut App, selected: &mut usize) -> Option<Action> {
    let len = app.inbounds.len();
    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => { let c = &mut app.command_cursor; if *c + 1 < COMMANDS.len() { *c += 1; } None }
        KeyCode::Left  => { if *selected > 0 { *selected -= 1; } None }
        KeyCode::Right => { if *selected + 1 < len { *selected += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => Some(Action::PushScreen(Screen::InboundWizard(InboundWizardState::new()))),
            1 if len > 0 => Some(Action::PushScreen(Screen::InboundWizard(InboundWizardState::edit(*selected, app.inbounds[*selected].clone())))),
            2 if len > 0 => {
                let mut cloned = app.inbounds[*selected].clone();
                let tag = cloned.tag.get_or_insert_with(|| "cloned".into()).clone();
                cloned.tag = Some(format!("{}-copy", tag));
                Some(Action::SaveInbound(cloned))
            }
            3 if len > 0 => Some(Action::PushScreen(Screen::ConfirmDialog { message: format!("Delete '{}'?", app.inbounds[*selected].tag.as_deref().unwrap_or("unnamed")), on_confirm: ConfirmedAction::DeleteInbound(*selected) })),
            4 if len > 0 => Some(Action::PushScreen(Screen::UserManager { inbound_idx: *selected, selected: 0, editing: None })),
            5 if len > 0 => { let ip = app.settings.server_public_ip.clone().unwrap_or("your-server-ip".into()); xray_services::SubscriptionService::generate_share_link(&app.inbounds[*selected], &ip, 0).map(|l| Action::PushScreen(Screen::ShareExport { content: l })) }
            6 => { let ip = app.settings.server_public_ip.clone().unwrap_or("your-server-ip".into()); Some(Action::ExportSubscription) }
            7 if len > 0 => { let json = serde_json::to_string_pretty(&app.inbounds[*selected]).unwrap_or_default(); Some(Action::PushScreen(Screen::ShareExport { content: json })) }
            8 if len > 0 => { let inb = &mut app.inbounds[*selected]; let tag = inb.tag.clone().unwrap_or_default(); if tag.starts_with("[DISABLED]") { inb.tag = Some(tag[10..].to_string()); } else { inb.tag = Some(format!("[DISABLED]{}", tag)); } Some(Action::ShowMessage("Toggled".into())) }
            _ => None,
        },
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App, selected: usize) {
    let chunks = Layout::vertical([Constraint::Length(2 + app.inbounds.len().max(1) as u16), Constraint::Min(1)]).split(area);
    let header = Row::new(["#", "Tag", "Protocol", "Port", "Transport", "Sec", "Users"]).style(Style::default().fg(Color::Cyan));
    let rows: Vec<Row> = app.inbounds.iter().enumerate().map(|(i, inb)| {
        let hl = i == selected; let s = if hl { Style::default().fg(Color::Black).bg(Color::White) } else { Style::default() };
        let disabled = inb.tag.as_deref().map(|t| t.starts_with("[DISABLED]")).unwrap_or(false);
        let tag_color = if disabled { Style::default().fg(Color::DarkGray) } else { s };
        Row::new(vec![
            Cell::from((i+1).to_string()).style(s), Cell::from(inb.tag.as_deref().unwrap_or("-")).style(tag_color),
            Cell::from(inb.protocol.to_string()).style(s), Cell::from(inb.port.to_string()).style(s),
            Cell::from(inb.stream_settings.network.to_string()).style(s), Cell::from(inb.stream_settings.security.to_string()).style(s),
            Cell::from(inb.user_count().to_string()).style(s),
        ])
    }).collect();
    let tbl = Table::new(if rows.is_empty() { vec![Row::new(["", "(empty)", "", "", "", "", ""])] } else { rows },
        [Constraint::Length(3), Constraint::Length(20), Constraint::Length(8), Constraint::Length(5), Constraint::Length(9), Constraint::Length(5), Constraint::Length(5)])
        .header(header).block(Block::default().borders(Borders::ALL).title("Inbounds"));
    f.render_widget(tbl, chunks[0]);

    let ctx = if let Some(inb) = app.inbounds.get(selected) {
        format!("{}:{}  {} + {}  sniff:{}  users:{}", inb.protocol, inb.port, inb.stream_settings.network, inb.stream_settings.security, if inb.sniffing.enabled {"on"} else {"off"}, inb.user_count())
    } else { "no selection".into() };
    let items: Vec<Line> = std::iter::once(Line::from(Span::styled(format!("  Selected: {}", ctx), Style::default().fg(Color::Yellow)))).chain(COMMANDS.iter().enumerate().map(|(i, (l, d))| {
        let hl = i == app.command_cursor; let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
        Line::from(vec![Span::styled(if hl { format!(" ▶ {}", l) } else { format!("   {}", l) }, s), Span::styled(format!("  — {}", d), Style::default().fg(Color::DarkGray))])
    })).collect();
    f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).title("Commands — ↑↓ select  ←→ switch entry  Enter execute")), chunks[1]);
}
