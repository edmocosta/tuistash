use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

pub trait RunnableCommand<T> {
    fn run(&self, out: &mut Output, args: &T, config: &Config) -> Result<(), AnyError>;
}
