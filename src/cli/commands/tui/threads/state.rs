use std::collections::{HashMap, VecDeque};

use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::commands::tui::app::AppData;
use crate::commands::tui::events::EventsListener;
use crate::commands::tui::widgets::StatefulTable;

pub const THREAD_LIST: usize = 0;
pub const THREAD_TRACES_VIEW: usize = 1;

pub struct ThreadsState {
    pub current_focus: usize,
    pub show_selected_thread: bool,
    pub threads_table: StatefulTable<ThreadTableItem>,
    pub threads_table_states: HashMap<i64, VecDeque<String>>,
    pub threads_table_states_times: VecDeque<OffsetDateTime>,
    pub selected_thread_traces: StatefulTable<String>,
    pub selected_thread_trace_value_offset: usize,
}

pub struct ThreadTableItem {
    pub time: String,
    pub name: String,
    pub id: i64,
    pub percent_of_cpu_time: f64,
    pub state: String,
    pub traces: Vec<String>,
}

impl ThreadsState {
    pub(crate) fn new() -> Self {
        ThreadsState {
            current_focus: THREAD_LIST,
            show_selected_thread: false,
            threads_table: StatefulTable::new(),
            threads_table_states: Default::default(),
            threads_table_states_times: Default::default(),
            selected_thread_traces: StatefulTable::new(),
            selected_thread_trace_value_offset: 0,
        }
    }

    fn update_selected_thread_traces(&mut self, _app_data: &AppData) {
        if let Some(selected_thread) = self.threads_table.selected_item() {
            self.selected_thread_traces
                .items
                .clone_from(&selected_thread.traces);
        }
    }
}

impl StatefulTable<ThreadTableItem> {
    fn update(&mut self, data: &AppData) {
        if let Some(threads) = data.hot_threads() {
            let mut new_items = Vec::with_capacity(threads.hot_threads.threads.len());
            for thread in threads.hot_threads.threads.values() {
                let new_item = ThreadTableItem {
                    id: thread.thread_id,
                    name: thread.name.to_string(),
                    percent_of_cpu_time: thread.percent_of_cpu_time,
                    state: thread.state.to_string(),
                    traces: thread.traces.clone(),
                    time: threads.hot_threads.time.to_string(),
                };
                new_items.push(new_item);
            }

            new_items.sort_by(|a, b| {
                if a.percent_of_cpu_time == b.percent_of_cpu_time {
                    a.name.cmp(&b.name)
                } else {
                    a.percent_of_cpu_time
                        .total_cmp(&b.percent_of_cpu_time)
                        .reverse()
                }
            });

            let new_selected_index = if let Some(selected) = self.selected_item() {
                // Not optimal, but should work for now to keep the selected line
                // selected after an update with order changes.
                new_items.iter().position(|p| p.id == selected.id)
            } else {
                None
            };

            self.items = new_items;
            self.select(new_selected_index);
        }
    }
}

pub(crate) const MAX_THREAD_STATES: usize = 500;

impl EventsListener for ThreadsState {
    fn update(&mut self, app_data: &AppData) {
        self.threads_table.update(app_data);

        for thread_item in &self.threads_table.items {
            self.threads_table_states.entry(thread_item.id).or_default();

            let states = self.threads_table_states.get_mut(&thread_item.id).unwrap();

            if let Ok(time) = OffsetDateTime::parse(&thread_item.time, &Iso8601::DEFAULT) {
                self.threads_table_states_times.push_back(time);
            } else {
                self.threads_table_states_times
                    .push_back(OffsetDateTime::now_utc());
            }

            states.push_back(thread_item.state.to_string());

            if states.len() > MAX_THREAD_STATES {
                states.pop_front();
                self.threads_table_states_times.pop_front();
            }
        }
    }

    fn reset(&mut self) {
        self.current_focus = THREAD_LIST;
        self.show_selected_thread = false;
        self.selected_thread_trace_value_offset = 0;

        self.threads_table = StatefulTable::new();
        self.selected_thread_traces = StatefulTable::new();
        self.threads_table_states.clear();
        self.threads_table_states_times.clear();
    }

    fn on_enter(&mut self, _app_data: &AppData) {
        if self.current_focus == THREAD_LIST {
            if self.threads_table.selected_item().is_some() {
                self.show_selected_thread = !self.show_selected_thread;
            }
        } else {
            self.show_selected_thread = false;
            self.selected_thread_trace_value_offset = 0;
            self.selected_thread_traces.unselect();
            self.current_focus = THREAD_LIST;
        }
    }

    fn on_left(&mut self, _app_data: &AppData) {
        if self.current_focus != THREAD_LIST {
            self.current_focus = THREAD_LIST;
            self.selected_thread_traces.unselect();
        }
    }

    fn on_right(&mut self, _app_data: &AppData) {
        if self.current_focus == THREAD_LIST && self.show_selected_thread {
            self.current_focus = THREAD_TRACES_VIEW;
            self.selected_thread_traces.next();
        } else if self.selected_thread_traces.selected_item().is_some() {
            self.selected_thread_trace_value_offset += 3;
        }
    }

    fn on_up(&mut self, app_data: &AppData) {
        if self.current_focus == THREAD_LIST {
            self.threads_table.previous();
            self.update_selected_thread_traces(app_data);
        } else {
            self.selected_thread_trace_value_offset = 0;
            self.selected_thread_traces.previous();
        }
    }

    fn on_down(&mut self, app_data: &AppData) {
        if self.current_focus == THREAD_LIST {
            self.threads_table.next();
            self.update_selected_thread_traces(app_data);
        } else {
            self.selected_thread_trace_value_offset = 0;
            self.selected_thread_traces.next();
        }
    }
}
