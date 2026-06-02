use ratatui::{Frame, layout::Rect, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action};

const COMMANDS: &[(&str, &str)] = &[
    ("Enable BBR",          "Enable TCP BBR congestion control (sysctl)"),
    ("Disable BBR",         "Revert to cubic + fq_codel"),
    ("Check BBR Status",    "Show current TCP congestion control algorithm"),
    ("Update Geo Files",    "Download latest geoip.dat & geosite.dat"),
    ("Sync NTP Time",       "Sync system clock via ntpdate / chronyd"),
    ("Show NTP Status",     "Check current NTP sync status"),
];

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<Action> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => { app.command_cursor = app.command_cursor.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.command_cursor + 1 < COMMANDS.len() { app.command_cursor += 1; }
            None
        }
        KeyCode::Enter => match app.command_cursor {
            0 => Some(Action::EnableBBR),
            1 => Some(Action::DisableBBR),
            2 => Some(Action::CheckBBR),
            3 => Some(Action::UpdateGeoFiles),
            4 => Some(Action::SyncNTP),
            5 => Some(Action::CheckNTP),
            _ => None,
        },
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<Line> = COMMANDS.iter().enumerate().map(|(i, (l, d))| {
        let hl = i == app.command_cursor;
        let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
        Line::from(vec![Span::styled(if hl { format!(" ▶ {}", l) } else { format!("   {}", l) }, s), Span::styled(format!("  — {}", d), Style::default().fg(Color::DarkGray))])
    }).collect();
    f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).title("System Tools — ↑↓ select  Enter execute")), area);
}
