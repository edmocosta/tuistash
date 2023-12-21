use crate::commands::tui::app::AppData;
use crate::commands::tui::events::EventsListener;
use crate::commands::tui::widgets::{StatefulTable, TabsState};
use crossterm::event::{KeyCode, KeyEvent};

pub(crate) struct PipelineFlowTableItem {
    pub id: String,
    pub name: String,
}

pub(crate) struct FlowsState {
    pub pipelines_flow_table: StatefulTable<PipelineFlowTableItem>,
    pub analysis_window_tabs: TabsState,
}

impl FlowsState {
    pub(crate) fn new() -> Self {
        FlowsState {
            pipelines_flow_table: StatefulTable::new(),
            analysis_window_tabs: TabsState::with_default_index(1),
        }
    }

    pub(crate) fn update(&mut self, _: &AppData) {}

    pub(crate) fn reset(&mut self) {}
}

impl EventsListener for FlowsState {
    fn focus_gained(&mut self, _: &AppData) {}

    fn focus_lost(&mut self, _: &AppData) {}

    fn on_enter(&mut self, _: &AppData) {}

    fn on_left(&mut self, _: &AppData) {}

    fn on_right(&mut self, _: &AppData) {}

    fn on_up(&mut self, _: &AppData) {}

    fn on_down(&mut self, _: &AppData) {}

    fn on_other(&mut self, key_event: KeyEvent, _: &AppData) {
        if let KeyCode::Char(c) = key_event.code {
            if c.is_numeric() {
                if let Some(number) = c.to_digit(10) {
                    if number > 0 && number < 8 {
                        self.analysis_window_tabs.select(number as usize);
                    }
                }
            }
        };
    }
}
