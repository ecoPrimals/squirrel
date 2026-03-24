// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Metrics Visualization
//!
//! This module provides visualization capabilities for performance metrics,
//! system health, and usage analytics.

use std::collections::HashMap;
use serde_json::{json, Value};
use async_trait::async_trait;

use crate::{Result, ContextError};
use super::{VisualizationConfig, VisualizationFormat, VisualizationMetrics, RuleImpact, Visualizable, VisualizationData};

/// Metrics visualizer for performance and system metrics
#[derive(Debug)]
pub struct MetricsVisualizer {
    config: VisualizationConfig,
}

impl MetricsVisualizer {
    /// Create new metrics visualizer
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }
    
    /// Render metrics dashboard
    pub async fn render_dashboard(&self, metrics: &VisualizationMetrics, format: VisualizationFormat) -> Result<String> {
        let content = json!({
            "type": "metrics_dashboard",
            "metrics": metrics
        });
        
        self.render_with_format(content, format).await
    }
    
    /// Render performance heatmap
    pub async fn render_performance_heatmap(&self, rule_impacts: &HashMap<String, RuleImpact>, format: VisualizationFormat) -> Result<String> {
        let heatmap_data = self.generate_heatmap_data(rule_impacts);
        
        let content = json!({
            "type": "performance_heatmap",
            "heatmap_data": heatmap_data
        });
        
        self.render_with_format(content, format).await
    }
    
    /// Render system health overview
    pub async fn render_system_health(&self, metrics: &VisualizationMetrics, rule_impacts: &HashMap<String, RuleImpact>, format: VisualizationFormat) -> Result<String> {
        let health_data = self.calculate_system_health(metrics, rule_impacts);
        
        let content = json!({
            "type": "system_health",
            "health_data": health_data,
            "metrics": metrics
        });
        
        self.render_with_format(content, format).await
    }
    
    /// Generate heatmap data from rule impacts
    fn generate_heatmap_data(&self, rule_impacts: &HashMap<String, RuleImpact>) -> Value {
        let mut heatmap = json!({
            "execution_time_bands": {},
            "success_rate_bands": {},
            "usage_frequency_bands": {},
            "overall_distribution": {}
        });
        
        if rule_impacts.is_empty() {
            return heatmap;
        }
        
        // Define bands for categorization
        let time_bands = vec![
            (0u64, 1000u64, "Very Fast"),
            (1000, 5000, "Fast"),
            (5000, 10000, "Moderate"),
            (10000, 50000, "Slow"),
            (50000, u64::MAX, "Very Slow"),
        ];
        
        let success_bands = vec![
            (0.0, 0.5, "Poor"),
            (0.5, 0.7, "Fair"),
            (0.7, 0.8, "Good"),
            (0.8, 0.9, "Very Good"),
            (0.9, 1.0, "Excellent"),
        ];
        
        let usage_bands = vec![
            (0u64, 10u64, "Rarely Used"),
            (10, 50, "Occasionally Used"),
            (50, 100, "Frequently Used"),
            (100, 500, "Heavily Used"),
            (500, u64::MAX, "Extremely Used"),
        ];
        
        // Initialize counters
        let mut time_counters = HashMap::new();
        let mut success_counters = HashMap::new();
        let mut usage_counters = HashMap::new();
        
        for band in &time_bands {
            time_counters.insert(band.2, 0);
        }
        for band in &success_bands {
            success_counters.insert(band.2, 0);
        }
        for band in &usage_bands {
            usage_counters.insert(band.2, 0);
        }
        
        // Categorize rules
        for impact in rule_impacts.values() {
            // Execution time categorization
            for (min_time, max_time, label) in &time_bands {
                if impact.avg_execution_time_us >= *min_time && impact.avg_execution_time_us < *max_time {
                    *time_counters.get_mut(label).expect("example") += 1;
                    break;
                }
            }
            
            // Success rate categorization
            for (min_rate, max_rate, label) in &success_bands {
                if impact.success_rate >= *min_rate && impact.success_rate < *max_rate {
                    *success_counters.get_mut(label).expect("example") += 1;
                    break;
                }
            }
            
            // Usage frequency categorization
            for (min_usage, max_usage, label) in &usage_bands {
                if impact.apply_count >= *min_usage && impact.apply_count < *max_usage {
                    *usage_counters.get_mut(label).expect("example") += 1;
                    break;
                }
            }
        }
        
        heatmap["execution_time_bands"] = json!(time_counters);
        heatmap["success_rate_bands"] = json!(success_counters);
        heatmap["usage_frequency_bands"] = json!(usage_counters);
        
        // Overall distribution matrix (execution time vs success rate)
        let mut distribution = HashMap::new();
        for impact in rule_impacts.values() {
            let time_band = time_bands.iter().find(|(min, max, _)| {
                impact.avg_execution_time_us >= *min && impact.avg_execution_time_us < *max
            }).map(|(_, _, label)| *label).unwrap_or("Unknown");
            
            let success_band = success_bands.iter().find(|(min, max, _)| {
                impact.success_rate >= *min && impact.success_rate < *max
            }).map(|(_, _, label)| *label).unwrap_or("Unknown");
            
            let key = format!("{}_{}", time_band, success_band);
            *distribution.entry(key).or_insert(0) += 1;
        }
        
        heatmap["overall_distribution"] = json!(distribution);
        heatmap
    }
    
    /// Calculate system health metrics
    fn calculate_system_health(&self, metrics: &VisualizationMetrics, rule_impacts: &HashMap<String, RuleImpact>) -> Value {
        let mut health = json!({
            "overall_score": 0.0,
            "performance_score": 0.0,
            "reliability_score": 0.0,
            "efficiency_score": 0.0,
            "health_status": "Unknown",
            "recommendations": []
        });
        
        let mut performance_score = 0.0;
        let mut reliability_score = 0.0;
        let mut efficiency_score = 0.0;
        let mut recommendations = Vec::new();
        
        // Performance scoring (based on render times and memory usage)
        if metrics.total_render_time_us > 0 {
            let avg_render_time = metrics.total_render_time_us as f64 / metrics.total_state_changes.max(1) as f64;
            performance_score = if avg_render_time < 1000.0 {
                1.0 // Excellent
            } else if avg_render_time < 5000.0 {
                0.8 // Good
            } else if avg_render_time < 10000.0 {
                0.6 // Fair
            } else {
                0.4 // Poor
            };
            
            if performance_score < 0.7 {
                recommendations.push("Consider optimizing render performance");
            }
        } else {
            performance_score = 1.0; // No data, assume good
        }
        
        // Memory efficiency
        let memory_mb = metrics.memory_usage_bytes as f64 / 1024.0 / 1024.0;
        let memory_score = if memory_mb < 100.0 {
            1.0
        } else if memory_mb < 500.0 {
            0.8
        } else if memory_mb < 1000.0 {
            0.6
        } else {
            0.4
        };
        
        performance_score = (performance_score + memory_score) / 2.0;
        
        // Reliability scoring (based on rule success rates)
        if !rule_impacts.is_empty() {
            let avg_success_rate: f64 = rule_impacts.values().map(|r| r.success_rate).sum::<f64>() / rule_impacts.len() as f64;
            reliability_score = avg_success_rate;
            
            if reliability_score < 0.8 {
                recommendations.push("Some rules have low success rates - review rule logic");
            }
        } else {
            reliability_score = 1.0; // No rules to fail
        }
        
        // Efficiency scoring (based on cache hit rate and change frequency)
        efficiency_score = metrics.cache_hit_rate;
        
        if metrics.cache_hit_rate < 0.7 {
            recommendations.push("Cache hit rate is low - consider cache optimization");
        }
        
        if metrics.avg_change_frequency > 10.0 {
            recommendations.push("High change frequency detected - monitor system load");
        }
        
        // Overall score
        let overall_score = (performance_score + reliability_score + efficiency_score) / 3.0;
        
        let health_status = if overall_score >= 0.9 {
            "Excellent"
        } else if overall_score >= 0.8 {
            "Good"
        } else if overall_score >= 0.7 {
            "Fair"
        } else if overall_score >= 0.5 {
            "Poor"
        } else {
            "Critical"
        };
        
        health["overall_score"] = json!(overall_score);
        health["performance_score"] = json!(performance_score);
        health["reliability_score"] = json!(reliability_score);
        health["efficiency_score"] = json!(efficiency_score);
        health["health_status"] = json!(health_status);
        health["recommendations"] = json!(recommendations);
        
        health
    }
    
    /// Render metrics dashboard as text
    pub fn render_dashboard_text(&self, metrics: &VisualizationMetrics, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            output.push_str("\x1b[1;36m📊 Metrics Dashboard\x1b[0m\n");
        } else {
            output.push_str("📊 Metrics Dashboard\n");
        }
        
        // System overview
        if use_colors {
            output.push_str("\x1b[1;34m🔧 System Overview\x1b[0m\n");
            output.push_str(&format!("\x1b[33m├─ State Changes: {}\x1b[0m\n", metrics.total_state_changes));
            output.push_str(&format!("\x1b[33m├─ Change Frequency: {:.2}/s\x1b[0m\n", metrics.avg_change_frequency));
            output.push_str(&format!("\x1b[33m├─ Active Rules: {}\x1b[0m\n", metrics.active_rules_count));
            output.push_str(&format!("\x1b[33m└─ Memory Usage: {}\x1b[0m\n", self.format_memory_size(metrics.memory_usage_bytes)));
        } else {
            output.push_str("🔧 System Overview\n");
            output.push_str(&format!("├─ State Changes: {}\n", metrics.total_state_changes));
            output.push_str(&format!("├─ Change Frequency: {:.2}/s\n", metrics.avg_change_frequency));
            output.push_str(&format!("├─ Active Rules: {}\n", metrics.active_rules_count));
            output.push_str(&format!("└─ Memory Usage: {}\n", self.format_memory_size(metrics.memory_usage_bytes)));
        }
        
        output.push_str("\n");
        
        // Performance metrics
        if use_colors {
            output.push_str("\x1b[1;32m⚡ Performance Metrics\x1b[0m\n");
            output.push_str(&format!("\x1b[32m├─ Total Render Time: {}\x1b[0m\n", self.format_duration(metrics.total_render_time_us)));
            output.push_str(&format!("\x1b[32m├─ Avg Render Time: {}\x1b[0m\n", 
                self.format_duration(if metrics.total_state_changes > 0 {
                    metrics.total_render_time_us / metrics.total_state_changes
                } else {
                    0
                })));
            output.push_str(&format!("\x1b[32m└─ Cache Hit Rate: {:.1}%\x1b[0m\n", metrics.cache_hit_rate * 100.0));
        } else {
            output.push_str("⚡ Performance Metrics\n");
            output.push_str(&format!("├─ Total Render Time: {}\n", self.format_duration(metrics.total_render_time_us)));
            output.push_str(&format!("├─ Avg Render Time: {}\n", 
                self.format_duration(if metrics.total_state_changes > 0 {
                    metrics.total_render_time_us / metrics.total_state_changes
                } else {
                    0
                })));
            output.push_str(&format!("└─ Cache Hit Rate: {:.1}%\n", metrics.cache_hit_rate * 100.0));
        }
        
        output
    }
    
    /// Render performance heatmap as text
    pub fn render_performance_heatmap_text(&self, rule_impacts: &HashMap<String, RuleImpact>, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            output.push_str("\x1b[1;31m🔥 Performance Heatmap\x1b[0m\n");
        } else {
            output.push_str("🔥 Performance Heatmap\n");
        }
        
        if rule_impacts.is_empty() {
            output.push_str("No performance data available\n");
            return output;
        }
        
        let heatmap_data = self.generate_heatmap_data(rule_impacts);
        
        // Execution time distribution
        if use_colors {
            output.push_str("\x1b[1;33m⏱️  Execution Time Distribution\x1b[0m\n");
        } else {
            output.push_str("⏱️  Execution Time Distribution\n");
        }
        
        if let Some(time_bands) = heatmap_data["execution_time_bands"].as_object() {
            for (band, count) in time_bands {
                let count_val = count.as_u64().unwrap_or(0);
                let bar = self.create_text_bar(count_val, 20);
                
                if use_colors {
                    output.push_str(&format!("\x1b[36m├─ {}: {} ({})\x1b[0m\n", band, bar, count_val));
                } else {
                    output.push_str(&format!("├─ {}: {} ({})\n", band, bar, count_val));
                }
            }
        }
        
        output.push_str("\n");
        
        // Success rate distribution
        if use_colors {
            output.push_str("\x1b[1;32m✅ Success Rate Distribution\x1b[0m\n");
        } else {
            output.push_str("✅ Success Rate Distribution\n");
        }
        
        if let Some(success_bands) = heatmap_data["success_rate_bands"].as_object() {
            for (band, count) in success_bands {
                let count_val = count.as_u64().unwrap_or(0);
                let bar = self.create_text_bar(count_val, 20);
                
                if use_colors {
                    output.push_str(&format!("\x1b[32m├─ {}: {} ({})\x1b[0m\n", band, bar, count_val));
                } else {
                    output.push_str(&format!("├─ {}: {} ({})\n", band, bar, count_val));
                }
            }
        }
        
        output
    }
    
    /// Create a text-based bar chart
    fn create_text_bar(&self, value: u64, max_width: usize) -> String {
        if value == 0 {
            return "▫".repeat(max_width);
        }
        
        // Find max value for scaling
        let max_value = 100; // Assume reasonable max for scaling
        let width = ((value as f64 / max_value as f64) * max_width as f64).ceil() as usize;
        let width = width.min(max_width);
        
        "▪".repeat(width) + &"▫".repeat(max_width - width)
    }
    
    /// Format memory size for display
    fn format_memory_size(&self, bytes: u64) -> String {
        super::format_memory_size(bytes)
    }
    
    /// Format duration for display
    fn format_duration(&self, microseconds: u64) -> String {
        super::format_duration(microseconds)
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
                if let Some(viz_type) = content.get("type").and_then(|v| v.as_str()) {
                    match viz_type {
                        "metrics_dashboard" => {
                            if let Some(metrics_data) = content.get("metrics") {
                                let metrics: VisualizationMetrics = serde_json::from_value(metrics_data.clone())
                                    .map_err(|e| ContextError::VisualizationError(format!("Failed to parse metrics: {}", e)))?;
                                Ok(self.render_dashboard_text(&metrics, self.config.terminal_colors))
                            } else {
                                Err(ContextError::VisualizationError("Missing metrics data".to_string()))
                            }
                        }
                        "performance_heatmap" => {
                            if let Some(heatmap_data) = content.get("heatmap_data") {
                                // For now, render the heatmap data as text
                                Ok(serde_json::to_string_pretty(heatmap_data)
                                    .map_err(|e| ContextError::VisualizationError(format!("Failed to render heatmap: {}", e)))?)
                            } else {
                                Err(ContextError::VisualizationError("Missing heatmap data".to_string()))
                            }
                        }
                        "system_health" => {
                            if let Some(health_data) = content.get("health_data") {
                                let health_status = health_data["health_status"].as_str().unwrap_or("Unknown");
                                let overall_score = health_data["overall_score"].as_f64().unwrap_or(0.0);
                                
                                if self.config.terminal_colors {
                                    Ok(format!("\x1b[1;35m🏥 System Health: {} ({:.1}%)\x1b[0m\n{}", 
                                        health_status, overall_score * 100.0,
                                        serde_json::to_string_pretty(health_data).unwrap_or_default()))
                                } else {
                                    Ok(format!("🏥 System Health: {} ({:.1}%)\n{}", 
                                        health_status, overall_score * 100.0,
                                        serde_json::to_string_pretty(health_data).unwrap_or_default()))
                                }
                            } else {
                                Err(ContextError::VisualizationError("Missing health data".to_string()))
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
                let content_str = serde_json::to_string_pretty(&content)
                    .map_err(|e| ContextError::VisualizationError(format!("HTML rendering failed: {}", e)))?;
                Ok(format!("<pre>{}</pre>", content_str))
            }
            VisualizationFormat::PlainText => {
                serde_json::to_string(&content).map_err(|e| {
                    ContextError::VisualizationError(format!("Plain text rendering failed: {}", e))
                })
            }
            VisualizationFormat::Svg => {
                Ok(format!("<svg><text>{}</text></svg>", serde_json::to_string(&content).unwrap_or_default()))
            }
        }
    }
}

#[async_trait]
impl Visualizable for MetricsVisualizer {
    async fn render(&self, format: VisualizationFormat) -> Result<String> {
        let default_metrics = VisualizationMetrics::default();
        self.render_dashboard(&default_metrics, format).await
    }
    
    async fn get_data(&self) -> Result<VisualizationData> {
        Ok(VisualizationData {
            current_state: crate::ContextState {
                id: "default".to_string(),
                version: 1,
                timestamp: chrono::Utc::now().timestamp() as u64,
                data: HashMap::new(),
            },
            history: Vec::new(),
            rule_impacts: HashMap::new(),
            metrics: VisualizationMetrics::default(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        })
    }
    
    async fn update(&mut self, _data: VisualizationData) -> Result<()> {
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
    async fn test_metrics_visualizer_creation() {
        let config = VisualizationConfig::default();
        let visualizer = MetricsVisualizer::new(config);
        
        assert!(visualizer.supports_format(&VisualizationFormat::Json));
        assert!(visualizer.supports_format(&VisualizationFormat::Terminal));
    }
    
    #[tokio::test]
    async fn test_dashboard_rendering() {
        let config = VisualizationConfig::default();
        let visualizer = MetricsVisualizer::new(config);
        
        let metrics = VisualizationMetrics {
            total_state_changes: 100,
            avg_change_frequency: 5.5,
            memory_usage_bytes: 1048576, // 1MB
            active_rules_count: 10,
            total_render_time_us: 50000,
            cache_hit_rate: 0.85,
        };
        
        let result = visualizer.render_dashboard(&metrics, VisualizationFormat::Json).await;
        assert!(result.is_ok());
        
        let rendered = result.expect("should succeed");
        assert!(rendered.contains("metrics_dashboard"));
        assert!(rendered.contains("100"));
    }
    
    #[tokio::test]
    async fn test_heatmap_generation() {
        let config = VisualizationConfig::default();
        let visualizer = MetricsVisualizer::new(config);
        
        let mut rule_impacts = HashMap::new();
        rule_impacts.insert("fast_rule".to_string(), RuleImpact {
            rule_id: "fast_rule".to_string(),
            rule_name: "Fast Rule".to_string(),
            apply_count: 100,
            avg_execution_time_us: 500,
            success_rate: 0.95,
            last_applied: 1640995200,
            impact_score: 0.8,
        });
        
        rule_impacts.insert("slow_rule".to_string(), RuleImpact {
            rule_id: "slow_rule".to_string(),
            rule_name: "Slow Rule".to_string(),
            apply_count: 10,
            avg_execution_time_us: 15000,
            success_rate: 0.7,
            last_applied: 1640995200,
            impact_score: 0.3,
        });
        
        let heatmap_data = visualizer.generate_heatmap_data(&rule_impacts);
        
        assert!(heatmap_data["execution_time_bands"].is_object());
        assert!(heatmap_data["success_rate_bands"].is_object());
        assert!(heatmap_data["usage_frequency_bands"].is_object());
    }
    
    #[tokio::test]
    async fn test_system_health_calculation() {
        let config = VisualizationConfig::default();
        let visualizer = MetricsVisualizer::new(config);
        
        let metrics = VisualizationMetrics {
            total_state_changes: 100,
            avg_change_frequency: 2.0,
            memory_usage_bytes: 50 * 1024 * 1024, // 50MB
            active_rules_count: 5,
            total_render_time_us: 100000,
            cache_hit_rate: 0.9,
        };
        
        let mut rule_impacts = HashMap::new();
        rule_impacts.insert("rule1".to_string(), RuleImpact {
            rule_id: "rule1".to_string(),
            rule_name: "Good Rule".to_string(),
            apply_count: 50,
            avg_execution_time_us: 1000,
            success_rate: 0.95,
            last_applied: 1640995200,
            impact_score: 0.8,
        });
        
        let health_data = visualizer.calculate_system_health(&metrics, &rule_impacts);
        
        assert!(health_data["overall_score"].as_f64().expect("should succeed") > 0.0);
        assert!(health_data["health_status"].as_str().is_some());
        assert!(health_data["recommendations"].is_array());
    }
    
    #[test]
    fn test_text_bar_creation() {
        let config = VisualizationConfig::default();
        let visualizer = MetricsVisualizer::new(config);
        
        let bar = visualizer.create_text_bar(50, 10);
        assert_eq!(bar.len(), 10);
        assert!(bar.contains("▪"));
        assert!(bar.contains("▫"));
        
        let empty_bar = visualizer.create_text_bar(0, 10);
        assert_eq!(empty_bar, "▫".repeat(10));
    }
    
    #[test]
    fn test_text_rendering() {
        let config = VisualizationConfig::default();
        let visualizer = MetricsVisualizer::new(config);
        
        let metrics = VisualizationMetrics {
            total_state_changes: 100,
            avg_change_frequency: 5.5,
            memory_usage_bytes: 1048576,
            active_rules_count: 10,
            total_render_time_us: 50000,
            cache_hit_rate: 0.85,
        };
        
        let rendered = visualizer.render_dashboard_text(&metrics, false);
        assert!(rendered.contains("Metrics Dashboard"));
        assert!(rendered.contains("100"));
        assert!(rendered.contains("5.50"));
        
        let rendered_with_colors = visualizer.render_dashboard_text(&metrics, true);
        assert!(rendered_with_colors.contains("\x1b["));
    }
} 