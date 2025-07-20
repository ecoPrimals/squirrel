//! Visualization Types
//!
//! This module defines the core data structures and types used throughout the visualization system.

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
    pub config: VisualizationConfig,

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

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Output format (json, html, terminal, markdown)
    pub format: String,

    /// Theme configuration
    pub theme: VisualizationTheme,

    /// Layout configuration
    pub layout: VisualizationLayout,

    /// Interactive features enabled
    pub interactive: bool,

    /// Animation settings
    pub animation: AnimationConfig,

    /// Export settings
    pub export: ExportConfig,

    /// Custom options
    pub custom_options: HashMap<String, Value>,
}

/// Visualization theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationTheme {
    /// Primary color
    pub primary_color: String,

    /// Secondary color
    pub secondary_color: String,

    /// Background color
    pub background_color: String,

    /// Text color
    pub text_color: String,

    /// Border color
    pub border_color: String,

    /// Font family
    pub font_family: String,

    /// Font size
    pub font_size: u32,

    /// Color palette for data visualization
    pub color_palette: Vec<String>,

    /// Dark mode enabled
    pub dark_mode: bool,
}

/// Visualization layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationLayout {
    /// Width of the visualization
    pub width: u32,

    /// Height of the visualization
    pub height: u32,

    /// Margin configuration
    pub margin: MarginConfig,

    /// Padding configuration
    pub padding: PaddingConfig,

    /// Grid configuration
    pub grid: GridConfig,

    /// Responsive design enabled
    pub responsive: bool,
}

/// Margin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginConfig {
    /// Top margin
    pub top: u32,

    /// Right margin
    pub right: u32,

    /// Bottom margin
    pub bottom: u32,

    /// Left margin
    pub left: u32,
}

/// Padding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaddingConfig {
    /// Top padding
    pub top: u32,

    /// Right padding
    pub right: u32,

    /// Bottom padding
    pub bottom: u32,

    /// Left padding
    pub left: u32,
}

/// Grid configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    /// Enable grid
    pub enabled: bool,

    /// Grid color
    pub color: String,

    /// Grid line width
    pub line_width: u32,

    /// Grid spacing
    pub spacing: u32,
}

/// Animation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Animation enabled
    pub enabled: bool,

    /// Animation duration in milliseconds
    pub duration: u32,

    /// Animation easing function
    pub easing: String,

    /// Animation delay in milliseconds
    pub delay: u32,
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Supported formats
    pub formats: Vec<String>,

    /// Default format
    pub default_format: String,

    /// Quality settings
    pub quality: QualityConfig,
}

/// Quality configuration for exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Image quality (1-100)
    pub image_quality: u32,

    /// DPI for image exports
    pub dpi: u32,

    /// Compression level
    pub compression: u32,
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

// Default implementations
impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            format: "html".to_string(),
            theme: VisualizationTheme::default(),
            layout: VisualizationLayout::default(),
            interactive: true,
            animation: AnimationConfig::default(),
            export: ExportConfig::default(),
            custom_options: HashMap::new(),
        }
    }
}

impl Default for VisualizationTheme {
    fn default() -> Self {
        Self {
            primary_color: "#3498db".to_string(),
            secondary_color: "#2ecc71".to_string(),
            background_color: "#ffffff".to_string(),
            text_color: "#333333".to_string(),
            border_color: "#dddddd".to_string(),
            font_family: "Arial, sans-serif".to_string(),
            font_size: 14,
            color_palette: vec![
                "#3498db".to_string(),
                "#2ecc71".to_string(),
                "#e74c3c".to_string(),
                "#f39c12".to_string(),
                "#9b59b6".to_string(),
                "#1abc9c".to_string(),
                "#34495e".to_string(),
                "#95a5a6".to_string(),
            ],
            dark_mode: false,
        }
    }
}

impl Default for VisualizationLayout {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            margin: MarginConfig::default(),
            padding: PaddingConfig::default(),
            grid: GridConfig::default(),
            responsive: true,
        }
    }
}

impl Default for MarginConfig {
    fn default() -> Self {
        Self {
            top: 20,
            right: 20,
            bottom: 20,
            left: 20,
        }
    }
}

impl Default for PaddingConfig {
    fn default() -> Self {
        Self {
            top: 10,
            right: 10,
            bottom: 10,
            left: 10,
        }
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: "#f0f0f0".to_string(),
            line_width: 1,
            spacing: 20,
        }
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 1000,
            easing: "ease-in-out".to_string(),
            delay: 0,
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            formats: vec![
                "png".to_string(),
                "svg".to_string(),
                "pdf".to_string(),
                "json".to_string(),
            ],
            default_format: "png".to_string(),
            quality: QualityConfig::default(),
        }
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            image_quality: 95,
            dpi: 300,
            compression: 80,
        }
    }
}

impl std::fmt::Display for VisualizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisualizationType::ContextState => write!(f, "context_state"),
            VisualizationType::RuleDependencyGraph => write!(f, "rule_dependency_graph"),
            VisualizationType::Timeline => write!(f, "timeline"),
            VisualizationType::MetricsDashboard => write!(f, "metrics_dashboard"),
            VisualizationType::StateDiff => write!(f, "state_diff"),
            VisualizationType::PerformanceHeatmap => write!(f, "performance_heatmap"),
            VisualizationType::InteractiveGraph => write!(f, "interactive_graph"),
            VisualizationType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl std::str::FromStr for VisualizationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "context_state" => Ok(VisualizationType::ContextState),
            "rule_dependency_graph" => Ok(VisualizationType::RuleDependencyGraph),
            "timeline" => Ok(VisualizationType::Timeline),
            "metrics_dashboard" => Ok(VisualizationType::MetricsDashboard),
            "state_diff" => Ok(VisualizationType::StateDiff),
            "performance_heatmap" => Ok(VisualizationType::PerformanceHeatmap),
            "interactive_graph" => Ok(VisualizationType::InteractiveGraph),
            custom => Ok(VisualizationType::Custom(custom.to_string())),
        }
    }
}
