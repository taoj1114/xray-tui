use ratatui::{Frame, layout::{Layout, Constraint, Direction, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action};

const ITEMS: &[(&str, &str)] = &[
    ("Install Xray",     "Download & install Xray via official script"),
    ("Restart Xray",     "Reload config & restart service"),
    ("Stop Xray",        "Stop the running Xray service"),
];

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<Action> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => { let c = &mut app.command_cursor; if *c + 1 < ITEMS.len() { *c += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => Some(Action::InstallXray),
            1 => Some(Action::RestartXray),
            2 => Some(Action::StopXray),
            _ => None,
        },
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(1)]).split(area);

    let status = app.xray_status.clone();
    let mut lines = vec![
        Line::from(vec![
            Span::raw("Status: "),
            if status.is_running {
                Span::styled("● Running", Style::default().fg(Color::Green))
            } else if status.is_installed {
                Span::styled("○ Stopped", Style::default().fg(Color::Red))
            } else {
                Span::styled("✖ Not installed", Style::default().fg(Color::Red))
            },
        ]),
        Line::from(format!("PID:     {}", status.pid.map(|p| p.to_string()).unwrap_or_else(|| "---".into()))),
        Line::from(format!("Version: {}", status.version.as_deref().unwrap_or("(not found)"))),
        Line::from(format!("Inbounds: {}  │  Rules: {}  │  Certs: {}", app.inbounds.len(), app.routing_rules.len(), app.certificates.len())),
    ];
    if !status.is_installed {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  ⚡  Select \"Install Xray\" below  ↓", Style::default().fg(Color::Yellow))));
    }
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("System").style(Style::default().fg(Color::Cyan))),
        chunks[0],
    );

    let items: Vec<Line> = ITEMS.iter().enumerate().map(|(i, (label, desc))| {
        let hl = i == app.command_cursor;
        let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
        Line::from(vec![
            Span::styled(if hl { format!(" ▶ {}", label) } else { format!("   {}", label) }, s),
            Span::styled(format!("  — {}", desc), Style::default().fg(Color::DarkGray)),
        ])
    }).collect();
    f.render_widget(
        Paragraph::new(items).block(Block::default().borders(Borders::ALL).title("Commands — ↑↓ select  Enter execute")),
        chunks[1],
    );
}
