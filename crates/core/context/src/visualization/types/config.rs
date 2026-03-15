// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_config_default() {
        let config = VisualizationConfig::default();
        assert_eq!(config.format, "html");
        assert!(config.interactive);
        assert!(config.custom_options.is_empty());
    }

    #[test]
    fn test_grid_config_default() {
        let config = GridConfig::default();
        assert!(config.enabled);
        assert_eq!(config.color, "#f0f0f0");
        assert_eq!(config.line_width, 1);
        assert_eq!(config.spacing, 20);
    }

    #[test]
    fn test_animation_config_default() {
        let config = AnimationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.duration, 1000);
        assert_eq!(config.easing, "ease-in-out");
        assert_eq!(config.delay, 0);
    }

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        assert_eq!(config.formats.len(), 4);
        assert!(config.formats.contains(&"png".to_string()));
        assert!(config.formats.contains(&"svg".to_string()));
        assert!(config.formats.contains(&"pdf".to_string()));
        assert!(config.formats.contains(&"json".to_string()));
        assert_eq!(config.default_format, "png");
    }

    #[test]
    fn test_quality_config_default() {
        let config = QualityConfig::default();
        assert_eq!(config.image_quality, 95);
        assert_eq!(config.dpi, 300);
        assert_eq!(config.compression, 80);
    }

    #[test]
    fn test_visualization_config_serde_roundtrip() {
        let config = VisualizationConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: VisualizationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.format, config.format);
        assert_eq!(deserialized.interactive, config.interactive);
    }

    #[test]
    fn test_grid_config_serde_roundtrip() {
        let config = GridConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GridConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.color, config.color);
        assert_eq!(deserialized.line_width, config.line_width);
    }

    #[test]
    fn test_animation_config_serde_roundtrip() {
        let config = AnimationConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AnimationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.duration, config.duration);
        assert_eq!(deserialized.easing, config.easing);
    }

    #[test]
    fn test_quality_config_equality() {
        let q1 = QualityConfig::default();
        let q2 = QualityConfig::default();
        assert_eq!(q1, q2);

        let q3 = QualityConfig {
            image_quality: 50,
            dpi: 72,
            compression: 50,
        };
        assert_ne!(q1, q3);
    }

    #[test]
    fn test_visualization_config_custom_options() {
        let mut config = VisualizationConfig::default();
        config
            .custom_options
            .insert("key".to_string(), serde_json::json!("value"));
        assert_eq!(config.custom_options.len(), 1);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: VisualizationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.custom_options.len(), 1);
    }
}
