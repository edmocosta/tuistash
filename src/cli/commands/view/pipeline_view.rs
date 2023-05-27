use humansize::{format_size_i, DECIMAL};
use std::cell::RefCell;
use std::collections::HashSet;
use std::vec;

use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use tui::Frame;
use uuid::Uuid;

use crate::api::node::Vertex;
use crate::api::stats::PipelineStats;
use crate::commands::view::app::{App, AppState, PipelineItem};
use crate::commands::view::flow_charts::{render_flow_chart, render_plugins_flow_chart};
use crate::commands::view::formatter::{DurationFormatter, NumberFormatter};
use crate::commands::view::graph::PipelineGraph;

pub(crate) fn render_pipeline_vertices<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let render_pipeline_charts =
        app.show_selected_pipeline_charts && app.pipelines.selected_item().is_some();
    let render_vertex_charts =
        app.show_selected_vertex_details && app.selected_pipeline_vertex.selected_item().is_some();

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
                pipeline_graph.as_ref().unwrap(),
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
                Constraint::Percentage(49), // Name
                Constraint::Percentage(8),  // Kind
                Constraint::Percentage(11), // In
                Constraint::Percentage(11), // Out
                Constraint::Percentage(21), // Duration
            ]);

        f.render_stateful_widget(
            vertices_table,
            chunks[0],
            &mut app.selected_pipeline_vertex.state,
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
    let plugin_name_cell = if vertex.explicit_id {
        Cell::from(Spans::from(vec![
            Span::from(format!("{}{} ", ident_spaces, vertex.config_name)),
            Span::styled(
                format!("({})", vertex.id.as_str()),
                Style::default().fg(Color::Blue),
            ),
        ]))
    } else {
        Cell::from(Text::from(format!(
            "{}{}",
            ident_spaces, vertex.config_name
        )))
    };

    let mut cells = vec![
        plugin_name_cell,
        Cell::from(Text::from(vertex.plugin_type.to_string())),
    ];

    if let Some(stats) = pipeline_stats {
        let events = stats.vertices.get(vertex.id.as_str()).unwrap();
        let events_count = if vertex.plugin_type == "input" {
            events.events_out
        } else {
            events.events_in
        };

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

type GraphVisitorStack<'b> = RefCell<Vec<(&'b str, Option<i32>, i32, bool)>>;

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
    let mut stack: GraphVisitorStack = RefCell::new(Vec::new());

    let mut head_stack = graph.heads.to_vec();
    if head_stack.is_empty() {
        return vec![];
    }

    // Add first head
    let first_head = head_stack.pop().unwrap();
    visited.get_mut().insert(first_head);
    stack.get_mut().push((first_head, None, 0, false));

    let mut table_rows: Vec<Row> = Vec::new();
    while let Some((vertex_id, mut next_row_index, ident_level, add_else_row)) =
        stack.get_mut().pop()
    {
        visited.get_mut().insert(vertex_id);

        let vertex = graph.data.get(vertex_id).unwrap().value;
        let ident_spaces = " ".repeat(ident_level as usize);
        let mut next_ident_level = ident_level;

        let mut push_row = |row| {
            if let Some(i) = next_row_index {
                table_rows.insert(i as usize, row);
                next_row_index = Some(i + 1);
            } else {
                table_rows.push(row);
            }
        };

        if add_else_row {
            push_row(create_else_row(&ident_spaces));
        }

        match vertex.r#type.as_str() {
            "if" => {
                push_row(create_if_row(vertex, ident_spaces));
                next_ident_level += 1;
            }
            "queue" => push_row(create_queue_row(ident_spaces, selected_pipeline_stats)),
            _ => push_row(create_plugin_row(
                vertex,
                ident_spaces,
                selected_pipeline_stats,
            )),
        };

        process_vertices_edges(
            &vertex.r#type,
            vertex_id,
            next_ident_level,
            next_row_index,
            graph,
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

pub fn create_pipeline_vertex_ids(
    graph: &PipelineGraph,
    selected_pipeline: &PipelineItem,
) -> Vec<String> {
    let mut visited: RefCell<HashSet<&str>> = RefCell::new(HashSet::with_capacity(
        selected_pipeline.graph.vertices.len(),
    ));
    let mut stack: GraphVisitorStack = RefCell::new(Vec::new());
    let mut head_stack = graph.heads.to_vec();
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

        let vertex = graph.data.get(vertex_id).unwrap().value;

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

        process_vertices_edges(
            &vertex.r#type,
            vertex_id,
            0,
            next_row_index,
            graph,
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

fn process_vertices_edges<'a>(
    vertex_type: &'a str,
    vertex_id: &'a str,
    next_ident_level: i32,
    next_row_index: Option<i32>,
    graph: &'a PipelineGraph,
    visited: &mut RefCell<HashSet<&str>>,
    stack: &mut GraphVisitorStack<'a>,
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
                        .push((edge.vertex_id, next_row_index, next_ident_level, false));
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
                    stack
                        .get_mut()
                        .push((edge.vertex_id, next_row_index, next_ident_level, false));
                }
            }
        } else {
            for edge in vertices {
                if !visited.borrow().contains(edge.vertex_id) {
                    stack
                        .get_mut()
                        .push((edge.vertex_id, next_row_index, next_ident_level, false));
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
            render_flow_chart(f, "Queue Backpressure", None, flow, pipeline_flow_chunks[1]);
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

    if pipeline_graph.is_none() || app.selected_pipeline_vertex.selected_item().is_none() {
        return;
    }

    let vertex_id = app.selected_pipeline_vertex.selected_item().unwrap();
    let vertex = pipeline_graph
        .as_ref()
        .unwrap()
        .data
        .get(vertex_id.as_str());
    if vertex.is_none() {
        return;
    }

    let vertex = vertex.unwrap();
    let details_text_constraint = if vertex.value.meta.is_some() {
        Constraint::Length(3)
    } else {
        Constraint::Length(2)
    };

    let chunks = Layout::default()
        .constraints([details_text_constraint, Constraint::Percentage(96)])
        .margin(1)
        .direction(Direction::Vertical)
        .split(area);

    let vertex_custom_id: &str = if vertex.value.explicit_id {
        &vertex.value.id
    } else {
        "-"
    };

    let vertex_name = if vertex.value.r#type.as_str() != "plugin" {
        &vertex.value.r#type
    } else {
        &vertex.value.config_name
    };

    let mut details_text = vec![
        Spans::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::from(vertex_custom_id),
        ]),
        Spans::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
            Span::raw(vertex_name),
        ]),
    ];

    if let Some(meta) = &vertex.value.meta {
        details_text.push(Spans::from(vec![
            Span::styled("Source: ", Style::default().fg(Color::DarkGray)),
            Span::from(format!(
                "{} - position {}:{}",
                meta.source.id, meta.source.line, meta.source.column
            )),
        ]))
    }

    let w = Paragraph::new(details_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(w, chunks[0]);

    // Charts
    match vertex.value.r#type.as_str() {
        "plugin" => render_selected_plugin_vertex_details(f, app, chunks[1]),
        "queue" => render_queue_vertex_details(f, app, chunks[1]),
        _ => {}
    }
}

fn render_queue_vertex_details<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    if app.state.node_stats.is_none() || app.pipelines.selected_item().is_none() {
        return;
    }

    let selected_pipeline = app.pipelines.selected_item().unwrap();
    let node_stats = app.state.node_stats.as_ref().unwrap();

    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(3), Constraint::Percentage(97)])
        .direction(Direction::Vertical)
        .split(area);

    let pipeline_stats = node_stats.pipelines.get(&selected_pipeline.name);
    if pipeline_stats.is_none() {
        return;
    }

    let pipeline_stats = pipeline_stats.unwrap();
    let queue_details = vec![
        Spans::from(vec![
            Span::styled("Capacity: ", Style::default().fg(Color::DarkGray)),
            Span::from(format!(
                "Size: {}, Max: {}",
                format_size_i(pipeline_stats.queue.capacity.queue_size_in_bytes, DECIMAL),
                format_size_i(
                    pipeline_stats.queue.capacity.max_queue_size_in_bytes,
                    DECIMAL,
                )
            )),
        ]),
        Spans::from(vec![
            Span::styled("Free space: ", Style::default().fg(Color::DarkGray)),
            Span::from(format_size_i(
                pipeline_stats.queue.data.free_space_in_bytes,
                DECIMAL,
            )),
        ]),
        Spans::from(vec![
            Span::styled("Events: ", Style::default().fg(Color::DarkGray)),
            Span::from(pipeline_stats.queue.events.format_number_with_decimals(2)),
        ]),
    ];

    let w = Paragraph::new(queue_details)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(w, chunks[0]);

    // Charts
    let chart_chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .direction(Direction::Vertical)
        .split(chunks[1]);

    if let Some(persisted_growth_bytes) = app
        .state
        .chart_flow_pipeline_queue_persisted_growth_bytes
        .get(&selected_pipeline.name)
    {
        render_flow_chart(
            f,
            "Bytes Growth",
            Some("bytes"),
            persisted_growth_bytes,
            chart_chunks[0],
        );
    }

    if let Some(persisted_growth_events) = app
        .state
        .chart_flow_pipeline_queue_persisted_growth_events
        .get(&selected_pipeline.name)
    {
        render_flow_chart(
            f,
            "Events Growth",
            Some("events"),
            persisted_growth_events,
            chart_chunks[1],
        );
    }
}

fn render_selected_plugin_vertex_details<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
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
        .split(area);

    if let Some(throughput) = throughput_state {
        render_flow_chart(
            f,
            "Throughput",
            Some("e/s"),
            throughput,
            chart_chunks[next_chunk_index],
        );
        next_chunk_index += 1;
    }

    if let Some(worker_utilization_state) = worker_utilization_state {
        render_flow_chart(
            f,
            "Worker Utilization",
            None,
            worker_utilization_state,
            chart_chunks[next_chunk_index],
        );
    }
}

pub(crate) fn render_pipeline_events_block<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let events_block = Block::default()
        .title("Pipeline events")
        .borders(Borders::ALL);

    let events_text: Vec<Spans>;
    if let Some(selected_pipeline) = &app.pipelines.selected_item() {
        if let Some(node_stats) = &app.state.node_stats {
            let selected_pipeline_stats =
                node_stats.pipelines.get(&selected_pipeline.name).unwrap();

            events_text = vec![Spans::from(vec![
                Span::styled("In: ", Style::default().fg(Color::DarkGray)),
                Span::from(selected_pipeline_stats.events.r#in.format_number()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Filtered: ", Style::default().fg(Color::DarkGray)),
                Span::from(selected_pipeline_stats.events.filtered.format_number()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Out: ", Style::default().fg(Color::DarkGray)),
                Span::from(selected_pipeline_stats.events.out.format_number()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "Queue push duration (ms/e): ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::from(
                    selected_pipeline_stats
                        .events
                        .queue_push_duration_in_millis
                        .format_duration_per_event(selected_pipeline_stats.events.r#in as u64),
                ),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Duration (ms/e): ", Style::default().fg(Color::DarkGray)),
                Span::from(
                    selected_pipeline_stats
                        .events
                        .duration_in_millis
                        .format_duration_per_event(selected_pipeline_stats.events.out as u64),
                ),
            ])];
        } else {
            events_text = vec![];
        }
    } else {
        events_text = vec![Spans::from(vec![Span::styled(
            "Select a pipeline",
            Style::default().fg(Color::DarkGray),
        )])];
    }

    let info_paragraph = Paragraph::new(events_text)
        .block(events_block)
        .wrap(Wrap { trim: true });
    f.render_widget(info_paragraph, area);
}
