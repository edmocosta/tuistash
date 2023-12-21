use crate::api::stats::NodeStats;
use crate::commands::tui::app::AppData;
use crate::commands::tui::charts::{ChartDataPoint, TimestampChartState, DEFAULT_MAX_DATA_POINTS};
use crate::commands::tui::events::EventsListener;
use crate::commands::tui::flow_charts::{FlowMetricDataPoint, PluginFlowMetricDataPoint};
use crate::commands::tui::now_local_unix_timestamp;
use crossterm::event::KeyEvent;

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

pub struct NodeState {
    pub chart_jvm_heap_state: TimestampChartState<JvmMemHeapDataPoint>,
    pub chart_jvm_non_heap_state: TimestampChartState<JvmMemNonHeapDataPoint>,
    pub chart_process_cpu: TimestampChartState<ProcessCpuDataPoint>,
    pub chart_flow_plugins_throughput: TimestampChartState<PluginFlowMetricDataPoint>,
    pub chart_flow_queue_backpressure: TimestampChartState<FlowMetricDataPoint>,
}

impl NodeState {
    pub(crate) fn new() -> Self {
        NodeState {
            chart_jvm_heap_state: TimestampChartState::new(DEFAULT_MAX_DATA_POINTS),
            chart_jvm_non_heap_state: TimestampChartState::new(DEFAULT_MAX_DATA_POINTS),
            chart_process_cpu: TimestampChartState::new(DEFAULT_MAX_DATA_POINTS),
            chart_flow_plugins_throughput: Default::default(),
            chart_flow_queue_backpressure: Default::default(),
        }
    }

    pub(crate) fn update(&mut self, app_data: &AppData) {
        if app_data.node_stats().is_none() {
            self.reset();
            return;
        }

        self.update_chart_states(app_data.node_stats().unwrap());
    }

    fn update_chart_states(&mut self, node_stats: &NodeStats) {
        self.chart_process_cpu
            .push(ProcessCpuDataPoint::new(node_stats.process.cpu.percent));

        self.update_jvm_charts_states(node_stats);

        self.chart_flow_plugins_throughput
            .push(PluginFlowMetricDataPoint::new(
                node_stats.flow.input_throughput.current,
                node_stats.flow.filter_throughput.current,
                node_stats.flow.filter_throughput.current,
            ));

        self.chart_flow_queue_backpressure
            .push(FlowMetricDataPoint::new(
                node_stats.flow.queue_backpressure.current,
            ));
    }

    pub(crate) fn reset(&mut self) {
        self.chart_jvm_heap_state.reset();
        self.chart_jvm_non_heap_state.reset();
        self.chart_process_cpu.reset();
        self.chart_flow_plugins_throughput.reset();
        self.chart_flow_queue_backpressure.reset();
    }

    fn update_jvm_charts_states(&mut self, node_stats: &NodeStats) {
        self.chart_jvm_heap_state.push(JvmMemHeapDataPoint {
            timestamp: now_local_unix_timestamp(),
            heap_committed_in_bytes: node_stats.jvm.mem.heap_committed_in_bytes,
            heap_max_in_bytes: node_stats.jvm.mem.heap_max_in_bytes,
            heap_used_in_bytes: node_stats.jvm.mem.heap_used_in_bytes,
            heap_used_percent: node_stats.jvm.mem.heap_used_percent,
        });

        self.chart_jvm_non_heap_state.push(JvmMemNonHeapDataPoint {
            timestamp: now_local_unix_timestamp(),
            non_heap_committed_in_bytes: node_stats.jvm.mem.non_heap_committed_in_bytes,
            non_heap_used_in_bytes: node_stats.jvm.mem.non_heap_used_in_bytes,
        });
    }
}

impl EventsListener for NodeState {
    fn focus_gained(&mut self, _app_data: &AppData) {}

    fn focus_lost(&mut self, _app_data: &AppData) {}

    fn on_enter(&mut self, _app_data: &AppData) {}

    fn on_left(&mut self, _app_data: &AppData) {}

    fn on_right(&mut self, _app_data: &AppData) {}

    fn on_up(&mut self, _app_data: &AppData) {}

    fn on_down(&mut self, _app_data: &AppData) {}

    fn on_other(&mut self, _event: KeyEvent, _app_data: &AppData) {}
}
