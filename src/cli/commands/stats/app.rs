use std::collections::HashSet;
use std::sync::Arc;

use crate::api;
use crate::api::node::{GraphDefinition, NodeInfo, NodeInfoType, Vertex};
use crate::api::stats::NodeStats;
use crate::commands::stats::widgets::{StatefulTable, TabsState};

pub struct StatsSignal {
    api: Arc<api::Client>,
}

impl StatsSignal {
    pub fn new(api: Arc<api::Client>) -> StatsSignal {
        StatsSignal {
            api,
        }
    }

    pub fn fetch_info(&self) -> Option<NodeInfo> {
        match self.api.get_node_info(&[NodeInfoType::Pipelines], &vec![]) {
            Ok(stats) => {
                Some(stats)
            }
            Err(err) => {
                //println!("{:?}", err);
                None
            }
        }
    }

    pub fn fetch_stats(&self) -> Option<NodeStats> {
        match self.api.get_node_stats() {
            Ok(stats) => {
                Some(stats)
            }
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

        let pipeline_item = selected_pipeline.unwrap();
        let mut new_items: Vec<String> = Vec::with_capacity(pipeline_item.graph.vertices.len());
        let mut if_vertices: HashSet<&str> = HashSet::new();

        for vertex in &pipeline_item.graph.vertices {
            new_items.push(vertex.id.to_string());
            if vertex.r#type == "if" {
                if_vertices.insert(vertex.id.as_str());
            }
        }

        let mut visited = HashSet::new();
        for edge in &pipeline_item.graph.edges {
            let vertex_id = edge.from.as_str();

            if visited.contains(vertex_id) {
                continue;
            }

            if edge.r#type == "boolean" && edge.when == Some(false) && if_vertices.contains(vertex_id) {
                new_items.push(edge.id.to_string());
                visited.insert(edge.from.as_str());
            }
        }

        self.items = new_items;

        // let node_info = &app_state.node_info.as_ref().unwrap();
        // if node_info.pipelines.is_none() || node_info.pipelines.as_ref().unwrap().is_empty() {
        //     return;
        // }
        //
        // let mut new_items = vec![];
        // let pipeline_item = selected_pipeline.unwrap();
        // let pipeline_stats: Option<&PipelineStats>;
        // if let Some(node_stats) = &app_state.node_stats {
        //     pipeline_stats = node_stats.pipelines.get(&pipeline_item.name);
        // } else {
        //     pipeline_stats = None;
        // }

        // for item in &pipeline_item.vertices {
        //     let events_in: Option<i64>;
        //     let events_out: Option<i64>;
        //     let duration_in_millis: Option<u64>;
        //     let queue_push_duration_in_millis: Option<u64>;
        //
        //     if let Some(stats) = pipeline_stats {
        //         events_in = Some(stats.events.r#in);
        //         events_out = Some(stats.events.out);
        //         duration_in_millis = Some(stats.events.duration_in_millis);
        //         queue_push_duration_in_millis = Some(stats.events.queue_push_duration_in_millis);
        //     } else {
        //         events_in = None;
        //         events_out = None;
        //         duration_in_millis = None;
        //         queue_push_duration_in_millis = None;
        //     }
        //
        //     new_items.push(
        //         PluginItem {
        //             id: item.id.clone(),
        //             name: item.config_name.clone(),
        //             kind: item.plugin_type.to_uppercase(),
        //             kind_description: None,
        //             events_in,
        //             events_out,
        //             duration_in_millis,
        //             queue_push_duration_in_millis,
        //         }
        //     );
        // }

        // if let Some(value) = selected_pipeline {
        //     let mut input_codecs: Vec<PluginItem> = vec![];
        //     let mut output_codecs: Vec<PluginItem> = vec![];
        //
        //     for codec_plugin in &value.pipeline.plugins.codecs {
        //         let all_codecs = codec_plugin.decode != Events::default() && codec_plugin.encode != Events::default();
        //
        //         if all_codecs || codec_plugin.decode != Events::default() {
        //             input_codecs.push(
        //                 PluginItem {
        //                     id: codec_plugin.id.clone(),
        //                     name: codec_plugin.name.clone(),
        //                     kind: "CODEC".to_string(),
        //                     kind_description: None,
        //                     events_in: Some(codec_plugin.decode.writes_in),
        //                     events_out: Some(codec_plugin.decode.out),
        //                     duration_in_millis: Some(codec_plugin.decode.duration_in_millis),
        //                     queue_push_duration_in_millis: None,
        //                 }
        //             );
        //         }
        //
        //         if all_codecs || codec_plugin.encode != Events::default() {
        //             output_codecs.push(
        //                 PluginItem {
        //                     id: codec_plugin.id.clone(),
        //                     name: codec_plugin.name.clone(),
        //                     kind: "CODEC".to_string(),
        //                     kind_description: None,
        //                     events_in: Some(codec_plugin.encode.writes_in),
        //                     events_out: Some(codec_plugin.encode.out),
        //                     duration_in_millis: Some(codec_plugin.encode.duration_in_millis),
        //                     queue_push_duration_in_millis: None,
        //                 }
        //             )
        //         }
        //     }
        //
        //     for input_plugin in &value.pipeline.plugins.inputs {
        //         let mut queue_push_duration_in_millis: Option<u64> = None;
        //         if input_plugin.events.queue_push_duration_in_millis > 0 {
        //             queue_push_duration_in_millis = Some(input_plugin.events.queue_push_duration_in_millis);
        //         }
        //
        //         new_items.push(
        //             PluginItem {
        //                 id: input_plugin.id.clone(),
        //                 name: input_plugin.name.clone(),
        //                 kind: "INPUT".to_string(),
        //                 kind_description: None,
        //                 events_in: Some(input_plugin.events.r#in),
        //                 events_out: Some(input_plugin.events.out),
        //                 duration_in_millis: Some(input_plugin.events.duration_in_millis),
        //                 queue_push_duration_in_millis,
        //             }
        //         );
        //
        //         if let Some(input_codec) = input_codecs.pop() {
        //             new_items.push(input_codec);
        //         }
        //     }
        //
        //     for input_codec in input_codecs {
        //         new_items.push(input_codec);
        //     }
        //
        //     for filter_plugin in &value.pipeline.plugins.filters {
        //         new_items.push(
        //             PluginItem {
        //                 id: filter_plugin.id.clone(),
        //                 name: filter_plugin.name.clone(),
        //                 kind: "FILTER".to_string(),
        //                 kind_description: None,
        //                 events_in: Some(filter_plugin.events.r#in),
        //                 events_out: Some(filter_plugin.events.out),
        //                 duration_in_millis: Some(filter_plugin.events.duration_in_millis),
        //                 queue_push_duration_in_millis: None,
        //             }
        //         )
        //     }
        //
        //     for output_plugin in &value.pipeline.plugins.outputs {
        //         if let Some(output_codec) = output_codecs.pop() {
        //             new_items.push(output_codec);
        //         }
        //
        //         new_items.push(
        //             PluginItem {
        //                 id: output_plugin.id.clone(),
        //                 name: output_plugin.name.clone(),
        //                 kind: "OUTPUT".to_string(),
        //                 kind_description: None,
        //                 events_in: Some(output_plugin.events.r#in),
        //                 events_out: Some(output_plugin.events.out),
        //                 duration_in_millis: Some(output_plugin.events.duration_in_millis),
        //                 queue_push_duration_in_millis: None,
        //             }
        //         )
        //     }
        //
        //     for output_codec in output_codecs {
        //         new_items.push(output_codec);
        //     }
        // }

        // self.items = new_items;
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
                if let Some(selected_pipeline_name) = self.selected_item().map(|p| p.name.as_str()) {
                    if let Some(new_index) = new_items.iter().position(|p| p.name == selected_pipeline_name) {
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

pub struct PluginItem {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub kind_description: Option<String>,
    pub events_in: Option<i64>,
    pub events_out: Option<i64>,
    pub duration_in_millis: Option<u64>,
    pub queue_push_duration_in_millis: Option<u64>,
}

pub struct PipelineVertex {
    vertex: Vertex,

}

pub struct AppState {
    pub node_info: Option<NodeInfo>,
    pub node_stats: Option<NodeStats>,
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub pipelines: StatefulTable<PipelineItem>,
    pub selected_pipeline_graph: StatefulTable<String>,
    pub stats_signal: StatsSignal,
    pub state: AppState,
    pub show_chart: bool,
    pub focused: u8,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, api: Arc<api::Client>) -> App<'a> {
        let app_state = AppState {
            node_info: None,
            node_stats: None,
        };

        App {
            title,
            show_chart: true,
            should_quit: false,
            tabs: TabsState::new(vec!["Pipelines", "Environment"]),
            pipelines: StatefulTable::new(),
            selected_pipeline_graph: StatefulTable::new(),
            stats_signal: StatsSignal::new(api),
            state: app_state,
            focused: 0,
        }
    }

    pub fn on_up(&mut self) {
        if self.focused == 0 {
            self.selected_pipeline_graph.update(&self.state, self.pipelines.previous());
        } else {
            self.selected_pipeline_graph.previous();
        }
    }

    pub fn on_down(&mut self) {
        if self.focused == 0 {
            self.selected_pipeline_graph.update(&self.state, self.pipelines.next());
        } else {
            self.selected_pipeline_graph.next();
        }
    }

    pub fn on_right(&mut self) {
        self.focused = 1;
        self.selected_pipeline_graph.next();
    }

    pub fn on_left(&mut self) {
        self.focused = 0;
        self.selected_pipeline_graph.unselect();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            't' => {
                self.show_chart = !self.show_chart;
            }
            'p' => {
                self.tabs.select(0);
            }
            'e' => {
                self.tabs.select(1);
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        self.state.node_info = self.stats_signal.fetch_info();
        // for x in graph.graph.nodes() {
        //     if let Some(v) = graph.node(x){
        //         println!("{}: {:?}", v.vertex.r#type, v.vertex.config_name);
        //     }
        // }

        // for i in graph.input_nodes() {
        //     println!("{:?}", graph.data.get(i).unwrap().vertex);
        // }

        // for i in &graph.graph.node_indices() {
        //     let mut bfs = Dfs::new(&graph.graph, i);
        //     while let Some(x) = bfs.next(&graph.graph) {
        //         if let Some(v) = &graph.node(x){
        //             println!("{}: {:?}", v.vertex.r#type, v.vertex.config_name);
        //         }
        //     }
        // }

        // let mut bfs = Bfs::new(&graph.graph, );
        // while let Some(x) = bfs.next(&graph.graph) {
        //     if let Some(v) = graph.node(x){
        //         println!("{}: {:?}", v.vertex.r#type, v.vertex.config_name);
        //     }
        // }


        self.state.node_stats = self.stats_signal.fetch_stats();
        self.pipelines.update(&self.state);
        self.selected_pipeline_graph.update(&self.state, self.pipelines.selected_item());
    }
}