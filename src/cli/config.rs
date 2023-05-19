use std::sync::Arc;
use crate::api;

pub struct Config {
    pub api: Arc<api::Client>,
}