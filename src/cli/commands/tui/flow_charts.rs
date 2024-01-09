use crate::commands::tui::charts::{
    create_chart_float_label_spans, create_chart_timestamp_label_spans, ChartDataPoint,
    TimestampChartState, DEFAULT_LABELS_COUNT,
};
use crate::commands::tui::now_local_unix_timestamp;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::{Color, Span, Style};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};
use ratatui::{symbols, Frame};

pub struct PluginFlowMetricDataPoint {
    pub timestamp: i64,
    pub input: f64,
    pub filter: f64,
    pub output: f64,
}

impl PluginFlowMetricDataPoint {
    pub fn new(input: f64, filter: f64, output: f64) -> Self {
        PluginFlowMetricDataPoint {
            timestamp: now_local_unix_timestamp(),
            input,
            filter,
            output,
        }
    }
}

impl ChartDataPoint for PluginFlowMetricDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        [
            f64::min(f64::min(self.input, self.filter), self.output),
            f64::max(f64::max(self.input, self.filter), self.output),
        ]
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        [self.timestamp as f64, self.timestamp as f64]
    }
}

pub struct FlowMetricDataPoint {
    pub timestamp: i64,
    pub value: f64,
}

impl FlowMetricDataPoint {
    pub fn new(value: f64) -> Self {
        FlowMetricDataPoint {
            timestamp: now_local_unix_timestamp(),
            value,
        }
    }
}

impl ChartDataPoint for FlowMetricDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2] {
        [self.value, self.value]
    }

    fn x_axis_bounds(&self) -> [f64; 2] {
        [self.timestamp as f64, self.timestamp as f64]
    }
}

pub(crate) fn draw_plugin_throughput_flow_chart(
    f: &mut Frame,
    title: &str,
    state: &TimestampChartState<PluginFlowMetricDataPoint>,
    area: Rect,
) {
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

    f.render_widget(create_flow_metric_chart(title, datasets, state), area);
}

pub(crate) fn draw_flow_metric_chart(
    f: &mut Frame,
    title: &str,
    label_suffix: Option<&str>,
    state: &TimestampChartState<FlowMetricDataPoint>,
    area: Rect,
) {
    let metric_data: Vec<(f64, f64)> = state
        .data_points
        .iter()
        .map(|p| (p.timestamp as f64, p.value))
        .collect();

    let current_value = state.data_points.front().map(|p| p.value).unwrap_or(0.0);

    let datasets = vec![Dataset::default()
        .name(format!(
            "Current: {:.3} {}",
            current_value,
            label_suffix.unwrap_or("")
        ))
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Blue))
        .data(&metric_data)];

    f.render_widget(create_flow_metric_chart(title, datasets, state), area);
}

fn create_flow_metric_chart<'a>(
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
                .labels(create_chart_timestamp_label_spans(
                    state.x_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(*state.y_axis_bounds())
                .labels(create_chart_float_label_spans(
                    state.y_axis_labels_values(DEFAULT_LABELS_COUNT),
                )),
        )
}
