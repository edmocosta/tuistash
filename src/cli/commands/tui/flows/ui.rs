use std::cmp::Ordering;
use std::collections::HashMap;
use std::string::ToString;
use std::vec;

use crate::api::node::Vertex;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, Tabs};
use ratatui::Frame;

use crate::api::stats::FlowMetricValue;
use crate::commands::formatter::NumberFormatter;
use crate::commands::tui::app::App;
use crate::commands::tui::widgets::{
    TABLE_HEADER_CELL_STYLE, TABLE_HEADER_ROW_STYLE, TABLE_SELECTED_ROW_STYLE,
    TABLE_SELECTED_ROW_SYMBOL,
};

const ANALYSIS_WINDOW_10_SEC: usize = 1;
const ANALYSIS_WINDOW_1_MIN: usize = 2;
const ANALYSIS_WINDOW_5_MIN: usize = 3;
const ANALYSIS_WINDOW_15_MIN: usize = 4;
const ANALYSIS_WINDOW_1_HOUR: usize = 5;
const ANALYSIS_WINDOW_24_HOUR: usize = 6;
const DEFAULT_COLORS: (Color, Color, Color) = (Color::Red, Color::Reset, Color::Green);
const REVERSED_COLORS: (Color, Color, Color) = (Color::Green, Color::Reset, Color::Red);

pub(crate) fn draw_flows_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area);

    draw_flow_widgets(f, app, chunks[0]);
}

pub(crate) fn flows_tab_shortcuts_help(_: &App) -> HashMap<String, String> {
    let mut keys = HashMap::with_capacity(4);
    keys.insert("[↵]".to_string(), "plugins flow".to_string());
    keys.insert("[1-6]".to_string(), "analysis window".to_string());
    keys.insert("[V]".to_string(), "diff as number/%".to_string());
    keys.insert("[L]".to_string(), "lifetimes".to_string());
    keys
}

fn draw_flow_widgets(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(2), Constraint::Min(0)])
        .direction(Direction::Vertical)
        .split(area);

    draw_analysis_window_options(f, app, chunks[0]);

    let tables_constraints = if app.flows_state.show_selected_plugin {
        vec![Constraint::Percentage(30), Constraint::Percentage(70)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let tables_chunks = Layout::default()
        .constraints(tables_constraints)
        .direction(Direction::Horizontal)
        .split(chunks[1]);

    draw_pipelines_table(f, app, tables_chunks[0]);

    if app.flows_state.show_selected_plugin {
        draw_selected_pipeline_widgets(f, app, tables_chunks[1]);
    }
}

fn draw_analysis_window_options(f: &mut Frame, app: &mut App, area: Rect) {
    let options: Vec<&str> = vec![
        "Compare most recent: ",
        "~10 secs",
        "~1 min",
        "~5 min",
        "~15 min",
        "~1 h",
        "~24 h",
        "with lifetime (Σ)",
    ];

    let tab_options: Vec<Line> = options
        .iter()
        .map(|value| {
            Line::from(vec![Span::styled(
                value.to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            )])
        })
        .collect();

    let tabs = Tabs::new(tab_options)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .select(app.flows_state.analysis_window_tabs.index);

    f.render_widget(tabs, area);
}

pub fn create_flow_metric_cell<'a>(
    metric: &Option<FlowMetricValue>,
    analysis_window: &usize,
    percentage: bool,
    lifetimes: bool,
    colors: (Color, Color, Color),
) -> Cell<'a> {
    let empty_cell = Cell::new(Text::from("-"));
    if let Some(metric) = metric {
        let value_op = match *analysis_window {
            ANALYSIS_WINDOW_10_SEC => Some(metric.current),
            ANALYSIS_WINDOW_1_MIN => metric.last_1_minute,
            ANALYSIS_WINDOW_5_MIN => metric.last_5_minutes,
            ANALYSIS_WINDOW_15_MIN => metric.last_15_minutes,
            ANALYSIS_WINDOW_1_HOUR => metric.last_1_hour,
            ANALYSIS_WINDOW_24_HOUR => metric.last_24_hours,
            _ => None,
        };

        if value_op.is_none() {
            return empty_cell;
        }

        let value = value_op.unwrap();
        let difference = if value.is_infinite() || metric.lifetime.is_infinite() {
            f64::INFINITY
        } else if percentage {
            let diff = 100.0 * (value - metric.lifetime) / ((value + metric.lifetime) / 2.0);
            if diff.is_nan() {
                0.0
            } else {
                diff
            }
        } else {
            value - metric.lifetime
        };

        let (color, icon) = match difference.total_cmp(&0.0) {
            Ordering::Less => (colors.0, '↓'),
            Ordering::Equal => (colors.1, ' '),
            Ordering::Greater => (colors.2, '↑'),
        };

        let decimals = 4;
        let mut difference_text = if percentage {
            difference.format_number_with_decimals(2)
        } else {
            difference.format_number_with_decimals(decimals)
        };

        if percentage {
            difference_text.insert(difference_text.len(), '%');
        }

        let mut spans = vec![
            Span::styled(
                value.format_number_with_decimals(decimals),
                Style::default(),
            ),
            Span::raw(" "),
            Span::styled(difference_text, Style::default().fg(color)),
            Span::styled(
                icon.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ];

        if lifetimes {
            spans.insert(
                2,
                Span::styled(
                    metric.lifetime.format_number_with_decimals(decimals) + "Σ ",
                    Style::default().fg(Color::Reset),
                ),
            );
        }

        Cell::from(Text::from(Line::from(spans)))
    } else {
        empty_cell
    }
}

fn draw_pipelines_table(f: &mut Frame, app: &mut App, area: Rect) {
    let show_as_percentage = app.flows_state.show_as_percentage;
    let show_lifetimes = app.flows_state.show_lifetime_values;
    let hide_flow_cells = app.flows_state.show_selected_plugin;

    let rows: Vec<Row> = app
        .flows_state
        .pipelines_flow_table
        .items
        .iter()
        .map(|i| {
            let flow_metric_cells = if hide_flow_cells {
                vec![]
            } else {
                vec![
                    create_flow_metric_cell(
                        &i.input_throughput,
                        &app.flows_state.analysis_window_tabs.index,
                        show_as_percentage,
                        show_lifetimes,
                        DEFAULT_COLORS,
                    ),
                    create_flow_metric_cell(
                        &i.filter_throughput,
                        &app.flows_state.analysis_window_tabs.index,
                        show_as_percentage,
                        show_lifetimes,
                        DEFAULT_COLORS,
                    ),
                    create_flow_metric_cell(
                        &i.output_throughput,
                        &app.flows_state.analysis_window_tabs.index,
                        show_as_percentage,
                        show_lifetimes,
                        DEFAULT_COLORS,
                    ),
                    create_flow_metric_cell(
                        &i.queue_backpressure,
                        &app.flows_state.analysis_window_tabs.index,
                        show_as_percentage,
                        show_lifetimes,
                        REVERSED_COLORS,
                    ),
                    create_flow_metric_cell(
                        &i.worker_concurrency,
                        &app.flows_state.analysis_window_tabs.index,
                        show_as_percentage,
                        show_lifetimes,
                        REVERSED_COLORS,
                    ),
                ]
            };

            let mut cells = Vec::with_capacity(flow_metric_cells.len() + 2);
            cells.push(Cell::from(Text::from(i.name.to_string())));
            cells.push(Cell::from(Text::from(i.workers.to_string())));
            cells.extend(flow_metric_cells);

            Row::new(cells)
        })
        .collect();

    let headers = if hide_flow_cells {
        vec!["Name", "Workers"]
    } else {
        vec![
            "Name",
            "Workers",
            "Input",
            "Filter",
            "Output",
            "Queue Backpressure",
            "Worker Concurrency ▼",
        ]
    };

    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(TABLE_HEADER_CELL_STYLE));

    let header = Row::new(header_cells)
        .style(TABLE_HEADER_ROW_STYLE)
        .height(1);

    let widths: Vec<Constraint> = vec![Constraint::Ratio(rows.len() as u32, 1); rows.len()];
    let (name_cell_constraint, workers_cell_constraint) = if hide_flow_cells {
        (Constraint::Percentage(80), Constraint::Percentage(20))
    } else {
        (Constraint::Percentage(15), Constraint::Percentage(5))
    };

    let pipelines = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Pipelines"))
        .column_spacing(2)
        .highlight_style(TABLE_SELECTED_ROW_STYLE)
        .highlight_symbol(TABLE_SELECTED_ROW_SYMBOL)
        .widths([
            name_cell_constraint,       // Name
            workers_cell_constraint,    // Workers
            Constraint::Percentage(13), // Input
            Constraint::Percentage(13), // Filter
            Constraint::Percentage(13), // Output
            Constraint::Percentage(13), // Queue Backpressure
            Constraint::Percentage(13), // Worker Concurrency
        ]);

    f.render_stateful_widget(
        pipelines,
        area,
        &mut app.flows_state.pipelines_flow_table.state,
    );
}

fn create_plugin_name_cell<'a>(id: &String, vertex: Option<&&Vertex>) -> Cell<'a> {
    if let Some(vertex) = vertex {
        return if vertex.explicit_id {
            Cell::from(Line::from(vec![
                Span::raw(vertex.config_name.to_string()),
                Span::styled(
                    format!(" ({})", vertex.id.as_str()),
                    Style::default().fg(Color::Blue),
                ),
            ]))
        } else {
            Cell::from(Span::raw(vertex.config_name.to_string()))
        };
    }

    Cell::from(Span::raw(id.to_string()))
}

fn draw_selected_pipeline_widgets(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
        .direction(Direction::Vertical)
        .split(area);

    draw_selected_pipeline_input_plugins(f, app, chunks[0]);
    draw_selected_pipeline_other_plugins(f, app, chunks[1]);
}

fn draw_selected_pipeline_input_plugins(f: &mut Frame, app: &mut App, area: Rect) {
    let show_as_percentage = app.flows_state.show_as_percentage;
    let show_lifetimes = app.flows_state.show_lifetime_values;
    let pipeline_vertices = &app.flows_state.selected_pipeline_vertices(&app.data);

    let rows: Vec<Row> = app
        .flows_state
        .input_plugins_flow_table
        .items
        .iter()
        .map(|i| {
            Row::new(vec![
                create_plugin_name_cell(&i.id, pipeline_vertices.get(&i.id)),
                Cell::from(Text::from("input")),
                create_flow_metric_cell(
                    &i.throughput,
                    &app.flows_state.analysis_window_tabs.index,
                    show_as_percentage,
                    show_lifetimes,
                    DEFAULT_COLORS,
                ),
            ])
        })
        .collect();

    let headers = ["Name", "Type", "Throughput ▼"];
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(TABLE_HEADER_CELL_STYLE));

    let header = Row::new(header_cells)
        .style(TABLE_HEADER_ROW_STYLE)
        .height(1);

    let widths: Vec<Constraint> = vec![Constraint::Ratio(rows.len() as u32, 1); rows.len()];
    let pipelines = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Inputs"))
        .column_spacing(2)
        .highlight_style(TABLE_SELECTED_ROW_STYLE)
        .highlight_symbol(TABLE_SELECTED_ROW_SYMBOL)
        .widths([
            Constraint::Percentage(30), // Name
            Constraint::Percentage(30), // Type
            Constraint::Percentage(40), // Throughput
        ]);

    f.render_stateful_widget(
        pipelines,
        area,
        &mut app.flows_state.input_plugins_flow_table.state,
    );
}

fn draw_selected_pipeline_other_plugins(f: &mut Frame, app: &mut App, area: Rect) {
    let show_as_percentage = app.flows_state.show_as_percentage;
    let show_lifetimes = app.flows_state.show_lifetime_values;
    let pipeline_vertices = &app.flows_state.selected_pipeline_vertices(&app.data);

    let rows: Vec<Row> = app
        .flows_state
        .other_plugins_flow_table
        .items
        .iter()
        .map(|i| {
            Row::new(vec![
                create_plugin_name_cell(&i.id, pipeline_vertices.get(&i.id)),
                Cell::from(Text::raw(&i.plugin_type)),
                create_flow_metric_cell(
                    &i.worker_millis_per_event,
                    &app.flows_state.analysis_window_tabs.index,
                    show_as_percentage,
                    show_lifetimes,
                    REVERSED_COLORS,
                ),
                create_flow_metric_cell(
                    &i.worker_utilization,
                    &app.flows_state.analysis_window_tabs.index,
                    show_as_percentage,
                    show_lifetimes,
                    REVERSED_COLORS,
                ),
            ])
        })
        .collect();

    let headers = [
        "Name",
        "Type",
        "Worker millis per event",
        "Worker utilization ▼",
    ];

    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(TABLE_HEADER_CELL_STYLE));

    let header = Row::new(header_cells)
        .style(TABLE_HEADER_ROW_STYLE)
        .height(1);

    let widths: Vec<Constraint> = vec![Constraint::Ratio(rows.len() as u32, 1); rows.len()];
    let pipelines = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Plugins"))
        .column_spacing(2)
        .highlight_style(TABLE_SELECTED_ROW_STYLE)
        .highlight_symbol(TABLE_SELECTED_ROW_SYMBOL)
        .widths([
            Constraint::Percentage(40), // Name
            Constraint::Percentage(10), // Type
            Constraint::Percentage(25), // Worker millis per event
            Constraint::Percentage(25), // Worker utilization
        ]);

    f.render_stateful_widget(
        pipelines,
        area,
        &mut app.flows_state.other_plugins_flow_table.state,
    );
}
