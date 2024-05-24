use std::fmt::{Display, Formatter};
use std::{error::Error, fmt};

pub type AnyError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct TuiError {
    message: String,
}

impl TuiError {
    pub fn from(message: &str) -> Self {
        TuiError {
            message: message.to_string(),
        }
    }
}

impl Error for TuiError {}

impl Display for TuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
