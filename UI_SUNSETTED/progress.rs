//! Progress tracking and display functionality.
//!
//! This module provides components for tracking and displaying progress information,
//! including progress bars, speed calculations, and estimated time remaining.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::ui::{
    layout::LayoutManager,
    components::progress::{ProgressStyle, SPEED_WINDOW_SIZE},
};

/// A progress tracker that calculates speed and estimated time remaining.
///
/// This struct maintains a history of progress updates and uses them to
/// calculate current speed and estimated time to completion using linear
/// regression.
pub struct Progress {
    /// Style configuration for progress display
    style: ProgressStyle,
    /// Layout manager for progress bar positioning
    layout: LayoutManager,
    /// History of progress updates with timestamps
    progress_history: VecDeque<(Instant, f64)>,
    /// Current progress value (between 0.0 and 1.0)
    current_progress: f64,
    /// Total size of the operation (if known)
    total_size: Option<u64>,
    /// Time when progress tracking started
    start_time: Instant,
}

impl Progress {
    /// Creates a new progress tracker with default settings.
    pub fn new() -> Self {
        Self {
            style: ProgressStyle::default(),
            layout: LayoutManager::new(),
            progress_history: VecDeque::with_capacity(SPEED_WINDOW_SIZE),
            current_progress: 0.0,
            total_size: None,
            start_time: Instant::now(),
        }
    }

    /// Updates the current progress value and maintains progress history.
    ///
    /// # Arguments
    ///
    /// * `progress` - The new progress value (between 0.0 and 1.0)
    pub fn update_progress(&mut self, progress: f64) {
        self.current_progress = progress;
        let now = Instant::now();
        
        // Add new progress point
        self.progress_history.push_back((now, progress));
        
        // Remove old points outside the window
        while self.progress_history.len() >= SPEED_WINDOW_SIZE {
            self.progress_history.pop_front();
        }
    }

    /// Calculates the estimated time remaining and current speed.
    ///
    /// Uses linear regression on the progress history to calculate
    /// the current speed and estimate the time remaining.
    ///
    /// # Returns
    ///
    /// Returns Some((eta, speed)) if there is enough data to calculate,
    /// where eta is the estimated time remaining and speed is the current
    /// progress per second. Returns None if there isn't enough data or
    /// if the calculation fails.
    pub fn calculate_eta_and_speed(&self) -> Option<(Duration, f64)> {
        if self.progress_history.len() < 2 {
            return None;
        }

        // Calculate speed using linear regression
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;
        let n = self.progress_history.len() as f64;

        let first_time = self.progress_history[0].0;
        for (time, progress) in &self.progress_history {
            let x = time.duration_since(first_time).as_secs_f64();
            let y = *progress;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        if slope <= 0.0 {
            return None;
        }

        let speed = slope;
        let remaining_progress = self.total_size.map(|t| t as f64 - self.current_progress).unwrap_or(0.0);
        let eta = Duration::from_secs_f64(remaining_progress / speed);

        Some((eta, speed))
    }
}

impl Default for Progress {
    /// Creates a new progress tracker with default settings.
    fn default() -> Self {
        Self::new()
    }
} 