use ratatui::{Frame, layout::{Layout, Constraint, Direction, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Block, Paragraph, Tabs}};
use crate::app::App;
use crate::app::Screen;
use crate::screens;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(10), Constraint::Length(1)]).split(area);
    render_top_bar(f, layout[0], app); render_tab_bar(f, layout[1], app); render_content(f, layout[2], app); render_help_bar(f, layout[3], app);
}

fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let sc = if app.xray_status.is_running { Color::Green } else { Color::Red };
    let si = if app.xray_status.is_running { "●" } else { "○" };
    let ver = app.xray_status.version.as_deref().unwrap_or("---");
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled(si, Style::default().fg(sc)), Span::raw(" xray "), Span::styled(ver, Style::default().fg(Color::Cyan))])).style(Style::default().bg(Color::Rgb(30, 30, 40))), area);
}

fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let tnames = ["Dashboard", "Inbounds", "SSL", "Logs", "Settings", "Others"];
    let cur = match &app.current_screen { Screen::Dashboard=>0, Screen::InboundList|Screen::ConfigPicker{..}=>1, Screen::SslManagement{..}=>2, Screen::LogViewer(_)=>3, Screen::Settings(_)=>4, Screen::Others=>5, _=>0 };
    let tabs: Vec<Span> = tnames.iter().enumerate().map(|(i,n)| if i==cur { Span::styled(format!(" {} ",n), Style::default().fg(Color::Black).bg(Color::Cyan)) } else { Span::styled(format!(" {} ",n), Style::default().fg(Color::Gray)) }).collect();
    f.render_widget(Tabs::new(tabs).block(Block::default().style(Style::default().bg(Color::Rgb(25, 25, 35)))), area);
}

fn render_content(f: &mut Frame, area: Rect, app: &App) {
    match &app.current_screen {
        Screen::Dashboard => screens::dashboard::render(f, area, app),
        Screen::InboundList => screens::inbound_list::render(f, area, app),
        Screen::ConfigPicker { selected, action } => screens::inbound_list::render_picker(f, area, app, *selected, action),
        Screen::InboundWizard(ref wiz) => screens::wizard::render(f, area, app, wiz),
        Screen::UserManager { inbound_idx: i, selected: s, .. } => screens::user_manager::render(f, area, app, *i, *s),
        Screen::SslManagement { selected: s, editing } => screens::ssl_manager::render(f, area, app, *s, editing),
        Screen::LogViewer(ref st) => screens::log_viewer::render(f, area, st, app.command_cursor),
        Screen::Settings(editing) => screens::settings_page::render(f, area, app, editing),
        Screen::ConfirmDialog { message, .. } => screens::confirm::render(f, area, message),
        Screen::ShareExport { content } => screens::share_export::render(f, area, content),
        Screen::Others => screens::others::render(f, area, app),
    }
}

fn render_help_bar(f: &mut Frame, area: Rect, app: &App) {
    if app.is_busy {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = (app.tick_count / 5) % 10;
        let line = Line::from(vec![Span::styled(spinner[frame as usize], Style::default().fg(Color::Yellow)), Span::raw(" "), Span::styled(&app.busy_msg, Style::default().fg(Color::White))]);
        f.render_widget(Paragraph::new(line).style(Style::default().bg(Color::Rgb(40, 40, 60))), area);
        return;
    }
    let help = match &app.current_screen {
        Screen::Dashboard => "q:Quit  Tab:Tab  ↑↓:Select  Enter:Execute",
        Screen::InboundList|Screen::ConfigPicker{..}=>"↑↓:Command  Enter:Execute  Esc:Back  Tab:Tab",
        Screen::InboundWizard(_)=>"Tab:Field  Esc:Close/Back  Enter:Confirm  ←→:Steps",
        Screen::UserManager{..}=>"Esc:Back  ↑↓:Command  ←→:User  Enter:Execute",
        Screen::SslManagement{..}=>"Esc:Back  ↑↓:Command  ←→:Cert  Enter:Execute",
        Screen::LogViewer(_)=>"Esc:Back  ↑↓:Scroll  ←→:Commands  Enter:Execute",
        Screen::Settings(_) =>"Esc:Back  ↑↓:Command  Enter:Edit",
        Screen::Others => "↑↓:Select  Enter:Execute  Tab:Tab",
        Screen::ConfirmDialog{..}=>"y:Yes  n/Esc:No",
        Screen::ShareExport{..}=>"Esc:Back  y:Copy  o:Open",    };
    if let Some((msg,_))= &app.status_message { f.render_widget(Paragraph::new(Line::from(vec![Span::styled(" ⓘ ", Style::default().fg(Color::Green)), Span::raw(msg)])), area); }
    else { f.render_widget(Paragraph::new(help).style(Style::default().fg(Color::DarkGray)), area); }
}
