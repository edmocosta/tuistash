use std::cell::RefCell;
use std::collections::HashSet;
use std::vec;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use uuid::Uuid;

use crate::api::node::Vertex;
use crate::api::stats::PipelineStats;
use crate::commands::stats::app::{App, AppState, PipelineItem};
use crate::commands::stats::flow_charts::{render_flow_chart, render_plugins_flow_chart};
use crate::commands::stats::formatter::{DurationFormatter, NumberFormatter};
use crate::commands::stats::graph::PipelineGraph;

pub(crate) fn render_pipeline_viewer<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{
    let render_pipeline_charts =
        app.show_selected_pipeline_charts && app.pipelines.selected_item().is_some();
    let render_vertex_charts =
        app.show_selected_plugin_charts && app.selected_pipeline_graph.selected_item().is_some();

    let constraints = if render_pipeline_charts || render_vertex_charts {
        vec![Constraint::Percentage(70), Constraint::Percentage(30)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Horizontal)
        .split(area);
    {
        let rows: Vec<Row>;
        let pipeline_graph: Option<PipelineGraph>;

        if let Some(selected_pipeline) = app.pipelines.selected_item() {
            pipeline_graph = Some(PipelineGraph::from(&selected_pipeline.graph));
            rows = create_rows(
                &pipeline_graph.as_ref().unwrap(),
                selected_pipeline,
                &app.state,
            );
        } else {
            pipeline_graph = None;
            rows = vec![];
        }

        let headers = vec!["Name", "Kind", "Events in", "Events out", "Duration (ms/e)"];
        let header_style: Style = Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD);
        let row_style: Style = Style::default().bg(Color::DarkGray);
        let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);

        let header_cells = headers.iter().map(|h| Cell::from(*h).style(header_style));

        let header = Row::new(header_cells).style(row_style).height(1);

        let vertices_table = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Pipeline"))
            .column_spacing(2)
            .highlight_style(selected_style)
            .widths(&[
                Constraint::Percentage(50), // Name
                Constraint::Percentage(7),  // Kind
                Constraint::Percentage(14),  // In
                Constraint::Percentage(14),  // Out
                Constraint::Percentage(15), // Duration
            ]);

        f.render_stateful_widget(
            vertices_table,
            chunks[0],
            &mut app.selected_pipeline_graph.state,
        );

        if render_pipeline_charts {
            render_selected_pipeline_flow_charts(f, app, chunks[1]);
        } else if render_vertex_charts {
            render_selected_vertex_details(f, app, pipeline_graph, chunks[1]);
        }
    }
}

fn create_if_row(vertex: &Vertex, ident_spaces: String) -> Row {
    let if_text = vec![Spans::from(vec![
        Span::raw(ident_spaces),
        Span::styled("if ", Style::default().fg(Color::DarkGray)),
        Span::styled(&vertex.condition, Style::default().fg(Color::Gray)),
    ])];

    Row::new(vec![Cell::from(if_text)])
}

fn create_queue_row(ident_spaces: String, pipeline_stats: Option<&PipelineStats>) -> Row {
    let (queue_type, events, push_duration_millis) = match pipeline_stats {
        None => ("-", 0, 0),
        Some(stats) => (
            stats.queue.r#type.as_str(),
            stats.queue.events,
            stats.events.queue_push_duration_in_millis,
        ),
    };

    let cells = vec![
        Cell::from(Text::styled(
            format!("{}{}", ident_spaces, "queue"),
            Style::default().fg(Color::Cyan),
        )), // Name
        Cell::from(Text::raw(queue_type)),             // Kind
        Cell::from(Text::raw(events.format_number())), // Events in
        Cell::from(Text::raw("-")),                    // Events out
        Cell::from(Text::raw(
            push_duration_millis.format_duration_per_event(events as u64),
        )), // Duration
    ];

    Row::new(cells)
}

fn create_plugin_row<'a>(
    vertex: &'a Vertex,
    ident_spaces: String,
    pipeline_stats: Option<&'a PipelineStats>,
) -> Row<'a> {
    let plugin_name_cell;
    if vertex.explicit_id {
        plugin_name_cell = Cell::from(Spans::from(vec![
            Span::from(format!(
                "{}{} ",
                ident_spaces.to_string(),
                vertex.config_name
            )),
            Span::styled(
                format!("({})", vertex.id.as_str()),
                Style::default().fg(Color::Blue),
            ),
        ]));
    } else {
        plugin_name_cell = Cell::from(Text::from(format!(
            "{}{}",
            ident_spaces.to_string(),
            vertex.config_name
        )));
    }

    let mut cells = vec![
        plugin_name_cell,
        Cell::from(Text::from(vertex.plugin_type.to_string())),
    ];

    if let Some(stats) = pipeline_stats {
        let events = stats.vertices.get(vertex.id.as_str()).unwrap();
        let events_count;
        if vertex.plugin_type == "input" {
            events_count = events.events_out
        } else {
            events_count = events.events_in
        }

        cells.push(Cell::from(Text::from(events.events_in.format_number())));
        cells.push(Cell::from(Text::from(events.events_out.format_number())));
        cells.push(Cell::from(Text::from(
            events
                .duration_in_millis
                .format_duration_per_event(events_count as u64),
        )));
    }

    Row::new(cells)
}

fn create_else_row<'a>(ident_spaces: &str) -> Row<'a> {
    let else_text = vec![Spans::from(vec![
        Span::raw(ident_spaces[1..].to_string()),
        Span::styled("else", Style::default().fg(Color::DarkGray)),
    ])];

    Row::new(vec![Cell::from(else_text)])
}

fn create_rows<'a>(
    graph: &'a PipelineGraph,
    selected_pipeline: &PipelineItem,
    state: &'a AppState,
) -> Vec<Row<'a>> {
    let selected_pipeline_stats: Option<&PipelineStats> = match &state.node_stats {
        None => None,
        Some(stats) => stats.pipelines.get(selected_pipeline.name.as_str()),
    };

    let mut visited: RefCell<HashSet<&str>> = RefCell::new(HashSet::with_capacity(
        selected_pipeline.graph.vertices.len(),
    ));
    let mut stack: RefCell<Vec<(&str, i32, bool)>> = RefCell::new(Vec::new());

    //TODO: Handle multiple inputs
    for input_id in &graph.inputs {
        stack.get_mut().push((input_id, 0, false));
        visited.get_mut().insert(input_id);
    }

    let mut table_rows: Vec<Row> = Vec::with_capacity(selected_pipeline.graph.edges.len());
    while let Some((vertex_id, ident_level, add_else_row)) = stack.get_mut().pop() {
        visited.get_mut().insert(vertex_id);

        let vertex = graph.data.get(vertex_id).unwrap().value;
        let ident_spaces = std::iter::repeat(" ")
            .take(ident_level as usize)
            .collect::<String>();
        let mut next_ident_level = ident_level;

        if add_else_row {
            table_rows.push(create_else_row(&ident_spaces));
        }

        match vertex.r#type.as_str() {
            "if" => {
                table_rows.push(create_if_row(vertex, ident_spaces));
                next_ident_level += 1;
            }
            "queue" => table_rows.push(create_queue_row(ident_spaces, selected_pipeline_stats)),
            _ => table_rows.push(create_plugin_row(
                vertex,
                ident_spaces,
                selected_pipeline_stats,
            )),
        };

        process_vertices_edges(
            &vertex.r#type,
            vertex_id,
            next_ident_level,
            graph,
            &mut visited,
            &mut stack,
        );
    }

    return table_rows;
}

pub fn create_rows_ids<'a>(
    graph: &'a PipelineGraph,
    selected_pipeline: &PipelineItem,
) -> Vec<String> {
    let mut visited: RefCell<HashSet<&str>> = RefCell::new(HashSet::with_capacity(
        selected_pipeline.graph.vertices.len(),
    ));
    let mut stack: RefCell<Vec<(&str, i32, bool)>> = RefCell::new(Vec::new());

    //TODO: Handle multiple inputs
    for input_id in &graph.inputs {
        stack.get_mut().push((input_id, 0, false));
        visited.get_mut().insert(input_id);
    }

    let mut table_rows: Vec<String> = Vec::with_capacity(selected_pipeline.graph.edges.len());
    while let Some((vertex_id, _, add_else_row)) = stack.get_mut().pop() {
        visited.get_mut().insert(vertex_id);

        let vertex = graph.data.get(vertex_id).unwrap().value;

        if add_else_row {
            table_rows.push(Uuid::new_v4().to_string());
        }

        table_rows.push(vertex.id.to_string());
        process_vertices_edges(
            &vertex.r#type,
            vertex_id,
            0,
            graph,
            &mut visited,
            &mut stack,
        );
    }

    return table_rows;
}

fn process_vertices_edges<'a>(
    vertex_type: &'a str,
    vertex_id: &'a str,
    next_ident_level: i32,
    graph: &'a PipelineGraph,
    visited: &mut RefCell<HashSet<&str>>,
    stack: &mut RefCell<Vec<(&'a str, i32, bool)>>,
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
                    stack
                        .get_mut()
                        .push((edge.vertex_id, next_ident_level, false));
                }
            }

            for (i, edge) in when_vertices[0].iter().rev().enumerate() {
                if !visited.borrow().contains(edge.vertex_id) {
                    if (i + 1) == when_vertices[0].len() {
                        stack
                            .get_mut()
                            .push((edge.vertex_id, next_ident_level, true));
                    } else {
                        stack
                            .get_mut()
                            .push((edge.vertex_id, next_ident_level, false));
                    }
                }
            }

            for edge in when_vertices[1].iter().rev() {
                if !visited.borrow().contains(edge.vertex_id) {
                    stack
                        .get_mut()
                        .push((edge.vertex_id, next_ident_level, false));
                }
            }
        } else {
            for edge in vertices {
                if !visited.borrow().contains(edge.vertex_id) {
                    stack
                        .get_mut()
                        .push((edge.vertex_id, next_ident_level, false));
                }
            }
        }
    }
}

fn render_selected_pipeline_flow_charts<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
    where
        B: Backend,
{
    let pipeline_flow_chunks = Layout::default()
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .direction(Direction::Vertical)
        .split(area);

    if let Some(selected_pipeline) = app.pipelines.selected_item() {
        if let Some(flow) = app
            .state
            .chart_flow_pipeline_plugins_throughput
            .get(&selected_pipeline.name)
        {
            render_plugins_flow_chart(f, "Pipeline Throughput", flow, pipeline_flow_chunks[0]);
        }

        if let Some(flow) = app
            .state
            .chart_flow_pipeline_queue_backpressure
            .get(&selected_pipeline.name)
        {
            render_flow_chart(f, "Queue Backpressure", flow, pipeline_flow_chunks[1]);
        }
    }
}

fn render_selected_vertex_details<B>(
    f: &mut Frame<B>,
    app: &App,
    pipeline_graph: Option<PipelineGraph>,
    area: Rect,
) where
    B: Backend,
{
    let main_block = Block::default().borders(Borders::ALL).title("Details");

    f.render_widget(main_block, area);

    if pipeline_graph.is_none() || app.selected_pipeline_graph.selected_item().is_none() {
        return;
    }

    let chunks = Layout::default()
        .constraints([Constraint::Length(6), Constraint::Percentage(90)])
        .margin(1)
        .direction(Direction::Vertical)
        .split(area);

    let vertex_id = app.selected_pipeline_graph.selected_item().unwrap();
    let vertex = pipeline_graph
        .as_ref()
        .unwrap()
        .data
        .get(vertex_id.as_str());
    if vertex.is_none() {
        return;
    }

    let vertex = vertex.unwrap();
    let mut status_text = vec![
        Spans::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::from(vertex.value.id.to_string()),
        ]),
        Spans::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
            Span::from(vertex.value.config_name.to_string()),
        ]),
    ];

    if let Some(meta) = &vertex.value.meta {
        status_text.push(Spans::from(vec![
            Span::styled("Source: ", Style::default().fg(Color::DarkGray)),
            Span::from(format!("{}:{}", meta.source.line, meta.source.column)),
        ]))
    }

    let w = Paragraph::new(status_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(w, chunks[0]);

    // Charts
    let throughput_state = &app.state.chart_pipeline_vertex_id_state.throughput;
    let worker_utilization_state = &app.state.chart_pipeline_vertex_id_state.worker_utilization;

    let constraints = if throughput_state.is_some() && worker_utilization_state.is_some() {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let mut next_chunk_index = 0;
    let chart_chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Vertical)
        .split(chunks[1]);

    if let Some(throughput) = throughput_state {
        render_flow_chart(f, "Throughput", &throughput, chart_chunks[next_chunk_index]);
        next_chunk_index += 1;
    }

    if let Some(worker_utilization_state) = worker_utilization_state {
        render_flow_chart(
            f,
            "Worker Utilization",
            &worker_utilization_state,
            chart_chunks[next_chunk_index],
        );
    }
}