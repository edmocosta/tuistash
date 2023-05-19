use std::time::Duration;
use clap::Args;
use crate::commands::stats::backend::run;
use crate::commands::traits::RunnableCommand;
use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

#[derive(Args)]
pub struct StatsArgs {
    #[arg(default_value = "1000", short = 'i', long)]
    pub interval: u32,
}

pub struct StatsCommand;

impl RunnableCommand<StatsArgs> for StatsCommand {
    fn run(&self, _: &mut Output, args: &StatsArgs, config: &Config) -> Result<(), AnyError> {
        let tick_rate = Duration::from_millis(1000);
        run(tick_rate, config);
        Ok(())
    }
}