use std::collections::HashMap;
use std::time::Duration;
use std::{f64, i64};

use crate::api::node::{GraphDefinition, NodeInfo, NodeInfoType};
use crate::api::stats::NodeStats;
use crate::api::Client;
use crate::commands::view::charts::{ChartDataPoint, TimestampChartState, DEFAULT_MAX_DATA_POINTS};
use crate::commands::view::now_local_unix_timestamp;
use crate::commands::view::pipeline_graph::PipelineGraph;
use crate::commands::view::widgets::{StatefulTable, TabsState};
use crate::errors::AnyError;

struct DataFetcher<'a> {
    api: &'a Client<'a>,
}

impl<'a> DataFetcher<'a> {
    pub fn new(api: &'a Client) -> DataFetcher<'a> {
        DataFetcher { api }
    }

    pub fn fetch_info(&self) -> Result<NodeInfo, AnyError> {
        self.api.get_node_info(
            &[NodeInfoType::Pipelines],
            Some(Client::QUERY_NODE_INFO_GRAPH),
        )
    }

    pub fn fetch_stats(&self) -> Result<NodeStats, AnyError> {
        self.api
            .get_node_stats(Some(Client::QUERY_NODE_STATS_VERTICES))
    }
}

impl StatefulTable<String> {
    fn update(&mut self, app_state: &AppState, selected_pipeline: Option<&PipelineItem>) {
        if selected_pipeline.is_none() || app_state.node_info.is_none() {
            self.items = vec![];
            self.unselect();
            return;
        }

        self.items = PipelineGraph::from(&selected_pipeline.unwrap().graph)
            .create_pipeline_vertex_ids(selected_pipeline.unwrap());
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

impl PluginFlowMetricDataPoint {
    pub fn new(input: f64, filter: f64, output: f64) -> Self {
        PluginFlowMetricDataPoint {
            timestamp: now_local_unix_timestamp(),
            input,
            filter,
            output,
        }
    }
}

impl ChartDataPoint for PluginFlowMetricDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        [
            f64::min(f64::min(self.input, self.filter), self.output),
            f64::max(f64::max(self.input, self.filter), self.output),
        ]
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        [self.timestamp as f64, self.timestamp as f64]
    }
}

pub struct FlowMetricDataPoint {
    pub timestamp: i64,
    pub value: f64,
}

impl FlowMetricDataPoint {
    pub fn new(value: f64) -> Self {
        FlowMetricDataPoint {
            timestamp: now_local_unix_timestamp(),
            value,
        }
    }
}

impl ChartDataPoint for FlowMetricDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        [self.value, self.value]
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        [self.timestamp as f64, self.timestamp as f64]
    }
}

pub struct ProcessCpuDataPoint {
    pub timestamp: i64,
    pub percent: i64,
}

impl ProcessCpuDataPoint {
    pub fn new(percent: i64) -> Self {
        ProcessCpuDataPoint {
            timestamp: now_local_unix_timestamp(),
            percent,
        }
    }
}

impl ChartDataPoint for ProcessCpuDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        [self.percent as f64, self.percent as f64]
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        [self.timestamp as f64, self.timestamp as f64]
    }
}

pub struct JvmMemNonHeapDataPoint {
    pub timestamp: i64,
    pub non_heap_committed_in_bytes: i64,
    pub non_heap_used_in_bytes: i64,
}

impl ChartDataPoint for JvmMemNonHeapDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        [
            f64::min(
                self.non_heap_used_in_bytes as f64,
                self.non_heap_committed_in_bytes as f64,
            ),
            f64::max(
                self.non_heap_used_in_bytes as f64,
                self.non_heap_committed_in_bytes as f64,
            ),
        ]
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        [self.timestamp as f64, self.timestamp as f64]
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
        [
            f64::min(
                self.heap_max_in_bytes as f64,
                self.heap_used_in_bytes as f64,
            ),
            f64::max(
                self.heap_max_in_bytes as f64,
                self.heap_used_in_bytes as f64,
            ),
        ]
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        [self.timestamp as f64, self.timestamp as f64]
    }
}

pub struct SelectedPipelineVertexChartState {
    pub throughput: Option<TimestampChartState<FlowMetricDataPoint>>,
    pub worker_utilization: Option<TimestampChartState<FlowMetricDataPoint>>,
    pub worker_millis_per_event: Option<TimestampChartState<FlowMetricDataPoint>>,
}

impl SelectedPipelineVertexChartState {
    pub fn new() -> Self {
        SelectedPipelineVertexChartState {
            throughput: None,
            worker_utilization: None,
            worker_millis_per_event: None,
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

        if let Some(worker_millis_per_event) = &self.worker_millis_per_event {
            if !worker_millis_per_event.data_points.is_empty() {
                return false;
            }
        }

        true
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
    pub chart_flow_pipeline_queue_persisted_growth_bytes:
        HashMap<String, TimestampChartState<FlowMetricDataPoint>>,
    pub chart_flow_pipeline_queue_persisted_growth_events:
        HashMap<String, TimestampChartState<FlowMetricDataPoint>>,
}

impl AppState {
    fn reset(&mut self) {
        self.node_stats = None;
        self.chart_pipeline_vertex_id = None;
        self.chart_pipeline_vertex_id_state = SelectedPipelineVertexChartState::new();
        self.chart_jvm_heap_state.reset();
        self.chart_jvm_non_heap_state.reset();
        self.chart_process_cpu.reset();
        self.chart_flow_plugins_throughput.reset();
        self.chart_flow_queue_backpressure.reset();
        self.chart_flow_pipeline_queue_persisted_growth_bytes
            .clear();
        self.chart_flow_pipeline_queue_persisted_growth_events
            .clear();
        self.chart_flow_pipeline_plugins_throughput.clear();
        self.chart_flow_pipeline_queue_backpressure.clear();
    }
}

pub(crate) struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub connected: bool,
    pub show_help: bool,
    pub show_error: Option<String>,
    pub tabs: TabsState,
    pub pipelines: StatefulTable<PipelineItem>,
    pub selected_pipeline_vertex: StatefulTable<String>,
    pub state: AppState,
    pub show_selected_pipeline_charts: bool,
    pub show_selected_vertex_details: bool,
    pub focused: usize,
    pub host: &'a str,
    pub refresh_interval: Duration,
    data_fetcher: DataFetcher<'a>,
}

impl<'a> App<'a> {
    pub const TAB_PIPELINES: usize = 0;
    pub const TAB_FLOW: usize = 1;
    pub const TAB_NODE: usize = 2;
    const TAB_PIPELINES_CMP_LIST: usize = 0;
    const TAB_PIPELINES_CMP_VIEW: usize = 1;

    pub fn new(title: &'a str, api: &'a Client, refresh_interval: Duration) -> App<'a> {
        let app_state = AppState {
            node_info: None,
            node_stats: None,
            chart_pipeline_vertex_id: None,
            chart_pipeline_vertex_id_state: SelectedPipelineVertexChartState::new(),
            chart_jvm_heap_state: TimestampChartState::new(DEFAULT_MAX_DATA_POINTS),
            chart_jvm_non_heap_state: TimestampChartState::new(DEFAULT_MAX_DATA_POINTS),
            chart_process_cpu: TimestampChartState::new(DEFAULT_MAX_DATA_POINTS),
            chart_flow_plugins_throughput: TimestampChartState::new(DEFAULT_MAX_DATA_POINTS),
            chart_flow_queue_backpressure: TimestampChartState::new(DEFAULT_MAX_DATA_POINTS),
            chart_flow_pipeline_queue_persisted_growth_bytes: HashMap::new(),
            chart_flow_pipeline_queue_persisted_growth_events: HashMap::new(),
            chart_flow_pipeline_plugins_throughput: HashMap::new(),
            chart_flow_pipeline_queue_backpressure: HashMap::new(),
        };

        App {
            title,
            refresh_interval,
            show_selected_pipeline_charts: false,
            show_selected_vertex_details: false,
            show_help: false,
            show_error: None,
            should_quit: false,
            connected: false,
            tabs: TabsState::new(),
            pipelines: StatefulTable::new(),
            selected_pipeline_vertex: StatefulTable::new(),
            data_fetcher: DataFetcher::new(api),
            state: app_state,
            focused: 0,
            host: api.base_url(),
        }
    }

    fn reset(&mut self) {
        self.focused = Self::TAB_PIPELINES_CMP_LIST;
        self.show_selected_pipeline_charts = false;
        self.show_selected_vertex_details = false;
        self.pipelines = StatefulTable::new();
        self.selected_pipeline_vertex = StatefulTable::new();
        self.state.reset();
    }

    pub fn on_up(&mut self) {
        if self.tabs.index == Self::TAB_PIPELINES {
            if self.focused == Self::TAB_PIPELINES_CMP_LIST {
                self.selected_pipeline_vertex
                    .update(&self.state, self.pipelines.previous());
            } else {
                self.selected_pipeline_vertex.previous();
                self.update_selected_vertex_details();
            }
        }
    }

    pub fn on_down(&mut self) {
        if self.tabs.index == Self::TAB_PIPELINES {
            if self.focused == Self::TAB_PIPELINES_CMP_LIST {
                self.selected_pipeline_vertex
                    .update(&self.state, self.pipelines.next());
            } else {
                self.selected_pipeline_vertex.next();
                self.update_selected_vertex_details();
            }
        }
    }

    pub fn on_right(&mut self) {
        if self.tabs.index == Self::TAB_PIPELINES && self.focused == Self::TAB_PIPELINES_CMP_LIST {
            self.focused = Self::TAB_PIPELINES_CMP_VIEW;
            self.selected_pipeline_vertex.next();
            self.update_selected_vertex_details();
        }
    }

    pub fn on_left(&mut self) {
        if self.tabs.index == Self::TAB_PIPELINES && self.focused == Self::TAB_PIPELINES_CMP_VIEW {
            self.focused = Self::TAB_PIPELINES_CMP_LIST;
            self.selected_pipeline_vertex.unselect();
            self.show_selected_vertex_details = false;
            self.update_selected_vertex_details();
        }
    }

    pub fn on_enter(&mut self) {
        if self.tabs.index == Self::TAB_PIPELINES {
            if self.focused == Self::TAB_PIPELINES_CMP_LIST {
                if self.pipelines.selected_item().is_some() {
                    self.show_selected_vertex_details = false;
                    self.show_selected_pipeline_charts = !self.show_selected_pipeline_charts;
                }
            } else if self.focused == Self::TAB_PIPELINES_CMP_VIEW {
                self.show_selected_pipeline_charts = false;
                self.show_selected_vertex_details = !self.show_selected_vertex_details;
                self.update_selected_vertex_details();
            }
        }
    }

    fn update_selected_vertex_details(&mut self) {
        if !self.show_selected_vertex_details && self.state.chart_pipeline_vertex_id.is_some() {
            self.state.chart_pipeline_vertex_id = None;
            self.state.chart_pipeline_vertex_id_state.reset();
        } else if self.show_selected_vertex_details {
            let selected_vertex = self
                .selected_pipeline_vertex
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
        match c.to_lowercase().to_string().as_str() {
            "q" => {
                self.on_esc();
            }
            "h" => {
                self.show_help = !self.show_help;
            }
            "f" => {
                self.tabs.select(Self::TAB_FLOW);
            }
            "p" => {
                self.tabs.select(Self::TAB_PIPELINES);
            }
            "n" => {
                self.tabs.select(Self::TAB_NODE);
            }
            _ => {}
        }
    }

    pub fn on_esc(&mut self) {
        self.should_quit = true;
    }

    pub fn on_tick(&mut self) {
        match self.data_fetcher.fetch_info() {
            Ok(info) => {
                self.state.node_info = Some(info);
                self.connected = true;
                self.show_error = None;
            }
            Err(err) => {
                self.state.node_info = None;
                self.connected = false;
                self.show_error = Some(err.to_string());
            }
        };

        match self.data_fetcher.fetch_stats() {
            Ok(stats) => {
                self.state.node_stats = Some(stats);
                self.connected = true;
                self.show_error = None;
            }
            Err(err) => {
                self.state.node_stats = None;
                self.connected = false;
                self.show_error = Some(err.to_string());
            }
        };

        if !self.connected || (self.state.node_stats.is_none() || self.state.node_info.is_none()) {
            self.reset();
            return;
        }

        let node_stats = &self.state.node_stats.clone().unwrap();

        self.update_jvm_charts_states(node_stats);
        self.state
            .chart_process_cpu
            .push(ProcessCpuDataPoint::new(node_stats.process.cpu.percent));

        self.state
            .chart_flow_plugins_throughput
            .push(PluginFlowMetricDataPoint::new(
                node_stats.flow.input_throughput.current,
                node_stats.flow.filter_throughput.current,
                node_stats.flow.filter_throughput.current,
            ));

        self.state
            .chart_flow_queue_backpressure
            .push(FlowMetricDataPoint::new(
                node_stats.flow.queue_backpressure.current,
            ));

        self.update_pipeline_vertices_charts_states(node_stats);
        self.update_current_vertex_id_sampling();

        self.pipelines.update(&self.state);
        let selected_pipeline_item =
            if self.pipelines.selected_item().is_none() && !self.pipelines.items.is_empty() {
                self.pipelines.next()
            } else {
                self.pipelines.selected_item()
            };

        self.selected_pipeline_vertex
            .update(&self.state, selected_pipeline_item);
    }

    fn update_jvm_charts_states(&mut self, node_stats: &NodeStats) {
        self.state.chart_jvm_heap_state.push(JvmMemHeapDataPoint {
            timestamp: now_local_unix_timestamp(),
            heap_committed_in_bytes: node_stats.jvm.mem.heap_committed_in_bytes,
            heap_max_in_bytes: node_stats.jvm.mem.heap_max_in_bytes,
            heap_used_in_bytes: node_stats.jvm.mem.heap_used_in_bytes,
            heap_used_percent: node_stats.jvm.mem.heap_used_percent,
        });

        self.state
            .chart_jvm_non_heap_state
            .push(JvmMemNonHeapDataPoint {
                timestamp: now_local_unix_timestamp(),
                non_heap_committed_in_bytes: node_stats.jvm.mem.non_heap_committed_in_bytes,
                non_heap_used_in_bytes: node_stats.jvm.mem.non_heap_used_in_bytes,
            });
    }

    fn update_pipeline_vertices_charts_states(&mut self, node_stats: &NodeStats) {
        for (name, stats) in &node_stats.pipelines {
            Self::add_to_pipeline_flow_state(
                name,
                &mut self.state.chart_flow_pipeline_plugins_throughput,
                PluginFlowMetricDataPoint::new(
                    stats.flow.input_throughput.current,
                    stats.flow.filter_throughput.current,
                    stats.flow.output_throughput.current,
                ),
            );

            Self::add_to_pipeline_flow_state(
                name,
                &mut self.state.chart_flow_pipeline_queue_backpressure,
                FlowMetricDataPoint::new(stats.flow.queue_backpressure.current),
            );

            Self::add_to_pipeline_flow_state(
                name,
                &mut self.state.chart_flow_pipeline_queue_persisted_growth_bytes,
                FlowMetricDataPoint::new(stats.flow.queue_persisted_growth_bytes.current),
            );

            Self::add_to_pipeline_flow_state(
                name,
                &mut self.state.chart_flow_pipeline_queue_persisted_growth_events,
                FlowMetricDataPoint::new(stats.flow.queue_persisted_growth_events.current),
            );
        }
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

        if self.pipelines.selected_item().is_none() || self.state.chart_pipeline_vertex_id.is_none()
        {
            return;
        }

        let node_stats = self.state.node_stats.as_ref().unwrap();
        let vertex_id = self.state.chart_pipeline_vertex_id.as_ref().unwrap();
        let selected_pipeline = self.pipelines.selected_item().unwrap();

        if let Some(pipeline_stats) = node_stats.pipelines.get(&selected_pipeline.name) {
            if let Some(vertex_stats) = pipeline_stats.plugins.get(vertex_id) {
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
                        .push(FlowMetricDataPoint::new(throughput.current));
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
                        .push(FlowMetricDataPoint::new(worker_utilization.current));
                }

                if let Some(worker_millis_per_event) = &plugin_flow.worker_millis_per_event {
                    if self
                        .state
                        .chart_pipeline_vertex_id_state
                        .worker_millis_per_event
                        .is_none()
                    {
                        self.state
                            .chart_pipeline_vertex_id_state
                            .worker_millis_per_event =
                            Some(TimestampChartState::new(DEFAULT_MAX_DATA_POINTS));
                    }

                    self.state
                        .chart_pipeline_vertex_id_state
                        .worker_millis_per_event
                        .as_mut()
                        .unwrap()
                        .push(FlowMetricDataPoint::new(worker_millis_per_event.current));
                }
            }
        }
    }

    fn add_to_pipeline_flow_state<T>(
        pipeline_name: &str,
        chart_state: &mut HashMap<String, TimestampChartState<T>>,
        value: T,
    ) where
        T: ChartDataPoint,
    {
        if !chart_state.contains_key(pipeline_name) {
            chart_state.insert(
                pipeline_name.to_string(),
                TimestampChartState::new(DEFAULT_MAX_DATA_POINTS),
            );
        }

        chart_state.get_mut(pipeline_name).unwrap().push(value);
    }
}
