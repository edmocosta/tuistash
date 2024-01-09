use crate::commands::Command;
use std::panic;
use std::process::exit;

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

    match cli.command {
        Some(cmd) => {
            cmd.execute(&mut out, &config)?;
        }
        None => {
            Command::execute_default_command(&mut out, &config)?;
        }
    }

    Ok(0)
}

fn main() {
    setup_panic_hook();

    let result = run();
    match result {
        Err(err) => {
            println!("{}", err);
            exit(1);
        }
        Ok(exit_code) => {
            exit(exit_code);
        }
    }
}

fn setup_panic_hook() {
    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to clear the terminal
        print!("{}[2J", 27 as char);

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

        exit(-1)
    }));
}
