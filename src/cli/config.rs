use crate::api;

pub struct Config<'a> {
    pub api: &'a mut api::Client,
}