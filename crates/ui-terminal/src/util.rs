use std::time::Duration;
use chrono::{DateTime, Utc, Duration as ChronoDuration, TimeDelta};

/// Format a duration for display
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format a value with the given unit
pub fn format_value(value: f64, unit: &str) -> String {
    format!("{:.2} {}", value, unit)
}

/// Format bytes for display
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format bytes with a rate unit (e.g., for network bandwidth)
pub fn format_bytes_rate(bytes: u64, rate_suffix: &str) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB{}", bytes as f64 / GB as f64, rate_suffix)
    } else if bytes >= MB {
        format!("{:.2} MB{}", bytes as f64 / MB as f64, rate_suffix)
    } else if bytes >= KB {
        format!("{:.2} KB{}", bytes as f64 / KB as f64, rate_suffix)
    } else {
        format!("{} B{}", bytes, rate_suffix)
    }
}

/// Format a percentage for display
pub fn format_percentage(value: f64) -> String {
    format!("{:.2}%", value)
}

/// Calculate a bar representation of a value
pub fn calculate_bar(value: f64, width: usize) -> String {
    // Ensure value is between 0 and 100
    let clamped_value = value.max(0.0).min(100.0);
    
    // Calculate the number of filled characters
    let filled_count = ((clamped_value / 100.0) * width as f64).round() as usize;
    
    // Create the bar string
    let filled = "█".repeat(filled_count);
    let empty = "░".repeat(width - filled_count);
    
    format!("{}{}", filled, empty)
}

/// Convert a timestamp to a human-readable string
pub fn format_timestamp(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format a duration between the given timestamp and now
pub fn format_time_ago(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(timestamp);
    
    let total_seconds = duration.num_seconds();
    
    if total_seconds < 60 {
        format!("{}s ago", total_seconds)
    } else if total_seconds < 3600 {
        format!("{}m ago", total_seconds / 60)
    } else if total_seconds < 86400 {
        format!("{}h ago", total_seconds / 3600)
    } else {
        format!("{}d ago", total_seconds / 86400)
    }
}

/// A memory-efficient container for time-series data using delta encoding.
/// Significantly reduces memory usage by storing differences between timestamps
/// and values instead of the full data.
#[derive(Debug, Clone)]
pub struct CompressedTimeSeries {
    /// Milliseconds since first timestamp (delta encoding)
    timestamps_deltas: Vec<u32>,
    /// Base timestamp for reference
    base_timestamp: DateTime<Utc>,
    /// Deltas from previous value * scale_factor (delta encoding)
    values_deltas: Vec<i16>,
    /// Base value for reference
    base_value: f64,
    /// Scale factor for value encoding
    scale_factor: f64,
    /// Maximum number of points to store
    max_points: usize,
}

impl CompressedTimeSeries {
    /// Create a new compressed time-series with given scale factor and max points
    pub fn new(scale_factor: f64, max_points: usize) -> Self {
        Self {
            timestamps_deltas: Vec::with_capacity(max_points),
            base_timestamp: Utc::now(), // Will be updated on first point
            values_deltas: Vec::with_capacity(max_points),
            base_value: 0.0, // Will be updated on first point
            scale_factor,
            max_points,
        }
    }

    /// Add a new data point to the time series
    pub fn add_point(&mut self, timestamp: DateTime<Utc>, value: f64) {
        if self.timestamps_deltas.is_empty() {
            // First point becomes the base reference
            self.base_timestamp = timestamp;
            self.base_value = value;
            self.timestamps_deltas.push(0);
            self.values_deltas.push(0);
        } else {
            // Calculate delta in milliseconds from base timestamp
            let delta_ms = timestamp
                .signed_duration_since(self.base_timestamp)
                .num_milliseconds() as u32;
                
            // Calculate value delta scaled by the factor
            let value_delta = ((value - self.base_value) * self.scale_factor) as i16;
            
            self.timestamps_deltas.push(delta_ms);
            self.values_deltas.push(value_delta);

            // Remove oldest point if we exceed max_points
            if self.timestamps_deltas.len() > self.max_points {
                self.timestamps_deltas.remove(0);
                self.values_deltas.remove(0);
            }
        }
    }
    
    /// Get all stored points as (timestamp, value) pairs
    pub fn get_points(&self) -> Vec<(DateTime<Utc>, f64)> {
        let mut result = Vec::with_capacity(self.timestamps_deltas.len());
        
        for i in 0..self.timestamps_deltas.len() {
            let timestamp = self.base_timestamp + 
                ChronoDuration::milliseconds(self.timestamps_deltas[i] as i64);
                
            let value = self.base_value + 
                (self.values_deltas[i] as f64) / self.scale_factor;
                
            result.push((timestamp, value));
        }
        
        result
    }

    /// Get the number of points stored
    pub fn len(&self) -> usize {
        self.timestamps_deltas.len()
    }

    /// Check if the time series is empty
    pub fn is_empty(&self) -> bool {
        self.timestamps_deltas.is_empty()
    }

    /// Get points within a specific time range
    pub fn get_points_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<(DateTime<Utc>, f64)> {
        let points = self.get_points();
        points
            .into_iter()
            .filter(|(ts, _)| *ts >= start && *ts <= end)
            .collect()
    }

    /// Get downsampled points (useful for rendering large datasets)
    pub fn get_downsampled_points(&self, max_points: usize) -> Vec<(DateTime<Utc>, f64)> {
        let points = self.get_points();
        if points.len() <= max_points {
            return points;
        }

        let step = (points.len() as f64 / max_points as f64).ceil() as usize;
        points
            .into_iter()
            .step_by(step)
            .take(max_points)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressed_time_series() {
        let mut series = CompressedTimeSeries::new(100.0, 10);
        
        // Test empty state
        assert!(series.is_empty());
        assert_eq!(series.len(), 0);

        // Add some points
        let now = Utc::now();
        series.add_point(now, 10.5);
        series.add_point(now + TimeDelta::seconds(1), 11.2);
        series.add_point(now + TimeDelta::seconds(2), 12.1);
        
        // Check points
        assert_eq!(series.len(), 3);
        assert!(!series.is_empty());
        
        // Verify values were stored correctly
        let points = series.get_points();
        assert_eq!(points.len(), 3);
        
        // Check base point
        assert!(
            (points[0].1 - 10.5).abs() < 0.01, 
            "Expected 10.5, got {}", points[0].1
        );
        
        // Check later points
        assert!(
            (points[1].1 - 11.2).abs() < 0.01, 
            "Expected 11.2, got {}", points[1].1
        );
        assert!(
            (points[2].1 - 12.1).abs() < 0.01, 
            "Expected 12.1, got {}", points[2].1
        );
        
        // Test max points limit
        for i in 3..15 {
            series.add_point(now + TimeDelta::seconds(i), i as f64);
        }
        
        // Should be capped at 10 points
        assert_eq!(series.len(), 10);
        
        // First point should now be different due to removal of oldest points
        let updated_points = series.get_points();
        assert!(updated_points[0].0 > now, "First point should be newer than original first point");
    }
} 