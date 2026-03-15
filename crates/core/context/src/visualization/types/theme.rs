// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_theme_default() {
        let theme = VisualizationTheme::default();
        assert_eq!(theme.primary_color, "#3498db");
        assert_eq!(theme.secondary_color, "#2ecc71");
        assert_eq!(theme.background_color, "#ffffff");
        assert_eq!(theme.text_color, "#333333");
        assert_eq!(theme.border_color, "#dddddd");
        assert_eq!(theme.font_family, "Arial, sans-serif");
        assert_eq!(theme.font_size, 14);
        assert_eq!(theme.color_palette.len(), 8);
        assert!(!theme.dark_mode);
    }

    #[test]
    fn test_visualization_layout_default() {
        let layout = VisualizationLayout::default();
        assert_eq!(layout.width, 800);
        assert_eq!(layout.height, 600);
        assert!(layout.responsive);
    }

    #[test]
    fn test_margin_config_default() {
        let margin = MarginConfig::default();
        assert_eq!(margin.top, 20);
        assert_eq!(margin.right, 20);
        assert_eq!(margin.bottom, 20);
        assert_eq!(margin.left, 20);
    }

    #[test]
    fn test_padding_config_default() {
        let padding = PaddingConfig::default();
        assert_eq!(padding.top, 10);
        assert_eq!(padding.right, 10);
        assert_eq!(padding.bottom, 10);
        assert_eq!(padding.left, 10);
    }

    #[test]
    fn test_theme_serde_roundtrip() {
        let theme = VisualizationTheme::default();
        let json = serde_json::to_string(&theme).unwrap();
        let deserialized: VisualizationTheme = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.primary_color, theme.primary_color);
        assert_eq!(deserialized.font_size, theme.font_size);
        assert_eq!(deserialized.dark_mode, theme.dark_mode);
        assert_eq!(deserialized.color_palette.len(), theme.color_palette.len());
    }

    #[test]
    fn test_layout_serde_roundtrip() {
        let layout = VisualizationLayout::default();
        let json = serde_json::to_string(&layout).unwrap();
        let deserialized: VisualizationLayout = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.width, layout.width);
        assert_eq!(deserialized.height, layout.height);
        assert_eq!(deserialized.responsive, layout.responsive);
    }

    #[test]
    fn test_margin_config_serde_roundtrip() {
        let margin = MarginConfig {
            top: 30,
            right: 40,
            bottom: 50,
            left: 60,
        };
        let json = serde_json::to_string(&margin).unwrap();
        let deserialized: MarginConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.top, 30);
        assert_eq!(deserialized.right, 40);
        assert_eq!(deserialized.bottom, 50);
        assert_eq!(deserialized.left, 60);
    }

    #[test]
    fn test_padding_config_serde_roundtrip() {
        let padding = PaddingConfig {
            top: 5,
            right: 15,
            bottom: 25,
            left: 35,
        };
        let json = serde_json::to_string(&padding).unwrap();
        let deserialized: PaddingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.top, 5);
        assert_eq!(deserialized.right, 15);
        assert_eq!(deserialized.bottom, 25);
        assert_eq!(deserialized.left, 35);
    }

    #[test]
    fn test_theme_custom_values() {
        let theme = VisualizationTheme {
            primary_color: "#ff0000".to_string(),
            secondary_color: "#00ff00".to_string(),
            background_color: "#000000".to_string(),
            text_color: "#ffffff".to_string(),
            border_color: "#888888".to_string(),
            font_family: "Monospace".to_string(),
            font_size: 16,
            color_palette: vec!["#111".to_string(), "#222".to_string()],
            dark_mode: true,
        };
        assert!(theme.dark_mode);
        assert_eq!(theme.font_size, 16);
        assert_eq!(theme.color_palette.len(), 2);
    }
}
