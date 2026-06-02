use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table, Cell}};
use crossterm::event::{KeyEvent, KeyCode};
use xray_model::RoutingRule;
use crate::{App, Action, Screen};

#[derive(Debug, Clone)]
pub enum RoutingEditMode { New, Edit(usize) }

const COMMANDS: &[(&str, &str)] = &[
    ("New Rule",      "Create a new routing rule"),
    ("Edit Rule",     "Edit the selected rule"),
    ("Delete Rule",   "Remove the selected rule"),
    ("Load Presets",  "Load CN direct / ads block / private presets"),
    ("Save & Apply",  "Save rules and regenerate Xray config"),
];

pub fn handle_key(key: KeyEvent, app: &mut App, selected: &mut usize, editing: &mut Option<RoutingEditMode>) -> Option<Action> {
    let len = app.routing_rules.len();
    match key.code {
        KeyCode::Up | KeyCode::Char('k')   => { let c = &mut app.command_cursor; *c = c.saturating_sub(1); None }
        KeyCode::Down | KeyCode::Char('j') => { let c = &mut app.command_cursor; if *c + 1 < COMMANDS.len() { *c += 1; } None }
        KeyCode::Left  => { if *selected > 0 { *selected -= 1; } None }
        KeyCode::Right => { if *selected + 1 < len { *selected += 1; } None }
        KeyCode::Enter => match app.command_cursor {
            0 => { *editing = Some(RoutingEditMode::New); None }
            1 if len > 0 => { *editing = Some(RoutingEditMode::Edit(*selected)); None }
            2 if len > 0 => { app.routing_rules.remove(*selected); *selected = (*selected).min(app.routing_rules.len().saturating_sub(1)); Some(Action::SaveRouting(app.routing_rules.clone())) }
            3 => { for p in RoutingRule::all_presets() { if !app.routing_rules.iter().any(|r| r.domain == p.domain && r.outbound_tag == p.outbound_tag) { app.routing_rules.push(p); } } Some(Action::SaveRouting(app.routing_rules.clone())) }
            4 => Some(Action::SaveRouting(app.routing_rules.clone())),
            _ => None,
        },
        KeyCode::Esc if editing.is_some() => { *editing = None; None }
        _ => None,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App, selected: usize, _editing: Option<&RoutingEditMode>) {
    let chunks = Layout::vertical([Constraint::Length(3+app.routing_rules.len().max(1) as u16), Constraint::Min(1)]).split(area);
    let header = Row::new(["#","Type","Match","Action"]).style(Style::default().fg(Color::Cyan));
    let rows: Vec<Row> = if app.routing_rules.is_empty() { vec![Row::new(["","(empty)","",""])] } else { app.routing_rules.iter().enumerate().map(|(i,r)| { let hl=i==selected; let s=if hl{Style::default().fg(Color::Black).bg(Color::White)}else{Style::default()}; let t=if r.domain.is_some(){"domain"}else if r.ip.is_some(){"ip"}else{"any"}; let m=r.domain.as_ref().map(|d|d.join(",")).or_else(||r.ip.as_ref().map(|d|d.join(","))).unwrap_or("*".into()); Row::new(vec![Cell::from((i+1).to_string()).style(s),Cell::from(t).style(s),Cell::from(m).style(s),Cell::from(r.outbound_tag.clone()).style(s)]) }).collect() };
    f.render_widget(Table::new(rows,[Constraint::Length(3),Constraint::Length(7),Constraint::Length(35),Constraint::Length(8)]).header(header).block(Block::default().borders(Borders::ALL).title("Rules")), chunks[0]);
    let items: Vec<Line> = COMMANDS.iter().enumerate().map(|(i,(l,d))| { let hl=i==app.command_cursor; let s=if hl{Style::default().fg(Color::Black).bg(Color::Cyan)}else{Style::default()}; Line::from(vec![Span::styled(if hl{format!(" ▶ {}",l)}else{format!("   {}",l)},s),Span::styled(format!("  — {}",d),Style::default().fg(Color::DarkGray))]) }).collect();
    f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).title("Commands — ↑↓ select  ←→ switch rule  Enter execute")), chunks[1]);
}
