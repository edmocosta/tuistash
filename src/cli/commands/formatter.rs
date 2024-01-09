use std::time::Duration;

pub trait DurationFormatter {
    fn format_duration(&self) -> String;
    fn format_duration_per_event(&self, events_count: u64) -> String;
}

impl DurationFormatter for u64 {
    fn format_duration(&self) -> String {
        if *self == 0 {
            return "-".to_string();
        }

        let secs = *self / 1000;
        let duration = if secs > 0 {
            Duration::from_secs(secs)
        } else {
            Duration::from_millis(*self)
        };

        humantime::format_duration(duration).to_string()
    }

    fn format_duration_per_event(&self, events_count: u64) -> String {
        if *self == 0 || events_count == 0 {
            return "-".to_string();
        }

        let duration = *self as f64 / events_count as f64;
        human_format::Formatter::new().format(duration)
    }
}

pub trait NumberFormatter {
    fn format_number(&self) -> String {
        self.format_number_with_decimals(3)
    }

    fn format_number_with_decimals(&self, decimals: usize) -> String;
}

impl NumberFormatter for i64 {
    fn format_number(&self) -> String {
        let decimals = if *self < 1000 { 0 } else { 3 };
        self.format_number_with_decimals(decimals)
    }

    fn format_number_with_decimals(&self, decimals: usize) -> String {
        match self {
            0 => "0".into(),
            _ => human_format::Formatter::new()
                .with_decimals(decimals)
                .format(*self as f64),
        }
    }
}

impl NumberFormatter for f64 {
    fn format_number_with_decimals(&self, decimals: usize) -> String {
        format!("{:.1$}", self, decimals)
    }
}
