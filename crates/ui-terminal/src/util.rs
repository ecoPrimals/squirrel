use std::time::{Instant, Duration};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

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
    let clamped_value = value.clamp(0.0, 100.0);
    
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

/// CompressedTimeSeries maintains a time series with efficient storage
#[derive(Debug, Clone)]
pub struct CompressedTimeSeries<T: Copy + std::ops::Sub<Output = T> + std::ops::Add<Output = T> + Default> {
    /// Base timestamp
    pub base_timestamp: DateTime<Utc>,
    /// Deltas from base timestamp in milliseconds
    pub timestamp_deltas: Vec<i64>,
    /// Base value
    pub base_value: T,
    /// Deltas from base value
    pub value_deltas: Vec<T>,
    /// Maximum capacity
    pub max_capacity: usize,
    /// Whether we've set a base point yet
    pub has_base_point: bool,
}

/// Resampling strategy for time series data
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResampleStrategy<T> {
    /// Evenly spaced points
    EvenlySpaced,
    /// Points with largest values
    LargestValues,
    /// Significant changes only
    SignificantChanges(T),
}

impl<T: Copy + std::ops::Sub<Output = T> + std::ops::Add<Output = T> + Default + PartialOrd> CompressedTimeSeries<T> {
    /// Create a new CompressedTimeSeries with specified maximum capacity
    pub fn new(max_capacity: usize) -> Self {
        Self {
            base_timestamp: Utc::now(),
            timestamp_deltas: Vec::with_capacity(max_capacity),
            base_value: T::default(),
            value_deltas: Vec::with_capacity(max_capacity),
            max_capacity,
            has_base_point: false,
        }
    }
    
    /// Add a point to the time series
    pub fn add(&mut self, timestamp: DateTime<Utc>, value: T) {
        // If this is the first point, set it as the base
        if !self.has_base_point {
            self.base_timestamp = timestamp;
            self.base_value = value;
            self.has_base_point = true;
            return;
        }
        
        // Calculate delta from base timestamp in milliseconds
        let delta_ms = (timestamp - self.base_timestamp).num_milliseconds();
        
        // Calculate delta from base value
        let value_delta = value - self.base_value;
        
        // Add deltas to series
        self.timestamp_deltas.push(delta_ms);
        self.value_deltas.push(value_delta);
        
        // If we've exceeded capacity, remove oldest point
        if self.timestamp_deltas.len() > self.max_capacity {
            self.timestamp_deltas.remove(0);
            self.value_deltas.remove(0);
        }
    }
    
    /// Get all points in the time series
    pub fn points(&self) -> Vec<(DateTime<Utc>, T)> {
        let mut result = Vec::with_capacity(self.len() + if self.has_base_point { 1 } else { 0 });
        
        // Add base point
        if self.has_base_point {
            result.push((self.base_timestamp, self.base_value));
        }
        
        // Add all delta points
        for i in 0..self.timestamp_deltas.len() {
            let timestamp = self.base_timestamp + chrono::Duration::milliseconds(self.timestamp_deltas[i]);
            let value = self.base_value + self.value_deltas[i];
            result.push((timestamp, value));
        }
        
        result
    }
    
    /// Get points within a specific time range
    pub fn points_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<(DateTime<Utc>, T)> {
        self.points().into_iter()
            .filter(|(ts, _)| *ts >= start && *ts <= end)
            .collect()
    }
    
    /// Downsample the time series to a specified number of points
    pub fn downsample(&self, count: usize) -> Vec<(DateTime<Utc>, T)> {
        self.resample(count, ResampleStrategy::EvenlySpaced)
    }
    
    /// Resample the time series to a specified number of points using the given strategy
    pub fn resample(&self, count: usize, strategy: ResampleStrategy<T>) -> Vec<(DateTime<Utc>, T)> {
        let points = self.points();
        
        // If we have fewer points than requested or no points, return all
        if points.len() <= count || points.is_empty() {
            return points;
        }
        
        match strategy {
            ResampleStrategy::EvenlySpaced => {
                // Calculate the step size
                let step = (points.len() - 1) as f64 / (count - 1) as f64;
                
                let mut result = Vec::with_capacity(count);
                
                // Always include the first point
                result.push(points[0]);
                
                // Add evenly spaced points
                for i in 1..count - 1 {
                    let idx = (i as f64 * step).round() as usize;
                    result.push(points[idx]);
                }
                
                // Always include the last point
                result.push(points[points.len() - 1]);
                
                result
            },
            ResampleStrategy::LargestValues => {
                // Sort points by value in descending order
                let mut sorted_points = points.clone();
                sorted_points.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                
                // Take the top 'count' points
                let mut largest_points: Vec<_> = sorted_points.into_iter().take(count).collect();
                
                // Resort by timestamp
                largest_points.sort_by_key(|(ts, _)| *ts);
                
                largest_points
            },
            ResampleStrategy::SignificantChanges(threshold) => {
                if points.len() <= 2 {
                    return points;
                }
                
                let mut result = Vec::with_capacity(count);
                
                // Always include the first point
                result.push(points[0]);
                
                // Add points with significant changes
                let mut last_included = points[0];
                for i in 1..points.len() - 1 {
                    let current = points[i];
                    let _next = points[i + 1];
                    
                    // Calculate change from last included point
                    let diff = if current.1 > last_included.1 {
                        current.1 - last_included.1
                    } else {
                        last_included.1 - current.1
                    };
                    
                    // Include point if change is significant
                    if diff > threshold {
                        result.push(current);
                        last_included = current;
                    }
                    
                    // Stop if we've reached the requested count
                    if result.len() >= count - 1 {
                        break;
                    }
                }
                
                // Always include the last point
                if !points.is_empty() && (result.is_empty() || result.last().unwrap().0 != points.last().unwrap().0) {
                    result.push(*points.last().unwrap());
                }
                
                result
            }
        }
    }
    
    /// Calculate statistics for a time range
    pub fn statistics(&self, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Option<Statistics<T>> 
    where T: std::cmp::PartialOrd + std::ops::Div<f64, Output = T> + Into<f64> + From<f64>
    {
        let points = if let (Some(s), Some(e)) = (start, end) {
            self.points_in_range(s, e)
        } else {
            self.points()
        };
        
        if points.is_empty() {
            return None;
        }
        
        let mut min_value = points[0].1;
        let mut max_value = points[0].1;
        let mut sum: f64 = points[0].1.into();
        
        for (_, value) in points.iter().skip(1) {
            if *value < min_value {
                min_value = *value;
            }
            if *value > max_value {
                max_value = *value;
            }
            sum += (*value).into();
        }
        
        let avg: T = T::from(sum / points.len() as f64);
        
        Some(Statistics {
            min: min_value,
            max: max_value,
            avg,
            count: points.len(),
        })
    }
    
    /// Get the number of points in the time series
    pub fn len(&self) -> usize {
        self.timestamp_deltas.len()
    }
    
    /// Check if the time series is empty
    pub fn is_empty(&self) -> bool {
        !self.has_base_point && self.timestamp_deltas.is_empty()
    }
    
    /// Clear all points
    pub fn clear(&mut self) {
        self.timestamp_deltas.clear();
        self.value_deltas.clear();
        self.has_base_point = false;
    }
    
    /// Get the time range of the series
    pub fn time_range(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        if self.is_empty() {
            return None;
        }
        
        let points = self.points();
        if points.is_empty() {
            return None;
        }
        
        Some((points[0].0, points[points.len() - 1].0))
    }
}

/// Statistics calculated from a time series
#[derive(Debug, Clone, Copy)]
pub struct Statistics<T> {
    /// Minimum value
    pub min: T,
    /// Maximum value
    pub max: T,
    /// Average value
    pub avg: T,
    /// Number of points
    pub count: usize,
}

/// CachedMetrics provides time-based caching for metrics
#[derive(Debug, Clone)]
pub struct CachedMetrics<T: Clone> {
    /// Cached metrics value
    value: Option<T>,
    /// When the value was last updated
    last_updated: Option<Instant>,
    /// Cache time-to-live
    pub ttl: Duration,
}

impl<T: Clone> CachedMetrics<T> {
    /// Create a new cached metrics with specified TTL
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            value: None,
            last_updated: None,
            ttl: Duration::from_millis(ttl_ms),
        }
    }
    
    /// Update the cached value
    pub fn update(&mut self, value: T) {
        self.value = Some(value);
        self.last_updated = Some(Instant::now());
    }
    
    /// Get the cached value if it's still valid, otherwise returns None
    pub fn get(&self) -> Option<T> {
        match (&self.value, &self.last_updated) {
            (Some(value), Some(last_updated)) => {
                if last_updated.elapsed() <= self.ttl {
                    Some(value.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Check if the cached value is still valid
    pub fn is_valid(&self) -> bool {
        match self.last_updated {
            Some(last_updated) => last_updated.elapsed() <= self.ttl,
            None => false,
        }
    }
    
    /// Check if cache has expired
    pub fn is_expired(&self) -> bool {
        !self.is_valid()
    }
    
    /// Get time since last update
    pub fn time_since_update(&self) -> Option<Duration> {
        self.last_updated.map(|t| t.elapsed())
    }
    
    /// Force invalidate the cache
    pub fn invalidate(&mut self) {
        self.last_updated = None;
    }
    
    /// Set a new TTL
    pub fn set_ttl(&mut self, ttl_ms: u64) {
        self.ttl = Duration::from_millis(ttl_ms);
    }
}

/// CachedMap provides time-based caching for a collection of metrics
#[derive(Debug, Clone)]
pub struct CachedMap<K: Eq + std::hash::Hash + Clone, V: Clone> {
    /// Map of cached values
    pub data: HashMap<K, (V, Instant)>,
    /// Default TTL for all entries
    pub ttl: Duration,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> CachedMap<K, V> {
    /// Create a new cached map with specified TTL
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            data: HashMap::new(),
            ttl: Duration::from_millis(ttl_ms),
        }
    }
    
    /// Insert a value into the cache
    pub fn insert(&mut self, key: K, value: V) {
        self.data.insert(key, (value, Instant::now()));
    }
    
    /// Get a value if it's still valid
    pub fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).and_then(|(value, timestamp)| {
            if timestamp.elapsed() <= self.ttl {
                Some(value.clone())
            } else {
                None
            }
        })
    }
    
    /// Check if entry exists and is valid
    pub fn contains_valid(&self, key: &K) -> bool {
        match self.data.get(key) {
            Some((_, timestamp)) => timestamp.elapsed() <= self.ttl,
            None => false,
        }
    }
    
    /// Remove entries that have expired
    pub fn cleanup(&mut self) {
        self.data.retain(|_, (_, timestamp)| {
            timestamp.elapsed() <= self.ttl
        });
    }
    
    /// Set a new TTL for the entire cache
    pub fn set_ttl(&mut self, ttl_ms: u64) {
        self.ttl = Duration::from_millis(ttl_ms);
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    /// Get number of entries
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Widget caching helper for efficient rendering
#[derive(Debug, Clone)]
pub struct CachedWidget<T: Clone> {
    /// Cached widget data
    data: Option<T>,
    /// Last update time
    last_updated: Option<Instant>,
    /// Cache TTL
    ttl: Duration,
    /// Render time statistics
    render_times: Vec<Duration>,
    /// Maximum number of render times to track
    max_render_times: usize,
}

impl<T: Clone> CachedWidget<T> {
    /// Create a new cached widget with specified TTL
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            data: None,
            last_updated: None,
            ttl: Duration::from_millis(ttl_ms),
            render_times: Vec::with_capacity(10),
            max_render_times: 10,
        }
    }
    
    /// Get cached data if valid
    pub fn get(&self) -> Option<T> {
        match (&self.data, &self.last_updated) {
            (Some(data), Some(last_updated)) => {
                if last_updated.elapsed() <= self.ttl {
                    Some(data.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Update cached data
    pub fn update(&mut self, data: T) {
        self.data = Some(data);
        self.last_updated = Some(Instant::now());
    }
    
    /// Check if cache is valid
    pub fn is_valid(&self) -> bool {
        match self.last_updated {
            Some(last_updated) => last_updated.elapsed() <= self.ttl,
            None => false,
        }
    }
    
    /// Render with caching
    pub fn render<F>(&mut self, render_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        // Check if cache is valid
        if let Some(data) = self.get() {
            return data;
        }
        
        // Cache is invalid, render and update
        let start = Instant::now();
        let data = render_fn();
        let duration = start.elapsed();
        
        // Track render time
        self.add_render_time(duration);
        
        // Update cache
        self.update(data.clone());
        
        data
    }
    
    /// Add render time to statistics
    fn add_render_time(&mut self, duration: Duration) {
        self.render_times.push(duration);
        
        // Keep render times at max capacity
        if self.render_times.len() > self.max_render_times {
            self.render_times.remove(0);
        }
    }
    
    /// Get average render time
    pub fn avg_render_time(&self) -> Option<Duration> {
        if self.render_times.is_empty() {
            return None;
        }
        
        let total = self.render_times.iter().sum::<Duration>();
        Some(total / self.render_times.len() as u32)
    }
    
    /// Invalidate the cache
    pub fn invalidate(&mut self) {
        self.last_updated = None;
    }
    
    /// Set a new TTL
    pub fn set_ttl(&mut self, ttl_ms: u64) {
        self.ttl = Duration::from_millis(ttl_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_compressed_time_series() {
        let mut series = CompressedTimeSeries::<f64>::new(5);
        
        // Add some points
        let now = Utc::now();
        println!("Adding first point");
        series.add(now, 1.0);
        println!("Adding second point");
        series.add(now + chrono::Duration::seconds(1), 2.0);
        println!("Adding third point");
        series.add(now + chrono::Duration::seconds(2), 3.0);
        
        // Debug prints
        println!("After adding 3 points:");
        println!("timestamp_deltas length: {}", series.timestamp_deltas.len());
        println!("value_deltas length: {}", series.value_deltas.len());
        println!("series.len(): {}", series.len());
        
        // Check points
        let points = series.points();
        println!("points.len(): {}", points.len());
        for (i, (ts, val)) in points.iter().enumerate() {
            println!("Point {}: ts={}, val={}", i, ts, val);
        }
        
        // IMPORTANT: When adding 3 points to an empty series:
        // - The first point becomes the base point (not stored in deltas)
        // - The other 2 points are stored as deltas
        // - So series.len() will return 2 (number of deltas)
        // - But points() will return 3 (base point + deltas)
        
        assert_eq!(points.len(), 3);  // 1 base point + 2 delta points
        assert!((points[0].1 - 1.0).abs() < 0.0001);
        assert!((points[1].1 - 2.0).abs() < 0.0001);
        assert!((points[2].1 - 3.0).abs() < 0.0001);
        
        // The len() method only counts delta points (not the base point)
        assert_eq!(series.len(), 2);
        
        // Test downsampling
        let downsampled = series.downsample(2);
        println!("Downsampled:");
        println!("downsampled.len(): {}", downsampled.len());
        for (i, (ts, val)) in downsampled.iter().enumerate() {
            println!("Downsampled point {}: ts={}, val={}", i, ts, val);
        }
        
        // The downsampling logic correctly returns just 2 points
        // - The first point (base point)
        // - The last point
        assert_eq!(downsampled.len(), 2);
        assert!((downsampled[0].1 - 1.0).abs() < 0.0001);
        assert!((downsampled[1].1 - 3.0).abs() < 0.0001);
    }
    
    #[test]
    fn test_cached_metrics() {
        let mut cache = CachedMetrics::<i32>::new(100);
        
        // Cache should be invalid initially
        assert!(!cache.is_valid());
        assert!(cache.get().is_none());
        
        // Update cache
        cache.update(42);
        
        // Cache should be valid
        assert!(cache.is_valid());
        assert_eq!(cache.get(), Some(42));
        
        // Wait for cache to expire
        thread::sleep(Duration::from_millis(150));
        
        // Cache should be invalid
        assert!(!cache.is_valid());
        assert!(cache.get().is_none());
    }
    
    #[test]
    fn test_cached_map() {
        let mut cache = CachedMap::<String, i32>::new(100);
        
        // Insert some values
        cache.insert("one".to_string(), 1);
        cache.insert("two".to_string(), 2);
        
        // Check values
        assert_eq!(cache.get(&"one".to_string()), Some(1));
        assert_eq!(cache.get(&"two".to_string()), Some(2));
        assert_eq!(cache.get(&"three".to_string()), None);
        
        // Wait for cache to expire
        thread::sleep(Duration::from_millis(150));
        
        // Values should be expired
        assert_eq!(cache.get(&"one".to_string()), None);
        assert_eq!(cache.get(&"two".to_string()), None);
        
        // Cleanup should remove expired entries
        cache.cleanup();
        assert_eq!(cache.len(), 0);
    }
    
    #[test]
    fn test_cached_widget() {
        let mut cache = CachedWidget::<String>::new(100);
        
        // Cache should be invalid initially
        assert!(!cache.is_valid());
        
        // Render with caching
        let result = cache.render(|| "Test".to_string());
        assert_eq!(result, "Test");
        
        // Cache should be valid
        assert!(cache.is_valid());
        assert_eq!(cache.get(), Some("Test".to_string()));
        
        // Render again should use cache
        let result = cache.render(|| "Changed".to_string());
        assert_eq!(result, "Test"); // Still returns cached value
        
        // Wait for cache to expire
        thread::sleep(Duration::from_millis(150));
        
        // Cache should be invalid
        assert!(!cache.is_valid());
        
        // Render again should update cache
        let result = cache.render(|| "Changed".to_string());
        assert_eq!(result, "Changed");
    }
} 