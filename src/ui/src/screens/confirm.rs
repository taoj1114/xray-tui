use ratatui::{Frame, layout::Rect, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Clear}};
use crossterm::event::{KeyEvent, KeyCode};
use crate::{App, Action, Screen};

pub fn handle_key(key: KeyEvent, app: &mut App) -> Option<Action> {
    let Screen::ConfirmDialog { on_confirm, .. } = &app.current_screen else { return None; };
    match key.code {
        KeyCode::Enter => {
            let action = match on_confirm {
                crate::ConfirmedAction::DeleteInbound(idx) => Action::DeleteInbound(*idx),
                crate::ConfirmedAction::DeleteUser { inbound_idx, user_idx } => Action::DeleteUser(*inbound_idx, *user_idx),
                crate::ConfirmedAction::DeleteCert(idx) => { app.certificates.remove(*idx); Action::ShowMessage("Deleted".into()) }
                crate::ConfirmedAction::RestartXray => Action::RestartXray,
                crate::ConfirmedAction::StopXray => Action::StopXray,
            };
            app.pop_screen();
            Some(action)
        }
        KeyCode::Esc => { app.pop_screen(); None }
        _ => None,
    }
}

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
