use crossterm::event::{KeyCode, KeyEvent};
use std::marker::PhantomData;

use crate::api::node::{GraphDefinition, NodeInfo};
use crate::commands::tui::app::AppData;
use crate::commands::tui::events::EventsListener;
use crate::commands::tui::pipelines::graph::PipelineGraph;
use crate::commands::tui::widgets::StatefulTable;

pub const PIPELINE_VERTEX_LIST: usize = 0;
pub const PIPELINE_VERTEX_VIEW: usize = 1;

pub struct PipelineTableItem {
    pub name: String,
    pub graph: GraphDefinition,
}

impl StatefulTable<PipelineTableItem> {
    fn update(&mut self, data: &AppData) {
        if let Some(node_info) = &data.node_info() {
            if let Some(pipelines) = &node_info.pipelines {
                let mut new_items = Vec::with_capacity(pipelines.len());
                for (name, pipeline_info) in pipelines {
                    let new_item = PipelineTableItem {
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

type SelectedPipelineVertexTableItem = String;

impl StatefulTable<SelectedPipelineVertexTableItem> {
    fn update(
        &mut self,
        node_info: &Option<&NodeInfo>,
        selected_pipeline: &Option<&PipelineTableItem>,
    ) {
        if selected_pipeline.is_none() || node_info.is_none() {
            self.items = vec![];
            self.unselect();
            return;
        }

        self.items = PipelineGraph::from(&selected_pipeline.unwrap().graph)
            .create_pipeline_vertex_ids(selected_pipeline.unwrap());
    }
}

pub struct PipelinesState<'a> {
    pub current_focus: usize,
    pub pipelines_table: StatefulTable<PipelineTableItem>,
    pub selected_pipeline_vertex: StatefulTable<SelectedPipelineVertexTableItem>,
    pub show_selected_pipeline_charts: bool,
    pub show_selected_vertex_details: bool,
    _marker: PhantomData<&'a ()>,
}

impl PipelinesState<'_> {
    pub fn new() -> Self {
        PipelinesState {
            current_focus: 0,
            pipelines_table: StatefulTable::new(),
            selected_pipeline_vertex: StatefulTable::new(),
            show_selected_pipeline_charts: false,
            show_selected_vertex_details: false,
            _marker: PhantomData,
        }
    }

    pub fn selected_pipeline_name(&self) -> Option<&String> {
        self.pipelines_table.selected_item().map(|p| &p.name)
    }

    pub fn selected_pipeline_vertex(&self) -> Option<&String> {
        self.selected_pipeline_vertex.selected_item()
    }
}

impl EventsListener for PipelinesState<'_> {
    fn update(&mut self, app_data: &AppData) {
        self.pipelines_table.update(app_data);

        let selected_pipeline_item = if self.pipelines_table.selected_item().is_none()
            && !self.pipelines_table.items.is_empty()
        {
            self.pipelines_table.next()
        } else {
            self.pipelines_table.selected_item()
        };

        self.selected_pipeline_vertex
            .update(&app_data.node_info(), &selected_pipeline_item);
    }

    fn reset(&mut self) {
        // UI
        self.current_focus = PIPELINE_VERTEX_LIST;
        self.show_selected_pipeline_charts = false;
        self.show_selected_vertex_details = false;

        self.pipelines_table = StatefulTable::new();
        self.selected_pipeline_vertex = StatefulTable::new();
    }

    fn on_enter(&mut self, _app_data: &AppData) {
        if self.current_focus == PIPELINE_VERTEX_LIST {
            if self.pipelines_table.selected_item().is_some() {
                self.show_selected_vertex_details = false;
                self.show_selected_pipeline_charts = !self.show_selected_pipeline_charts;
            }
        } else if self.current_focus == PIPELINE_VERTEX_VIEW {
            self.show_selected_pipeline_charts = false;
            self.show_selected_vertex_details = !self.show_selected_vertex_details;
        }
    }

    fn on_left(&mut self, _: &AppData) {
        if self.current_focus == PIPELINE_VERTEX_VIEW {
            self.current_focus = PIPELINE_VERTEX_LIST;
            self.selected_pipeline_vertex.unselect();
            self.show_selected_vertex_details = false;
        }
    }

    fn on_right(&mut self, _: &AppData) {
        if self.current_focus == PIPELINE_VERTEX_LIST
            && !self.selected_pipeline_vertex.items.is_empty()
        {
            self.current_focus = PIPELINE_VERTEX_VIEW;
            self.selected_pipeline_vertex.next();
        }
    }

    fn on_up(&mut self, app_data: &AppData) {
        if self.current_focus == PIPELINE_VERTEX_LIST {
            self.selected_pipeline_vertex
                .update(&app_data.node_info(), &self.pipelines_table.previous());
        } else {
            self.selected_pipeline_vertex.previous();
        }
    }

    fn on_down(&mut self, app_data: &AppData) {
        if self.current_focus == PIPELINE_VERTEX_LIST {
            self.selected_pipeline_vertex
                .update(&app_data.node_info(), &self.pipelines_table.next());
        } else {
            self.selected_pipeline_vertex.next();
        }
    }

    fn on_other(&mut self, key_event: KeyEvent, app_data: &AppData) {
        // Tab navigation
        if key_event.code == KeyCode::Tab {
            if self.current_focus == PIPELINE_VERTEX_LIST {
                self.on_right(app_data);
            } else {
                self.on_left(app_data);
            }

            return;
        }

        if let KeyCode::Char(c) = key_event.code {
            if c.eq_ignore_ascii_case(&'c') && self.pipelines_table.selected_item().is_some() {
                self.show_selected_vertex_details = false;
                self.show_selected_pipeline_charts = !self.show_selected_pipeline_charts;
            }
        };
    }
}
