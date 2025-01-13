use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crate::api::hot_threads::NodeHotThreads;
use crate::api::node::NodeInfo;
use crate::api::stats::NodeStats;
use crate::commands::tui::data_decorator;
use crate::commands::tui::data_fetcher::{DataFetcher, NodeData};
use crate::commands::tui::events::EventsListener;
use crate::commands::tui::flows::state::FlowsState;
use crate::commands::tui::node::state::NodeState;
use crate::commands::tui::pipelines::state::PipelinesState;
use crate::commands::tui::shared_state::SharedState;
use crate::commands::tui::threads::state::ThreadsState;
use crate::commands::tui::widgets::TabsState;
use crate::errors::AnyError;
use crossterm::event::{KeyCode, KeyEvent};

pub(crate) struct AppData {
    errored: bool,
    last_error_message: Option<String>,
    node_info: Option<NodeInfo>,
    node_stats: Option<NodeStats>,
    hot_threads: Option<NodeHotThreads>,
}

impl AppData {
    fn new() -> Self {
        AppData {
            errored: false,
            last_error_message: None,
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

    fn fetch_and_set(&mut self, data_fetcher: &dyn DataFetcher) {
        if let Ok(mut node_data) = data_fetcher.fetch_node_data(None) {
            data_decorator::decorate(&mut node_data.info, &mut node_data.stats);
            self.node_info = Some(node_data.info);
            self.node_stats = Some(node_data.stats);
        }

        if let Ok(hot_threads) = data_fetcher.fetch_hot_threads(None) {
            self.hot_threads = Some(hot_threads);
        }
    }

    fn get_fetched_data(
        data_fetcher: &dyn DataFetcher,
        data_tx: Sender<(NodeInfo, NodeStats, Option<NodeHotThreads>)>,
        error_tx: Sender<AnyError>,
    ) {
        let mut node_data: NodeData = match data_fetcher.fetch_node_data(None) {
            Ok(value) => value,
            Err(e) => {
                if e.downcast_ref::<RecvTimeoutError>().is_none() {
                    _ = error_tx.send(e);
                }
                return;
            }
        };

        let hot_threads = match data_fetcher.fetch_hot_threads(Some(Duration::from_millis(100))) {
            Ok(data) => Some(data),
            Err(e) => {
                if e.downcast_ref::<RecvTimeoutError>().is_none() {
                    _ = error_tx.send(e);
                }
                None
            }
        };

        data_decorator::decorate(&mut node_data.info, &mut node_data.stats);
        let result = (node_data.info, node_data.stats, hot_threads);
        _ = data_tx.send(result);
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
    pub title: String,
    pub should_quit: bool,
    pub show_help: bool,
    pub tabs: TabsState,
    pub shared_state: SharedState,
    pub node_state: NodeState,
    pub pipelines_state: PipelinesState<'a>,
    pub flows_state: FlowsState,
    pub threads_state: ThreadsState,
    pub data: Arc<RwLock<AppData>>,
    pub host: String,
    pub sampling_interval: Option<Duration>,
}

impl<'a> App<'a> {
    pub const TAB_PIPELINES: usize = 0;
    pub const TAB_FLOWS: usize = 1;
    pub const TAB_THREADS: usize = 2;
    pub const TAB_NODE: usize = 3;

    pub fn new(title: String, host: String, sampling_interval: Option<Duration>) -> App<'a> {
        App {
            title,
            sampling_interval,
            show_help: false,
            should_quit: false,
            tabs: TabsState::new(),
            pipelines_state: PipelinesState::new(),
            node_state: NodeState::new(),
            data: Arc::new(RwLock::new(AppData::new())),
            host,
            shared_state: SharedState::new(),
            flows_state: FlowsState::new(),
            threads_state: ThreadsState::new(),
        }
    }

    fn reset(&mut self) {
        {
            self.data.write().unwrap().reset();
        }
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

    pub fn set_data(&mut self, data_fetcher: &dyn DataFetcher) {
        self.data.write().unwrap().fetch_and_set(data_fetcher);
    }

    pub fn start_reading_data(&self, data_fetcher: Box<dyn DataFetcher>, interval: Duration) {
        let (data_tx, data_rx) = channel::<(NodeInfo, NodeStats, Option<NodeHotThreads>)>();
        let (error_tx, error_rx) = channel::<AnyError>();

        thread::Builder::new()
            .name("app-data-get-fetched-data".to_string())
            .spawn(move || loop {
                AppData::get_fetched_data(data_fetcher.as_ref(), data_tx.clone(), error_tx.clone());
                sleep(interval);
            })
            .unwrap();

        let data = self.data.clone();
        thread::Builder::new()
            .name("app-data-fetched-data-receiver".to_string())
            .spawn(move || loop {
                if let Ok(values) = data_rx.recv() {
                    let mut data = data.write().unwrap();
                    data.node_info = Some(values.0);
                    data.node_stats = Some(values.1);
                    data.hot_threads = values.2;
                    data.errored = false;
                    data.last_error_message = None;
                }
                sleep(interval);
            })
            .unwrap();

        let data = self.data.clone();
        thread::Builder::new()
            .name("app-data-fetch-api-errors".to_string())
            .spawn(move || loop {
                if let Ok(values) = error_rx.recv() {
                    data.write().unwrap().handle_error(&values);
                }
                sleep(interval);
            })
            .unwrap();
    }

    pub fn wait_node_data(&self) {
        loop {
            {
                let data = self.data.read().unwrap();
                if data.errored {
                    break;
                }
                if data.node_info.is_some() && data.node_stats.is_some() {
                    return;
                }
            }
            sleep(Duration::from_millis(100));
        }
    }

    pub fn on_tick(&mut self) {
        {
            if self.data.read().unwrap().errored {
                self.reset();
                return;
            }
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
            func(listener, &self.data.read().unwrap());
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
            func(&self.data.read().unwrap(), value);
        }
    }
}
