use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::commands::tui::app::App;
use crate::commands::tui::data_fetcher::{ApiDataFetcher, PathDataFetcher};
use crate::commands::tui::ui;
use crate::config::Config;
use crate::errors::AnyError;

const APP_TITLE: &str = "Logstash";

pub fn run(interval: Duration, config: &Config) -> Result<(), AnyError> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    if let Some(path) = &config.diagnostic_path {
        match PathDataFetcher::new(path.to_string()) {
            Ok(file_data_fetcher) => {
                run_app(
                    &mut terminal,
                    App::new(
                        APP_TITLE,
                        &file_data_fetcher,
                        path.to_string().as_str(),
                        None,
                    ),
                )?;
            }
            Err(err) => {
                return Err(err);
            }
        };
    } else {
        run_app(
            &mut terminal,
            App::new(
                APP_TITLE,
                &ApiDataFetcher::new(config.api),
                config.api.base_url(),
                Some(interval),
            ),
        )?;
    };

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut last_tick = Instant::now();

    app.on_tick();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let interval = app.sampling_interval.unwrap_or(Duration::from_secs(1));

        let timeout = interval
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => app.on_esc(),
                    _ => app.handle_key_event(key),
                }
            }
        }

        if app.sampling_interval.is_some() && last_tick.elapsed() >= interval {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
