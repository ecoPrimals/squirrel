//! Dashboard configuration options.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

/// Dashboard configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// How frequently to update dashboard data (in seconds)
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,
    
    /// How long to keep historical metrics (in hours)
    #[serde(default = "default_history_retention")]
    pub history_retention: u64,
    
    /// Maximum number of alerts to keep in history
    #[serde(default = "default_alert_history_size")]
    pub alert_history_size: usize,
    
    /// Custom metrics to display
    #[serde(default)]
    pub custom_metrics: Vec<String>,
    
    /// Enable automatic data refresh
    #[serde(default = "default_auto_refresh")]
    pub auto_refresh: bool,
    
    /// Which metric categories to display
    pub displayed_categories: HashSet<MetricCategory>,
    
    /// Custom dashboard panels
    pub custom_panels: Vec<PanelConfig>,
    
    /// Alert display settings
    pub alert_settings: AlertDisplaySettings,
    
    /// Data retention period
    pub retention_period: Duration,
}

fn default_update_interval() -> u64 {
    5 // 5 seconds
}

fn default_history_retention() -> u64 {
    24 // 24 hours
}

fn default_alert_history_size() -> usize {
    100
}

fn default_auto_refresh() -> bool {
    true
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            update_interval: default_update_interval(),
            history_retention: default_history_retention(),
            alert_history_size: default_alert_history_size(),
            custom_metrics: Vec::new(),
            auto_refresh: default_auto_refresh(),
            displayed_categories: HashSet::from_iter([
                MetricCategory::System,
                MetricCategory::Protocol,
                MetricCategory::Network,
            ]),
            custom_panels: Vec::new(),
            alert_settings: AlertDisplaySettings::default(),
            retention_period: Duration::from_secs(60 * 60 * 24 * 30), // 30 days
        }
    }
}

impl DashboardConfig {
    /// Create a new dashboard configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the update interval
    pub fn with_update_interval(mut self, seconds: u64) -> Self {
        self.update_interval = seconds;
        self
    }
    
    /// Set the history retention period
    pub fn with_history_retention(mut self, hours: u64) -> Self {
        self.history_retention = hours;
        self
    }
    
    /// Set the alert history size
    pub fn with_alert_history_size(mut self, size: usize) -> Self {
        self.alert_history_size = size;
        self
    }
    
    /// Set custom metrics to display
    pub fn with_custom_metrics(mut self, metrics: Vec<String>) -> Self {
        self.custom_metrics = metrics;
        self
    }
    
    /// Set auto refresh option
    pub fn with_auto_refresh(mut self, auto_refresh: bool) -> Self {
        self.auto_refresh = auto_refresh;
        self
    }
    
    /// Get the update interval as a Duration
    pub fn update_interval_duration(&self) -> Duration {
        Duration::from_secs(self.update_interval)
    }
}

/// Metric categories to display
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricCategory {
    /// System metrics (CPU, memory, etc.)
    System,
    
    /// Protocol metrics (message processing, etc.)
    Protocol,
    
    /// Tool-specific metrics
    Tool,
    
    /// Network metrics
    Network,
    
    /// Custom metrics
    Custom(String),
}

/// Panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    /// Panel ID
    pub id: String,
    
    /// Panel title
    pub title: String,
    
    /// Panel type
    pub panel_type: PanelType,
    
    /// Metrics to display
    pub metrics: Vec<String>,
    
    /// Panel position
    pub position: PanelPosition,
    
    /// Panel size
    pub size: PanelSize,
    
    /// Refresh rate in seconds
    pub refresh_rate: u64,
}

/// Panel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    /// Line chart
    LineChart,
    
    /// Bar chart
    BarChart,
    
    /// Gauge
    Gauge,
    
    /// Table
    Table,
    
    /// Status panel
    StatusPanel,
    
    /// Custom panel
    Custom(String),
}

/// Panel position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelPosition {
    /// X position (grid units)
    pub x: u32,
    
    /// Y position (grid units)
    pub y: u32,
}

/// Panel size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSize {
    /// Width (grid units)
    pub width: u32,
    
    /// Height (grid units)
    pub height: u32,
}

/// Alert display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDisplaySettings {
    /// Whether to show acknowledged alerts
    pub show_acknowledged: bool,
    
    /// Maximum number of alerts to show
    pub max_alerts: usize,
    
    /// Whether to group alerts by source
    pub group_by_source: bool,
    
    /// Whether to auto-refresh alerts
    pub auto_refresh: bool,
}

impl Default for AlertDisplaySettings {
    fn default() -> Self {
        Self {
            show_acknowledged: false,
            max_alerts: 50,
            group_by_source: true,
            auto_refresh: true,
        }
    }
}

/// History resolution for metric history queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HistoryResolution {
    /// Minute resolution
    Minute,
    
    /// Hour resolution
    Hour,
    
    /// Day resolution
    Day,
} 