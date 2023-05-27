use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

mod api;
mod cli;
mod commands;
mod config;
mod errors;
mod output;

type ExitCode = i32;

fn run() -> Result<ExitCode, AnyError> {
    let cli = cli::build_cli();
    let username = cli.username.as_deref();
    let password = cli.password.as_deref();
    let api = api::Client::new(&cli.host, username, password, cli.skip_tls_verification).unwrap();
    let config = Config { api: &api };

    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    let mut out = Output::new(&mut stdout_lock);

    cli.command.execute(&mut out, &config)?;
    Ok(0)
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
