use crate::api;

pub struct Config<'a> {
    pub api: &'a api::Client<'a>,
    pub diagnostic_path: Option<String>,
}
