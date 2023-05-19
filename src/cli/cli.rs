use clap::Parser;
use commands::Command;

use crate::commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, default_value = "http://localhost", global = true)]
    pub host: String,

    #[arg(long, default_value_t = 9600, global = true)]
    pub port: u16,

    #[arg(long, global = true)]
    pub username: Option<String>,

    #[arg(long, global = true)]
    pub password: Option<String>,
}

pub fn build_cli() -> Cli {
    Cli::parse()
}