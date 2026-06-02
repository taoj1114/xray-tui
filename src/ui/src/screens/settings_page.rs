use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action};

const COMMANDS: &[(&str, &str)] = &[
    ("Edit Binary Path", "Path to the Xray executable"),
    ("Edit Config Path", "Path to generated Xray config.json"),
    ("Cycle Log Level",  "warning → info → debug → error → warning"),
    ("Edit Server IP",   "Public IP for share link generation"),
    ("Edit State Dir",   "Directory for persistent state"),
    ("Save & Apply",     "Write settings to disk"),
];

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<Action> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => { let c = &mut app.command_cursor; if *c + 1 < COMMANDS.len() { *c += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => { let path = app.settings.xray_binary_path.clone(); app.show_msg(&format!("Binary: {} (edit in next version)", path)); None }
            1 => { let path = app.settings.config_path.clone(); app.show_msg(&format!("Config: {} (edit in next version)", path)); None }
            2 => {
                app.settings.log_level = match app.settings.log_level.as_str() {
                    "warning" => "info".into(), "info" => "debug".into(),
                    "debug" => "error".into(), "error" => "warning".into(), _ => "warning".into(),
                };
                Some(Action::UpdateSettings(app.settings.clone()))
            }
            3 => { let ip = app.settings.server_public_ip.clone(); app.show_msg(&format!("IP: {}", ip.as_deref().unwrap_or("not set"))); None }
            4 => { let dir = app.settings.state_dir.clone(); app.show_msg(&format!("Dir: {}", dir)); None }
            5 => Some(Action::UpdateSettings(app.settings.clone())),
            _ => None,
        },
        _ => None,
    }
}

fn app_fmt(s: &Option<String>) -> &str { s.as_deref().unwrap_or("(not set)") }

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(7),
        Constraint::Min(4),
    ]).split(area);

    let info = vec![
        Line::from(format!("  Xray binary:  {}", app.settings.xray_binary_path)),
        Line::from(format!("  Config path:  {}", app.settings.config_path)),
        Line::from(format!("  Log level:    {}", app.settings.log_level)),
        Line::from(format!("  Server IP:    {}", app_fmt(&app.settings.server_public_ip))),
        Line::from(format!("  State dir:    {}", app.settings.state_dir)),
    ];
    f.render_widget(Paragraph::new(info).block(Block::default().borders(Borders::ALL).title("Settings")), chunks[0]);

    let items: Vec<Line> = COMMANDS.iter().enumerate().map(|(i, (l, d))| {
        let hl = i == app.command_cursor;
        let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
        Line::from(vec![Span::styled(if hl { format!(" ▶ {}", l) } else { format!("   {}", l) }, s), Span::styled(format!("  — {}", d), Style::default().fg(Color::DarkGray))])
    }).collect();
    f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).title("Commands — ↑↓ select  Enter execute")), chunks[1]);
}
