use std::cell::RefCell;
use std::collections::HashSet;
use std::vec;

use humansize::{format_size_i, DECIMAL};
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use crate::api::node::Vertex;
use crate::api::stats::PipelineStats;
use crate::commands::formatter::{DurationFormatter, NumberFormatter};
use crate::commands::view::app::App;
use crate::commands::view::app::{AppState, PipelineItem};
use crate::commands::view::flow_metrics_charts::{
    draw_flow_metric_chart, draw_plugin_throughput_flow_chart,
};
use crate::commands::view::pipeline_graph::PipelineGraph;

pub(crate) fn draw_pipelines_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area);

    draw_pipelines_widgets(f, app, chunks[0]);
}

fn draw_pipelines_widgets<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(18), Constraint::Percentage(82)])
        .direction(Direction::Horizontal)
        .split(area);
    {
        draw_pipelines_table(f, app, chunks[0]);
        draw_selected_pipeline_section(f, app, chunks[1]);
    }
}

fn draw_pipelines_table<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let rows: Vec<Row> = app
        .pipelines
        .items
        .iter()
        .map(|i| Row::new(vec![Cell::from(Text::from(i.name.to_string()))]))
        .collect();

    let headers = vec!["Name"];
    let header_style: Style = Style::default()
        .fg(Color::Gray)
        .add_modifier(Modifier::BOLD);

    let row_style: Style = Style::default().bg(Color::DarkGray);
    let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);

    let header_cells = headers.iter().map(|h| Cell::from(*h).style(header_style));
    let header = Row::new(header_cells).style(row_style).height(1);

    let pipelines = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Pipelines"))
        .column_spacing(2)
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Percentage(100), // Name
        ]);

    f.render_stateful_widget(pipelines, area, &mut app.pipelines.state);
}

fn draw_selected_pipeline_section<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(3), Constraint::Percentage(82)])
        .direction(Direction::Vertical)
        .split(area);
    {
        draw_selected_pipeline_events_block(f, app, chunks[0]);
        draw_selected_pipeline_vertices(f, app, chunks[1]);
    }
}

fn draw_selected_pipeline_vertices<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let draw_pipeline_charts =
        app.show_selected_pipeline_charts && app.pipelines.selected_item().is_some();
    let draw_vertex_charts =
        app.show_selected_vertex_details && app.selected_pipeline_vertex.selected_item().is_some();

    let constraints = if draw_pipeline_charts || draw_vertex_charts {
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
            rows = create_selected_pipeline_vertices_rows(
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

        if draw_pipeline_charts {
            draw_selected_pipeline_flow_charts(f, app, chunks[1]);
        } else if draw_vertex_charts {
            draw_selected_pipeline_vertex_details(f, app, pipeline_graph, chunks[1]);
        }
    }
}

fn draw_selected_pipeline_flow_charts<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
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
            draw_plugin_throughput_flow_chart(
                f,
                "Pipeline Throughput",
                flow,
                pipeline_flow_chunks[0],
            );
        }

        if let Some(flow) = app
            .state
            .chart_flow_pipeline_queue_backpressure
            .get(&selected_pipeline.name)
        {
            draw_flow_metric_chart(f, "Queue Backpressure", None, flow, pipeline_flow_chunks[1]);
        }
    }
}

fn draw_selected_pipeline_vertex_details<B>(
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
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::from(vertex_custom_id),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
            Span::raw(vertex_name),
        ]),
    ];

    if let Some(meta) = &vertex.value.meta {
        details_text.push(Line::from(vec![
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
        "plugin" => draw_selected_pipeline_plugin_vertex_details(f, app, vertex.value, chunks[1]),
        "queue" => draw_selected_pipeline_queue_vertex_details(f, app, chunks[1]),
        _ => {}
    }
}

fn draw_selected_pipeline_events_block<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let events_block = Block::default()
        .title("Pipeline events")
        .borders(Borders::ALL);

    let events_text: Line;
    if let Some(selected_pipeline) = &app.pipelines.selected_item() {
        if let Some(node_stats) = &app.state.node_stats {
            let selected_pipeline_stats =
                node_stats.pipelines.get(&selected_pipeline.name).unwrap();

            events_text = Line::from(vec![
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
            ]);
        } else {
            events_text = Line::default();
        }
    } else {
        events_text = Line::from(vec![Span::styled(
            "Select a pipeline",
            Style::default().fg(Color::DarkGray),
        )]);
    }

    let info_paragraph = Paragraph::new(events_text)
        .block(events_block)
        .wrap(Wrap { trim: true });

    f.render_widget(info_paragraph, area);
}

fn create_pipeline_vertex_if_row(vertex: &Vertex, ident_spaces: String) -> Row {
    let if_text = Line::from(vec![
        Span::raw(ident_spaces),
        Span::styled("if ", Style::default().fg(Color::DarkGray)),
        Span::styled(&vertex.condition, Style::default().fg(Color::Gray)),
    ]);

    Row::new(vec![Cell::from(if_text)])
}

fn create_pipeline_vertex_queue_row(
    ident_spaces: String,
    pipeline_stats: Option<&PipelineStats>,
) -> Row {
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

fn create_pipeline_vertex_plugin_row<'a>(
    vertex: &'a Vertex,
    ident_spaces: String,
    pipeline_stats: Option<&'a PipelineStats>,
) -> Row<'a> {
    let plugin_name_cell = if vertex.explicit_id {
        Cell::from(Line::from(vec![
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
        if let Some(events) = stats.vertices.get(vertex.id.as_str()) {
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
    }

    Row::new(cells)
}

fn create_pipeline_vertex_else_row<'a>(ident_spaces: &str) -> Row<'a> {
    let else_text: Line = Line::from(vec![
        Span::raw(ident_spaces[1..].to_string()),
        Span::styled("else", Style::default().fg(Color::DarkGray)),
    ]);

    Row::new(vec![Cell::from(else_text)])
}

type GraphVisitorStack<'b> = RefCell<Vec<(&'b str, Option<i32>, i32, bool)>>;

fn create_selected_pipeline_vertices_rows<'a>(
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
            push_row(create_pipeline_vertex_else_row(&ident_spaces));
        }

        match vertex.r#type.as_str() {
            "if" => {
                push_row(create_pipeline_vertex_if_row(vertex, ident_spaces));
                next_ident_level += 1;
            }
            "queue" => push_row(create_pipeline_vertex_queue_row(
                ident_spaces,
                selected_pipeline_stats,
            )),
            _ => push_row(create_pipeline_vertex_plugin_row(
                vertex,
                ident_spaces,
                selected_pipeline_stats,
            )),
        };

        PipelineGraph::process_vertices_edges(
            graph,
            &vertex.r#type,
            vertex_id,
            next_ident_level,
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

fn draw_selected_pipeline_queue_vertex_details<B>(f: &mut Frame<B>, app: &App, area: Rect)
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
        Line::from(vec![
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
        Line::from(vec![
            Span::styled("Free space: ", Style::default().fg(Color::DarkGray)),
            Span::from(format_size_i(
                pipeline_stats.queue.data.free_space_in_bytes,
                DECIMAL,
            )),
        ]),
        Line::from(vec![
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
        draw_flow_metric_chart(
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
        draw_flow_metric_chart(
            f,
            "Events Growth",
            Some("events"),
            persisted_growth_events,
            chart_chunks[1],
        );
    }
}

fn draw_selected_pipeline_plugin_vertex_details<B>(
    f: &mut Frame<B>,
    app: &App,
    vertex: &Vertex,
    area: Rect,
) where
    B: Backend,
{
    let throughput_state = &app.state.chart_pipeline_vertex_id_state.throughput;
    let worker_utilization_state = &app.state.chart_pipeline_vertex_id_state.worker_utilization;
    let worker_millis_per_event_state = &app
        .state
        .chart_pipeline_vertex_id_state
        .worker_millis_per_event;

    let is_input_plugin = vertex.plugin_type == "input";
    let constraints = if is_input_plugin {
        vec![Constraint::Percentage(100)]
    } else {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    };

    let chart_chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Vertical)
        .split(area);

    if is_input_plugin {
        if let Some(throughput) = throughput_state {
            draw_flow_metric_chart(f, "Throughput", Some("e/s"), throughput, chart_chunks[0]);
        }
    } else {
        if let Some(worker_utilization_state) = worker_utilization_state {
            draw_flow_metric_chart(
                f,
                "Worker Utilization",
                None,
                worker_utilization_state,
                chart_chunks[0],
            );
        }

        if let Some(worker_millis_per_event_state) = worker_millis_per_event_state {
            draw_flow_metric_chart(
                f,
                "Worker Millis Per Event",
                None,
                worker_millis_per_event_state,
                chart_chunks[1],
            );
        }
    }
}
