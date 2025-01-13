use crate::api::node::{PipelineInfo, Vertex};
use crate::api::stats::FlowMetricValue;
use crate::commands::tui::app::AppData;
use crate::commands::tui::events::EventsListener;
use crate::commands::tui::widgets::{StatefulTable, TabsState};
use crossterm::event::{KeyCode, KeyEvent};
use std::cmp::Ordering;
use std::collections::HashMap;

const PIPELINES_LIST: usize = 0;
const PIPELINE_INPUTS_LIST: usize = 1;
const PIPELINE_PLUGINS_LIST: usize = 2;

pub(crate) struct PluginFlowTableInputItem {
    pub(crate) id: String,
    pub(crate) throughput: Option<FlowMetricValue>,
}

pub(crate) struct PluginFlowTableItem {
    pub(crate) id: String,
    pub(crate) plugin_type: String,
    pub(crate) worker_utilization: Option<FlowMetricValue>,
    pub(crate) worker_millis_per_event: Option<FlowMetricValue>,
}

pub(crate) struct PipelineFlowTableItem {
    pub _id: String,
    pub name: String,
    pub workers: i64,
    pub vertices: HashMap<String, Vertex>,
    pub input_throughput: Option<FlowMetricValue>,
    pub filter_throughput: Option<FlowMetricValue>,
    pub output_throughput: Option<FlowMetricValue>,
    pub queue_backpressure: Option<FlowMetricValue>,
    pub worker_concurrency: Option<FlowMetricValue>,
}

impl StatefulTable<PluginFlowTableInputItem> {
    fn update(&mut self, selected_pipeline: &String, app_data: &AppData) {
        if let Some(node_stats) = app_data.node_stats() {
            if let Some(pipeline_stats) = node_stats.pipelines.get(selected_pipeline) {
                let mut new_items = Vec::with_capacity(self.items.len());
                for (id, plugin) in &pipeline_stats.plugins.inputs {
                    new_items.push(PluginFlowTableInputItem {
                        id: id.to_string(),
                        throughput: plugin.flow.as_ref().and_then(|p| p.throughput.clone()),
                    });
                }

                new_items.sort_by(|a, b| {
                    let a_value = a.throughput.as_ref().map(|p| p.current).unwrap_or(0.0);
                    let b_value = b.throughput.as_ref().map(|p| p.current).unwrap_or(0.0);
                    a_value.total_cmp(&b_value).reverse()
                });

                self.items = new_items;
            }
        }
    }
}

impl StatefulTable<PluginFlowTableItem> {
    fn update(&mut self, selected_pipeline: &String, app_data: &AppData) {
        if let Some(node_stats) = app_data.node_stats() {
            if let Some(pipeline_stats) = node_stats.pipelines.get(selected_pipeline) {
                let mut new_items = Vec::new();
                for (id, (plugin_type, plugin)) in pipeline_stats.plugins.all_with_type() {
                    if plugin_type == "input" {
                        continue;
                    }

                    if let Some(plugin_flow) = &plugin.flow {
                        new_items.push(PluginFlowTableItem {
                            id: id.to_string(),
                            plugin_type,
                            worker_utilization: plugin_flow.worker_utilization.clone(),
                            worker_millis_per_event: plugin_flow.worker_millis_per_event.clone(),
                        });
                    }
                }

                new_items.sort_by(|a, b| {
                    let a_value = a
                        .worker_utilization
                        .as_ref()
                        .map(|p| p.current)
                        .unwrap_or(0.0);

                    let b_value = b
                        .worker_utilization
                        .as_ref()
                        .map(|p| p.current)
                        .unwrap_or(0.0);

                    let ordering = approx_cmp(a_value, b_value, 4);
                    if ordering.is_eq() {
                        a.id.cmp(&b.id)
                    } else {
                        ordering.reverse()
                    }
                });

                self.items = new_items;
            }
        }
    }
}

impl StatefulTable<PipelineFlowTableItem> {
    fn create_pipeline_vertices_map(
        &self,
        pipeline_info: &PipelineInfo,
    ) -> HashMap<String, Vertex> {
        let mut map = HashMap::new();
        for vertex in &pipeline_info.graph.graph.vertices {
            map.insert(vertex.id.to_string(), vertex.clone());
        }
        map
    }

    fn update(&mut self, data: &AppData) {
        if let Some(node_info) = &data.node_info() {
            if let Some(node_stats) = &data.node_stats() {
                if let Some(pipelines) = &node_info.pipelines {
                    let mut new_items = Vec::with_capacity(pipelines.len());
                    for (name, pipeline_info) in pipelines {
                        let mut new_item = PipelineFlowTableItem {
                            _id: pipeline_info.ephemeral_id.to_string(),
                            name: name.to_string(),
                            workers: pipeline_info.workers,
                            vertices: self.create_pipeline_vertices_map(pipeline_info),
                            input_throughput: None,
                            filter_throughput: None,
                            output_throughput: None,
                            queue_backpressure: None,
                            worker_concurrency: None,
                        };

                        if let Some(stats) = node_stats.pipelines.get(name) {
                            let flow = &stats.flow;
                            new_item.input_throughput = Some(flow.input_throughput.clone());
                            new_item.filter_throughput = Some(flow.filter_throughput.clone());
                            new_item.output_throughput = Some(flow.output_throughput.clone());
                            new_item.queue_backpressure = Some(flow.queue_backpressure.clone());
                            new_item.worker_concurrency = Some(flow.worker_concurrency.clone());
                        }

                        new_items.push(new_item);
                    }

                    new_items.sort_by(|a, b| {
                        let a_value = a
                            .worker_concurrency
                            .as_ref()
                            .map(|p| p.current)
                            .unwrap_or(f64::MIN);

                        let b_value = b
                            .worker_concurrency
                            .as_ref()
                            .map(|p| p.current)
                            .unwrap_or(f64::MIN);

                        let ordering = approx_cmp(a_value, b_value, 4);
                        if ordering.is_eq() {
                            a.name.cmp(&b.name)
                        } else {
                            ordering.reverse()
                        }
                    });

                    if let Some(selected_pipeline) = self.selected_item().map(|p| &p.name) {
                        if let Some(new_index) =
                            new_items.iter().position(|p| p.name == *selected_pipeline)
                        {
                            self.state.select(Some(new_index));
                        }
                    }

                    self.items = new_items;
                }
            }
        }
    }
}

fn approx_cmp(a: f64, b: f64, decimal_places: u8) -> Ordering {
    let factor = 10.0f64.powi(decimal_places as i32);
    let a = (a * factor).trunc();
    let b = (b * factor).trunc();

    if a > b {
        Ordering::Greater
    } else if a < b {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}

pub(crate) struct FlowsState {
    pub(crate) pipelines_flow_table: StatefulTable<PipelineFlowTableItem>,
    pub(crate) other_plugins_flow_table: StatefulTable<PluginFlowTableItem>,
    pub(crate) input_plugins_flow_table: StatefulTable<PluginFlowTableInputItem>,
    pub(crate) analysis_window_tabs: TabsState,
    pub(crate) show_as_percentage: bool,
    pub(crate) show_lifetime_values: bool,
    pub(crate) show_selected_pipeline: bool,
    pub(crate) current_focus: usize,
}

impl FlowsState {
    pub(crate) fn new() -> Self {
        FlowsState {
            pipelines_flow_table: StatefulTable::new(),
            input_plugins_flow_table: StatefulTable::new(),
            other_plugins_flow_table: StatefulTable::new(),
            analysis_window_tabs: TabsState::with_default_index(1),
            show_as_percentage: false,
            show_lifetime_values: false,
            show_selected_pipeline: false,
            current_focus: PIPELINES_LIST,
        }
    }
    fn update_selected_pipeline_tables(&mut self, app_data: &AppData) {
        if self.show_selected_pipeline {
            if let Some(selected_pipeline) = self.pipelines_flow_table.selected_item() {
                self.input_plugins_flow_table
                    .update(&selected_pipeline.name, app_data);
                self.other_plugins_flow_table
                    .update(&selected_pipeline.name, app_data);
            }
        }
    }
}

impl EventsListener for FlowsState {
    fn update(&mut self, app_data: &AppData) {
        self.pipelines_flow_table.update(app_data);
        self.update_selected_pipeline_tables(app_data);
    }

    fn reset(&mut self) {
        self.pipelines_flow_table.unselect();
        self.pipelines_flow_table.items.clear();
        self.input_plugins_flow_table.unselect();
        self.input_plugins_flow_table.items.clear();
        self.other_plugins_flow_table.unselect();
        self.other_plugins_flow_table.items.clear();
    }

    fn focus_gained(&mut self, _: &AppData) {
        if self.pipelines_flow_table.selected_item().is_none() {
            self.pipelines_flow_table.next();
        }
    }

    fn on_enter(&mut self, app_data: &AppData) {
        if self.current_focus == PIPELINES_LIST {
            self.show_selected_pipeline = !self.show_selected_pipeline;
            self.update_selected_pipeline_tables(app_data);
        }
    }

    fn on_left(&mut self, _: &AppData) {
        if self.current_focus != PIPELINES_LIST {
            self.current_focus = PIPELINES_LIST;
            self.input_plugins_flow_table.unselect();
            self.other_plugins_flow_table.unselect();
        }
    }

    fn on_right(&mut self, _: &AppData) {
        if self.current_focus == PIPELINES_LIST
            && self.show_selected_pipeline
            && !self.input_plugins_flow_table.items.is_empty()
        {
            self.current_focus = PIPELINE_INPUTS_LIST;
            self.input_plugins_flow_table.next();
        }
    }

    fn on_up(&mut self, app_data: &AppData) {
        if self.current_focus == PIPELINES_LIST {
            self.pipelines_flow_table.previous();
            self.update_selected_pipeline_tables(app_data);
        } else if self.current_focus == PIPELINE_INPUTS_LIST {
            if self.input_plugins_flow_table.has_previous() {
                self.input_plugins_flow_table.previous();
            } else {
                self.current_focus = PIPELINE_PLUGINS_LIST;
                self.input_plugins_flow_table.unselect();
                self.other_plugins_flow_table.previous();
            }
        } else if self.current_focus == PIPELINE_PLUGINS_LIST {
            if self.other_plugins_flow_table.has_previous() {
                self.other_plugins_flow_table.previous();
            } else {
                self.current_focus = PIPELINE_INPUTS_LIST;
                self.input_plugins_flow_table.previous();
                self.other_plugins_flow_table.unselect();
            }
        }
    }

    fn on_down(&mut self, app_data: &AppData) {
        if self.current_focus == PIPELINES_LIST {
            self.pipelines_flow_table.next();
            self.update_selected_pipeline_tables(app_data);
        } else if self.current_focus == PIPELINE_INPUTS_LIST {
            if self.input_plugins_flow_table.has_next() {
                self.input_plugins_flow_table.next();
            } else {
                self.current_focus = PIPELINE_PLUGINS_LIST;
                self.input_plugins_flow_table.unselect();
                self.other_plugins_flow_table.next();
            }
        } else if self.current_focus == PIPELINE_PLUGINS_LIST {
            if self.other_plugins_flow_table.has_next() {
                self.other_plugins_flow_table.next();
            } else {
                self.current_focus = PIPELINE_INPUTS_LIST;
                self.input_plugins_flow_table.next();
                self.other_plugins_flow_table.unselect();
            }
        }
    }

    fn on_other(&mut self, key_event: KeyEvent, _: &AppData) {
        // Tab navigation
        if key_event.code == KeyCode::Tab && self.show_selected_pipeline {
            if self.current_focus == PIPELINES_LIST {
                self.current_focus = PIPELINE_INPUTS_LIST;
                self.input_plugins_flow_table.next();
            } else if self.current_focus == PIPELINE_INPUTS_LIST {
                self.current_focus = PIPELINE_PLUGINS_LIST;
                self.input_plugins_flow_table.unselect();
                self.other_plugins_flow_table.next();
            } else {
                self.current_focus = PIPELINES_LIST;
                self.other_plugins_flow_table.unselect();
            }
            return;
        }

        if let KeyCode::Char(c) = key_event.code {
            if c.is_numeric() {
                if let Some(number) = c.to_digit(10) {
                    if number > 0 && number < 7 {
                        self.analysis_window_tabs.select(number as usize);
                    }
                }
            }

            match c.to_ascii_lowercase() {
                'v' => {
                    self.show_as_percentage = !self.show_as_percentage;
                }
                'l' => {
                    self.show_lifetime_values = !self.show_lifetime_values;
                }
                _ => {}
            }
        };
    }
}
