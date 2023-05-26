use clap::Subcommand;

use crate::commands::node::command::{NodeArgs, NodeCommand};
use crate::commands::view::command::{ViewArgs, StatsCommand};
use crate::commands::traits::RunnableCommand;
use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

mod node;
mod view;
pub mod traits;

#[derive(Subcommand)]
pub enum Command {
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
                    GetCommands::Node(args) => NodeCommand.run(out, &args, config),
                };
            }
            Command::View(args) => StatsCommand.run(out, &args, config),
        }
    }
}
