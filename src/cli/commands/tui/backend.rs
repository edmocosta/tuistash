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
use crate::commands::tui::ui;
use crate::config::Config;
use crate::errors::AnyError;

pub fn run(interval: Duration, config: &Config) -> Result<(), AnyError> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    run_app(
        &mut terminal,
        App::new("Logstash", config.api, interval),
        interval,
    )?;

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    interval: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    app.on_tick();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

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
        if last_tick.elapsed() >= interval {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
