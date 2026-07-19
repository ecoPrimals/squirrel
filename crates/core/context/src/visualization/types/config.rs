// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration types for visualizations

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Visualization configuration.
///
/// Squirrel serializes context state as JSON/terminal output.
/// Presentation concerns (themes, layout, animation, export) are
/// delegated to petalTongue via `visualization.render.*` IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Output format (json, terminal)
    pub format: String,

    /// Custom options
    pub custom_options: HashMap<String, Value>,
}

// Default implementations
impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            format: "json".to_string(),
            custom_options: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_config_default() {
        let config = VisualizationConfig::default();
        assert_eq!(config.format, "json");
        assert!(config.custom_options.is_empty());
    }

    #[test]
    fn test_visualization_config_serde_roundtrip() {
        let config = VisualizationConfig::default();
        let json = serde_json::to_string(&config).expect("should succeed");
        let deserialized: VisualizationConfig =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.format, config.format);
    }

    #[test]
    fn test_visualization_config_custom_options() {
        let mut config = VisualizationConfig::default();
        config
            .custom_options
            .insert("key".to_string(), serde_json::json!("value"));
        assert_eq!(config.custom_options.len(), 1);

        let json = serde_json::to_string(&config).expect("should succeed");
        let deserialized: VisualizationConfig =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.custom_options.len(), 1);
    }
}
