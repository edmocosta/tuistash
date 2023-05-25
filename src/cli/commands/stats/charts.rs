use std::collections::VecDeque;

use chrono::{Local, TimeZone};
use humansize::ToF64;
use humansize::{format_size_i, DECIMAL};
use tui::text::Span;

pub(crate) const DEFAULT_LABELS_COUNT: usize = 4;

pub(crate) const DEFAULT_MAX_DATA_POINTS: Option<usize> = Some(60);

pub(crate) fn create_float_label_spans<'a>(label_values: Vec<f64>) -> Vec<Span<'a>> {
    let spans: Vec<Span> = label_values
        .iter()
        .map(|value| Span::raw(format!("{:.2}", *value)))
        .collect();

    return spans;
}

pub(crate) fn create_timestamp_label_spans<'a>(label_values: Vec<f64>) -> Vec<Span<'a>> {
    let spans: Vec<Span> = label_values
        .iter()
        .map(|value| {
            Span::raw(
                Local
                    .timestamp_millis_opt(*value as i64)
                    .unwrap()
                    .format("%H:%M:%S")
                    .to_string(),
            )
        })
        .collect();

    return spans;
}

pub(crate) fn create_binary_size_label_spans<'a>(label_values: Vec<f64>) -> Vec<Span<'a>> {
    let spans: Vec<Span> = label_values
        .iter()
        .map(|value| Span::raw(format_size_i(*value, DECIMAL)))
        .collect();

    return spans;
}

pub(crate) fn create_percentage_label_spans<'a>(label_values: Vec<f64>) -> Vec<Span<'a>> {
    let spans: Vec<Span> = label_values
        .iter()
        .map(|value| Span::raw(format!("{:.2}%", *value)))
        .collect();

    return spans;
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
}

impl<Y> TimestampChartState<Y>
where
    Y: ChartDataPoint,
{
    pub fn new(max_data_points: Option<usize>) -> Self {
        TimestampChartState {
            data_points: VecDeque::new(),
            max_data_points,
            x_axis_bounds: [0.0, 0.0],
            y_axis_bounds: [0.0, 0.0],
        }
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
            f64::max(0.0, value_y_bounds[0] - 1.0),
            value_y_bounds[1] + 1.0,
        ];
        let inclusive_x_bounds = [
            f64::max(0.0, value_x_bounds[0] - 1.0),
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
        let mut values: Vec<f64> = Vec::with_capacity(count);

        let pieces = f64::max(
            1.0,
            (self.x_axis_bounds[1] - self.x_axis_bounds[0]) / count.to_f64(),
        );
        for i in 0..(count - 1) {
            let next = self.x_axis_bounds[0] + (pieces * i.to_f64());
            values.push(next);
        }

        values.push(self.x_axis_bounds[1]);
        return values;
    }

    pub fn y_axis_labels_values(&self, count: usize) -> Vec<f64> {
        let mut values: Vec<f64> = Vec::with_capacity(count);

        let pieces = f64::max(
            1.0,
            (self.y_axis_bounds[1] - self.y_axis_bounds[0]) / count.to_f64(),
        );
        for i in 0..(count - 1) {
            let next = self.y_axis_bounds[0] + (pieces * i.to_f64());
            values.push(next);
        }

        values.push(self.y_axis_bounds[1]);
        return values;
    }

    pub fn reset(&mut self) {
        self.data_points.clear();
        self.x_axis_bounds = [0.0, 0.00];
        self.y_axis_bounds = [0.0, 0.00];
    }
}
