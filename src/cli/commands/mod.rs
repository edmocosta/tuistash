use clap::Subcommand;

use crate::commands::node::command::{NodeArgs, NodeCommand};
use crate::commands::traits::RunnableCommand;
use crate::commands::tui::command::{TuiArgs, TuiCommand};
use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

mod formatter;
mod node;
pub mod traits;
mod tui;

#[derive(Subcommand)]
pub enum Command {
    /// Query data from the Logstash API
    #[command(subcommand)]
    Get(GetCommands),
    /// Logstash TUI
    Tui(TuiArgs),
}

#[derive(Subcommand)]
pub enum GetCommands {
    /// Prints the current Logstash node information
    Node(NodeArgs),
}

impl Command {
    pub fn execute_default_command(out: &mut Output, config: &Config) -> Result<(), AnyError> {
        TuiCommand.run(out, &TuiArgs::default(), config)
    }

    pub fn execute(&self, out: &mut Output, config: &Config) -> Result<(), AnyError> {
        match &self {
            Command::Get(subcommand) => match subcommand {
                GetCommands::Node(args) => NodeCommand.run(out, args, config),
            },
            Command::Tui(args) => TuiCommand.run(out, args, config),
        }
    }
}
