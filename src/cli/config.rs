use crate::api;
use std::sync::Arc;

pub struct Config {
    pub api: Arc<api::Client>,
}
