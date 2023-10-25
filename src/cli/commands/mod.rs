use clap::Subcommand;

use crate::commands::node::command::{NodeArgs, NodeCommand};
use crate::commands::traits::RunnableCommand;
use crate::commands::view::command::{ViewArgs, ViewCommand};
use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

mod formatter;
mod node;
pub mod traits;
mod view;

#[derive(Subcommand)]
pub enum Command {
    /// Get data from Logstash
    #[command(subcommand)]
    Get(GetCommands),
    /// Monitoring TUI
    View(ViewArgs),
}

#[derive(Subcommand)]
pub enum GetCommands {
    /// Prints the current node information
    Node(NodeArgs),
}

impl Command {
    pub fn execute_default_command(out: &mut Output, config: &Config) -> Result<(), AnyError> {
        ViewCommand.run(out, &ViewArgs::default(), config)
    }

    pub fn execute(&self, out: &mut Output, config: &Config) -> Result<(), AnyError> {
        match &self {
            Command::Get(subcommand) => {
                return match subcommand {
                    GetCommands::Node(args) => NodeCommand.run(out, args, config),
                };
            }
            Command::View(args) => ViewCommand.run(out, args, config),
        }
    }
}
