use humansize::{format_size_i, DECIMAL};
use ratatui::text::Span;
use std::collections::VecDeque;
use time::{format_description, OffsetDateTime, UtcOffset};

pub(crate) const DEFAULT_LABELS_COUNT: usize = 4;

pub(crate) const DEFAULT_MAX_DATA_POINTS: Option<usize> = Some(120);

pub(crate) fn create_chart_float_label_spans<'a>(label_values: Vec<f64>) -> Vec<Span<'a>> {
    label_values
        .iter()
        .map(|value| Span::raw(format!("{:.2}", *value)))
        .collect()
}

pub(crate) fn create_chart_timestamp_label_spans<'a>(label_values: Vec<f64>) -> Vec<Span<'a>> {
    let format = format_description::parse("[hour]:[minute]:[second]").unwrap();
    label_values
        .iter()
        .map(|value| {
            Span::raw(
                OffsetDateTime::from_unix_timestamp(*value as i64)
                    .unwrap()
                    .to_offset(UtcOffset::current_local_offset().unwrap())
                    .format(&format)
                    .unwrap(),
            )
        })
        .collect()
}

pub(crate) fn create_chart_binary_size_label_spans<'a>(label_values: Vec<f64>) -> Vec<Span<'a>> {
    label_values
        .iter()
        .map(|value| Span::raw(format_size_i(*value, DECIMAL)))
        .collect()
}

pub(crate) fn create_chart_percentage_label_spans<'a>(label_values: Vec<f64>) -> Vec<Span<'a>> {
    label_values
        .iter()
        .map(|value| Span::raw(format!("{:.2}%", *value)))
        .collect()
}

pub trait ChartDataPoint {
    fn y_axis_bounds(&self) -> [f64; 2];
    fn x_axis_bounds(&self) -> [f64; 2];
}

pub struct TimestampChartState<Y>
where
    Y: ChartDataPoint,
{
    pub data_points: VecDeque<Y>,
    pub max_data_points: Option<usize>,
    x_axis_bounds: [f64; 2],
    y_axis_bounds: [f64; 2],
    min_x_value: Option<f64>,
    min_y_value: Option<f64>,
}

impl<Y> Default for TimestampChartState<Y>
where
    Y: ChartDataPoint,
{
    fn default() -> Self {
        TimestampChartState::new(DEFAULT_MAX_DATA_POINTS)
    }
}

impl<Y> TimestampChartState<Y>
where
    Y: ChartDataPoint,
{
    pub fn with_min_bounds(
        max_data_points: Option<usize>,
        min_x_value: Option<f64>,
        min_y_value: Option<f64>,
    ) -> Self {
        TimestampChartState {
            data_points: VecDeque::new(),
            max_data_points,
            x_axis_bounds: [0.0, 0.0],
            y_axis_bounds: [0.0, 0.0],
            min_x_value,
            min_y_value,
        }
    }

    pub fn with_negative_bounds(max_data_points: Option<usize>) -> Self {
        Self::with_min_bounds(max_data_points, Some(f64::MIN), Some(f64::MIN))
    }

    pub fn new(max_data_points: Option<usize>) -> Self {
        Self::with_min_bounds(max_data_points, Some(0.0), Some(0.0))
    }

    fn update_bounds(bounds: &mut [f64; 2], value_bounds: [f64; 2]) {
        if value_bounds[0] < bounds[0] {
            bounds[0] = value_bounds[0];
        }

        if value_bounds[1] > bounds[1] {
            bounds[1] = value_bounds[1];
        }
    }

    pub fn push(&mut self, value: Y) {
        let value_y_bounds = value.y_axis_bounds();
        let value_x_bounds = value.x_axis_bounds();

        let inclusive_y_bounds = [
            f64::max(self.min_y_value.unwrap_or(0.0), value_y_bounds[0] - 1.0),
            value_y_bounds[1] + 1.0,
        ];
        let inclusive_x_bounds = [
            f64::max(self.min_x_value.unwrap_or(0.0), value_x_bounds[0] - 1.0),
            value_x_bounds[1] + 1.0,
        ];

        if self.data_points.is_empty() {
            self.x_axis_bounds = inclusive_x_bounds;
            self.y_axis_bounds = inclusive_y_bounds;
        } else {
            Self::update_bounds(&mut self.x_axis_bounds, inclusive_x_bounds);
            Self::update_bounds(&mut self.y_axis_bounds, inclusive_y_bounds);
        }

        self.data_points.push_front(value);

        if let Some(max_data_points) = self.max_data_points {
            if self.data_points.len() >= max_data_points {
                self.data_points.pop_back();
                if let Some(newest) = self.data_points.back() {
                    self.x_axis_bounds[0] = newest.x_axis_bounds()[0];
                }
            }
        }
    }

    pub fn x_axis_bounds(&self) -> &[f64; 2] {
        &self.x_axis_bounds
    }

    pub fn y_axis_bounds(&self) -> &[f64; 2] {
        &self.y_axis_bounds
    }

    pub fn x_axis_labels_values(&self, count: usize) -> Vec<f64> {
        let mut values: Vec<f64> = Vec::with_capacity(count + 1);

        let pieces = f64::max(
            self.min_x_value.map(|p| p + 1.0).unwrap_or(1.0),
            (self.x_axis_bounds[1] - self.x_axis_bounds[0]) / (count as f64),
        );

        let mut previous: Option<f64> = None;
        for i in 0..(count - 1) {
            let next = self.x_axis_bounds[0] + (pieces * (i as f64));
            if previous.is_some_and(|p| p.signum() != next.signum()) {
                values.push(0.0);
            }

            previous = Some(next);
            values.push(next);
        }

        values.push(self.x_axis_bounds[1]);
        values
    }

    pub fn y_axis_labels_values(&self, count: usize) -> Vec<f64> {
        let mut values: Vec<f64> = Vec::with_capacity(count + 1);

        let pieces = f64::max(
            self.min_y_value.map(|p| p + 1.0).unwrap_or(1.0),
            (self.y_axis_bounds[1] - self.y_axis_bounds[0]) / (count as f64),
        );

        let mut previous: Option<f64> = None;
        for i in 0..(count - 1) {
            let next = self.y_axis_bounds[0] + (pieces * (i as f64));
            if previous.is_some_and(|p| p.signum() != next.signum()) {
                values.push(0.0);
            }

            previous = Some(next);
            values.push(next);
        }

        values.push(self.y_axis_bounds[1]);
        values
    }

    pub fn reset(&mut self) {
        self.data_points.clear();
        self.x_axis_bounds = [0.0, 0.00];
        self.y_axis_bounds = [0.0, 0.00];
    }

    pub fn is_empty(&self) -> bool {
        self.data_points.is_empty()
    }
}
