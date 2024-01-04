use crate::api::stats::PipelineStats;
use crate::commands::tui::app::AppData;
use crate::commands::tui::charts::{TimestampChartState, DEFAULT_MAX_DATA_POINTS};
use crate::commands::tui::events::EventsListener;
use crate::commands::tui::flow_charts::{FlowMetricDataPoint, PluginFlowMetricDataPoint};
use std::collections::HashMap;

pub struct PluginFlowChartState {
    pub throughput: TimestampChartState<FlowMetricDataPoint>,
    pub worker_utilization: TimestampChartState<FlowMetricDataPoint>,
    pub worker_millis_per_event: TimestampChartState<FlowMetricDataPoint>,
}

impl PluginFlowChartState {
    pub fn new() -> Self {
        PluginFlowChartState {
            throughput: Default::default(),
            worker_utilization: Default::default(),
            worker_millis_per_event: Default::default(),
        }
    }
}

pub struct PipelineFlowChartState {
    pub plugins_throughput: TimestampChartState<PluginFlowMetricDataPoint>,
    pub input_throughput: TimestampChartState<FlowMetricDataPoint>,
    pub filter_throughput: TimestampChartState<FlowMetricDataPoint>,
    pub output_throughput: TimestampChartState<FlowMetricDataPoint>,
    pub queue_backpressure: TimestampChartState<FlowMetricDataPoint>,
    pub worker_concurrency: TimestampChartState<FlowMetricDataPoint>,
    pub queue_persisted_growth_bytes: TimestampChartState<FlowMetricDataPoint>,
    pub queue_persisted_growth_events: TimestampChartState<FlowMetricDataPoint>,
}

impl PipelineFlowChartState {
    pub fn new(pipeline_stats: &PipelineStats) -> Self {
        let mut state = PipelineFlowChartState {
            plugins_throughput: Default::default(),
            input_throughput: Default::default(),
            filter_throughput: Default::default(),
            output_throughput: Default::default(),
            queue_backpressure: Default::default(),
            worker_concurrency: Default::default(),
            queue_persisted_growth_bytes: TimestampChartState::with_negative_bounds(
                DEFAULT_MAX_DATA_POINTS,
            ),
            queue_persisted_growth_events: TimestampChartState::with_negative_bounds(
                DEFAULT_MAX_DATA_POINTS,
            ),
        };

        state.input_throughput.push(FlowMetricDataPoint::new(
            pipeline_stats.flow.input_throughput.current,
        ));

        state.filter_throughput.push(FlowMetricDataPoint::new(
            pipeline_stats.flow.filter_throughput.current,
        ));

        state.output_throughput.push(FlowMetricDataPoint::new(
            pipeline_stats.flow.output_throughput.current,
        ));

        state
            .plugins_throughput
            .push(PluginFlowMetricDataPoint::new(
                pipeline_stats.flow.input_throughput.current,
                pipeline_stats.flow.filter_throughput.current,
                pipeline_stats.flow.output_throughput.current,
            ));

        state.queue_backpressure.push(FlowMetricDataPoint::new(
            pipeline_stats.flow.queue_backpressure.current,
        ));

        state.worker_concurrency.push(FlowMetricDataPoint::new(
            pipeline_stats.flow.worker_concurrency.current,
        ));

        state
            .queue_persisted_growth_bytes
            .push(FlowMetricDataPoint::new(
                pipeline_stats.flow.queue_persisted_growth_bytes.current,
            ));

        state
            .queue_persisted_growth_events
            .push(FlowMetricDataPoint::new(
                pipeline_stats.flow.queue_persisted_growth_events.current,
            ));

        state
    }
}

pub struct PipelineChartState {
    pub pipeline: PipelineFlowChartState,
    pub plugins: HashMap<String, PluginFlowChartState>,
}

impl PipelineChartState {
    pub fn new(pipeline_stats: &PipelineStats) -> Self {
        let mut plugins_states: HashMap<String, PluginFlowChartState> =
            HashMap::with_capacity(pipeline_stats.vertices.len());

        for (name, plugin) in pipeline_stats.plugins.all() {
            if !plugins_states.contains_key(&name) {
                plugins_states.insert(name.to_string(), PluginFlowChartState::new());
            }

            if let Some(plugin_flow) = &plugin.flow {
                let state = plugins_states.get_mut(&name).unwrap();
                if let Some(metric) = &plugin_flow.throughput {
                    state
                        .throughput
                        .push(FlowMetricDataPoint::new(metric.current));
                }

                if let Some(metric) = &plugin_flow.worker_utilization {
                    state
                        .worker_utilization
                        .push(FlowMetricDataPoint::new(metric.current));
                }

                if let Some(metric) = &plugin_flow.worker_millis_per_event {
                    state
                        .worker_millis_per_event
                        .push(FlowMetricDataPoint::new(metric.current));
                }
            }
        }

        PipelineChartState {
            pipeline: PipelineFlowChartState::new(pipeline_stats),
            plugins: plugins_states,
        }
    }
}

pub struct SharedState {
    pipelines_flows_chart_state: HashMap<String, PipelineChartState>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            pipelines_flows_chart_state: Default::default(),
        }
    }

    pub(crate) fn pipeline_flows_chart_state(
        &self,
        pipeline: &String,
    ) -> Option<&PipelineChartState> {
        self.pipelines_flows_chart_state.get(pipeline)
    }

    pub(crate) fn pipeline_plugin_flows_chart_state(
        &self,
        pipeline: &String,
        plugin: &String,
    ) -> Option<&PluginFlowChartState> {
        if let Some(state) = self.pipeline_flows_chart_state(pipeline) {
            return state.plugins.get(plugin);
        }

        None
    }

    fn update_chart_flows_states(&mut self, app_data: &AppData) {
        if app_data.node_stats().is_none() {
            self.pipelines_flows_chart_state.clear();
            return;
        }

        let node_stats = app_data.node_stats().unwrap();
        for (pipeline_name, pipeline_stats) in &node_stats.pipelines {
            if !self.pipelines_flows_chart_state.contains_key(pipeline_name) {
                self.pipelines_flows_chart_state.insert(
                    pipeline_name.to_string(),
                    PipelineChartState::new(pipeline_stats),
                );
            }

            let pipeline_chart_state = self
                .pipelines_flows_chart_state
                .get_mut(pipeline_name)
                .unwrap();
            pipeline_chart_state
                .pipeline
                .input_throughput
                .push(FlowMetricDataPoint::new(
                    pipeline_stats.flow.input_throughput.current,
                ));

            pipeline_chart_state
                .pipeline
                .filter_throughput
                .push(FlowMetricDataPoint::new(
                    pipeline_stats.flow.filter_throughput.current,
                ));

            pipeline_chart_state
                .pipeline
                .output_throughput
                .push(FlowMetricDataPoint::new(
                    pipeline_stats.flow.output_throughput.current,
                ));

            pipeline_chart_state
                .pipeline
                .plugins_throughput
                .push(PluginFlowMetricDataPoint::new(
                    pipeline_stats.flow.input_throughput.current,
                    pipeline_stats.flow.filter_throughput.current,
                    pipeline_stats.flow.output_throughput.current,
                ));

            pipeline_chart_state
                .pipeline
                .worker_concurrency
                .push(FlowMetricDataPoint::new(
                    pipeline_stats.flow.worker_concurrency.current,
                ));

            pipeline_chart_state
                .pipeline
                .queue_backpressure
                .push(FlowMetricDataPoint::new(
                    pipeline_stats.flow.queue_backpressure.current,
                ));

            pipeline_chart_state
                .pipeline
                .queue_persisted_growth_bytes
                .push(FlowMetricDataPoint::new(
                    pipeline_stats.flow.queue_persisted_growth_bytes.current,
                ));

            pipeline_chart_state
                .pipeline
                .queue_persisted_growth_events
                .push(FlowMetricDataPoint::new(
                    pipeline_stats.flow.queue_persisted_growth_events.current,
                ));

            for (plugin_name, plugin) in pipeline_stats.plugins.all() {
                if !pipeline_chart_state.plugins.contains_key(&plugin_name) {
                    pipeline_chart_state
                        .plugins
                        .insert(plugin_name.to_string(), PluginFlowChartState::new());
                }

                if let Some(plugin_flow) = &plugin.flow {
                    let plugin_state = pipeline_chart_state.plugins.get_mut(&plugin_name).unwrap();
                    if let Some(metric) = &plugin_flow.throughput {
                        plugin_state
                            .throughput
                            .push(FlowMetricDataPoint::new(metric.current));
                    }
                    if let Some(metric) = &plugin_flow.worker_utilization {
                        plugin_state
                            .worker_utilization
                            .push(FlowMetricDataPoint::new(metric.current));
                    }
                    if let Some(metric) = &plugin_flow.worker_millis_per_event {
                        plugin_state
                            .worker_millis_per_event
                            .push(FlowMetricDataPoint::new(metric.current));
                    }
                }
            }
        }
    }
}

impl EventsListener for SharedState {
    fn update(&mut self, app_data: &AppData) {
        self.update_chart_flows_states(app_data);
    }

    fn reset(&mut self) {
        self.pipelines_flows_chart_state.clear();
    }
}
