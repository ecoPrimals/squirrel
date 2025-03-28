use std::time::Duration;

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