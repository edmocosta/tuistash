use crate::config::Config;
use crate::output::Output;
use crate::result::GenericResult;

pub trait RunnableCommand<T> {
    fn run(&self, out: &mut Output, args: &T, config: &Config) -> GenericResult<()>;
}