use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action, Screen, PickerAction};
use crate::screens::InboundWizardState;

const COMMANDS: &[(&str, &str)] = &[
    ("New Config",        "Create a new inbound configuration"),
    ("Edit Config",       "Modify an existing configuration"),
    ("Delete Config",     "Remove a configuration file"),
    ("Toggle Status",     "Enable or disable a configuration"),
    ("Manage Users",      "Add / remove / edit users for an inbound"),
    ("Copy Share Link",   "Copy subscription link for the first user"),
    ("Export All Links",  "Export subscription links for ALL inbounds"),
];

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<Action> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => { app.command_cursor = app.command_cursor.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.command_cursor + 1 < COMMANDS.len() { app.command_cursor += 1; }
            None
        }
        KeyCode::Enter => execute_command(app.command_cursor, app),
        _ => None,
    }
}

fn execute_command(cmd: usize, app: &mut App) -> Option<Action> {
    let len = app.inbounds.len();
    match cmd {
        0 => Some(Action::PushScreen(Screen::InboundWizard(InboundWizardState::new()))),
        1 if len > 0 => Some(Action::PushScreen(Screen::ConfigPicker { selected: 0, action: PickerAction::EditConfig })),
        2 if len > 0 => Some(Action::PushScreen(Screen::ConfigPicker { selected: 0, action: PickerAction::DeleteConfig })),
        3 if len > 0 => Some(Action::PushScreen(Screen::ConfigPicker { selected: 0, action: PickerAction::ToggleConfig })),
        4 if len > 0 => Some(Action::PushScreen(Screen::ConfigPicker { selected: 0, action: PickerAction::ManageUsers })),
        5 if len > 0 => Some(Action::PushScreen(Screen::ConfigPicker { selected: 0, action: PickerAction::CopyLink })),
        6 => Some(Action::ExportSubscription),
        _ => None,
    }
}

// ─ 渲染 ─

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<Line> = COMMANDS.iter().enumerate().map(|(i, (l, d))| {
        let hl = i == app.command_cursor;
        let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
        Line::from(vec![Span::styled(if hl { format!(" ▶ {}", l) } else { format!("   {}", l) }, s), Span::styled(format!("  — {}", d), Style::default().fg(Color::DarkGray))])
    }).collect();
    let summary = format!("{} config(s) loaded", app.inbounds.len());
    let header = Paragraph::new(Line::from(vec![Span::styled(summary, Style::default().fg(Color::Cyan))]));
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);
    f.render_widget(header, chunks[0]);
    f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).title("Commands")), chunks[1]);
}

pub fn render_picker(f: &mut Frame, area: Rect, app: &App, selected: usize, action: &PickerAction) {
    let title = match action {
        PickerAction::EditConfig => "Select a config to edit",
        PickerAction::DeleteConfig => "Select a config to delete",
        PickerAction::ToggleConfig => "Select a config to toggle",
        PickerAction::ManageUsers => "Select a config to manage users",
        PickerAction::CopyLink => "Select a config to copy link",
    };
    let items: Vec<Line> = app.inbounds.iter().enumerate().map(|(i, entry)| {
        let hl = i == selected;
        let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
        let status = if entry.enabled { "●" } else { "○" };
        let desc = format!("  {}  {}:{}  [{}]  {}", if hl { "▶" } else { " " }, entry.config.protocol, entry.config.port, status, entry.filename);
        Line::from(Span::styled(desc, s))
    }).collect();
    if items.is_empty() {
        f.render_widget(Paragraph::new("(no configuration files found)").style(Style::default().fg(Color::DarkGray)), area);
    } else {
        f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).title(title)), area);
    }
}
