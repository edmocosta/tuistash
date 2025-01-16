use crate::commands::tui::app::App;
use crate::commands::tui::data_fetcher::{ApiDataFetcher, PathDataFetcher};
use crate::commands::tui::ui;
use crate::config::Config;
use crate::errors::AnyError;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::ops::Add;
use std::{
    io,
    time::{Duration, Instant},
};

const APP_TITLE: &str = "Logstash";

pub fn run(interval: Duration, config: &Config) -> Result<(), AnyError> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    if let Some(path) = &config.diagnostic_path {
        let path = path.to_string();
        match PathDataFetcher::new(path.to_string()) {
            Ok(file_data_fetcher) => {
                let mut app = App::new(APP_TITLE.to_string(), path, None);
                app.set_data(&file_data_fetcher);
                run_app(&mut terminal, app)?;
            }
            Err(err) => {
                return Err(err);
            }
        };
    } else {
        let app = App::new(
            APP_TITLE.to_string(),
            config.api.base_url().to_string(),
            Some(interval),
        );

        let fetcher = ApiDataFetcher::new(config.api.clone());
        fetcher.start_polling(interval);

        app.start_reading_data(Box::new(fetcher), interval);
        run_app(&mut terminal, app)?;
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
    app.wait_node_data();
    app.on_tick();

    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let tick_interval = app
            .sampling_interval
            .unwrap_or(Duration::from_secs(1))
            .add(Duration::from_millis(300));

        let timeout = tick_interval
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

        if app.sampling_interval.is_some() && last_tick.elapsed() >= tick_interval {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
