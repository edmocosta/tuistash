use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::vec;

use humansize::{format_size_i, DECIMAL};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;
use serde_json::Value;

use crate::api::node::Vertex;
use crate::api::stats::PipelineStats;
use crate::commands::formatter::{DurationFormatter, NumberFormatter};
use crate::commands::tui::app::{App, AppData};
use crate::commands::tui::flow_charts::{
    draw_flow_metric_chart, draw_plugin_throughput_flow_chart,
};
use crate::commands::tui::pipelines::graph::PipelineGraph;
use crate::commands::tui::pipelines::state::PipelineTableItem;
use crate::commands::tui::widgets::{
    TABLE_HEADER_CELL_STYLE, TABLE_HEADER_ROW_STYLE, TABLE_SELECTED_ROW_STYLE,
    TABLE_SELECTED_ROW_SYMBOL,
};

pub(crate) fn draw_pipelines_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area);

    draw_pipelines_widgets(f, app, chunks[0]);
}

pub(crate) fn pipelines_tab_shortcuts_help(_: &App) -> HashMap<String, String> {
    let mut keys = HashMap::with_capacity(2);
    keys.insert(
        "[↵]".to_string(),
        "open pipeline charts/vertex details".to_string(),
    );
    keys.insert("[C]".to_string(), "pipeline charts".to_string());
    keys
}

fn draw_pipelines_widgets(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(18), Constraint::Percentage(82)])
        .direction(Direction::Horizontal)
        .split(area);
    {
        draw_pipelines_table(f, app, chunks[0]);
        draw_selected_pipeline_section(f, app, chunks[1]);
    }
}

fn draw_pipelines_table(f: &mut Frame, app: &mut App, area: Rect) {
    let rows: Vec<Row> = app
        .pipelines_state
        .pipelines_table
        .items
        .iter()
        .map(|i| Row::new(vec![Cell::from(Text::from(i.name.to_string()))]))
        .collect();

    let headers = ["Name"];
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(TABLE_HEADER_CELL_STYLE));

    let header = Row::new(header_cells)
        .style(TABLE_HEADER_ROW_STYLE)
        .height(1);

    let pipelines_count = rows.len();
    let pipelines = Table::new(rows, vec![Constraint::Percentage(100)])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Pipelines({})", pipelines_count)),
        )
        .column_spacing(2)
        .row_highlight_style(TABLE_SELECTED_ROW_STYLE)
        .highlight_symbol(TABLE_SELECTED_ROW_SYMBOL);

    f.render_stateful_widget(
        pipelines,
        area,
        &mut app.pipelines_state.pipelines_table.state,
    );
}

fn draw_selected_pipeline_section(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(3), Constraint::Percentage(97)])
        .direction(Direction::Vertical)
        .split(area);
    {
        draw_selected_pipeline_events_block(f, app, chunks[0]);
        draw_selected_pipeline_vertices(f, app, chunks[1]);
    }
}

fn draw_selected_pipeline_vertices(f: &mut Frame, app: &mut App, area: Rect) {
    let draw_pipeline_charts = app.pipelines_state.show_selected_pipeline_charts
        && app
            .pipelines_state
            .pipelines_table
            .selected_item()
            .is_some();
    let draw_vertex_charts = app.pipelines_state.show_selected_vertex_details
        && app.pipelines_state.selected_pipeline_vertex().is_some();

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

        if let Some(selected_pipeline) = app.pipelines_state.pipelines_table.selected_item() {
            pipeline_graph = Some(PipelineGraph::from(&selected_pipeline.graph));
            rows = create_selected_pipeline_vertices_rows(
                pipeline_graph.as_ref().unwrap(),
                selected_pipeline,
                &app.data,
            );
        } else {
            pipeline_graph = None;
            rows = vec![];
        }

        let headers = ["Name", "Kind", "Events in", "Events out", "Duration (ms/e)"];
        let header_cells = headers
            .iter()
            .map(|h| Cell::from(*h).style(TABLE_HEADER_CELL_STYLE));

        let header = Row::new(header_cells)
            .style(TABLE_HEADER_ROW_STYLE)
            .height(1);

        let widths: Vec<Constraint> = vec![
            Constraint::Percentage(49), // Name
            Constraint::Percentage(8),  // Kind
            Constraint::Percentage(11), // In
            Constraint::Percentage(15), // Out
            Constraint::Percentage(18), // Duration
        ];

        let vertices_table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Pipeline"))
            .column_spacing(2)
            .row_highlight_style(TABLE_SELECTED_ROW_STYLE)
            .highlight_symbol(TABLE_SELECTED_ROW_SYMBOL);

        f.render_stateful_widget(
            vertices_table,
            chunks[0],
            &mut app.pipelines_state.selected_pipeline_vertex.state,
        );

        if draw_pipeline_charts {
            draw_selected_pipeline_flow_charts(f, app, chunks[1]);
        } else if draw_vertex_charts {
            draw_selected_pipeline_vertex_details(f, app, pipeline_graph, chunks[1]);
        }
    }
}

fn draw_selected_pipeline_flow_charts(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_pipeline = app.pipelines_state.selected_pipeline_name();
    if selected_pipeline.is_none() {
        return;
    }

    if let Some(selected_pipeline_state) = app
        .shared_state
        .pipeline_flows_chart_state(selected_pipeline.unwrap())
        .map(|p| &p.pipeline)
    {
        let pipeline_flow_chunks = Layout::default()
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .direction(Direction::Vertical)
            .split(area);

        if !selected_pipeline_state.plugins_throughput.is_empty() {
            draw_plugin_throughput_flow_chart(
                f,
                "Pipeline Throughput",
                &selected_pipeline_state.plugins_throughput,
                pipeline_flow_chunks[0],
            );
        }

        if !selected_pipeline_state.queue_backpressure.is_empty() {
            draw_flow_metric_chart(
                f,
                "Queue Backpressure",
                None,
                &selected_pipeline_state.queue_backpressure,
                pipeline_flow_chunks[1],
                false,
            );
        }
    }
}

fn draw_selected_pipeline_vertex_details(
    f: &mut Frame,
    app: &App,
    pipeline_graph: Option<PipelineGraph>,
    area: Rect,
) {
    let main_block = Block::default().borders(Borders::ALL).title("Details");

    f.render_widget(main_block, area);

    if pipeline_graph.is_none()
        || app
            .pipelines_state
            .selected_pipeline_vertex
            .selected_item()
            .is_none()
    {
        return;
    }

    let vertex_id = app
        .pipelines_state
        .selected_pipeline_vertex
        .selected_item()
        .unwrap();

    let vertex = pipeline_graph
        .as_ref()
        .unwrap()
        .data
        .get(vertex_id.as_str());

    if vertex.is_none() {
        return;
    }

    let unknown_vertex_id = "-".to_string();
    let vertex = vertex.unwrap();

    let (vertex_id, vertex_name) = if vertex.value.r#type.as_str() != "plugin" {
        (&unknown_vertex_id, &vertex.value.r#type)
    } else {
        (&vertex.value.id, &vertex.value.config_name)
    };

    let mut details_text = vec![
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::from(vertex_id),
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

    let details_text_len = w.line_count(area.width) as u16;
    let chunks = Layout::default()
        .constraints([
            Constraint::Length(details_text_len),
            Constraint::Percentage(96),
        ])
        .margin(1)
        .direction(Direction::Vertical)
        .split(area);

    f.render_widget(w, chunks[0]);

    match vertex.value.r#type.as_str() {
        "plugin" => draw_selected_pipeline_plugin_vertex_details(f, app, vertex.value, chunks[1]),
        "queue" => draw_selected_pipeline_queue_vertex_details(f, app, chunks[1]),
        _ => {}
    }
}

fn draw_selected_pipeline_events_block(f: &mut Frame, app: &mut App, area: Rect) {
    let events_block = Block::default()
        .title("Pipeline events")
        .borders(Borders::ALL);

    let events_text: Line;
    if let Some(selected_pipeline) = &app.pipelines_state.pipelines_table.selected_item() {
        if let Some(node_stats) = &app.data.node_stats() {
            let selected_pipeline_stats =
                node_stats.pipelines.get(&selected_pipeline.name).unwrap();

            let duration_text = format!(
                "{} ({} ms/e)",
                selected_pipeline_stats.events.duration_in_millis,
                selected_pipeline_stats
                    .events
                    .duration_in_millis
                    .format_duration_per_event(selected_pipeline_stats.events.out as u64)
                    .trim()
            );

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
                Span::styled("Duration (ms): ", Style::default().fg(Color::DarkGray)),
                Span::from(duration_text),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Workers: ", Style::default().fg(Color::DarkGray)),
                Span::from(
                    get_pipeline_workers(app, &selected_pipeline.name)
                        .map(|p| p.to_string())
                        .unwrap_or("-".to_string()),
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

fn get_pipeline_workers(app: &App, pipeline_name: &str) -> Option<i64> {
    if let Some(node_info) = app.data.node_info() {
        if let Some(pipelines) = &node_info.pipelines {
            return pipelines
                .get(pipeline_name)
                .map(|p| p.workers)
                .or(Some(node_info.node.pipeline.workers));
        }
    }
    None
}

fn create_pipeline_vertex_if_row(vertex: &Vertex, ident_spaces: String) -> Row {
    let if_text = Line::from(vec![
        Span::raw(ident_spaces),
        Span::styled("if ", Style::default().fg(Color::Red)),
        Span::styled(&vertex.condition, Style::default().fg(Color::DarkGray)),
    ]);

    Row::new(vec![Cell::from(if_text)])
}

fn create_pipeline_vertex_queue_row(
    ident_spaces: String,
    pipeline_stats: Option<&PipelineStats>,
) -> Row {
    let (queue_type, events_in, events_out, queue_push_duration, backpressure) =
        match pipeline_stats {
            None => ("-", 0, 0, 0, None),
            Some(stats) => (
                stats.queue.r#type.as_str(),
                stats.events.r#in,
                stats.events.r#in - stats.queue.events,
                stats.events.queue_push_duration_in_millis,
                Some(&stats.flow.queue_backpressure),
            ),
        };

    let duration_without_backpressure =
        Span::raw(queue_push_duration.format_duration_per_event(events_in as u64));
    let duration_spans = if let Some(backpressure_metric) = backpressure {
        if backpressure_metric.current >= 0.01 {
            vec![
                Span::raw(format!(
                    "{} ",
                    queue_push_duration.format_duration_per_event(events_in as u64)
                )),
                Span::styled("BP(", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    backpressure_metric.current.strip_number_decimals(2),
                    Style::default().fg(Color::Red),
                ),
                Span::styled(")", Style::default().fg(Color::DarkGray)),
            ]
        } else {
            vec![duration_without_backpressure]
        }
    } else {
        vec![duration_without_backpressure]
    };

    let cells = vec![
        Cell::from(Text::styled(
            format!("{}{}", ident_spaces, "queue"),
            Style::default().fg(Color::Cyan),
        )), // Name
        Cell::from(Text::raw(queue_type)),                 // Kind
        Cell::from(Text::raw(events_in.format_number())),  // Events in
        Cell::from(Text::raw(events_out.format_number())), // Events out
        Cell::from(Line::from(duration_spans)),            // Duration
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
        if let Some(vertex_stats) = stats.vertices.get(vertex.id.as_str()) {
            let is_input_plugin = vertex.plugin_type == "input";
            let vertex_events_in = if is_input_plugin {
                vertex_stats.events_out
            } else {
                vertex_stats.events_in
            };

            if is_input_plugin {
                let mut empty_events_in = true;
                if let Some(plugin) = stats.plugins.get(&vertex.id) {
                    if let Some(plugin_flow) = &plugin.flow {
                        if let Some(plugin_throughput) = &plugin_flow.throughput {
                            empty_events_in = false;
                            cells.push(Cell::from(Line::from(vec![Span::styled(
                                format!("{} e/s", plugin_throughput.current.format_number().trim()),
                                Style::default().fg(Color::Blue),
                            )])));
                        }
                    }
                }
                if empty_events_in {
                    cells.push(Cell::from(Text::from("-")));
                }
            } else {
                cells.push(Cell::from(Text::from(vertex_events_in.format_number())));
            }

            // Drop percentage
            let events_in_out_diff = vertex_stats.events_out - vertex_events_in;
            match events_in_out_diff.cmp(&0) {
                Ordering::Less => {
                    let drop_percentage = 100.00
                        - ((vertex_stats.events_out as f64 / vertex_events_in as f64) * 100.0);
                    let drop_percentage_text = if drop_percentage > 0.01 {
                        format!(" {}% ↓", drop_percentage.strip_number_decimals(2))
                    } else {
                        "".to_string()
                    };

                    let drop_percentage_color =
                        if vertex.plugin_type == "filter" && vertex.config_name == "drop" {
                            Color::DarkGray
                        } else {
                            Color::Yellow
                        };

                    cells.push(Cell::from(Line::from(vec![
                        Span::raw(vertex_stats.events_out.format_number()),
                        Span::styled(
                            drop_percentage_text,
                            Style::default().fg(drop_percentage_color),
                        ),
                    ])));
                }
                Ordering::Equal | Ordering::Greater => {
                    cells.push(Cell::from(Text::from(
                        vertex_stats.events_out.format_number(),
                    )));
                }
            }

            // Duration
            let (duration_in_millis, total_duration_in_millis) = if is_input_plugin {
                (
                    vertex_stats.queue_push_duration_in_millis,
                    stats.events.queue_push_duration_in_millis,
                )
            } else {
                (
                    vertex_stats.duration_in_millis,
                    stats.events.duration_in_millis,
                )
            };

            let mut duration_spans = vec![Span::raw(
                duration_in_millis.format_duration_per_event(vertex_events_in as u64),
            )];

            if !is_input_plugin {
                let mut duration_percentage =
                    (duration_in_millis as f64 / total_duration_in_millis as f64) * 100.0;
                if duration_percentage.is_nan() || duration_percentage.is_infinite() {
                    duration_percentage = 0.0
                }

                if duration_percentage >= 0.01 || (duration_in_millis > 0 && vertex_events_in > 0) {
                    let avg_plugins_percentage = stats
                        .plugins
                        .avg_duration_in_millis_percentage(total_duration_in_millis);

                    let duration_percentage_color: Color =
                        if (duration_percentage - 10.0) > avg_plugins_percentage {
                            Color::Yellow
                        } else {
                            Color::DarkGray
                        };

                    duration_spans.push(Span::styled(
                        format!(" {}%", duration_percentage.strip_number_decimals(2)),
                        Style::default().fg(duration_percentage_color),
                    ));
                }
            }

            cells.push(Cell::from(Line::from(duration_spans)));
        }
    }

    Row::new(cells)
}

fn create_pipeline_vertex_else_row<'a>(ident_spaces: &str) -> Row<'a> {
    let else_text: Line = Line::from(vec![
        Span::raw(ident_spaces[1..].to_string()),
        Span::styled("else", Style::default().fg(Color::Red)),
    ]);

    Row::new(vec![Cell::from(else_text)])
}

type GraphVisitorStack<'b> = RefCell<Vec<(&'b str, Option<i32>, i32, bool)>>;

fn create_selected_pipeline_vertices_rows<'a>(
    graph: &'a PipelineGraph,
    selected_pipeline: &PipelineTableItem,
    data: &'a AppData,
) -> Vec<Row<'a>> {
    let selected_pipeline_stats: Option<&PipelineStats> = match &data.node_stats() {
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

fn draw_selected_pipeline_queue_vertex_details(f: &mut Frame, app: &App, area: Rect) {
    if app.data.node_stats().is_none()
        || app
            .pipelines_state
            .pipelines_table
            .selected_item()
            .is_none()
    {
        return;
    }

    let selected_pipeline = app.pipelines_state.pipelines_table.selected_item().unwrap();
    let node_stats = app.data.node_stats().unwrap();

    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(2), Constraint::Percentage(98)])
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
                "Size: {}, Max: {}, Free: {}",
                format_size_i(pipeline_stats.queue.capacity.queue_size_in_bytes, DECIMAL),
                format_size_i(
                    pipeline_stats.queue.capacity.max_queue_size_in_bytes,
                    DECIMAL,
                ),
                format_size_i(pipeline_stats.queue.data.free_space_in_bytes, DECIMAL,)
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
        .constraints(vec![
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .direction(Direction::Vertical)
        .split(chunks[1]);

    if let Some(selected_pipeline_state) = app
        .shared_state
        .pipeline_flows_chart_state(&selected_pipeline.name)
        .map(|p| &p.pipeline)
    {
        if !selected_pipeline_state.queue_backpressure.is_empty() {
            draw_flow_metric_chart(
                f,
                "Backpressure",
                None,
                &selected_pipeline_state.queue_backpressure,
                chart_chunks[0],
                false,
            );
        }

        if !selected_pipeline_state
            .queue_persisted_growth_bytes
            .is_empty()
        {
            draw_flow_metric_chart(
                f,
                "Bytes Growth",
                Some("bytes"),
                &selected_pipeline_state.queue_persisted_growth_bytes,
                chart_chunks[1],
                false,
            );
        }

        if !selected_pipeline_state
            .queue_persisted_growth_events
            .is_empty()
        {
            draw_flow_metric_chart(
                f,
                "Events Growth",
                Some("events"),
                &selected_pipeline_state.queue_persisted_growth_events,
                chart_chunks[2],
                false,
            );
        }
    }
}

fn draw_selected_pipeline_plugin_vertex_details(
    f: &mut Frame,
    app: &App,
    vertex: &Vertex,
    area: Rect,
) {
    let selected_pipeline = app.pipelines_state.selected_pipeline_name();
    let selected_vertex = app.pipelines_state.selected_pipeline_vertex();

    if selected_pipeline.is_none() || selected_vertex.is_none() {
        return;
    }

    let node_stats = app.data.node_stats();
    if node_stats.is_none() {
        return;
    }

    let pipeline_stats = node_stats
        .unwrap()
        .pipelines
        .get(selected_pipeline.unwrap());
    if pipeline_stats.is_none() {
        return;
    }

    let (custom_details, constraints) = if let Some(p) =
        get_selected_pipeline_plugin_vertex_custom_details(vertex, pipeline_stats.unwrap())
    {
        let custom_details_len = p.line_count(area.width);
        (
            Some(p),
            vec![
                Constraint::Length(custom_details_len as u16),
                Constraint::Percentage(100),
            ],
        )
    } else {
        (None, vec![Constraint::Percentage(100)])
    };

    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Vertical)
        .split(area);

    if let Some(custom_details) = &custom_details {
        f.render_widget(custom_details, chunks[0]);
    }

    // Charts and custom widgets
    draw_selected_pipeline_plugin_vertex_widgets(
        f,
        app,
        vertex,
        selected_pipeline.unwrap(),
        selected_vertex.unwrap(),
        chunks[chunks.len() - 1],
    );
}

fn draw_selected_pipeline_plugin_vertex_widgets(
    f: &mut Frame,
    app: &App,
    vertex: &Vertex,
    selected_pipeline: &String,
    selected_vertex: &String,
    area: Rect,
) {
    match vertex.plugin_type.as_str() {
        "input" => draw_selected_pipeline_input_plugin_widgets(
            f,
            app,
            vertex,
            selected_pipeline,
            selected_vertex,
            area,
        ),
        _ => draw_selected_pipeline_worker_plugin_widgets(
            f,
            app,
            selected_pipeline,
            selected_vertex,
            area,
        ),
    }
}

fn draw_selected_pipeline_worker_plugin_widgets(
    f: &mut Frame,
    app: &App,
    selected_pipeline: &String,
    selected_vertex: &String,
    area: Rect,
) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .direction(Direction::Vertical)
        .split(area);

    let flow_state = app
        .shared_state
        .pipeline_plugin_flows_chart_state(selected_pipeline, selected_vertex);

    let worker_utilization_state = flow_state.map(|p| &p.worker_utilization);
    let worker_millis_per_event_state = flow_state.map(|p| &p.worker_millis_per_event);

    if let Some(worker_utilization_state) = worker_utilization_state {
        if !worker_utilization_state.is_empty() {
            draw_flow_metric_chart(
                f,
                "Worker Utilization",
                None,
                worker_utilization_state,
                chunks[0],
                false,
            );
        }
    }

    if let Some(worker_millis_per_event_state) = worker_millis_per_event_state {
        if !worker_millis_per_event_state.is_empty() {
            draw_flow_metric_chart(
                f,
                "Worker Millis Per Event",
                None,
                worker_millis_per_event_state,
                chunks[1],
                false,
            );
        }
    }
}

fn draw_selected_pipeline_input_plugin_widgets(
    f: &mut Frame,
    app: &App,
    vertex: &Vertex,
    selected_pipeline: &String,
    selected_vertex: &String,
    area: Rect,
) {
    let constraints = if vertex.config_name == "pipeline" {
        vec![Constraint::Percentage(70), Constraint::Percentage(30)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Vertical)
        .split(area);

    let throughput_state = app
        .shared_state
        .pipeline_plugin_flows_chart_state(selected_pipeline, selected_vertex)
        .map(|p| &p.throughput);

    if let Some(throughput) = throughput_state {
        if !throughput.is_empty() {
            draw_flow_metric_chart(f, "Throughput", Some("e/s"), throughput, chunks[0], true);
        }
    }

    if vertex.config_name == "pipeline" {
        draw_selected_pipeline_input_pipeline_plugin_widgets(
            f,
            app,
            vertex,
            selected_pipeline,
            chunks[1],
        )
    }
}

fn draw_selected_pipeline_input_pipeline_plugin_widgets(
    f: &mut Frame,
    app: &App,
    vertex: &Vertex,
    selected_pipeline: &String,
    area: Rect,
) {
    let mut plugin_option = None;
    if let Some(node_stats) = app.data.node_stats() {
        if let Some(pipeline_stats) = node_stats.pipelines.get(selected_pipeline) {
            if let Some(p) = pipeline_stats.plugins.inputs.get(&vertex.id) {
                plugin_option = Some(p);
            }
        }
    }
    if plugin_option.is_none() {
        return;
    }

    let plugin = plugin_option.unwrap();
    if let Some(listen_address) = plugin.get_other("address", |v| v.as_str(), None) {
        let mut writing_pipelines: Vec<(&String, i64)> = vec![];
        for (pipeline_id, stats) in &app.data.node_stats().unwrap().pipelines {
            let mut has_producers = false;
            let pipeline_plugins_events_out = stats
                .plugins
                .outputs
                .iter()
                .filter(|(_, source_plugin)| {
                    if source_plugin.name.as_ref().is_some_and(|v| v != "pipeline") {
                        return false;
                    }
                    if let Some(send_to) =
                        source_plugin.get_other("send_to", |p| p.as_array(), None)
                    {
                        let addresses: Vec<&str> =
                            send_to.iter().map(|v| v.as_str().unwrap_or("-")).collect();
                        if addresses.contains(&listen_address) {
                            has_producers = true;
                            return true;
                        }
                    }
                    false
                })
                .map(|(_id, plugin)| plugin.events.out)
                .sum();

            if has_producers {
                writing_pipelines.push((pipeline_id, pipeline_plugins_events_out));
            }
        }

        writing_pipelines.sort_by(|a, b| b.1.cmp(&a.1));

        let headers = ["Pipeline", "Events"];
        let header_cells = headers
            .iter()
            .map(|h| Cell::from(*h).style(TABLE_HEADER_CELL_STYLE));

        let header = Row::new(header_cells)
            .style(TABLE_HEADER_ROW_STYLE)
            .height(1);

        let widths: Vec<Constraint> = vec![
            Constraint::Percentage(75), // Pipeline
            Constraint::Percentage(25), // Events
        ];

        let rows: Vec<Row> = writing_pipelines
            .iter()
            .map(|(pipeline_id, events_out)| {
                Row::new(vec![
                    Cell::from(Text::from(Line::from(pipeline_id.to_string()))),
                    Cell::from(Text::from(Line::from(events_out.format_number()))),
                ])
            })
            .collect();

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Top upstream producers"),
            )
            .column_spacing(2)
            .row_highlight_style(TABLE_SELECTED_ROW_STYLE)
            .highlight_symbol(TABLE_SELECTED_ROW_SYMBOL);

        f.render_widget(table, area);
    }
}

fn get_selected_pipeline_plugin_vertex_custom_details<'a>(
    vertex: &'a Vertex,
    stats: &'a PipelineStats,
) -> Option<Paragraph<'a>> {
    let mut custom_fields = vec![];

    // elasticsearch {}
    if vertex.config_name == "elasticsearch" && vertex.plugin_type == "output" {
        if let Some(plugin) = stats.plugins.outputs.get(&vertex.id) {
            let int_value_mapper: fn(&Value) -> i64 = |p| p.as_i64().unwrap_or(0);

            if plugin.other.contains_key("documents") {
                let successes = plugin.get_other("documents.successes", int_value_mapper, 0);
                let failures =
                    plugin.get_other("documents.non_retryable_failures", int_value_mapper, 0);
                let dlq_routed = plugin.get_other("documents.dlq_routed", int_value_mapper, 0);

                custom_fields.push(Line::from(vec![
                    Span::styled("Documents: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!(
                        "OK: {}, Errors: {}, DLQ: {}",
                        successes, failures, dlq_routed
                    )),
                ]));
            }

            if plugin.other.contains_key("bulk_requests") {
                let successes = plugin.get_other("bulk_requests.successes", int_value_mapper, 0);
                let with_errors =
                    plugin.get_other("bulk_requests.with_errors", int_value_mapper, 0);
                let failures = plugin.get_other("bulk_requests.failures", int_value_mapper, 0);

                custom_fields.push(Line::from(vec![
                    Span::styled("Bulk Req.: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!(
                        "OK: {}, w/Errors: {}, Fails: {}",
                        successes, with_errors, failures
                    )),
                ]));
            }
        }
    }

    // pipeline {}
    if vertex.config_name == "pipeline" {
        if vertex.plugin_type == "input" {
            if let Some(plugin) = stats.plugins.inputs.get(&vertex.id) {
                if let Some(address) = plugin.get_other("address", |v| v.as_str(), None) {
                    custom_fields.push(Line::from(vec![
                        Span::styled("Address: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(address),
                    ]));
                }
            }
        }

        if vertex.plugin_type == "output" {
            if let Some(plugin) = stats.plugins.outputs.get(&vertex.id) {
                if let Some(send_to) = plugin.get_other("send_to", |v| v.as_array(), None) {
                    let addresses: Vec<&str> =
                        send_to.iter().map(|v| v.as_str().unwrap_or("-")).collect();

                    custom_fields.push(Line::from(vec![
                        Span::styled("Send to: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(addresses.join(", ")),
                    ]));
                }
            }
        }
    }

    if custom_fields.is_empty() {
        return None;
    }

    Some(
        Paragraph::new(custom_fields)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true }),
    )
}
