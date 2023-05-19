use std::time::Duration;

use num_format::{Locale, ToFormattedString};

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
        let duration: Duration;
        if secs > 0 {
            duration = Duration::from_secs(secs);
        } else {
            duration = Duration::from_millis(*self);
        }

        return humantime::format_duration(duration).to_string();
    }

    fn format_duration_per_event(&self, events_count: u64) -> String {
        if *self == 0 || events_count == 0 {
            return "-".to_string();
        }

        let secs = *self as f64 * 1000 as f64;
        let es: f64 = events_count as f64 / secs;
        return format!("{:.2} e/s", es);
    }
}

pub trait NumberFormatter {
    fn format_number(&self) -> String;
}

impl NumberFormatter for i64 {
    fn format_number(&self) -> String {
        match self {
            0 => "0".into(),
            _ => self.to_formatted_string(&Locale::en),
        }
    }
}