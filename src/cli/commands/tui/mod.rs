use time::OffsetDateTime;

mod app;
mod backend;
mod charts;
pub mod command;
mod data_fetcher;
mod events;
mod flow_charts;

mod data_decorator;
mod flows;
mod node;
mod pipelines;
mod shared_state;
mod ui;
mod widgets;

fn now_local_unix_timestamp() -> i64 {
    OffsetDateTime::now_local().unwrap().unix_timestamp()
}
