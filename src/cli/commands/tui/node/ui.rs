use humansize::{format_size_i, DECIMAL};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType, Paragraph, Wrap};
use ratatui::{symbols, Frame};

use crate::commands::formatter::{DurationFormatter, NumberFormatter};
use crate::commands::tui::app::App;
use crate::commands::tui::charts::{
    create_chart_binary_size_label_spans, create_chart_percentage_label_spans,
    create_chart_timestamp_label_spans, DEFAULT_LABELS_COUNT,
};
use crate::commands::tui::flow_charts::{
    draw_flow_metric_chart, draw_plugin_throughput_flow_chart,
};

pub(crate) fn draw_node_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(area);

    draw_node_widgets(f, app, chunks[0]);
}

fn draw_node_widgets(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Length(3), Constraint::Percentage(82)])
        .direction(Direction::Vertical)
        .split(area);
    {
        // Node overview
        let events_block = Block::default().title("Overview").borders(Borders::ALL);
        let overview_text: Line = if let Some(node_stats) = &app.data.node_stats() {
            Line::from(vec![
                Span::styled("Events in: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.events.r#in.format_number()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Events out: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.events.out.format_number()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Reloads: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.reloads.successes.format_number()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Pipeline workers: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.pipeline.workers.to_string()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "Pipeline batch size: ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::from(node_stats.pipeline.batch_size.to_string()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Version: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.version.as_str()),
                Span::styled(" | ", Style::default().fg(Color::Yellow)),
                Span::styled("Uptime: ", Style::default().fg(Color::DarkGray)),
                Span::from(node_stats.jvm.uptime_in_millis.format_duration()),
            ])
        } else {
            Line::from(vec![Span::styled(
                "-",
                Style::default().fg(Color::DarkGray),
            )])
        };

        let info_paragraph = Paragraph::new(overview_text)
            .block(events_block)
            .wrap(Wrap { trim: true });

        f.render_widget(info_paragraph.clone(), chunks[0]);

        draw_node_charts(f, app, chunks[1]);
    }
}

fn draw_node_charts(f: &mut Frame, app: &mut App, area: Rect) {
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

    draw_process_file_descriptor_gauge(f, app, flow_chart_chunks[0]);
    draw_plugin_throughput_flow_chart(
        f,
        "Throughput",
        &app.node_state.chart_flow_plugins_throughput,
        flow_chart_chunks[1],
    );
    draw_flow_metric_chart(
        f,
        "Queue Backpressure",
        None,
        &app.node_state.chart_flow_queue_backpressure,
        flow_chart_chunks[2],
        false,
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

    draw_jvm_heap_chart(f, app, node_chart_chunks[0]);
    draw_jvm_non_heap_chart(f, app, node_chart_chunks[1]);
    draw_process_cpu_chart(f, app, node_chart_chunks[2]);
}

fn draw_jvm_heap_chart(f: &mut Frame, app: &mut App, area: Rect) {
    let mut heap_max_data: Vec<(f64, f64)> = vec![];
    let mut heap_used_data: Vec<(f64, f64)> = vec![];

    for data in &app.node_state.chart_jvm_heap_state.data_points {
        heap_max_data.push((data.timestamp as f64, data.heap_max_in_bytes as f64));
        heap_used_data.push((data.timestamp as f64, data.heap_used_in_bytes as f64));
    }

    let current_max_heap = app
        .node_state
        .chart_jvm_heap_state
        .data_points
        .front()
        .map(|p| p.heap_max_in_bytes)
        .unwrap_or(0);

    let current_used_heap = app
        .node_state
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
                .bounds(*app.node_state.chart_jvm_heap_state.x_axis_bounds())
                .labels(create_chart_timestamp_label_spans(
                    app.node_state
                        .chart_jvm_heap_state
                        .x_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(*app.node_state.chart_jvm_heap_state.y_axis_bounds())
                .labels(create_chart_binary_size_label_spans(
                    app.node_state
                        .chart_jvm_heap_state
                        .y_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        );

    f.render_widget(chart, area);
}

fn draw_jvm_non_heap_chart(f: &mut Frame, app: &mut App, area: Rect) {
    let mut non_heap_used: Vec<(f64, f64)> = vec![];
    let mut non_heap_committed: Vec<(f64, f64)> = vec![];

    for data in &app.node_state.chart_jvm_non_heap_state.data_points {
        non_heap_used.push((data.timestamp as f64, data.non_heap_used_in_bytes as f64));
        non_heap_committed.push((
            data.timestamp as f64,
            data.non_heap_committed_in_bytes as f64,
        ));
    }

    let current_heap_used = app
        .node_state
        .chart_jvm_non_heap_state
        .data_points
        .front()
        .map(|p| p.non_heap_used_in_bytes)
        .unwrap_or(0);

    let current_heap_committed = app
        .node_state
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
                .bounds(*app.node_state.chart_jvm_non_heap_state.x_axis_bounds())
                .labels(create_chart_timestamp_label_spans(
                    app.node_state
                        .chart_jvm_non_heap_state
                        .x_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(*app.node_state.chart_jvm_non_heap_state.y_axis_bounds())
                .labels(create_chart_binary_size_label_spans(
                    app.node_state
                        .chart_jvm_non_heap_state
                        .y_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        );

    f.render_widget(chart, area);
}

fn draw_process_cpu_chart(f: &mut Frame, app: &mut App, area: Rect) {
    let cpu_percentage_data: Vec<(f64, f64)> = app
        .node_state
        .chart_process_cpu
        .data_points
        .iter()
        .map(|p| (p.timestamp as f64, p.percent as f64))
        .collect();

    let current_value = app
        .node_state
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
                .bounds(*app.node_state.chart_process_cpu.x_axis_bounds())
                .labels(create_chart_timestamp_label_spans(
                    app.node_state
                        .chart_process_cpu
                        .x_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(*app.node_state.chart_process_cpu.y_axis_bounds())
                .labels(create_chart_percentage_label_spans(
                    app.node_state
                        .chart_process_cpu
                        .y_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        );

    f.render_widget(chart, area);
}

fn draw_process_file_descriptor_gauge(f: &mut Frame, app: &mut App, area: Rect) {
    let usage: u16;
    let max_file_descriptors;
    let open_file_descriptors;

    if let Some(stats) = &app.data.node_stats() {
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
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        )
        .percent(usage)
        .label(label)
        .use_unicode(true);

    f.render_widget(gauge, area);
}
