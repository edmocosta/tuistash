use std::{f64, i64};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Local;

use crate::api;
use crate::api::node::{GraphDefinition, NodeInfo, NodeInfoType};
use crate::api::stats::NodeStats;
use crate::commands::stats::charts::{
    ChartDataPoint, DEFAULT_MAX_DATA_POINTS, TimestampChartState,
};
use crate::commands::stats::graph::PipelineGraph;
use crate::commands::stats::pipeline_viewer;
use crate::commands::stats::widgets::{StatefulTable, TabsState};

struct DataFetcher {
    api: Arc<api::Client>,
}

impl DataFetcher {
    pub fn new(api: Arc<api::Client>) -> DataFetcher {
        DataFetcher { api }
    }

    pub fn fetch_info(&self) -> Option<NodeInfo> {
        match self.api.get_node_info(&[NodeInfoType::Pipelines], &vec![]) {
            Ok(stats) => Some(stats),
            Err(_) => {
                //println!("{:?}", err);
                None
            }
        }
    }

    pub fn fetch_stats(&self) -> Option<NodeStats> {
        match self.api.get_node_stats() {
            Ok(stats) => Some(stats),
            Err(err) => {
                //println!("{:?}", err);
                None
            }
        }
    }
}

impl StatefulTable<String> {
    fn update(&mut self, app_state: &AppState, selected_pipeline: Option<&PipelineItem>) {
        if selected_pipeline.is_none() || app_state.node_info.is_none() {
            self.items = vec![];
            self.unselect();
            return;
        }

        self.items = pipeline_viewer::create_rows_ids(
            &PipelineGraph::from(&selected_pipeline.unwrap().graph),
            selected_pipeline.unwrap(),
        );
    }
}

impl StatefulTable<PipelineItem> {
    fn update(&mut self, state: &AppState) {
        if let Some(node_info) = &state.node_info {
            if let Some(pipelines) = &node_info.pipelines {
                let mut new_items = Vec::with_capacity(pipelines.len());
                for (name, pipeline_info) in pipelines {
                    let new_item = PipelineItem {
                        id: pipeline_info.ephemeral_id.to_string(),
                        name: name.to_string(),
                        graph: pipeline_info.graph.graph.clone(),
                    };

                    new_items.push(new_item);
                }

                new_items.sort_by_key(|k| k.name.to_string());
                if let Some(selected_pipeline_name) = self.selected_item().map(|p| p.name.as_str())
                {
                    if let Some(new_index) = new_items
                        .iter()
                        .position(|p| p.name == selected_pipeline_name)
                    {
                        self.state.select(Some(new_index));
                    }
                }

                self.items = new_items;
            }
        }
    }
}

pub struct PipelineItem {
    pub id: String,
    pub name: String,
    pub graph: GraphDefinition,
}

pub struct PluginFlowMetricDataPoint {
    pub timestamp: i64,
    pub input: f64,
    pub filter: f64,
    pub output: f64,
}

impl ChartDataPoint for PluginFlowMetricDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        return [
            f64::min(f64::min(self.input, self.filter), self.output),
            f64::max(f64::max(self.input, self.filter), self.output),
        ];
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        return [self.timestamp as f64, self.timestamp as f64];
    }
}

pub struct FlowMetricDataPoint {
    pub timestamp: i64,
    pub value: f64,
}

impl ChartDataPoint for FlowMetricDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        return [self.value as f64, self.value as f64];
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        return [self.timestamp as f64, self.timestamp as f64];
    }
}

pub struct ProcessCpuDataPoint {
    pub timestamp: i64,
    pub percent: i64,
}

impl ChartDataPoint for ProcessCpuDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        return [self.percent as f64, self.percent as f64];
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        return [self.timestamp as f64, self.timestamp as f64];
    }
}

pub struct JvmMemNonHeapDataPoint {
    pub timestamp: i64,
    pub non_heap_committed_in_bytes: i64,
    pub non_heap_used_in_bytes: i64,
}

impl ChartDataPoint for JvmMemNonHeapDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        return [
            f64::min(
                self.non_heap_used_in_bytes as f64,
                self.non_heap_committed_in_bytes as f64,
            ),
            f64::max(
                self.non_heap_used_in_bytes as f64,
                self.non_heap_committed_in_bytes as f64,
            ),
        ];
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        return [self.timestamp as f64, self.timestamp as f64];
    }
}

pub struct JvmMemHeapDataPoint {
    pub timestamp: i64,
    pub heap_committed_in_bytes: i64,
    pub heap_max_in_bytes: i64,
    pub heap_used_in_bytes: i64,
    pub heap_used_percent: i64,
}

impl ChartDataPoint for JvmMemHeapDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        return [
            f64::min(
                self.heap_max_in_bytes as f64,
                self.heap_used_in_bytes as f64,
            ),
            f64::max(
                self.heap_max_in_bytes as f64,
                self.heap_used_in_bytes as f64,
            ),
        ];
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        return [self.timestamp as f64, self.timestamp as f64];
    }
}

pub struct SelectedPipelineVertexChartState {
    pub throughput: Option<TimestampChartState<FlowMetricDataPoint>>,
    pub worker_utilization: Option<TimestampChartState<FlowMetricDataPoint>>,
}

impl SelectedPipelineVertexChartState {
    pub fn new() -> Self {
        SelectedPipelineVertexChartState {
            throughput: None,
            worker_utilization: None,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        if let Some(throughput) = &self.throughput {
            if !throughput.data_points.is_empty() {
                return false;
            }
        }

        if let Some(worker_utilization) = &self.worker_utilization {
            if !worker_utilization.data_points.is_empty() {
                return false;
            }
        }

        return true;
    }

    pub fn reset(&mut self) {
        if let Some(throughput) = self.throughput.as_mut() {
            throughput.reset();
            self.throughput = None;
        }

        if let Some(worker_utilization) = self.worker_utilization.as_mut() {
            worker_utilization.reset();
            self.worker_utilization = None;
        }
    }
}

pub(crate) struct AppState {
    pub node_info: Option<NodeInfo>,
    pub node_stats: Option<NodeStats>,

    chart_pipeline_vertex_id: Option<String>,
    pub chart_pipeline_vertex_id_state: SelectedPipelineVertexChartState,

    pub chart_jvm_heap_state: TimestampChartState<JvmMemHeapDataPoint>,
    pub chart_jvm_non_heap_state: TimestampChartState<JvmMemNonHeapDataPoint>,
    pub chart_process_cpu: TimestampChartState<ProcessCpuDataPoint>,

    pub chart_flow_plugins_throughput: TimestampChartState<PluginFlowMetricDataPoint>,
    pub chart_flow_queue_backpressure: TimestampChartState<FlowMetricDataPoint>,

    pub chart_flow_pipeline_plugins_throughput:
    HashMap<String, TimestampChartState<PluginFlowMetricDataPoint>>,
    pub chart_flow_pipeline_queue_backpressure:
    HashMap<String, TimestampChartState<FlowMetricDataPoint>>,
}

pub(crate) struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub pipelines: StatefulTable<PipelineItem>,
    pub selected_pipeline_graph: StatefulTable<String>,
    pub state: AppState,
    pub show_selected_pipeline_charts: bool,
    pub show_selected_plugin_charts: bool,
    pub focused: usize,
    pub refresh_interval: Duration,
    data_fetcher: DataFetcher,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, api: Arc<api::Client>, refresh_interval: Duration) -> App<'a> {
        let max_data_points = Some(10);

        let app_state = AppState {
            node_info: None,
            node_stats: None,
            chart_pipeline_vertex_id: None,
            chart_pipeline_vertex_id_state: SelectedPipelineVertexChartState::new(),
            chart_jvm_heap_state: TimestampChartState::new(max_data_points),
            chart_jvm_non_heap_state: TimestampChartState::new(max_data_points),
            chart_process_cpu: TimestampChartState::new(max_data_points),
            chart_flow_plugins_throughput: TimestampChartState::new(max_data_points),
            chart_flow_queue_backpressure: TimestampChartState::new(max_data_points),
            chart_flow_pipeline_plugins_throughput: Default::default(),
            chart_flow_pipeline_queue_backpressure: Default::default(),
        };

        App {
            title,
            refresh_interval,
            show_selected_pipeline_charts: false,
            show_selected_plugin_charts: false,
            should_quit: false,
            tabs: TabsState::new(vec!["Pipelines", "Node"]),
            pipelines: StatefulTable::new(),
            selected_pipeline_graph: StatefulTable::new(),
            data_fetcher: DataFetcher::new(api),
            state: app_state,
            focused: 0,
        }
    }

    const PIPELINE_LIST: usize = 0;
    const PIPELINE_VIEWER_LIST: usize = 1;

    pub fn on_up(&mut self) {
        if self.focused == Self::PIPELINE_LIST {
            self.selected_pipeline_graph
                .update(&self.state, self.pipelines.previous());
        } else {
            self.selected_pipeline_graph.previous();
            self.update_selected_vertex_details();
        }
    }

    pub fn on_down(&mut self) {
        if self.focused == Self::PIPELINE_LIST {
            self.selected_pipeline_graph
                .update(&self.state, self.pipelines.next());
        } else {
            self.selected_pipeline_graph.next();
            self.update_selected_vertex_details();
        }
    }

    pub fn on_right(&mut self) {
        if self.focused == Self::PIPELINE_LIST {
            self.focused = Self::PIPELINE_VIEWER_LIST;
            self.selected_pipeline_graph.next();
            self.update_selected_vertex_details();
        }
    }

    pub fn on_left(&mut self) {
        if self.focused == Self::PIPELINE_VIEWER_LIST {
            self.focused = Self::PIPELINE_LIST;
            self.selected_pipeline_graph.unselect();
            self.show_selected_plugin_charts = false;
            self.update_selected_vertex_details();
        }
    }

    pub fn on_enter(&mut self) {
        if self.focused == Self::PIPELINE_VIEWER_LIST {
            self.show_selected_pipeline_charts = false;
            self.show_selected_plugin_charts = !self.show_selected_plugin_charts;
            self.update_selected_vertex_details();
        }
    }

    fn update_selected_vertex_details(&mut self) {
        if !self.show_selected_plugin_charts && self.state.chart_pipeline_vertex_id.is_some() {
            self.state.chart_pipeline_vertex_id = None;
            self.state.chart_pipeline_vertex_id_state.reset();
        } else if self.show_selected_plugin_charts {
            let selected_vertex = self
                .selected_pipeline_graph
                .selected_item()
                .map(|p| p.to_string());

            if self.state.chart_pipeline_vertex_id != selected_vertex {
                self.state.chart_pipeline_vertex_id = selected_vertex;
                self.state.chart_pipeline_vertex_id_state.reset();
                self.update_current_vertex_id_sampling();
            }
        }
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            'f' => {
                if self.pipelines.selected_item().is_some() {
                    self.show_selected_pipeline_charts = !self.show_selected_pipeline_charts;
                }
            }
            'p' => {
                self.tabs.select(Self::PIPELINE_LIST);
            }
            'n' => {
                self.tabs.select(Self::PIPELINE_VIEWER_LIST);
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        self.state.node_info = self.data_fetcher.fetch_info();
        self.state.node_stats = self.data_fetcher.fetch_stats();

        if let Some(node_stats) = self.state.node_stats.clone() {
            self.state.chart_jvm_heap_state.push(JvmMemHeapDataPoint {
                timestamp: Local::now().timestamp_millis(),
                heap_committed_in_bytes: node_stats.jvm.mem.heap_committed_in_bytes,
                heap_max_in_bytes: node_stats.jvm.mem.heap_max_in_bytes,
                heap_used_in_bytes: node_stats.jvm.mem.heap_used_in_bytes,
                heap_used_percent: node_stats.jvm.mem.heap_used_percent,
            });

            self.state
                .chart_jvm_non_heap_state
                .push(JvmMemNonHeapDataPoint {
                    timestamp: Local::now().timestamp_millis(),
                    non_heap_committed_in_bytes: node_stats.jvm.mem.non_heap_committed_in_bytes,
                    non_heap_used_in_bytes: node_stats.jvm.mem.non_heap_used_in_bytes,
                });

            self.state.chart_process_cpu.push(ProcessCpuDataPoint {
                timestamp: Local::now().timestamp_millis(),
                percent: node_stats.process.cpu.percent,
            });

            self.state
                .chart_flow_plugins_throughput
                .push(PluginFlowMetricDataPoint {
                    timestamp: Local::now().timestamp_millis(),
                    input: node_stats.flow.input_throughput.current,
                    filter: node_stats.flow.filter_throughput.current,
                    output: node_stats.flow.output_throughput.current,
                });

            self.state
                .chart_flow_queue_backpressure
                .push(FlowMetricDataPoint {
                    timestamp: Local::now().timestamp_millis(),
                    value: node_stats.flow.queue_backpressure.current,
                });

            for (name, stats) in &node_stats.pipelines {
                if !self
                    .state
                    .chart_flow_pipeline_plugins_throughput
                    .contains_key(name)
                {
                    self.state
                        .chart_flow_pipeline_plugins_throughput
                        .insert(name.to_string(), TimestampChartState::new(Some(60)));
                }

                self.state
                    .chart_flow_pipeline_plugins_throughput
                    .get_mut(name)
                    .unwrap()
                    .push(PluginFlowMetricDataPoint {
                        timestamp: Local::now().timestamp_millis(),
                        input: stats.flow.input_throughput.current,
                        filter: stats.flow.filter_throughput.current,
                        output: stats.flow.output_throughput.current,
                    });

                Self::add_to_pipeline_flow_state(
                    name,
                    stats.flow.queue_backpressure.current,
                    &mut self.state.chart_flow_pipeline_queue_backpressure,
                );
            }

            self.update_current_vertex_id_sampling();
        }

        self.pipelines.update(&self.state);
        self.selected_pipeline_graph
            .update(&self.state, self.pipelines.selected_item());
    }

    fn update_current_vertex_id_sampling(&mut self) {
        if self.state.chart_pipeline_vertex_id.is_none() {
            if !self.state.chart_pipeline_vertex_id_state.is_empty() {
                self.state.chart_pipeline_vertex_id_state.reset();
            }

            return;
        }

        if self.state.node_stats.is_none() {
            return;
        }

        if self.pipelines.selected_item().is_none() || self.state.chart_pipeline_vertex_id.is_none() {
            return;
        }

        let node_stats = self.state.node_stats.as_ref().unwrap();
        let vertex_id = self.state.chart_pipeline_vertex_id.as_ref().unwrap();
        let selected_pipeline = self.pipelines.selected_item().unwrap();

        if let Some(pipeline_stats) = node_stats.pipelines.get(&selected_pipeline.name) {
            if let Some(vertex_stats) = pipeline_stats.plugins.get(&vertex_id) {
                let flow = &vertex_stats.flow;
                if flow.is_none() {
                    return;
                }

                let plugin_flow = flow.as_ref().unwrap();
                if let Some(throughput) = &plugin_flow.throughput {
                    if self
                        .state
                        .chart_pipeline_vertex_id_state
                        .throughput
                        .is_none()
                    {
                        self.state.chart_pipeline_vertex_id_state.throughput =
                            Some(TimestampChartState::new(DEFAULT_MAX_DATA_POINTS));
                    }

                    self.state
                        .chart_pipeline_vertex_id_state
                        .throughput
                        .as_mut()
                        .unwrap()
                        .push(FlowMetricDataPoint {
                            timestamp: Local::now().timestamp_millis(),
                            value: throughput.current,
                        });
                }

                if let Some(worker_utilization) = &plugin_flow.worker_utilization {
                    if self
                        .state
                        .chart_pipeline_vertex_id_state
                        .worker_utilization
                        .is_none()
                    {
                        self.state.chart_pipeline_vertex_id_state.worker_utilization =
                            Some(TimestampChartState::new(DEFAULT_MAX_DATA_POINTS));
                    }

                    self.state
                        .chart_pipeline_vertex_id_state
                        .worker_utilization
                        .as_mut()
                        .unwrap()
                        .push(FlowMetricDataPoint {
                            timestamp: Local::now().timestamp_millis(),
                            value: worker_utilization.current,
                        });
                }
            }
        }
    }

    fn add_to_pipeline_flow_state(
        name: &str,
        value: f64,
        chart_state: &mut HashMap<String, TimestampChartState<FlowMetricDataPoint>>,
    ) {
        if !chart_state.contains_key(name) {
            chart_state.insert(name.to_string(), TimestampChartState::new(Some(10)));
        }

        chart_state
            .get_mut(name)
            .unwrap()
            .push(FlowMetricDataPoint {
                timestamp: Local::now().timestamp_millis(),
                value,
            });
    }
}
