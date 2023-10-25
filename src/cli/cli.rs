use clap::Parser;
use commands::Command;

use crate::commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, default_value = "http://localhost:9600", global = true)]
    pub host: String,

    #[arg(long, global = true)]
    pub username: Option<String>,

    #[arg(long, global = true)]
    pub password: Option<String>,

    #[arg(long, default_value_t = false, global = true)]
    pub skip_tls_verification: bool,
}

pub fn build_cli() -> Cli {
    Cli::parse()
}
