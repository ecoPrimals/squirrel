use crate::app::AppTab;
use crate::error::Error as UiError;
use chrono::{DateTime, Utc};
use dashboard_core::data::{Alert, Metrics as SystemMetrics, ProtocolData};
use std::collections::{HashMap, VecDeque};

// Type aliases for clarity
type ConnectionHealth = u8;

const MAX_PROVIDER_ERRORS: usize = 10; // Keep last 10 errors from the provider
const MAX_HISTORY_POINTS: usize = 100; // Keep last 100 data points for charts

/// Maximum number of recent errors to keep
const MAX_RECENT_ERRORS: usize = 10;

/// Represents the state of the application including all data and UI state
pub struct AppState {
    // Data fetched from provider
    pub metrics: Option<SystemMetrics>,
    pub protocol_data: Option<ProtocolData>,
    pub connection_health: u8,
    pub connection_status: String,
    pub alerts: Option<Vec<Alert>>,
    pub recent_errors: VecDeque<UiError>,

    // Internal UI State
    pub active_tab: AppTab,
    pub show_help: bool,
    pub should_quit: bool,
    pub last_update: Option<DateTime<Utc>>,

    // Time series data for charts
    pub time_series: HashMap<String, VecDeque<(f64, f64)>>,
    pub time_window_secs: u64,
    pub max_history_points: usize,

    // New time series data
    pub cpu_history: VecDeque<f64>,
    pub memory_history: VecDeque<f64>,

    // --- Dashboard Data ---
    /// System metrics from the provider
    pub metrics_system: Option<SystemMetrics>,
    /// Network statistics from the provider
    pub network: Option<SystemMetrics>,
    /// Protocol data from the provider
    pub protocol: Option<ProtocolData>,
    /// Alerts from the provider
    pub alerts_provider: Option<Vec<Alert>>,
    
    // --- UI State ---
    /// Whether data needs to be updated
    pub needs_update: bool,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("metrics", &self.metrics)
            .field("protocol_data", &self.protocol_data)
            .field("connection_health", &self.connection_health)
            .field("connection_status", &self.connection_status)
            .field("alerts", &self.alerts)
            .field("recent_errors", &self.recent_errors)
            .field("active_tab", &self.active_tab)
            .field("show_help", &self.show_help)
            .field("should_quit", &self.should_quit)
            .field("last_update", &self.last_update)
            .field("time_series", &self.time_series)
            .field("time_window_secs", &self.time_window_secs)
            .field("max_history_points", &self.max_history_points)
            .field("cpu_history", &self.cpu_history)
            .field("memory_history", &self.memory_history)
            .field("metrics_system", &self.metrics_system)
            .field("network", &self.network)
            .field("protocol", &self.protocol)
            .field("alerts_provider", &self.alerts_provider)
            .field("needs_update", &self.needs_update)
            .finish()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            metrics: None,
            protocol_data: None,
            connection_health: 0,
            alerts: None,
            connection_status: String::new(),
            recent_errors: VecDeque::with_capacity(MAX_RECENT_ERRORS),
            last_update: None,
            time_series: HashMap::new(),
            time_window_secs: 300, // 5 minutes
            max_history_points: 60, // 1 minute with 1 second intervals
            cpu_history: VecDeque::with_capacity(120),
            memory_history: VecDeque::with_capacity(120),
            active_tab: AppTab::Overview,
            show_help: false,
            should_quit: false,
            metrics_system: None,
            network: None,
            protocol: None,
            alerts_provider: None,
            needs_update: true,
        }
    }
}

impl AppState {
    /// Adds an error to the recent errors list, maintaining the maximum size
    pub fn add_error(&mut self, error: UiError) {
        // If we're at capacity, remove the oldest error
        if self.recent_errors.len() >= MAX_RECENT_ERRORS {
            self.recent_errors.pop_front();
        }
        
        self.recent_errors.push_back(error);
    }
    
    /// Check if a similar error already exists in the error list
    pub fn has_similar_error(&self, _error: &UiError) -> bool {
        // For now, we don't check for similar errors
        // This could be implemented in the future if needed
        false
    }
} 