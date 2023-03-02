use clap::Subcommand;

use crate::commands::node::command::{NodeArgs, NodeCommand};
use crate::commands::traits::RunnableCommand;
use crate::config::Config;
use crate::output::Output;
use crate::result::GenericResult;

pub mod traits;
mod node;

#[derive(Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Get(GetCommands),

    Stats,
}

#[derive(Subcommand)]
pub enum GetCommands {
    Node(NodeArgs),
}

impl Command {
    pub fn execute(&self, out: &mut Output, config: &Config) -> GenericResult<()> {
        match &self {
            Command::Get(subcommand) => {
                return match subcommand {
                    GetCommands::Node(args) => {
                        NodeCommand.run(out, &args, config)
                    }
                };
            }
            Command::Stats => {
                todo!()
            }
        }
    }
}