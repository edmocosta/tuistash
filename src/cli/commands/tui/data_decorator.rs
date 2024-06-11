use std::cmp::Ordering;
use std::collections::HashMap;

use uuid::Uuid;

use crate::api::node::{Edge, NodeInfo, Vertex};
use crate::api::stats::{NodeStats, NodeStatsVertex};

pub(crate) fn decorate(node_info: &mut NodeInfo, node_stats: &mut NodeStats) {
    if !is_graph_api_available(&node_info.node.version) || has_empty_vertices(node_info) {
        decorate_node_stats(node_stats);
        decorate_node_info(node_info, node_stats);
    }
}

fn has_empty_vertices(node_info: &NodeInfo) -> bool {
    node_info
        .pipelines
        .as_ref()
        .is_some_and(|p| p.values().any(|m| m.graph.graph.vertices.is_empty()))
}

// Pipeline graph/vertices API is available on version > 7.3.0
fn is_graph_api_available(version: &str) -> bool {
    let version: Vec<&str> = version.split('.').collect();
    if let Some(major) = version.first() {
        if let Ok(major) = major.parse::<u32>() {
            if major < 7 {
                return false;
            } else if major > 7 {
                return true;
            } else if let Some(minor) = version.get(1) {
                if let Ok(minor) = minor.parse::<u32>() {
                    return minor >= 3;
                }
            }
        }
    }

    false
}

fn decorate_node_stats(node_stats: &mut NodeStats) {
    for stats in &mut node_stats.pipelines.values_mut() {
        if stats.vertices.is_empty() {
            let mut new_vertices: HashMap<String, NodeStatsVertex> = HashMap::new();
            for (id, plugin) in stats.plugins.all() {
                new_vertices.insert(
                    id.to_string(),
                    NodeStatsVertex {
                        id: id.to_string(),
                        pipeline_ephemeral_id: stats
                            .ephemeral_id
                            .as_deref()
                            .map(|p| p.to_string())
                            .unwrap_or(id),
                        events_out: plugin.events.out,
                        events_in: plugin.events.r#in,
                        duration_in_millis: plugin.events.duration_in_millis,
                        queue_push_duration_in_millis: plugin.events.queue_push_duration_in_millis,
                    },
                );
            }
            stats.vertices = new_vertices;
        }
    }
}

fn decorate_node_info(node_info: &mut NodeInfo, node_stats: &NodeStats) {
    if let Some(pipelines) = &mut node_info.pipelines {
        for (pipeline, info) in pipelines {
            if info.graph.graph.vertices.is_empty() {
                let pipeline_stats = node_stats.pipelines.get(pipeline).unwrap();
                let mut new_vertices = vec![];
                let mut new_edges = vec![];

                for (id, (plugin_type, plugin)) in pipeline_stats.plugins.all_with_type() {
                    if plugin_type == "codec" {
                        continue;
                    }

                    new_vertices.push(Vertex {
                        id: id.to_string(),
                        explicit_id: id.len() != 64, // Guessed based on the generated ID size
                        config_name: plugin
                            .name
                            .as_ref()
                            .unwrap_or(&plugin.id.to_string())
                            .to_string(),
                        plugin_type: plugin_type.to_string(),
                        condition: "".to_string(),
                        r#type: "plugin".to_string(),
                        meta: None,
                    });
                }

                new_vertices.push(Vertex {
                    id: "__QUEUE__".to_string(),
                    explicit_id: false,
                    config_name: "".to_string(),
                    plugin_type: "".to_string(),
                    condition: "".to_string(),
                    r#type: "queue".to_string(),
                    meta: None,
                });

                new_vertices.sort_by(|v1, v2| {
                    if v1.plugin_type == v2.plugin_type {
                        return v1.id.cmp(&v2.id);
                    }

                    if v1.plugin_type == "input" {
                        return Ordering::Less;
                    }

                    if v2.plugin_type == "input" {
                        return Ordering::Greater;
                    }

                    v1.plugin_type.cmp(&v2.plugin_type)
                });

                for i in 0..new_vertices.len() - 1 {
                    let (plugin_type, from) = &new_vertices
                        .get(i)
                        .map(|p| (p.plugin_type.to_string(), p.id.to_string()))
                        .unwrap_or(("".to_string(), "".to_string()));

                    let to = if plugin_type == "input" {
                        "__QUEUE__".to_string()
                    } else {
                        new_vertices
                            .get(i + 1)
                            .map(|p| p.id.to_string())
                            .unwrap_or("".to_string())
                    };

                    new_edges.push(Edge {
                        id: Uuid::new_v4().to_string(),
                        from: from.to_string(),
                        to,
                        r#type: "".to_string(),
                        when: None,
                    });
                }

                info.graph.graph.vertices = new_vertices;
                info.graph.graph.edges = new_edges;
            }
        }
    }
}
