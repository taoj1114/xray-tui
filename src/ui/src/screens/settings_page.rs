use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action};

#[derive(Debug, Clone, Default)]
pub struct SettingsEditState {
    pub editing: bool,               // true = typing a value
    pub field: usize,                // command index (0..=5)
    pub value: String,               // current input
    pub cf_sub: Option<usize>,       // Some(0..2) = CF sub-menu, None = normal
}

const COMMANDS: &[(&str, &str)] = &[
    ("Edit Binary Path", "Path to the Xray executable"),
    ("Edit Config Path", "Path to generated Xray config.json"),
    ("Cycle Log Level",  "warning → info → debug → error → warning"),
    ("Edit Server IP",   "Public IP for share link generation"),
    ("Edit State Dir",   "Directory for persistent state"),
    ("Edit Cloudflare",  "Email, API Token, Zone ID"),
    ("Save & Apply",     "Write settings to disk"),
];

pub fn handle_key(key: KeyEvent, app: &mut App, edit: &mut Option<SettingsEditState>) -> Option<Action> {
    if let Some(ref mut ed) = edit {
        // ── CF sub-menu: browse Email/Token/Zone ──
        if let Some(ref mut sub) = ed.cf_sub {
            return match key.code {
                KeyCode::Esc => { *edit = None; None }
                KeyCode::Up | KeyCode::Char('k') => { *sub = sub.saturating_sub(1); None }
                KeyCode::Down | KeyCode::Char('j') => { if *sub + 1 < 3 { *sub += 1; } None }
                KeyCode::Enter => {
                    let cf_field = *sub + 5; // 5=email, 6=token, 7=zone
                    let val = match *sub {
                        0 => app.settings.cf_email.clone().unwrap_or_default(),
                        1 => app.settings.cf_token.clone().unwrap_or_default(),
                        _ => app.settings.cf_zone_id.clone().unwrap_or_default(),
                    };
                    *edit = Some(SettingsEditState { editing: true, field: cf_field, value: val, cf_sub: None });
                    None
                }
                _ => None,
            };
        }

        // ── Editing any field value ──
        if ed.editing {
            let is_cf = (5..=7).contains(&ed.field);
            return match key.code {
                KeyCode::Esc => {
                    if is_cf {
                        *edit = Some(SettingsEditState { editing: false, field: 5, value: String::new(), cf_sub: Some(ed.field - 5) });
                    } else {
                        *edit = None;
                    }
                    None
                }
                KeyCode::Enter => {
                    let val = std::mem::take(&mut ed.value);
                    let field = ed.field;
                    *edit = None;
                    match field {
                        0 => app.settings.xray_binary_path = val,
                        1 => app.settings.config_path = val,
                        3 => app.settings.server_public_ip = if val.is_empty() { None } else { Some(val) },
                        4 => app.settings.state_dir = val,
                        5 => app.settings.cf_email = if val.is_empty() { None } else { Some(val) },
                        6 => app.settings.cf_token = if val.is_empty() { None } else { Some(val) },
                        7 => app.settings.cf_zone_id = if val.is_empty() { None } else { Some(val) },
                        _ => {}
                    }
                    return Some(Action::UpdateSettings(app.settings.clone()));
                }
                KeyCode::Char(c) => { ed.value.push(c); None }
                KeyCode::Backspace => { ed.value.pop(); None }
                _ => None,
            };
        }

        return None;
    }

    // ── Main command list ──
    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { app.command_cursor = app.command_cursor.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => { if app.command_cursor + 1 < COMMANDS.len() { app.command_cursor += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => { *edit = Some(SettingsEditState { editing: true, field: 0, value: app.settings.xray_binary_path.clone(), cf_sub: None }); None }
            1 => { *edit = Some(SettingsEditState { editing: true, field: 1, value: app.settings.config_path.clone(), cf_sub: None }); None }
            2 => {
                app.settings.log_level = match app.settings.log_level.as_str() {
                    "warning" => "info".into(), "info" => "debug".into(),
                    "debug" => "error".into(), "error" => "warning".into(), _ => "warning".into(),
                };
                Some(Action::UpdateSettings(app.settings.clone()))
            }
            3 => { *edit = Some(SettingsEditState { editing: true, field: 3, value: app.settings.server_public_ip.clone().unwrap_or_default(), cf_sub: None }); None }
            4 => { *edit = Some(SettingsEditState { editing: true, field: 4, value: app.settings.state_dir.clone(), cf_sub: None }); None }
            5 => { *edit = Some(SettingsEditState { editing: false, field: 5, value: String::new(), cf_sub: Some(0) }); None }
            6 => Some(Action::UpdateSettings(app.settings.clone())),
            _ => None,
        },
        _ => None,
    }
}

fn app_fmt(s: &Option<String>, default: &str) -> String {
    s.as_ref().map(|s| s.to_string()).unwrap_or_else(|| default.to_string())
}

pub fn render(f: &mut Frame, area: Rect, app: &App, edit: &Option<SettingsEditState>) {
    let chunks = Layout::vertical([Constraint::Length(10), Constraint::Min(4)]).split(area);

    let cf_ok = app.settings.cf_email.is_some() && app.settings.cf_token.is_some();
    let cf_status = if cf_ok {
        Span::styled("● configured", Style::default().fg(Color::Green))
    } else {
        Span::styled("○ not set", Style::default().fg(Color::Red))
    };

    let info = vec![
        Line::from(format!("  Binary:     {}", app.settings.xray_binary_path)),
        Line::from(format!("  Config:     {}", app.settings.config_path)),
        Line::from(format!("  Log level:  {}", app.settings.log_level)),
        Line::from(format!("  Server IP:  {}", app_fmt(&app.settings.server_public_ip, "(not set)"))),
        Line::from(format!("  State dir:  {}", app.settings.state_dir)),
        Line::from(vec![Span::raw("  Cloudflare: "), cf_status]),
        Line::from(format!("    Email:    {}", app_fmt(&app.settings.cf_email, "(not set)"))),
        Line::from(format!("    Token:    {}", if app.settings.cf_token.is_some() { "●●●●●●●●" } else { "(not set)" })),
        Line::from(format!("    Zone ID:  {}", app_fmt(&app.settings.cf_zone_id, "(not set)"))),
    ];
    f.render_widget(Paragraph::new(info).block(Block::default().borders(Borders::ALL).title("Settings")), chunks[0]);

    // ── Bottom panel ──
    if let Some(ed) = edit {
        // CF sub-menu
        if let Some(sub) = ed.cf_sub {
            let cf_items: [(&str, String); 3] = [
                ("CF Email", app_fmt(&app.settings.cf_email, "(not set)")),
                ("CF Token", if app.settings.cf_token.is_some() { "●●●●●●●●".into() } else { "(not set)".into() }),
                ("CF Zone ID", app_fmt(&app.settings.cf_zone_id, "(not set)")),
            ];
            let lines: Vec<Line> = cf_items.iter().enumerate().map(|(i, (label, val))| {
                let hl = i == sub as usize;
                let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
                Line::from(vec![Span::styled(if hl { format!(" ▶ {}: {}", label, val) } else { format!("   {}: {}", label, val) }, s)])
            }).collect();
            f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Cloudflare — ↑↓ select  Enter edit  Esc back")), chunks[1]);
            return;
        }
        // Editing any field
        if ed.editing {
            let label = ["Xray binary path", "Config path", "", "Server IP", "State dir",
                         "CF Email", "CF Token", "CF Zone ID"][ed.field];
            let mask = ed.field == 6;
            let display = if mask && !ed.value.is_empty() { "●●●●●●●●".to_string() } else { ed.value.clone() };
            f.render_widget(Paragraph::new(vec![
                Line::from(Span::styled(format!("  Editing: {} — type, Enter:save, Esc:cancel", label), Style::default().fg(Color::Yellow))),
                Line::from(Span::styled(format!("  ▶ {}", display), Style::default().fg(Color::Cyan))),
            ]).block(Block::default().borders(Borders::ALL).title("Edit")), chunks[1]);
            return;
        }
    }

    let bottom_lines: Vec<Line> = COMMANDS.iter().enumerate().map(|(i, (l, d))| {
        let hl = i == app.command_cursor;
        let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
        Line::from(vec![Span::styled(if hl { format!(" ▶ {}", l) } else { format!("   {}", l) }, s), Span::styled(format!("  — {}", d), Style::default().fg(Color::DarkGray))])
    }).collect();
    f.render_widget(Paragraph::new(bottom_lines).block(Block::default().borders(Borders::ALL).title("Commands — ↑↓ select  Enter execute")), chunks[1]);
}
