use ratatui::{Frame, layout::Rect, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Clear}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action, Screen};

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<Action> {
    let content = match &app.current_screen {
        Screen::ShareExport { content } => content.clone(),
        _ => return None,
    };
    match key.code {
        KeyCode::Esc => Some(Action::PopScreen),
        KeyCode::Char('c') => {
            let pipe = std::process::Command::new("xclip")
                .arg("-sel").arg("clip")
                .stdin(std::process::Stdio::piped())
                .spawn();
            if let Ok(mut child) = pipe {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(content.as_bytes());
                }
                let _ = child.wait();
            }
            Some(Action::ShowMessage("Copied to clipboard".into()))
        }
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, content: &str) {
    let popup_w = 60;
    let popup_h = 12;
    let popup = Rect::new(
        area.x + (area.width.saturating_sub(popup_w)) / 2,
        area.y + (area.height.saturating_sub(popup_h)) / 2,
        popup_w,
        popup_h,
    );
    f.render_widget(Clear, popup);

    let preview = if content.len() > 500 { &content[..500] } else { content };
    let lines: Vec<Line> = std::iter::once(Line::from(""))
        .chain(preview.lines().map(|l| Line::from(l)))
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from(Span::styled("  c:Copy  Esc:Close  ", Style::default().fg(Color::Yellow)))))
        .collect();

    f.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Share Link / Subscription").style(Style::default().bg(Color::Rgb(25, 25, 35)))),
        popup,
    );
}
