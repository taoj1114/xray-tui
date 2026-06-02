use std::io;
use std::time::Duration;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use xray_model::{AppState, GlobalSettings};
use xray_services::{XrayService, SystemdService, Storage, ConfigManager};
use xray_ui::{App, render};

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let settings = GlobalSettings::default();
    let storage = Storage::new(&settings);
    let config_manager = ConfigManager::new(&settings.config_dir);
    let state = storage.load_or_default().unwrap_or_else(|_| AppState {
        settings: settings.clone(),
        stored_certs: vec![],
    });

    let xray_service = XrayService::new(state.settings.clone());
    let systemd_service = SystemdService::new(
        state.settings.xray_binary_path.clone(),
        state.settings.config_path.clone(),
    );

    let mut app = App::new(xray_service, systemd_service, storage, config_manager, state);
    app.refresh_status();

    let tick_rate = Duration::from_millis(16);
    let result = run_app(&mut terminal, &mut app, tick_rate);

    app.save_and_quit();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(result?)
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

        if app.should_quit {
            return Ok(());
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_event(Event::Key(key));
                }
            }
        } else {
            app.on_tick();
        }
    }
}
