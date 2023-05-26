use crate::api;
use std::sync::Arc;

pub struct Config<'a> {
    pub api: Arc<&'a api::Client<'a>>,
}
