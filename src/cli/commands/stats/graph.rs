use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use crate::api::node::{Edge, GraphDefinition, Vertex};

#[derive(Debug, Copy, Clone)]
pub struct Stats {
    pub events_in: Option<i64>,
    pub events_out: Option<i64>,
    pub duration_in_millis: Option<u64>,
    pub queue_push_duration_in_millis: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Node<'a> {
    pub vertex: &'a Vertex,
    pub stats: Option<&'a Stats>,

}

pub struct Data<'a, T> {
    pub value: &'a T,
}

pub struct VertexEdge<'a> {
    pub vertex_id: &'a str,
    pub r#type: String,
    pub when: Option<bool>,
}

pub struct PipelineGraph<'a> {
    pub vertices: HashMap<&'a str, Vec<VertexEdge<'a>>>,
    pub data: HashMap<&'a str, Data<'a, Vertex>>,
    pub inputs: HashSet<&'a str>,
}

impl<'a> PipelineGraph<'a> {
    pub fn from(graph_value: &'a GraphDefinition) -> Self {
        let mut vertices: HashMap<&'a str, Vec<VertexEdge<'a>>> = HashMap::new();
        let mut data: HashMap<&'a str, Data<'a, Vertex>> = HashMap::new();
        let mut inputs: HashSet<&'a str> = HashSet::new();

        for vertex in &graph_value.vertices {
            let id = vertex.id.as_str();
            if !vertices.contains_key(id) {
                vertices.insert(id, Vec::new());
            }

            if vertex.r#type == "plugin" && vertex.plugin_type == "input" {
                inputs.insert(id);
            }

            data.insert(id, Data { value: vertex });
        }

        for edge in &graph_value.edges {
            vertices.get_mut(edge.from.as_str()).unwrap().push(
                VertexEdge {
                    vertex_id: edge.to.as_str(),
                    r#type: edge.r#type.to_string(),
                    when: edge.when,
                }
            );
        }

        for vertex in &graph_value.vertices {
            let neighbours = vertices.get_mut(vertex.id.as_str()).unwrap();

            // Sort by source line or column
            neighbours.sort_by(|a, b| {
                let a_vertex = &data.get(a.vertex_id).unwrap().value;
                let b_vertex = &data.get(b.vertex_id).unwrap().value;

                if let Some(a_vertex_source) = a_vertex.meta.as_ref().map(|m| &m.source) {
                    if let Some(b_vertex_source) = b_vertex.meta.as_ref().map(|m| &m.source) {
                        let line_order = a_vertex_source.line.cmp(&b_vertex_source.line);
                        if line_order == Ordering::Equal {
                            return a_vertex_source.column.cmp(&b_vertex_source.column);
                        }

                        return line_order;
                    }
                }

                return Ordering::Equal;
            });
        }

        PipelineGraph {
            vertices,
            data,
            inputs,
        }
    }
}