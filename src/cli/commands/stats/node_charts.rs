use humansize::{format_size_i, ToF64, DECIMAL};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType};
use tui::{symbols, Frame};

use crate::commands::stats::app::App;
use crate::commands::stats::charts::{
    create_binary_size_label_spans, create_percentage_label_spans, create_timestamp_label_spans,
    DEFAULT_LABELS_COUNT,
};
use crate::commands::stats::flow_charts::{render_flow_chart, render_plugins_flow_chart};

pub(crate) fn render_node_charts<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let main_chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);

    let flow_chart_chunks = Layout::default()
        .constraints(
            [
                Constraint::Min(3),
                Constraint::Percentage(55),
                Constraint::Percentage(45),
            ]
            .as_ref(),
        )
        .split(main_chunks[0]);

    render_process_file_descriptor_gauge(f, app, flow_chart_chunks[0]);
    render_plugins_flow_chart(
        f,
        "Throughput",
        &app.state.chart_flow_plugins_throughput,
        flow_chart_chunks[1],
    );
    render_flow_chart(
        f,
        "Queue Backpressure",
        &app.state.chart_flow_queue_backpressure,
        flow_chart_chunks[2],
    );

    let node_chart_chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    render_jvm_heap_chart(f, app, node_chart_chunks[0]);
    render_jvm_non_heap_chart(f, app, node_chart_chunks[1]);
    render_process_cpu_chart(f, app, node_chart_chunks[2]);
}

fn render_jvm_heap_chart<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let mut heap_max_data: Vec<(f64, f64)> = vec![];
    let mut heap_used_data: Vec<(f64, f64)> = vec![];

    for data in &app.state.chart_jvm_heap_state.data_points {
        heap_max_data.push((data.timestamp.to_f64(), data.heap_max_in_bytes.to_f64()));
        heap_used_data.push((data.timestamp.to_f64(), data.heap_used_in_bytes.to_f64()));
    }

    let current_max_heap = app
        .state
        .chart_jvm_heap_state
        .data_points
        .front()
        .map(|p| p.heap_max_in_bytes)
        .unwrap_or(0);

    let current_used_heap = app
        .state
        .chart_jvm_heap_state
        .data_points
        .front()
        .map(|p| p.heap_used_in_bytes)
        .unwrap_or(0);

    let datasets = vec![
        Dataset::default()
            .name(format!("Max: {}", format_size_i(current_max_heap, DECIMAL)))
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Magenta))
            .graph_type(GraphType::Line)
            .data(&heap_max_data),
        Dataset::default()
            .name(format!(
                "Used: {}",
                format_size_i(current_used_heap, DECIMAL)
            ))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Blue))
            .data(&heap_used_data),
    ];

    let chart = Chart::new(datasets)
        .hidden_legend_constraints((Constraint::Percentage(90), Constraint::Percentage(90)))
        .block(
            Block::default()
                .title(Span::raw("JVM Heap"))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(app.state.chart_jvm_heap_state.x_axis_bounds().clone())
                .labels(create_timestamp_label_spans(
                    app.state
                        .chart_jvm_heap_state
                        .x_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(app.state.chart_jvm_heap_state.y_axis_bounds().clone())
                .labels(create_binary_size_label_spans(
                    app.state
                        .chart_jvm_heap_state
                        .y_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        );

    f.render_widget(chart, area);
}

fn render_jvm_non_heap_chart<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let mut non_heap_used: Vec<(f64, f64)> = vec![];
    let mut non_heap_committed: Vec<(f64, f64)> = vec![];

    for data in &app.state.chart_jvm_non_heap_state.data_points {
        non_heap_used.push((
            data.timestamp.to_f64(),
            data.non_heap_used_in_bytes.to_f64(),
        ));
        non_heap_committed.push((
            data.timestamp.to_f64(),
            data.non_heap_committed_in_bytes.to_f64(),
        ));
    }

    let current_heap_used = app
        .state
        .chart_jvm_non_heap_state
        .data_points
        .front()
        .map(|p| p.non_heap_used_in_bytes)
        .unwrap_or(0);

    let current_heap_committed = app
        .state
        .chart_jvm_non_heap_state
        .data_points
        .front()
        .map(|p| p.non_heap_committed_in_bytes)
        .unwrap_or(0);

    let datasets = vec![
        Dataset::default()
            .name(format!(
                "Used: {}",
                format_size_i(current_heap_used, DECIMAL)
            ))
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Magenta))
            .graph_type(GraphType::Line)
            .data(&non_heap_used),
        Dataset::default()
            .name(format!(
                "Committed: {}",
                format_size_i(current_heap_committed, DECIMAL)
            ))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Blue))
            .data(&non_heap_committed),
    ];

    let chart = Chart::new(datasets)
        .hidden_legend_constraints((Constraint::Percentage(90), Constraint::Percentage(90)))
        .block(
            Block::default()
                .title(Span::raw("JVM Non Heap"))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(app.state.chart_jvm_non_heap_state.x_axis_bounds().clone())
                .labels(create_timestamp_label_spans(
                    app.state
                        .chart_jvm_non_heap_state
                        .x_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(app.state.chart_jvm_non_heap_state.y_axis_bounds().clone())
                .labels(create_binary_size_label_spans(
                    app.state
                        .chart_jvm_non_heap_state
                        .y_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        );

    f.render_widget(chart, area);
}

fn render_process_cpu_chart<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let cpu_percentage_data: Vec<(f64, f64)> = app
        .state
        .chart_process_cpu
        .data_points
        .iter()
        .map(|p| (p.timestamp.to_f64(), p.percent.to_f64()))
        .collect();

    let current_value = app
        .state
        .chart_process_cpu
        .data_points
        .front()
        .map(|p| p.percent)
        .unwrap_or(0);

    let datasets = vec![Dataset::default()
        .name(format!("Usage: {}%", current_value))
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Blue))
        .data(&cpu_percentage_data)];

    let chart = Chart::new(datasets)
        .hidden_legend_constraints((Constraint::Percentage(90), Constraint::Percentage(90)))
        .block(
            Block::default()
                .title(Span::raw("CPU Usage"))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(app.state.chart_process_cpu.x_axis_bounds().clone())
                .labels(create_timestamp_label_spans(
                    app.state
                        .chart_process_cpu
                        .x_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(app.state.chart_process_cpu.y_axis_bounds().clone())
                .labels(create_percentage_label_spans(
                    app.state
                        .chart_process_cpu
                        .y_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        );

    f.render_widget(chart, area);
}

fn render_process_file_descriptor_gauge<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let usage: u16;
    let max_file_descriptors;
    let open_file_descriptors;

    if let Some(stats) = &app.state.node_stats {
        open_file_descriptors = stats.process.open_file_descriptors;
        max_file_descriptors = stats.process.max_file_descriptors;
        usage = ((open_file_descriptors * 100) / max_file_descriptors) as u16;
    } else {
        open_file_descriptors = 0;
        max_file_descriptors = 0;
        usage = 0;
    }

    let label = format!("{}/{}", open_file_descriptors, max_file_descriptors);
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(Span::raw("OS File Descriptors"))
                .borders(Borders::ALL),
        )
        .gauge_style(
            Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        )
        .percent(usage)
        .label(label)
        .use_unicode(true);

    f.render_widget(gauge, area);
}
