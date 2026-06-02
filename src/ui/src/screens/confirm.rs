use ratatui::{Frame, layout::Rect, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Clear}};

pub fn render(f: &mut Frame, area: Rect, message: &str) {
    let pw = 45; let ph = 5;
    let popup = Rect::new(area.x + (area.width.saturating_sub(pw))/2, area.y + (area.height.saturating_sub(ph))/2, pw, ph);
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""), Line::from(message), Line::from(""),
            Line::from(Span::styled("  Enter:Confirm    Esc:Cancel  ", Style::default().fg(Color::Yellow))),
        ]).block(Block::default().borders(Borders::ALL).title("Confirm").style(Style::default().bg(Color::Rgb(30, 30, 40)))),
        popup,
    );
}
