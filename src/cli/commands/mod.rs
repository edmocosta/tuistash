use clap::Subcommand;

use crate::commands::node::command::{NodeArgs, NodeCommand};
use crate::commands::stats::command::{StatsArgs, StatsCommand};
use crate::commands::traits::RunnableCommand;
use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

pub mod traits;
mod node;
mod stats;

#[derive(Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Get(GetCommands),

    Stats(StatsArgs),
}

#[derive(Subcommand)]
pub enum GetCommands {
    Node(NodeArgs),
}

impl Command {
    pub fn execute(&self, out: &mut Output, config: &Config) -> Result<(), AnyError> {
        match &self {
            Command::Get(subcommand) => {
                return match subcommand {
                    GetCommands::Node(args) => {
                        NodeCommand.run(out, &args, config)
                    }
                };
            }
            Command::Stats(args) => {
                StatsCommand.run(out, &args, config)
            }
        }
    }
}