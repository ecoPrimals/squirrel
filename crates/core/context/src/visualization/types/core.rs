//! Core visualization types and structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Visualization type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VisualizationType {
    /// Context state visualization
    ContextState,
    /// Rule dependency graph
    RuleDependencyGraph,
    /// Timeline visualization
    Timeline,
    /// Metrics dashboard
    MetricsDashboard,
    /// State diff visualization
    StateDiff,
    /// Performance heatmap
    PerformanceHeatmap,
    /// Interactive graph
    InteractiveGraph,
    /// Custom visualization
    Custom(String),
}

/// Visualization request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationRequest {
    /// Type of visualization
    pub visualization_type: VisualizationType,

    /// Visualization configuration
    pub config: super::VisualizationConfig,

    /// Data to visualize
    pub data: Value,

    /// Metadata for the visualization
    pub metadata: HashMap<String, Value>,

    /// Title for the visualization
    pub title: Option<String>,

    /// Description for the visualization
    pub description: Option<String>,
}

/// Visualization response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationResponse {
    /// Unique identifier for the visualization
    pub visualization_id: String,

    /// Type of visualization
    pub visualization_type: VisualizationType,

    /// Output format
    pub format: String,

    /// Rendered content
    pub content: String,

    /// Metadata
    pub metadata: HashMap<String, Value>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Visualization data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// X coordinate
    pub x: f64,

    /// Y coordinate
    pub y: f64,

    /// Optional Z coordinate for 3D visualizations
    pub z: Option<f64>,

    /// Data value
    pub value: Value,

    /// Label for the data point
    pub label: Option<String>,

    /// Color for the data point
    pub color: Option<String>,

    /// Size for the data point
    pub size: Option<f64>,
}

/// Visualization series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationSeries {
    /// Series name
    pub name: String,

    /// Series data points
    pub data: Vec<DataPoint>,

    /// Series color
    pub color: Option<String>,

    /// Series type (line, bar, scatter, etc.)
    pub series_type: String,

    /// Series metadata
    pub metadata: HashMap<String, Value>,
}

/// Interactive element configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveElement {
    /// Element type
    pub element_type: String,

    /// Element ID
    pub id: String,

    /// Element properties
    pub properties: HashMap<String, Value>,

    /// Event handlers
    pub event_handlers: HashMap<String, String>,
}

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Filter type
    pub filter_type: String,

    /// Filter parameters
    pub parameters: HashMap<String, Value>,

    /// Filter enabled
    pub enabled: bool,
}

/// Axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    /// Axis label
    pub label: String,

    /// Axis type (linear, logarithmic, time, etc.)
    pub axis_type: String,

    /// Minimum value
    pub min: Option<f64>,

    /// Maximum value
    pub max: Option<f64>,

    /// Tick configuration
    pub ticks: TickConfig,

    /// Grid lines enabled
    pub grid_lines: bool,
}

/// Tick configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickConfig {
    /// Number of ticks
    pub count: u32,

    /// Tick format
    pub format: String,

    /// Tick rotation
    pub rotation: f64,
}

/// Legend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendConfig {
    /// Legend enabled
    pub enabled: bool,

    /// Legend position
    pub position: String,

    /// Legend orientation
    pub orientation: String,

    /// Legend styling
    pub styling: HashMap<String, Value>,
}

/// Tooltip configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooltipConfig {
    /// Tooltip enabled
    pub enabled: bool,

    /// Tooltip format
    pub format: String,

    /// Tooltip styling
    pub styling: HashMap<String, Value>,
}
