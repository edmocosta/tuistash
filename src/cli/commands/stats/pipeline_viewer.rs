use std::collections::HashSet;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, Cell, Row, Table};

use crate::api::node::Vertex;
use crate::api::stats::PipelineStats;
use crate::commands::stats::app::{App, AppState, PipelineItem};
use crate::commands::stats::formatter::{DurationFormatter, NumberFormatter};
use crate::commands::stats::graph::PipelineGraph;

pub fn render_pipeline_viewer<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(100)])
        .direction(Direction::Vertical)
        .split(area);
    {
        let selected_pipeline = app.pipelines.selected_item();
        let rows: Vec<Row> = create_rows(selected_pipeline, &app.state);
        let headers = vec![
            "Name",
            "Kind",
            "Events in",
            "Events out",
            "Duration",
            "ID",
        ];

        let header_style: Style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
        let row_style: Style = Style::default().bg(Color::DarkGray);
        let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);

        let header_cells = headers
            .iter()
            .map(|h| Cell::from(*h).style(header_style));

        let header = Row::new(header_cells)
            .style(row_style)
            .height(1);

        let vertices_table = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(create_table_title(selected_pipeline)))
            .column_spacing(2)
            .highlight_style(selected_style)
            .widths(
                &[
                    Constraint::Percentage(40), // Name
                    Constraint::Percentage(8),  // Kind
                    Constraint::Percentage(10), // In
                    Constraint::Percentage(10), // Out
                    Constraint::Percentage(10), // Duration
                    Constraint::Percentage(12), // ID
                ]
            );

        f.render_stateful_widget(vertices_table, chunks[0], &mut app.selected_pipeline_graph.state);
    }
}

fn create_table_title(selected_pipeline: Option<&PipelineItem>) -> &str {
    return match selected_pipeline {
        None => "Pipeline",
        Some(p) => p.name.as_str()
    };
}

fn create_if_row(vertex: &Vertex, ident_spaces: String) -> Row {
    let if_text = vec![Spans::from(
        vec![
            Span::raw(ident_spaces),
            Span::styled("if ", Style::default().fg(Color::DarkGray)),
            Span::styled(&vertex.condition, Style::default().fg(Color::Gray)),
        ])];

    Row::new(vec![Cell::from(if_text)])
}

fn create_queue_row<'a>(vertex: &'a Vertex, ident_spaces: String, pipeline_stats: Option<&'a PipelineStats>) -> Row<'a> {
    let (queue_type, events, push_duration_millis) = match pipeline_stats {
        None => ("-", 0, 0),
        Some(stats) => (stats.queue.r#type.as_str(), stats.queue.events, stats.events.queue_push_duration_in_millis)
    };

    let cells = vec![
        Cell::from(Text::styled(format!("{}{}", ident_spaces, "queue"), Style::default().fg(Color::Cyan))), // Name
        Cell::from(Text::raw(queue_type)), // Kind
        Cell::from(Text::raw(events.format_number())), // Events in
        Cell::from(Text::raw("-")), // Events out
        Cell::from(Text::raw(push_duration_millis.format_duration_per_event(events as u64))), // Duration
        Cell::from(Text::raw(&vertex.id)), // ID
    ];

    Row::new(cells)
}

fn create_plugin_row<'a>(vertex: &'a Vertex, ident_spaces: String, pipeline_stats: Option<&'a PipelineStats>) -> Row<'a> {
    let mut cells = vec![
        Cell::from(Text::from(format!("{}{}", ident_spaces.to_string(), vertex.config_name))),
        Cell::from(Text::from(vertex.plugin_type.to_string())),
    ];

    if let Some(stats) = pipeline_stats {
        let events = stats.vertices.get(vertex.id.as_str()).unwrap();
        let eps_events;
        if vertex.plugin_type == "input" {
            eps_events = events.events_out
        } else {
            eps_events = events.events_in
        }


        cells.push(Cell::from(Text::from(events.events_in.format_number())));
        cells.push(Cell::from(Text::from(events.events_out.format_number())));
        cells.push(Cell::from(Text::from(events.duration_in_millis.format_duration_per_event(eps_events as u64))));
    }

    cells.push(Cell::from(Text::from(vertex.id.to_string())));
    Row::new(cells)
}

fn create_else_row<'a>(ident_spaces: &str) -> Row<'a> {
    let else_text = vec![Spans::from(
        vec![
            Span::raw(ident_spaces[1..].to_string()),
            Span::styled("else", Style::default().fg(Color::DarkGray)),
        ])];

    Row::new(vec![Cell::from(else_text)])
}

fn create_rows<'a>(selected_pipeline_option: Option<&'a PipelineItem>, state: &'a AppState) -> Vec<Row<'a>> {
    if selected_pipeline_option.is_none() {
        return Vec::new();
    }

    let selected_pipeline = selected_pipeline_option.unwrap();
    let selected_pipeline_stats: Option<&PipelineStats> = match &state.node_stats {
        None => None,
        Some(stats) => stats.pipelines.get(selected_pipeline.name.as_str())
    };

    let graph = PipelineGraph::from(&selected_pipeline.graph);
    let mut visited: HashSet<&str> = HashSet::with_capacity(selected_pipeline.graph.vertices.len());
    let mut stack: Vec<(&str, i32, bool)> = Vec::new();

    //TODO: Handle multiple inputs
    for input_id in &graph.inputs {
        stack.push((input_id, 0, false));
        visited.insert(input_id);
    }

    let mut table_rows: Vec<Row> = Vec::with_capacity(selected_pipeline.graph.edges.len());
    while let Some((vertex_id, ident_level, add_else_row)) = stack.pop() {
        visited.insert(vertex_id);

        let vertex = graph.data.get(vertex_id).unwrap().value;
        let ident_spaces = std::iter::repeat(" ").take(ident_level as usize).collect::<String>();
        let mut next_ident_level = ident_level;

        if add_else_row {
            table_rows.push(create_else_row(&ident_spaces));
        }

        match vertex.r#type.as_str() {
            "if" => {
                table_rows.push(create_if_row(vertex, ident_spaces));
                next_ident_level += 1;
            }
            "queue" => table_rows.push(create_queue_row(vertex, ident_spaces, selected_pipeline_stats)),
            _ => table_rows.push(create_plugin_row(vertex, ident_spaces, selected_pipeline_stats)),
        };

        if let Some(vertices) = graph.vertices.get(vertex_id) {
            if vertex.r#type == "if" {
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
                    if !visited.contains(edge.vertex_id) {
                        stack.push((edge.vertex_id, next_ident_level, false));
                    }
                }

                for (i, edge) in when_vertices[0].iter().rev().enumerate() {
                    if !visited.contains(edge.vertex_id) {
                        if (i + 1) == when_vertices[0].len() {
                            stack.push((edge.vertex_id, next_ident_level, true));
                        } else {
                            stack.push((edge.vertex_id, next_ident_level, false));
                        }
                    }
                }

                for edge in when_vertices[1].iter().rev() {
                    if !visited.contains(edge.vertex_id) {
                        stack.push((edge.vertex_id, next_ident_level, false));
                    }
                }
            } else {
                for edge in vertices {
                    if !visited.contains(edge.vertex_id) {
                        stack.push((edge.vertex_id, next_ident_level, false));
                    }
                }
            }
        }
    }

    return table_rows;
}