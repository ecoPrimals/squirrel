// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! State Visualization
//!
//! This module provides state visualization capabilities including state trees,
//! timelines, and state diff visualization.

use std::collections::HashMap;
use serde_json::{json, Value};
use async_trait::async_trait;

use crate::{ContextState, Result, ContextError};
use super::{VisualizationConfig, VisualizationFormat, Visualizable, VisualizationData};

/// State visualizer for context states
#[derive(Debug)]
pub struct StateVisualizer {
    config: VisualizationConfig,
}

impl StateVisualizer {
    /// Create new state visualizer
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }
    
    /// Render state tree visualization
    pub async fn render_state_tree(&self, state: &ContextState, format: VisualizationFormat) -> Result<String> {
        let content = json!({
            "type": "state_tree",
            "data": state
        });
        
        self.render_with_format(content, format).await
    }
    
    /// Render timeline visualization
    pub async fn render_timeline(&self, history: &[ContextState], format: VisualizationFormat) -> Result<String> {
        let content = json!({
            "type": "timeline",
            "history": history
        });
        
        self.render_with_format(content, format).await
    }
    
    /// Render state diff visualization
    pub async fn render_state_diff(&self, old_state: &ContextState, new_state: &ContextState, format: VisualizationFormat) -> Result<String> {
        let diff = self.calculate_state_diff(old_state, new_state);
        
        let content = json!({
            "type": "state_diff",
            "old_state": old_state,
            "new_state": new_state,
            "diff": diff
        });
        
        self.render_with_format(content, format).await
    }
    
    /// Calculate differences between two states
    fn calculate_state_diff(&self, old_state: &ContextState, new_state: &ContextState) -> Value {
        let mut diff = json!({
            "version_changed": old_state.version != new_state.version,
            "timestamp_changed": old_state.timestamp != new_state.timestamp,
            "data_changes": {
                "added": {},
                "removed": {},
                "modified": {}
            }
        });
        
        // Calculate data changes
        let mut added = json!({});
        let mut removed = json!({});
        let mut modified = json!({});
        
        // Find added and modified keys
        for (key, new_value) in &new_state.data {
            if let Some(old_value) = old_state.data.get(key) {
                if old_value != new_value {
                    modified[key] = json!({
                        "old": old_value,
                        "new": new_value
                    });
                }
            } else {
                added[key] = new_value.clone();
            }
        }
        
        // Find removed keys
        for (key, old_value) in &old_state.data {
            if !new_state.data.contains_key(key) {
                removed[key] = old_value.clone();
            }
        }
        
        diff["data_changes"]["added"] = added;
        diff["data_changes"]["removed"] = removed;
        diff["data_changes"]["modified"] = modified;
        
        diff
    }
    
    /// Render state tree as text
    pub fn render_state_tree_text(&self, state: &ContextState, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            output.push_str(&format!("\x1b[1;34m🌳 State Tree: {}\x1b[0m\n", state.id));
            output.push_str(&format!("\x1b[36m├─ ID: {}\x1b[0m\n", state.id));
            output.push_str(&format!("\x1b[36m├─ Version: {}\x1b[0m\n", state.version));
            output.push_str(&format!("\x1b[36m├─ Timestamp: {}\x1b[0m\n", self.format_timestamp(state.timestamp)));
            output.push_str(&format!("\x1b[36m└─ Data:\x1b[0m\n"));
            
            for (i, (key, value)) in state.data.iter().enumerate() {
                let connector = if i == state.data.len() - 1 { "└─" } else { "├─" };
                output.push_str(&format!("   \x1b[32m{} {}: {}\x1b[0m\n", connector, key, value));
            }
        } else {
            output.push_str(&format!("🌳 State Tree: {}\n", state.id));
            output.push_str(&format!("├─ ID: {}\n", state.id));
            output.push_str(&format!("├─ Version: {}\n", state.version));
            output.push_str(&format!("├─ Timestamp: {}\n", self.format_timestamp(state.timestamp)));
            output.push_str("└─ Data:\n");
            
            for (i, (key, value)) in state.data.iter().enumerate() {
                let connector = if i == state.data.len() - 1 { "└─" } else { "├─" };
                output.push_str(&format!("   {} {}: {}\n", connector, key, value));
            }
        }
        
        output
    }
    
    /// Render timeline as text
    pub fn render_timeline_text(&self, history: &[ContextState], use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            output.push_str("\x1b[1;35m📅 Timeline\x1b[0m\n");
        } else {
            output.push_str("📅 Timeline\n");
        }
        
        for (i, state) in history.iter().enumerate() {
            let connector = if i == history.len() - 1 { "└─" } else { "├─" };
            let timestamp = self.format_timestamp(state.timestamp);
            
            if use_colors {
                output.push_str(&format!(
                    "\x1b[33m{} [{}] {} (v{}) - {} items\x1b[0m\n",
                    connector, timestamp, state.id, state.version, state.data.len()
                ));
            } else {
                output.push_str(&format!(
                    "{} [{}] {} (v{}) - {} items\n",
                    connector, timestamp, state.id, state.version, state.data.len()
                ));
            }
        }
        
        output
    }
    
    /// Render state diff as text
    pub fn render_state_diff_text(&self, old_state: &ContextState, new_state: &ContextState, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            output.push_str("\x1b[1;36m🔄 State Diff\x1b[0m\n");
            output.push_str(&format!("\x1b[31m- {} (v{})\x1b[0m\n", old_state.id, old_state.version));
            output.push_str(&format!("\x1b[32m+ {} (v{})\x1b[0m\n", new_state.id, new_state.version));
        } else {
            output.push_str("🔄 State Diff\n");
            output.push_str(&format!("- {} (v{})\n", old_state.id, old_state.version));
            output.push_str(&format!("+ {} (v{})\n", new_state.id, new_state.version));
        }
        
        // Version changes
        if old_state.version != new_state.version {
            if use_colors {
                output.push_str(&format!("\x1b[33m~ Version: {} → {}\x1b[0m\n", old_state.version, new_state.version));
            } else {
                output.push_str(&format!("~ Version: {} → {}\n", old_state.version, new_state.version));
            }
        }
        
        // Timestamp changes
        if old_state.timestamp != new_state.timestamp {
            if use_colors {
                output.push_str(&format!("\x1b[33m~ Timestamp: {} → {}\x1b[0m\n", 
                    self.format_timestamp(old_state.timestamp), 
                    self.format_timestamp(new_state.timestamp)));
            } else {
                output.push_str(&format!("~ Timestamp: {} → {}\n", 
                    self.format_timestamp(old_state.timestamp), 
                    self.format_timestamp(new_state.timestamp)));
            }
        }
        
        // Data changes
        output.push_str("Data changes:\n");
        
        // Added keys
        for (key, value) in &new_state.data {
            if !old_state.data.contains_key(key) {
                if use_colors {
                    output.push_str(&format!("  \x1b[32m+ {}: {}\x1b[0m\n", key, value));
                } else {
                    output.push_str(&format!("  + {}: {}\n", key, value));
                }
            }
        }
        
        // Removed keys
        for (key, value) in &old_state.data {
            if !new_state.data.contains_key(key) {
                if use_colors {
                    output.push_str(&format!("  \x1b[31m- {}: {}\x1b[0m\n", key, value));
                } else {
                    output.push_str(&format!("  - {}: {}\n", key, value));
                }
            }
        }
        
        // Modified keys
        for (key, new_value) in &new_state.data {
            if let Some(old_value) = old_state.data.get(key) {
                if old_value != new_value {
                    if use_colors {
                        output.push_str(&format!("  \x1b[33m~ {}: {} → {}\x1b[0m\n", key, old_value, new_value));
                    } else {
                        output.push_str(&format!("  ~ {}: {} → {}\n", key, old_value, new_value));
                    }
                }
            }
        }
        
        output
    }
    
    /// Render with specific format
    async fn render_with_format(&self, content: Value, format: VisualizationFormat) -> Result<String> {
        match format {
            VisualizationFormat::Json => {
                serde_json::to_string_pretty(&content).map_err(|e| {
                    ContextError::VisualizationError(format!("JSON rendering failed: {}", e))
                })
            }
            VisualizationFormat::Terminal => {
                // Handle terminal-specific rendering
                if let Some(viz_type) = content.get("type").and_then(|v| v.as_str()) {
                    match viz_type {
                        "state_tree" => {
                            if let Some(state_data) = content.get("data") {
                                let state: ContextState = serde_json::from_value(state_data.clone())
                                    .map_err(|e| ContextError::VisualizationError(format!("Failed to parse state: {}", e)))?;
                                Ok(self.render_state_tree_text(&state, self.config.terminal_colors))
                            } else {
                                Err(ContextError::VisualizationError("Missing state data".to_string()))
                            }
                        }
                        "timeline" => {
                            if let Some(history_data) = content.get("history") {
                                let history: Vec<ContextState> = serde_json::from_value(history_data.clone())
                                    .map_err(|e| ContextError::VisualizationError(format!("Failed to parse history: {}", e)))?;
                                Ok(self.render_timeline_text(&history, self.config.terminal_colors))
                            } else {
                                Err(ContextError::VisualizationError("Missing history data".to_string()))
                            }
                        }
                        "state_diff" => {
                            if let Some(old_data) = content.get("old_state") {
                                if let Some(new_data) = content.get("new_state") {
                                    let old_state: ContextState = serde_json::from_value(old_data.clone())
                                        .map_err(|e| ContextError::VisualizationError(format!("Failed to parse old state: {}", e)))?;
                                    let new_state: ContextState = serde_json::from_value(new_data.clone())
                                        .map_err(|e| ContextError::VisualizationError(format!("Failed to parse new state: {}", e)))?;
                                    Ok(self.render_state_diff_text(&old_state, &new_state, self.config.terminal_colors))
                                } else {
                                    Err(ContextError::VisualizationError("Missing new state data".to_string()))
                                }
                            } else {
                                Err(ContextError::VisualizationError("Missing old state data".to_string()))
                            }
                        }
                        _ => {
                            Ok(serde_json::to_string_pretty(&content)
                                .map_err(|e| ContextError::VisualizationError(format!("Terminal rendering failed: {}", e)))?)
                        }
                    }
                } else {
                    Ok(serde_json::to_string_pretty(&content)
                        .map_err(|e| ContextError::VisualizationError(format!("Terminal rendering failed: {}", e)))?)
                }
            }
            VisualizationFormat::Html => {
                // Basic HTML rendering - could be enhanced with proper HTML renderer
                let content_str = serde_json::to_string_pretty(&content)
                    .map_err(|e| ContextError::VisualizationError(format!("HTML rendering failed: {}", e)))?;
                Ok(format!("<pre>{}</pre>", content_str))
            }
            VisualizationFormat::PlainText => {
                // Plain text rendering
                serde_json::to_string(&content).map_err(|e| {
                    ContextError::VisualizationError(format!("Plain text rendering failed: {}", e))
                })
            }
            VisualizationFormat::Svg => {
                // SVG rendering - placeholder for now
                Ok(format!("<svg><text>{}</text></svg>", serde_json::to_string(&content).unwrap_or_default()))
            }
        }
    }
    
    /// Format timestamp for display
    fn format_timestamp(&self, timestamp: u64) -> String {
        chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

#[async_trait]
impl Visualizable for StateVisualizer {
    async fn render(&self, format: VisualizationFormat) -> Result<String> {
        // Default state visualization
        let default_state = ContextState {
            id: "default".to_string(),
            version: 1,
            timestamp: chrono::Utc::now().timestamp() as u64,
            data: HashMap::new(),
        };
        
        self.render_state_tree(&default_state, format).await
    }
    
    async fn get_data(&self) -> Result<VisualizationData> {
        // Return default visualization data
        Ok(VisualizationData {
            current_state: ContextState {
                id: "default".to_string(),
                version: 1,
                timestamp: chrono::Utc::now().timestamp() as u64,
                data: HashMap::new(),
            },
            history: Vec::new(),
            rule_impacts: HashMap::new(),
            metrics: super::VisualizationMetrics::default(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        })
    }
    
    async fn update(&mut self, _data: VisualizationData) -> Result<()> {
        // State visualizer doesn't maintain its own state
        Ok(())
    }
    
    fn supports_format(&self, format: &VisualizationFormat) -> bool {
        matches!(format, 
            VisualizationFormat::Json | 
            VisualizationFormat::Terminal | 
            VisualizationFormat::Html | 
            VisualizationFormat::PlainText |
            VisualizationFormat::Svg
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_state_visualizer_creation() {
        let config = VisualizationConfig::default();
        let visualizer = StateVisualizer::new(config);
        
        assert!(visualizer.supports_format(&VisualizationFormat::Json));
        assert!(visualizer.supports_format(&VisualizationFormat::Terminal));
        assert!(visualizer.supports_format(&VisualizationFormat::Html));
    }
    
    #[tokio::test]
    async fn test_state_tree_rendering() {
        let config = VisualizationConfig::default();
        let visualizer = StateVisualizer::new(config);
        
        let mut data = HashMap::new();
        data.insert("key1".to_string(), "value1".to_string());
        data.insert("key2".to_string(), "value2".to_string());
        
        let state = ContextState {
            id: "test-state".to_string(),
            version: 1,
            timestamp: 1640995200,
            data,
        };
        
        let result = visualizer.render_state_tree(&state, VisualizationFormat::Json).await;
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        assert!(rendered.contains("state_tree"));
        assert!(rendered.contains("test-state"));
    }
    
    #[tokio::test]
    async fn test_timeline_rendering() {
        let config = VisualizationConfig::default();
        let visualizer = StateVisualizer::new(config);
        
        let state1 = ContextState {
            id: "state1".to_string(),
            version: 1,
            timestamp: 1640995200,
            data: HashMap::new(),
        };
        
        let state2 = ContextState {
            id: "state2".to_string(),
            version: 2,
            timestamp: 1640995260,
            data: HashMap::new(),
        };
        
        let history = vec![state1, state2];
        let result = visualizer.render_timeline(&history, VisualizationFormat::Json).await;
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        assert!(rendered.contains("timeline"));
        assert!(rendered.contains("state1"));
        assert!(rendered.contains("state2"));
    }
    
    #[tokio::test]
    async fn test_state_diff_rendering() {
        let config = VisualizationConfig::default();
        let visualizer = StateVisualizer::new(config);
        
        let mut old_data = HashMap::new();
        old_data.insert("key1".to_string(), "old_value".to_string());
        old_data.insert("key2".to_string(), "unchanged".to_string());
        
        let mut new_data = HashMap::new();
        new_data.insert("key1".to_string(), "new_value".to_string());
        new_data.insert("key2".to_string(), "unchanged".to_string());
        new_data.insert("key3".to_string(), "added".to_string());
        
        let old_state = ContextState {
            id: "state".to_string(),
            version: 1,
            timestamp: 1640995200,
            data: old_data,
        };
        
        let new_state = ContextState {
            id: "state".to_string(),
            version: 2,
            timestamp: 1640995260,
            data: new_data,
        };
        
        let result = visualizer.render_state_diff(&old_state, &new_state, VisualizationFormat::Json).await;
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        assert!(rendered.contains("state_diff"));
        assert!(rendered.contains("modified"));
        assert!(rendered.contains("added"));
    }
    
    #[test]
    fn test_state_diff_calculation() {
        let config = VisualizationConfig::default();
        let visualizer = StateVisualizer::new(config);
        
        let mut old_data = HashMap::new();
        old_data.insert("key1".to_string(), "old_value".to_string());
        old_data.insert("key2".to_string(), "unchanged".to_string());
        old_data.insert("key3".to_string(), "removed".to_string());
        
        let mut new_data = HashMap::new();
        new_data.insert("key1".to_string(), "new_value".to_string());
        new_data.insert("key2".to_string(), "unchanged".to_string());
        new_data.insert("key4".to_string(), "added".to_string());
        
        let old_state = ContextState {
            id: "state".to_string(),
            version: 1,
            timestamp: 1640995200,
            data: old_data,
        };
        
        let new_state = ContextState {
            id: "state".to_string(),
            version: 2,
            timestamp: 1640995260,
            data: new_data,
        };
        
        let diff = visualizer.calculate_state_diff(&old_state, &new_state);
        
        assert!(diff["version_changed"].as_bool().unwrap());
        assert!(diff["timestamp_changed"].as_bool().unwrap());
        assert!(diff["data_changes"]["added"]["key4"].as_str().unwrap() == "added");
        assert!(diff["data_changes"]["removed"]["key3"].as_str().unwrap() == "removed");
        assert!(diff["data_changes"]["modified"]["key1"]["new"].as_str().unwrap() == "new_value");
    }
    
    #[test]
    fn test_text_rendering() {
        let config = VisualizationConfig::default();
        let visualizer = StateVisualizer::new(config);
        
        let mut data = HashMap::new();
        data.insert("key1".to_string(), "value1".to_string());
        
        let state = ContextState {
            id: "test-state".to_string(),
            version: 1,
            timestamp: 1640995200,
            data,
        };
        
        let rendered = visualizer.render_state_tree_text(&state, false);
        assert!(rendered.contains("State Tree"));
        assert!(rendered.contains("test-state"));
        assert!(rendered.contains("key1"));
        assert!(rendered.contains("value1"));
        
        let rendered_with_colors = visualizer.render_state_tree_text(&state, true);
        assert!(rendered_with_colors.contains("\x1b["));
    }
} 