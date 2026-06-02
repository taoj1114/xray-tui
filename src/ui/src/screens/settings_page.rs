use ratatui::{Frame, layout::Rect, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action};

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::Navigate(crate::Screen::Dashboard)),
        KeyCode::Char('s') => {
            let mut s = app.settings.clone();
            s.log_level = match s.log_level.as_str() {
                "warning" => "info".into(),
                "info" => "debug".into(),
                "debug" => "error".into(),
                "error" => "warning".into(),
                _ => "warning".into(),
            };
            Some(Action::UpdateSettings(s))
        }
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let lines = vec![
        Line::from(format!("Xray binary:  {}", app.settings.xray_binary_path)),
        Line::from(format!("Config path:  {}", app.settings.config_path)),
        Line::from(format!("Log level:    {}", app.settings.log_level)),
        Line::from(format!("Server IP:    {}", app.settings.server_public_ip.as_deref().unwrap_or("not set"))),
        Line::from(format!("State dir:    {}", app.settings.state_dir)),
        Line::from(""),
        Line::from(Span::styled("  s: cycle log level  ", Style::default().fg(Color::Yellow).bg(Color::Rgb(40, 40, 50)))),
    ];

    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Settings")),
        area,
    );
}
