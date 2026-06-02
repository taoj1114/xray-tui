use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action};

#[derive(Debug, Clone)]
pub struct LogViewerState {
    pub lines: Vec<String>,
    pub auto_scroll: bool,
    pub scroll_offset: u16,
    pub level_filter: String,
    pub keyword: String,
    pub editing_keyword: bool,
}
impl Default for LogViewerState {
    fn default() -> Self {
        Self { lines: vec!["No logs. Select 'Refresh'. ".into()], auto_scroll: true, scroll_offset: 0,
            level_filter: "info".into(), keyword: String::new(), editing_keyword: false }
    }
}

const COMMANDS: &[(&str, &str)] = &[
    ("Refresh",        "Reload latest logs from journalctl"),
    ("Auto Scroll",    "Toggle auto-scroll"),
    ("Cycle Level",    "info → warn → err → debug → all"),
    ("Keyword Search", "Filter logs by keyword (press Enter to start typing)"),
    ("Clear Filter",   "Clear keyword search"),
];

pub fn handle_key(key: KeyEvent, app: &mut App, state: &mut LogViewerState) -> Option<Action> {
    if state.editing_keyword {
        match key.code {
            KeyCode::Esc => { state.editing_keyword = false; return None; }
            KeyCode::Enter => { state.editing_keyword = false; return None; }
            KeyCode::Char(c) => { state.keyword.push(c); return None; }
            KeyCode::Backspace => { state.keyword.pop(); return None; }
            _ => return None,
        }
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { if state.scroll_offset < state.lines.len().saturating_sub(1) as u16 { state.scroll_offset += 1; state.auto_scroll = false; } None }
        KeyCode::Down | KeyCode::Char('j') => { if state.scroll_offset > 0 { state.scroll_offset -= 1; state.auto_scroll = false; } None }
        KeyCode::Left  => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Right => { let c = &mut app.command_cursor; if *c + 1 < COMMANDS.len() { *c += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => match xray_services::JournalService::fetch_logs(500, Some(&state.level_filter), None) { Ok(lines) => { state.lines = lines; None } Err(e) => Some(Action::ShowMessage(format!("Error: {}", e))) }
            1 => { state.auto_scroll = !state.auto_scroll; if state.auto_scroll { state.scroll_offset = 0; } None }
            2 => { state.level_filter = match state.level_filter.as_str() { "info"=>"warn".into(),"warn"=>"err".into(),"err"=>"debug".into(),"debug"=>"all".into(),_=>"info".into() }; None }
            3 => { state.editing_keyword = true; None }
            4 => { state.keyword.clear(); None }
            _ => None,
        },
        KeyCode::PageUp   => { state.scroll_offset = (state.scroll_offset+15).min(state.lines.len().saturating_sub(1) as u16); state.auto_scroll=false; None }
        KeyCode::PageDown => { state.scroll_offset = state.scroll_offset.saturating_sub(15); None }
        KeyCode::Char('/') => { state.editing_keyword = true; None }
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &LogViewerState, command_cursor: usize) {
    let chunks = Layout::vertical([
        Constraint::Min(5),
        Constraint::Length(if state.editing_keyword { 2 } else { 0 }),
        Constraint::Length(3 + COMMANDS.len() as u16),
    ]).split(area);

    let filtered: Vec<&String> = if state.keyword.is_empty() {
        state.lines.iter().collect()
    } else {
        state.lines.iter().filter(|l| l.to_lowercase().contains(&state.keyword.to_lowercase())).collect()
    };
    let visible: Vec<Line> = filtered.iter().skip(state.scroll_offset as usize).take(chunks[0].height as usize - 2).map(|l| {
        let c = if l.contains("ERROR") || l.contains("ERR") { Color::Red } else if l.contains("WARN") { Color::Yellow } else if l.contains("INFO") { Color::Green } else if l.contains("DEBUG") { Color::DarkGray } else { Color::White };
        Line::from(Span::styled(l.as_str(), Style::default().fg(c)))
    }).collect();
    let title = format!("Logs — lvl:{}  kw:\"{}\"  {}", state.level_filter, state.keyword, if state.auto_scroll {"AUTO"} else {"HOLD"});
    f.render_widget(Paragraph::new(visible).block(Block::default().borders(Borders::ALL).title(title)), chunks[0]);

    if state.editing_keyword {
        let prompt = Span::styled(format!("Search: {}_", state.keyword), Style::default().fg(Color::Yellow));
        f.render_widget(Paragraph::new(Line::from(prompt)).block(Block::default().borders(Borders::ALL).style(Style::default().bg(Color::Rgb(30,30,40)))), chunks[1]);
    }

    let cmd_area = if state.editing_keyword { chunks[2] } else { *chunks.get(1).unwrap_or(&chunks[1]) };
    let items: Vec<Line> = COMMANDS.iter().enumerate().map(|(i,(l,d))| {
        let hl = i == command_cursor;
        let s = if hl { Style::default().fg(Color::Black).bg(Color::Cyan) } else { Style::default() };
        Line::from(vec![Span::styled(if hl { format!(" ▶ {}", l) } else { format!("   {}", l) }, s), Span::styled(format!("  — {}", d), Style::default().fg(Color::DarkGray))])
    }).collect();
    f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).title("Commands — ←→ select  Enter execute  /:search  ↑↓:scroll")), cmd_area);
}
