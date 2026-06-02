use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action};

#[derive(Debug, Clone)]
pub struct LogViewerState {
    pub lines: Vec<(String, LogLevel)>,
    pub error_lines: Vec<String>,
    pub auto_scroll: bool,
    pub scroll_offset: u16,
    pub level_filter: LogLevel,
    pub keyword: String,
    pub editing_keyword: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel { Error, Warn, Info, Debug, All }

impl LogLevel {
    fn name(&self) -> &str {
        match self { Self::Error=>"error", Self::Warn=>"warn", Self::Info=>"info", Self::Debug=>"debug", Self::All=>"all" }
    }
    fn next(&self) -> Self {
        match self { Self::Error=>Self::Warn, Self::Warn=>Self::Info, Self::Info=>Self::Debug, Self::Debug=>Self::All, Self::All=>Self::Error }
    }
    fn color(&self) -> Color {
        match self { Self::Error=>Color::Red, Self::Warn=>Color::Yellow, Self::Info=>Color::Green, Self::Debug=>Color::DarkGray, _=>Color::White }
    }
}

fn parse_level(line: &str) -> LogLevel {
    if line.contains("error") || line.contains("ERROR") || line.contains("err") || line.contains("ERR") || line.contains("failed") || line.contains("Failed") || line.contains("panic") || line.contains("PANIC") { LogLevel::Error }
    else if line.contains("warn") || line.contains("WARN") || line.contains("warning") || line.contains("WARNING") { LogLevel::Warn }
    else if line.contains("info") || line.contains("INFO") { LogLevel::Info }
    else if line.contains("debug") || line.contains("DEBUG") { LogLevel::Debug }
    else { LogLevel::Info }
}

impl Default for LogViewerState {
    fn default() -> Self {
        Self { lines: vec![("No logs. Press Refresh.".into(), LogLevel::Info)], error_lines: Vec::new(), auto_scroll: true, scroll_offset: 0,
            level_filter: LogLevel::All, keyword: String::new(), editing_keyword: false }
    }
}

const COMMANDS: &[(&str, &str)] = &[
    ("Refresh",        "Reload latest 50 lines from journalctl"),
    ("Auto Scroll",    "Toggle auto-scroll"),
    ("Cycle Level",    "all → error → warn → info → debug → all"),
    ("Search",         "Filter logs by keyword (/ or Enter to type)"),
    ("Clear Filter",   "Clear keyword / level filter"),
    ("Extract Errors", "Copy error lines to clipboard"),
];

pub fn handle_key(key: KeyEvent, app: &mut App, state: &mut LogViewerState) -> Option<Action> {
    if state.editing_keyword {
        return match key.code {
            KeyCode::Esc => { state.editing_keyword = false; None }
            KeyCode::Enter => { state.editing_keyword = false; None }
            KeyCode::Char(c) => { state.keyword.push(c); None }
            KeyCode::Backspace => { state.keyword.pop(); None }
            _ => None,
        };
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { if state.scroll_offset < state.lines.len().saturating_sub(1) as u16 { state.scroll_offset += 1; state.auto_scroll = false; } None }
        KeyCode::Down | KeyCode::Char('j') => { if state.scroll_offset > 0 { state.scroll_offset -= 1; state.auto_scroll = false; } None }
        KeyCode::Left  => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Right => { let c = &mut app.command_cursor; if *c + 1 < COMMANDS.len() { *c += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => match xray_services::JournalService::fetch_logs(50, None, None) {
                Ok(lines) => {
                    let parsed: Vec<_> = lines.iter().map(|l| (l.clone(), parse_level(l))).collect();
                    let errors: Vec<_> = parsed.iter().filter(|(_, lvl)| *lvl == LogLevel::Error).map(|(l, _)| l.clone()).collect();
                    state.lines = parsed; state.error_lines = errors; state.scroll_offset = 0;
                    None
                }
                Err(e) => Some(Action::ShowMessage(format!("journalctl: {}", e)))
            },
            1 => { state.auto_scroll = !state.auto_scroll; if state.auto_scroll { state.scroll_offset = 0; } None }
            2 => { state.level_filter = state.level_filter.next(); None }
            3 => { state.editing_keyword = true; None }
            4 => { state.keyword.clear(); state.level_filter = LogLevel::All; None }
            5 => {
                if state.error_lines.is_empty() { return Some(Action::ShowMessage("No errors found".into())); }
                let text = state.error_lines.join("\n");
                let pipe = std::process::Command::new("xclip").arg("-sel").arg("clip").stdin(std::process::Stdio::piped()).spawn();
                if let Ok(mut child) = pipe {
                    if let Some(mut stdin) = child.stdin.take() { use std::io::Write; let _ = stdin.write_all(text.as_bytes()); }
                    let _ = child.wait();
                }
                Some(Action::ShowMessage("Errors copied to clipboard".into()))
            }
            _ => None,
        },
        KeyCode::PageUp   => { state.scroll_offset = (state.scroll_offset + 25).min(state.lines.len().saturating_sub(1) as u16); state.auto_scroll = false; None }
        KeyCode::PageDown => { state.scroll_offset = state.scroll_offset.saturating_sub(25); None }
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

    let filtered: Vec<&(String, LogLevel)> = if state.level_filter == LogLevel::All && state.keyword.is_empty() {
        state.lines.iter().collect()
    } else {
        state.lines.iter().filter(|(l, lvl)| {
            let lvl_ok = state.level_filter == LogLevel::All || *lvl == state.level_filter;
            let kw_ok = state.keyword.is_empty() || l.to_lowercase().contains(&state.keyword.to_lowercase());
            lvl_ok && kw_ok
        }).collect()
    };

    let error_count = state.error_lines.len();
    let total = state.lines.len();
    let visible: Vec<Line> = filtered.iter().skip(state.scroll_offset as usize).take(chunks[0].height as usize - 2).map(|(l, lvl)| {
        let c = lvl.color();
        Line::from(Span::styled(l.as_str(), Style::default().fg(c)))
    }).collect();

    let status = if error_count > 0 {
        Span::styled(format!(" ⚠ {} errors", error_count), Style::default().fg(Color::Red))
    } else {
        Span::styled(" ✓ clean", Style::default().fg(Color::Green))
    };
    let title = Line::from(vec![
        Span::raw(format!("Logs {} | lvl:", if state.auto_scroll {"↓AUTO"} else {"⬆HOLD"})),
        Span::styled(state.level_filter.name(), Style::default().fg(Color::Cyan)),
        Span::raw(format!(" | {} lines", total)),
        Span::raw(" | "), status,
    ]);
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
    f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).title("Commands — ←→ select  /:search  ↑↓PgUp/Dn:scroll")), cmd_area);
}
