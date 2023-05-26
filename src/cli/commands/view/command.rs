use std::time::Duration;

use clap::Args;

use crate::commands::traits::RunnableCommand;
use crate::commands::view::backend::run;
use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

#[derive(Args)]
pub struct ViewArgs {
    /// Refresh interval in seconds
    #[arg(default_value = "1", short = 'i', long)]
    pub interval: u64,
}

pub struct StatsCommand;

impl RunnableCommand<ViewArgs> for StatsCommand {
    fn run(&self, _: &mut Output, args: &ViewArgs, config: &Config) -> Result<(), AnyError> {
        let tick_rate = Duration::from_secs(args.interval);
        if let Err(e) = run(tick_rate, config) {
            println!("{}", e);
        }

        Ok(())
    }
}
