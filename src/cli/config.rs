use crate::api;

pub struct Config {
    pub api: api::Client,
    pub diagnostic_path: Option<String>,
}
