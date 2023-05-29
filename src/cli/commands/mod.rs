use clap::Subcommand;

use crate::commands::node::command::{NodeArgs, NodeCommand};
use crate::commands::traits::RunnableCommand;
use crate::commands::view::command::{StatsCommand, ViewArgs};
use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

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
    pub fn execute(&self, out: &mut Output, config: &Config) -> Result<(), AnyError> {
        match &self {
            Command::Get(subcommand) => {
                return match subcommand {
                    GetCommands::Node(args) => NodeCommand.run(out, args, config),
                };
            }
            Command::View(args) => StatsCommand.run(out, args, config),
        }
    }
}
