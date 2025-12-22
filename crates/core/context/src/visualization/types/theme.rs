//! Theme and layout types

use serde::{Deserialize, Serialize};

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
    pub grid: super::GridConfig,

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

// Default implementations
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
            grid: super::GridConfig::default(),
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
