use humansize::ToF64;
use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};
use tui::{symbols, Frame};

use crate::commands::view::app::{FlowMetricDataPoint, PluginFlowMetricDataPoint};
use crate::commands::view::charts::TimestampChartState;
use crate::commands::view::charts::{
    create_float_label_spans, create_timestamp_label_spans, ChartDataPoint, DEFAULT_LABELS_COUNT,
};

pub(crate) fn render_plugins_flow_chart<B>(
    f: &mut Frame<B>,
    title: &str,
    state: &TimestampChartState<PluginFlowMetricDataPoint>,
    area: Rect,
) where
    B: Backend,
{
    let mut input_throughput_data: Vec<(f64, f64)> = vec![];
    let mut filter_throughput_data: Vec<(f64, f64)> = vec![];
    let mut output_throughput_data: Vec<(f64, f64)> = vec![];

    for data_point in &state.data_points {
        input_throughput_data.push((data_point.timestamp as f64, data_point.input));
        filter_throughput_data.push((data_point.timestamp as f64, data_point.filter));
        output_throughput_data.push((data_point.timestamp as f64, data_point.output));
    }

    let (input_throughput, filter_throughput, output_throughput) = state
        .data_points
        .front()
        .map(|p| (p.input, p.filter, p.output))
        .unwrap_or((0.0, 0.0, 0.0));

    let datasets = vec![
        Dataset::default()
            .name(format!("Input: {:.3} e/s", input_throughput))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Blue))
            .data(&input_throughput_data),
        Dataset::default()
            .name(format!("Filter: {:.3} e/s", filter_throughput))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&filter_throughput_data),
        Dataset::default()
            .name(format!("Output: {:.3} e/s", output_throughput))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Magenta))
            .data(&output_throughput_data),
    ];

    f.render_widget(create_flow_chart(title, datasets, state), area);
}

pub(crate) fn render_flow_chart<B>(
    f: &mut Frame<B>,
    title: &str,
    label_suffix: Option<&str>,
    state: &TimestampChartState<FlowMetricDataPoint>,
    area: Rect,
) where
    B: Backend,
{
    let throughput_data: Vec<(f64, f64)> = state
        .data_points
        .iter()
        .map(|p| (p.timestamp.to_f64(), p.value.to_f64()))
        .collect();

    let current_throughput = state.data_points.front().map(|p| p.value).unwrap_or(0.0);

    let datasets = vec![Dataset::default()
        .name(format!(
            "Current: {:.3} {}",
            current_throughput,
            label_suffix.unwrap_or("")
        ))
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Blue))
        .data(&throughput_data)];

    f.render_widget(create_flow_chart(title, datasets, state), area);
}

fn create_flow_chart<'a>(
    title: &'a str,
    datasets: Vec<Dataset<'a>>,
    state: &TimestampChartState<impl ChartDataPoint>,
) -> Chart<'a> {
    Chart::new(datasets)
        .hidden_legend_constraints((Constraint::Percentage(90), Constraint::Percentage(90)))
        .block(
            Block::default()
                .title(Span::raw(title))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(*state.x_axis_bounds())
                .labels(create_timestamp_label_spans(
                    state.x_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(*state.y_axis_bounds())
                .labels(create_float_label_spans(
                    state.y_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
}
