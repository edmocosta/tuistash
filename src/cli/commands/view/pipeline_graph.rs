use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use uuid::Uuid;

use crate::api::node::{GraphDefinition, Vertex};
use crate::commands::view::app::PipelineItem;

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
    pub heads: Vec<&'a str>,
}

type GraphVisitorStack<'b> = RefCell<Vec<(&'b str, Option<i32>, i32, bool)>>;

impl<'a> PipelineGraph<'a> {
    pub fn from(graph_value: &'a GraphDefinition) -> Self {
        let mut vertices: HashMap<&'a str, Vec<VertexEdge<'a>>> = HashMap::new();
        let mut inputs_incoming_edge: HashMap<&'a str, &'a str> = HashMap::new();
        let mut data: HashMap<&'a str, Data<'a, Vertex>> = HashMap::new();
        let mut inputs: HashSet<&'a str> = HashSet::new();
        let mut heads: Vec<&'a str> = Vec::new();

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
            vertices
                .get_mut(edge.from.as_str())
                .unwrap()
                .push(VertexEdge {
                    vertex_id: edge.to.as_str(),
                    r#type: edge.r#type.to_string(),
                    when: edge.when,
                });

            let edge_to_vertex = data.get(edge.to.as_str()).unwrap();
            if edge_to_vertex.value.plugin_type == "input" || edge_to_vertex.value.r#type == "if" {
                inputs_incoming_edge.insert(edge.to.as_str(), edge.from.as_str());
            }
        }

        let sort_by_source_fn = Self::sort_vertices_by_source();
        for vertex in &graph_value.vertices {
            let neighbours = vertices.get_mut(vertex.id.as_str()).unwrap();
            neighbours.sort_by(|a, b| {
                sort_by_source_fn(
                    data.get(a.vertex_id).unwrap().value,
                    data.get(b.vertex_id).unwrap().value,
                )
            });
        }

        for vertex_id in &inputs {
            if !inputs_incoming_edge.contains_key(vertex_id) {
                heads.push(vertex_id);
            } else {
                let mut current: &str = vertex_id;
                while inputs_incoming_edge.contains_key(current) {
                    current = inputs_incoming_edge.get(current).unwrap();
                }

                heads.push(current);
            }
        }

        heads.sort_by(|a, b| {
            sort_by_source_fn(data.get(a).unwrap().value, data.get(b).unwrap().value)
        });

        PipelineGraph {
            heads,
            vertices,
            data,
        }
    }

    fn sort_vertices_by_source() -> impl Fn(&'a Vertex, &'a Vertex) -> Ordering {
        |a_vertex: &'a Vertex, b_vertex: &'a Vertex| {
            if let Some(a_vertex_source) = a_vertex.meta.as_ref().map(|m| &m.source) {
                if let Some(b_vertex_source) = b_vertex.meta.as_ref().map(|m| &m.source) {
                    let line_order = a_vertex_source.line.cmp(&b_vertex_source.line);
                    if line_order == Ordering::Equal {
                        return a_vertex_source.column.cmp(&b_vertex_source.column);
                    }

                    return line_order;
                }
            }
            Ordering::Equal
        }
    }

    pub(crate) fn process_vertices_edges<'b>(
        graph: &'b PipelineGraph,
        vertex_type: &'b str,
        vertex_id: &'b str,
        next_ident_level: i32,
        next_row_index: Option<i32>,
        visited: &mut RefCell<HashSet<&str>>,
        stack: &mut GraphVisitorStack<'b>,
    ) {
        if let Some(vertices) = graph.vertices.get(vertex_id) {
            if vertex_type == "if" {
                let mut when_vertices = vec![vec![], vec![]];
                let mut other_vertices = vec![];

                for edge in vertices {
                    match edge.when {
                        None => {
                            other_vertices.push(edge);
                        }
                        Some(when) => {
                            if when {
                                when_vertices[1].push(edge);
                            } else {
                                when_vertices[0].push(edge);
                            }
                        }
                    }
                }

                for edge in other_vertices.iter().rev() {
                    if !visited.borrow().contains(edge.vertex_id) {
                        stack.get_mut().push((
                            edge.vertex_id,
                            next_row_index,
                            next_ident_level,
                            false,
                        ));
                    }
                }

                for (i, edge) in when_vertices[0].iter().rev().enumerate() {
                    if !visited.borrow().contains(edge.vertex_id) {
                        if (i + 1) == when_vertices[0].len() {
                            stack.get_mut().push((
                                edge.vertex_id,
                                next_row_index,
                                next_ident_level,
                                true,
                            ));
                        } else {
                            stack.get_mut().push((
                                edge.vertex_id,
                                next_row_index,
                                next_ident_level,
                                false,
                            ));
                        }
                    }
                }

                for edge in when_vertices[1].iter().rev() {
                    if !visited.borrow().contains(edge.vertex_id) {
                        stack.get_mut().push((
                            edge.vertex_id,
                            next_row_index,
                            next_ident_level,
                            false,
                        ));
                    }
                }
            } else {
                for edge in vertices {
                    if !visited.borrow().contains(edge.vertex_id) {
                        stack.get_mut().push((
                            edge.vertex_id,
                            next_row_index,
                            next_ident_level,
                            false,
                        ));
                    }
                }
            }
        }
    }

    pub fn create_pipeline_vertex_ids(&self, selected_pipeline: &PipelineItem) -> Vec<String> {
        let mut visited: RefCell<HashSet<&str>> = RefCell::new(HashSet::with_capacity(
            selected_pipeline.graph.vertices.len(),
        ));
        let mut stack: GraphVisitorStack = RefCell::new(Vec::new());
        let mut head_stack = self.heads.to_vec();
        if head_stack.is_empty() {
            return vec![];
        }

        // Add first head
        let first_head = head_stack.pop().unwrap();
        visited.get_mut().insert(first_head);
        stack.get_mut().push((first_head, None, 0, false));

        let mut table_rows: Vec<String> = Vec::with_capacity(selected_pipeline.graph.edges.len());
        while let Some((vertex_id, mut next_row_index, _, add_else_row)) = stack.get_mut().pop() {
            visited.get_mut().insert(vertex_id);

            let vertex = &self.data.get(vertex_id).unwrap().value;

            let mut push_row = |row| {
                if let Some(i) = next_row_index {
                    table_rows.insert(i as usize, row);
                    next_row_index = Some(i + 1);
                } else {
                    table_rows.push(row);
                }
            };

            if add_else_row {
                push_row(Uuid::new_v4().to_string());
            }

            push_row(vertex.id.to_string());

            Self::process_vertices_edges(
                self,
                &vertex.r#type,
                vertex_id,
                0,
                next_row_index,
                &mut visited,
                &mut stack,
            );

            if stack.borrow().is_empty() && !head_stack.is_empty() {
                while let Some(vertex_id) = head_stack.pop() {
                    stack.get_mut().push((vertex_id, Some(0), 0, false));
                }
            }
        }

        table_rows
    }
}
