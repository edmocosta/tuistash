use std::backtrace::Backtrace;
use std::panic;

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
    setup_panic_hook();

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

fn setup_panic_hook() {
    panic::set_hook(Box::new(move |panic_info| {
        let backtrace = Backtrace::force_capture().to_string();
        let loc = if let Some(location) = panic_info.location() {
            format!(" in file {} at line {}", location.file(), location.line())
        } else {
            String::new()
        };

        let message = if let Some(value) = panic_info.payload().downcast_ref::<&str>() {
            value
        } else if let Some(value) = panic_info.payload().downcast_ref::<String>() {
            value
        } else {
            "Payload not captured as it is not a string."
        };

        println!("Panic occurred{} with the message: {}", loc, message);
        println!("Panic backtrace: \n{}", backtrace);
    }));
}
