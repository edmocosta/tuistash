use std::sync::Arc;
use crate::config::Config;
use crate::errors::{AnyError};
use crate::output::Output;

mod cli;
mod commands;
mod output;
mod config;
mod api;
mod errors;

type ExitCode = i32;

fn run() -> Result<ExitCode, AnyError> {
    let cli = cli::build_cli();
    let api = api::Client::new(&cli.host, &cli.port, cli.username, cli.password).unwrap();
    let config = Config { api: Arc::new(api) };

    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    let mut out = Output::new(&mut stdout_lock);

    cli.command.execute(&mut out, &config)?;
    return Ok(0);
}

fn main() {
    let result = run();
    match result {
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
        Ok(exit_code) => {
            std::process::exit(exit_code);
        }
    }
}