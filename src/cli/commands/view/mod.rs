use time::OffsetDateTime;

mod app;
mod backend;
mod charts;
pub mod command;
mod flow_metrics_charts;
mod pipeline_graph;
mod ui;
mod ui_flow_tab;
mod ui_node_tab;
mod ui_pipelines_tab;
mod widgets;

fn now_local_unix_timestamp() -> i64 {
    OffsetDateTime::now_local().unwrap().unix_timestamp()
}
