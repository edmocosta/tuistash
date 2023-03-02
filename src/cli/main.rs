use crate::config::Config;
use crate::result::GenericResult;
use crate::output::Output;

mod cli;
mod commands;
mod output;
mod config;
mod api;
mod result;

type ExitCode = i32;

fn run() -> GenericResult<ExitCode> {
    let cli = cli::build_cli();
    let mut api = api::Client::new(&cli.host, &cli.port, None, None).unwrap();
    let config = Config { api: &mut api };

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