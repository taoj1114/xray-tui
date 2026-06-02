use ratatui::{Frame, layout::Rect, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Clear}};

pub fn render(f: &mut Frame, area: Rect, content: &str) {
    let popup_w = 60; let popup_h = 12;
    let popup = Rect::new(area.x + (area.width.saturating_sub(popup_w))/2, area.y + (area.height.saturating_sub(popup_h))/2, popup_w, popup_h);
    f.render_widget(Clear, popup);
    let preview = if content.len() > 500 { &content[..500] } else { content };
    let lines: Vec<Line> = std::iter::once(Line::from(""))
        .chain(preview.lines().map(|l| Line::from(l)))
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from(Span::styled("  y:Copy  Esc:Close  ", Style::default().fg(Color::Yellow)))))
        .collect();
    f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Share Link").style(Style::default().bg(Color::Rgb(25, 25, 35)))), popup);
}
