//! Configuration types for visualizations

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Output format (json, html, terminal, markdown)
    pub format: String,

    /// Theme configuration
    pub theme: super::VisualizationTheme,

    /// Layout configuration
    pub layout: super::VisualizationLayout,

    /// Interactive features enabled
    pub interactive: bool,

    /// Animation settings
    pub animation: AnimationConfig,

    /// Export settings
    pub export: ExportConfig,

    /// Custom options
    pub custom_options: HashMap<String, Value>,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityConfig {
    /// Image quality (1-100)
    pub image_quality: u32,

    /// DPI for image exports
    pub dpi: u32,

    /// Compression level
    pub compression: u32,
}

// Default implementations
impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            format: "html".to_string(),
            theme: super::VisualizationTheme::default(),
            layout: super::VisualizationLayout::default(),
            interactive: true,
            animation: AnimationConfig::default(),
            export: ExportConfig::default(),
            custom_options: HashMap::new(),
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
