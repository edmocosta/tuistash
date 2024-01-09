use std::time::Duration;

use clap::Args;

use crate::commands::traits::RunnableCommand;
use crate::commands::tui::backend::run;
use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

#[derive(Args)]
pub struct TuiArgs {
    /// Refresh interval in seconds
    #[arg(default_value_t = 1, short = 'i', long)]
    pub interval: u64,
}

impl Default for TuiArgs {
    fn default() -> Self {
        TuiArgs { interval: 1 }
    }
}

pub struct TuiCommand;

impl RunnableCommand<TuiArgs> for TuiCommand {
    fn run(&self, _: &mut Output, args: &TuiArgs, config: &Config) -> Result<(), AnyError> {
        let tick_rate = Duration::from_secs(args.interval);
        if let Err(e) = run(tick_rate, config) {
            println!("{}", e);
        }

        Ok(())
    }
}
