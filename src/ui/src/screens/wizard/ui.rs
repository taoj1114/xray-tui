use ratatui::{Frame, layout::{Layout, Constraint, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}};
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use xray_model;
use crate::{App, Action, InputMode};
use super::state::{InboundWizardState, WizardStep, WizardFieldType};
use super::templates::InboundTemplate;

// ─── Key Handler ──────────────────────────────────────────────────

pub fn handle_key(key: KeyEvent, app: &mut App, wiz: &mut InboundWizardState) -> Option<Action> {
    if wiz.current_step == WizardStep::Template {
        return handle_template_step(key, wiz);
    }
    match key.code {
        KeyCode::Tab => { let n = wiz.fields.len(); if n > 0 { wiz.focused = (wiz.focused + 1) % n; } return None; }
        KeyCode::BackTab => { let n = wiz.fields.len(); if n > 0 { wiz.focused = (wiz.focused + n - 1) % n; } return None; }
        KeyCode::Up => {
            if let Some(f) = wiz.fields.get(wiz.focused) {
                if f.is_open && f.field_type == WizardFieldType::Dropdown { return handle_dropdown_nav(wiz, false); }
            }
            let n = wiz.fields.len(); if n > 0 { wiz.focused = (wiz.focused + n - 1) % n; }
            return None;
        }
        KeyCode::Down => {
            if let Some(f) = wiz.fields.get(wiz.focused) {
                if f.is_open && f.field_type == WizardFieldType::Dropdown { return handle_dropdown_nav(wiz, true); }
            }
            let n = wiz.fields.len(); if n > 0 { wiz.focused = (wiz.focused + 1) % n; }
            return None;
        }
        KeyCode::Enter => return handle_enter(app, wiz),
        KeyCode::Esc => {
            if wiz.close_dropdowns() { app.mode = InputMode::Normal; return None; }
            if wiz.current_step == WizardStep::Basic { return Some(Action::PopScreen); }
            wiz.prev_step(); return None;
        }
        KeyCode::Right if key.modifiers.is_empty() => { if let Some(err) = wiz.next_step() { wiz.error_msg = Some(format!("Validation: {}", err)); } return None; }
        KeyCode::Left if key.modifiers.is_empty() => { wiz.prev_step(); return None; }
        KeyCode::Char('k') if !matches!(wiz.fields.get(wiz.focused).map(|f| &f.field_type), Some(WizardFieldType::TextInput)) => {
            if let Some(f) = wiz.fields.get(wiz.focused) {
                if f.is_open && f.field_type == WizardFieldType::Dropdown { return handle_dropdown_nav(wiz, false); }
            }
            let n = wiz.fields.len(); if n > 0 { wiz.focused = (wiz.focused + n - 1) % n; }
            return None;
        }
        KeyCode::Char('j') if !matches!(wiz.fields.get(wiz.focused).map(|f| &f.field_type), Some(WizardFieldType::TextInput)) => {
            if let Some(f) = wiz.fields.get(wiz.focused) {
                if f.is_open && f.field_type == WizardFieldType::Dropdown { return handle_dropdown_nav(wiz, true); }
            }
            let n = wiz.fields.len(); if n > 0 { wiz.focused = (wiz.focused + 1) % n; }
            return None;
        }
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(f) = wiz.fields.iter_mut().find(|f| f.label == "UUID") { f.value = uuid::Uuid::new_v4().to_string(); }
            return None;
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(Action::GenerateRealityKeys);
        }
        KeyCode::Char(c) => {
            if let Some(f) = wiz.fields.get_mut(wiz.focused) { if f.field_type == WizardFieldType::TextInput { f.value.push(c); app.mode = InputMode::Editing; } }
            return None;
        }
        KeyCode::Backspace => {
            if let Some(f) = wiz.fields.get_mut(wiz.focused) { if f.field_type == WizardFieldType::TextInput { f.value.pop(); } }
            return None;
        }
        _ => None,
    }
}

fn handle_template_step(key: KeyEvent, wiz: &mut InboundWizardState) -> Option<Action> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => { if wiz.focused > 0 { wiz.focused -= 1; wiz.selected_template = wiz.focused; } None }
        KeyCode::Down | KeyCode::Char('j') => {
            let max = InboundTemplate::all().len().saturating_sub(1);
            if wiz.focused < max { wiz.focused += 1; wiz.selected_template = wiz.focused; } None
        }
        KeyCode::Enter => {
            if let Some(t) = InboundTemplate::all().get(wiz.selected_template) { wiz.builder.apply_template(&t.resolve_params()); }
            wiz.next_step(); None
        }
        KeyCode::Esc => Some(Action::PopScreen),
        _ => None,
    }
}

fn handle_enter(app: &mut App, wiz: &mut InboundWizardState) -> Option<Action> {
    if let Some(field) = wiz.fields.get_mut(wiz.focused) {
        match field.field_type {
            WizardFieldType::Dropdown => { field.is_open = !field.is_open; if field.is_open { app.mode = InputMode::Selecting; } return None; }
            WizardFieldType::Toggle => { field.value = if field.value == "true" { "false".into() } else { "true".into() }; return None; }
            _ => {}
        }
    }
    if wiz.current_step == WizardStep::Confirm {
        let config = wiz.builder.build();
        return if let Some(idx) = wiz.edit_index { Some(Action::UpdateInbound(idx, config)) } else { Some(Action::SaveInbound(config)) };
    }
    wiz.next_step();
    None
}

fn handle_dropdown_nav(wiz: &mut InboundWizardState, down: bool) -> Option<Action> {
    if let Some(field) = wiz.fields.get_mut(wiz.focused) {
        if field.is_open && field.field_type == WizardFieldType::Dropdown {
            if down { if field.selected_option + 1 < field.options.len() { field.selected_option += 1; } }
            else if field.selected_option > 0 { field.selected_option -= 1; }
            field.value = field.options.get(field.selected_option).cloned().unwrap_or_default();
        }
    }
    None
}

// ─── Render ───────────────────────────────────────────────────────

pub fn render(f: &mut Frame, area: Rect, app: &App, wiz: &InboundWizardState) {
    let step_names = ["Template", "Basic", "Transport", "Sniffing", "Security", "Users", "Confirm"];
    let step_idx = wiz.current_step.clone() as usize;
    let title = format!("{} Inbound — Step {}/6: {}", if wiz.edit_index.is_some() { "Edit" } else { "New" }, step_idx, step_names[step_idx]);

    if wiz.current_step == WizardStep::Template { render_template_selector(f, area, wiz); return; }
    if wiz.current_step == WizardStep::Confirm { render_confirm(f, area, &title, wiz); return; }

    let form_h = (wiz.fields.len() + 4 + if wiz.error_msg.is_some() { 1 } else { 0 }) as u16;
    let chunks = Layout::vertical([Constraint::Length(form_h), Constraint::Min(1)]).split(area);

    let mut field_lines: Vec<Line> = if let Some(ref err) = wiz.error_msg {
        vec![Line::from(Span::styled(format!(" ⚠ {}", err), Style::default().fg(Color::Red)))]
    } else { vec![] };
    field_lines.extend(wiz.fields.iter().enumerate().map(|(i, fld)| {
        let is_focused = i == wiz.focused;
        let (ind, val) = match fld.field_type {
            WizardFieldType::Dropdown => {
                let a = if fld.is_open { "▾" } else { "▸" };
                (format!("{} {}", if is_focused { "▶" } else { " " }, a), Span::styled(format!(" {} ", fld.value), if is_focused { Style::default().bg(Color::Cyan).fg(Color::Black) } else { Style::default() }))
            }
            WizardFieldType::TextInput => {
                (if is_focused { ">" } else { " " }.into(), Span::styled(format!(" {}_", fld.value), if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() }))
            }
            WizardFieldType::Toggle => {
                (if is_focused { ">" } else { " " }.into(), Span::styled(format!("{} {}", if fld.value=="true"{"[X]"}else{"[ ]"}, fld.label), if is_focused { Style::default().fg(Color::Green) } else { Style::default() }))
            }
            _ => (">".into(), Span::raw(&fld.value)),
        };
        Line::from(vec![Span::raw(ind), if fld.field_type != WizardFieldType::Toggle { Span::raw(format!("{}:  ", fld.label)) } else { Span::raw("") }, val])
    }).collect::<Vec<_>>());
    f.render_widget(Paragraph::new(field_lines).block(Block::default().borders(Borders::ALL).title(title)), chunks[0]);

    for (i, fld) in wiz.fields.iter().enumerate() {
        if fld.is_open && fld.field_type == WizardFieldType::Dropdown {
            let pop = Rect::new(chunks[0].x + 18, chunks[0].y + (i + 1) as u16, 20, fld.options.len() as u16 + 2);
            let items: Vec<Line> = fld.options.iter().enumerate().map(|(oi, opt)|
                if oi == fld.selected_option { Line::from(Span::styled(format!(" ▶ {}", opt), Style::default().fg(Color::Black).bg(Color::Cyan))) }
                else { Line::from(Span::raw(format!("   {}", opt))) }
            ).collect();
            f.render_widget(Paragraph::new(items).block(Block::default().borders(Borders::ALL).style(Style::default().bg(Color::Rgb(20, 20, 30)))), pop);
        }
    }
    f.render_widget(Paragraph::new(vec![Line::from(Span::styled("←→ step  ↑↓/Tab field  Enter confirm/open  Esc back/close  ^G UUID  ^K RealityKeys", Style::default().fg(Color::DarkGray)))]), chunks[1]);

    // If on Security step, show available certs
    if wiz.current_step == WizardStep::Security && !app.certificates.is_empty() && wiz.builder.security == xray_model::StreamSecurity::Tls {
        let cert_y = chunks[0].y + chunks[0].height;
        let cert_lines: Vec<Line> = std::iter::once(Line::from(Span::styled("  Available certs:", Style::default().fg(Color::Cyan))))
            .chain(app.certificates.iter().map(|c| Line::from(Span::styled(format!("    {} → cert:{}, key:{}", c.domain, c.cert_path, c.key_path), Style::default().fg(Color::DarkGray)))))
            .collect();
        let cert_len = cert_lines.len() as u16;
        f.render_widget(Paragraph::new(cert_lines), Rect::new(chunks[0].x, cert_y, area.width, cert_len));
    }
}

fn render_confirm(f: &mut Frame, area: Rect, title: &str, wiz: &InboundWizardState) {
    let preview: Vec<Line> = wiz.json_preview.lines().take(area.height as usize - 5).map(Line::from).collect();
    f.render_widget(Paragraph::new(preview).block(Block::default().borders(Borders::ALL).title(title).style(Style::default().fg(Color::Green))), area);
}

fn render_template_selector(f: &mut Frame, area: Rect, wiz: &InboundWizardState) {
    let templates = InboundTemplate::all();
    let header = vec![
        Line::from(Span::styled(" Select a preset template to quickly configure your inbound:", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(" ─── 预设值已填好，选择后可按需修改 ───", Style::default().fg(Color::DarkGray))), Line::from(""),
    ];
    let content: Vec<Line> = templates.iter().enumerate().map(|(i, t)| {
        let (name, desc) = t.info();
        if i == wiz.selected_template {
            Line::from(vec![Span::styled(format!(" ▶ {}  ", name), Style::default().fg(Color::Black).bg(Color::Cyan)), Span::styled(desc, Style::default().fg(Color::Cyan))])
        } else { Line::from(vec![Span::raw(format!("   {}  ", name)), Span::raw(desc)]) }
    }).collect();
    let help = vec![Line::from(""), Line::from(Span::styled(" ↑↓:选择模板  Enter:确认并继续  Esc:返回", Style::default().fg(Color::DarkGray)))];
    f.render_widget(Paragraph::new(header.into_iter().chain(content).chain(help).collect::<Vec<_>>())
        .block(Block::default().borders(Borders::ALL).title("New Inbound — Step 0/6: Choose Template")), area);
}
