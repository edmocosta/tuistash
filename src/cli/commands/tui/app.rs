use std::time::Duration;

use crate::api::hot_threads::NodeHotThreads;
use crossterm::event::{KeyCode, KeyEvent};

use crate::api::node::NodeInfo;
use crate::api::stats::NodeStats;
use crate::commands::tui::data_decorator;
use crate::commands::tui::data_fetcher::DataFetcher;
use crate::commands::tui::events::EventsListener;
use crate::commands::tui::flows::state::FlowsState;
use crate::commands::tui::node::state::NodeState;
use crate::commands::tui::pipelines::state::PipelinesState;
use crate::commands::tui::shared_state::SharedState;
use crate::commands::tui::threads::state::ThreadsState;
use crate::commands::tui::widgets::TabsState;
use crate::errors::AnyError;

pub(crate) struct AppData<'a> {
    errored: bool,
    last_error_message: Option<String>,
    data_fetcher: &'a dyn DataFetcher<'a>,
    node_info: Option<NodeInfo>,
    node_stats: Option<NodeStats>,
    hot_threads: Option<NodeHotThreads>,
}

impl<'a> AppData<'a> {
    fn new(data_fetcher: &'a impl DataFetcher<'a>) -> Self {
        AppData {
            errored: false,
            last_error_message: None,
            data_fetcher,
            node_stats: None,
            node_info: None,
            hot_threads: None,
        }
    }

    fn reset(&mut self) {
        self.node_info = None;
        self.node_stats = None;
    }

    fn handle_error(&mut self, error: &AnyError) {
        self.errored = true;
        self.last_error_message = Some(error.to_string());
        self.reset();
    }

    fn fetch_all(&mut self) -> Result<&mut Self, AnyError> {
        match self.data_fetcher.fetch_info() {
            Ok(node_info) => {
                self.node_info = Some(node_info);
            }
            Err(e) => {
                self.handle_error(&e);
                return Err(e);
            }
        }

        match self.data_fetcher.fetch_stats() {
            Ok(node_stats) => {
                self.node_stats = Some(node_stats);
            }
            Err(e) => {
                self.handle_error(&e);
                return Err(e);
            }
        }

        match self.data_fetcher.fetch_hot_threads() {
            Ok(hot_threads) => {
                self.hot_threads = Some(hot_threads);
            }
            Err(e) => {
                self.handle_error(&e);
                return Err(e);
            }
        }

        data_decorator::decorate(
            self.node_info.as_mut().unwrap(),
            self.node_stats.as_mut().unwrap(),
        );

        self.errored = false;
        self.last_error_message = None;

        Ok(self)
    }

    pub(crate) fn node_info(&self) -> Option<&NodeInfo> {
        self.node_info.as_ref()
    }

    pub(crate) fn node_stats(&self) -> Option<&NodeStats> {
        self.node_stats.as_ref()
    }

    pub(crate) fn hot_threads(&self) -> Option<&NodeHotThreads> {
        self.hot_threads.as_ref()
    }

    pub(crate) fn errored(&self) -> bool {
        self.errored
    }

    pub(crate) fn last_error_message(&self) -> &Option<String> {
        &self.last_error_message
    }
}

pub(crate) struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub show_help: bool,
    pub tabs: TabsState,
    pub shared_state: SharedState,
    pub node_state: NodeState,
    pub pipelines_state: PipelinesState<'a>,
    pub flows_state: FlowsState,
    pub threads_state: ThreadsState,
    pub data: AppData<'a>,
    pub host: &'a str,
    pub sampling_interval: Option<Duration>,
}

impl<'a> App<'a> {
    pub const TAB_PIPELINES: usize = 0;
    pub const TAB_FLOWS: usize = 1;
    pub const TAB_THREADS: usize = 2;
    pub const TAB_NODE: usize = 3;

    pub fn new(
        title: &'a str,
        fetcher: &'a impl DataFetcher<'a>,
        host: &'a str,
        sampling_interval: Option<Duration>,
    ) -> App<'a> {
        App {
            title,
            sampling_interval,
            show_help: false,
            should_quit: false,
            tabs: TabsState::new(),
            pipelines_state: PipelinesState::new(),
            node_state: NodeState::new(),
            data: AppData::new(fetcher),
            host,
            shared_state: SharedState::new(),
            flows_state: FlowsState::new(),
            threads_state: ThreadsState::new(),
        }
    }

    fn reset(&mut self) {
        self.data.reset();
        self.trigger_states_event(|listener, _| {
            listener.reset();
        });
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let selected_tab = &self.tabs.index.clone();
        match key.code {
            KeyCode::Left => {
                self.trigger_tab_event(selected_tab, |app_data, listener| {
                    listener.on_left(app_data);
                });
            }
            KeyCode::Right => {
                self.trigger_tab_event(selected_tab, |app_data, listener| {
                    listener.on_right(app_data);
                });
            }
            KeyCode::Up => {
                self.trigger_tab_event(selected_tab, |app_data, listener| {
                    listener.on_up(app_data);
                });
            }
            KeyCode::Down => {
                self.trigger_tab_event(selected_tab, |app_data, listener| {
                    listener.on_down(app_data);
                });
            }
            KeyCode::Char(c) => {
                self.on_key(c);
                self.trigger_tab_event(selected_tab, |app_data, listener| {
                    listener.on_other(key, app_data);
                });
            }
            KeyCode::Enter => {
                self.trigger_tab_event(selected_tab, |app_data, listener| {
                    listener.on_enter(app_data);
                });
            }
            _ => {
                self.trigger_tab_event(selected_tab, |app_data, listener| {
                    listener.on_other(key, app_data);
                });
            }
        }
    }

    pub fn on_key(&mut self, c: char) {
        match c.to_lowercase().to_string().as_str() {
            "q" => {
                self.on_esc();
            }
            "h" => {
                let visible = !self.show_help;
                self.show_help = visible;
            }
            "p" => {
                self.select_tab(Self::TAB_PIPELINES);
            }
            "f" => {
                self.select_tab(Self::TAB_FLOWS);
            }
            "n" => {
                self.select_tab(Self::TAB_NODE);
            }
            "t" => {
                self.select_tab(Self::TAB_THREADS);
            }
            _ => {}
        }
    }

    pub fn on_esc(&mut self) {
        self.should_quit = true;
    }

    pub fn on_tick(&mut self) {
        if self.data.fetch_all().is_err() {
            self.reset();
            return;
        }

        self.trigger_states_event(|listener, app_data| {
            listener.update(app_data);
        });
    }

    fn trigger_states_event(&mut self, func: impl Fn(&mut dyn EventsListener, &AppData)) {
        let listeners: Vec<&mut dyn EventsListener> = vec![
            &mut self.shared_state,
            &mut self.pipelines_state,
            &mut self.flows_state,
            &mut self.node_state,
            &mut self.threads_state,
        ];

        for listener in listeners {
            func(listener, &self.data);
        }
    }

    fn select_tab(&mut self, new_tab: usize) {
        self.tabs.select(new_tab);

        self.trigger_tab_event(&self.tabs.index.clone(), |app_data, listener| {
            listener.focus_lost(app_data);
        });

        self.trigger_tab_event(&new_tab, |app_data, listener| {
            listener.focus_gained(app_data);
        });
    }

    fn trigger_tab_event(&mut self, tab: &usize, func: impl Fn(&AppData, &mut dyn EventsListener)) {
        let listener: Option<&mut dyn EventsListener> = match *tab {
            Self::TAB_PIPELINES => Some(&mut self.pipelines_state),
            Self::TAB_NODE => Some(&mut self.node_state),
            Self::TAB_FLOWS => Some(&mut self.flows_state),
            Self::TAB_THREADS => Some(&mut self.threads_state),
            _ => None,
        };

        if let Some(value) = listener {
            func(&self.data, value);
        }
    }
}
