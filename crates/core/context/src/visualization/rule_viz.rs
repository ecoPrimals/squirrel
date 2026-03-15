// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Rule Visualization
//!
//! This module provides visualization capabilities for rules including dependency graphs,
//! impact analysis, and performance metrics.

use std::collections::HashMap;
use serde_json::{json, Value};
use async_trait::async_trait;

use crate::{Result, ContextError};
use super::{VisualizationConfig, VisualizationFormat, RuleImpact, Visualizable, VisualizationData};

/// Rule visualizer for rule-related visualizations
#[derive(Debug)]
pub struct RuleVisualizer {
    config: VisualizationConfig,
}

impl RuleVisualizer {
    /// Create new rule visualizer
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }
    
    /// Render rule dependency graph
    pub async fn render_dependency_graph(&self, rule_impacts: &HashMap<String, RuleImpact>, format: VisualizationFormat) -> Result<String> {
        let content = json!({
            "type": "rule_dependency_graph",
            "rule_impacts": rule_impacts
        });
        
        self.render_with_format(content, format).await
    }
    
    /// Render rule impact analysis
    pub async fn render_impact_analysis(&self, rule_impacts: &HashMap<String, RuleImpact>, format: VisualizationFormat) -> Result<String> {
        let content = json!({
            "type": "rule_impact_analysis",
            "rule_impacts": rule_impacts
        });
        
        self.render_with_format(content, format).await
    }
    
    /// Render rule performance metrics
    pub async fn render_performance_metrics(&self, rule_impacts: &HashMap<String, RuleImpact>, format: VisualizationFormat) -> Result<String> {
        let performance_data = self.calculate_performance_metrics(rule_impacts);
        
        let content = json!({
            "type": "rule_performance_metrics",
            "performance_data": performance_data
        });
        
        self.render_with_format(content, format).await
    }
    
    /// Calculate performance metrics for rules
    fn calculate_performance_metrics(&self, rule_impacts: &HashMap<String, RuleImpact>) -> Value {
        let mut metrics = json!({
            "total_rules": rule_impacts.len(),
            "total_applications": 0,
            "avg_success_rate": 0.0,
            "avg_execution_time": 0.0,
            "top_performers": [],
            "bottom_performers": [],
            "most_used": [],
            "least_used": []
        });
        
        if rule_impacts.is_empty() {
            return metrics;
        }
        
        // Calculate totals
        let total_applications: u64 = rule_impacts.values().map(|r| r.apply_count).sum();
        let avg_success_rate: f64 = rule_impacts.values().map(|r| r.success_rate).sum::<f64>() / rule_impacts.len() as f64;
        let avg_execution_time: f64 = rule_impacts.values().map(|r| r.avg_execution_time_us as f64).sum::<f64>() / rule_impacts.len() as f64;
        
        metrics["total_applications"] = json!(total_applications);
        metrics["avg_success_rate"] = json!(avg_success_rate);
        metrics["avg_execution_time"] = json!(avg_execution_time);
        
        // Sort rules by different criteria
        let mut by_impact: Vec<_> = rule_impacts.iter().collect();
        by_impact.sort_by(|a, b| b.1.impact_score.partial_cmp(&a.1.impact_score).unwrap());
        
        let mut by_usage: Vec<_> = rule_impacts.iter().collect();
        by_usage.sort_by(|a, b| b.1.apply_count.cmp(&a.1.apply_count));
        
        // Top and bottom performers (by impact score)
        let top_performers: Vec<_> = by_impact.iter().take(5).map(|(id, impact)| {
            json!({
                "id": id,
                "name": impact.rule_name,
                "impact_score": impact.impact_score,
                "success_rate": impact.success_rate,
                "apply_count": impact.apply_count
            })
        }).collect();
        
        let bottom_performers: Vec<_> = by_impact.iter().rev().take(5).map(|(id, impact)| {
            json!({
                "id": id,
                "name": impact.rule_name,
                "impact_score": impact.impact_score,
                "success_rate": impact.success_rate,
                "apply_count": impact.apply_count
            })
        }).collect();
        
        // Most and least used rules
        let most_used: Vec<_> = by_usage.iter().take(5).map(|(id, impact)| {
            json!({
                "id": id,
                "name": impact.rule_name,
                "apply_count": impact.apply_count,
                "success_rate": impact.success_rate
            })
        }).collect();
        
        let least_used: Vec<_> = by_usage.iter().rev().take(5).map(|(id, impact)| {
            json!({
                "id": id,
                "name": impact.rule_name,
                "apply_count": impact.apply_count,
                "success_rate": impact.success_rate
            })
        }).collect();
        
        metrics["top_performers"] = json!(top_performers);
        metrics["bottom_performers"] = json!(bottom_performers);
        metrics["most_used"] = json!(most_used);
        metrics["least_used"] = json!(least_used);
        
        metrics
    }
    
    /// Render rule dependency graph as text
    pub fn render_dependency_graph_text(&self, rule_impacts: &HashMap<String, RuleImpact>, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            output.push_str("\x1b[1;35m🔗 Rule Dependency Graph\x1b[0m\n");
        } else {
            output.push_str("🔗 Rule Dependency Graph\n");
        }
        
        if rule_impacts.is_empty() {
            output.push_str("No rules available\n");
            return output;
        }
        
        // Sort rules by impact score
        let mut sorted_rules: Vec<_> = rule_impacts.iter().collect();
        sorted_rules.sort_by(|a, b| b.1.impact_score.partial_cmp(&a.1.impact_score).unwrap());
        
        for (i, (rule_id, impact)) in sorted_rules.iter().enumerate() {
            let connector = if i == sorted_rules.len() - 1 { "└─" } else { "├─" };
            let impact_indicator = self.get_impact_indicator(impact.impact_score);
            
            if use_colors {
                output.push_str(&format!(
                    "\x1b[36m{} {} {} ({})\x1b[0m\n",
                    connector, impact_indicator, impact.rule_name, rule_id
                ));
                output.push_str(&format!(
                    "   \x1b[37m├─ Impact: {:.3}\x1b[0m\n",
                    impact.impact_score
                ));
                output.push_str(&format!(
                    "   \x1b[37m├─ Applications: {}\x1b[0m\n",
                    impact.apply_count
                ));
                output.push_str(&format!(
                    "   \x1b[37m└─ Success Rate: {:.1}%\x1b[0m\n",
                    impact.success_rate * 100.0
                ));
            } else {
                output.push_str(&format!(
                    "{} {} {} ({})\n",
                    connector, impact_indicator, impact.rule_name, rule_id
                ));
                output.push_str(&format!(
                    "   ├─ Impact: {:.3}\n",
                    impact.impact_score
                ));
                output.push_str(&format!(
                    "   ├─ Applications: {}\n",
                    impact.apply_count
                ));
                output.push_str(&format!(
                    "   └─ Success Rate: {:.1}%\n",
                    impact.success_rate * 100.0
                ));
            }
        }
        
        output
    }
    
    /// Render rule impact analysis as text
    pub fn render_impact_analysis_text(&self, rule_impacts: &HashMap<String, RuleImpact>, use_colors: bool) -> String {
        let mut output = String::new();
        
        if use_colors {
            output.push_str("\x1b[1;31m📊 Rule Impact Analysis\x1b[0m\n");
        } else {
            output.push_str("📊 Rule Impact Analysis\n");
        }
        
        if rule_impacts.is_empty() {
            output.push_str("No rule impacts to analyze\n");
            return output;
        }
        
        let performance_data = self.calculate_performance_metrics(rule_impacts);
        
        // Summary statistics
        if use_colors {
            output.push_str("\x1b[1;34m📈 Summary Statistics\x1b[0m\n");
            output.push_str(&format!("\x1b[33m├─ Total Rules: {}\x1b[0m\n", performance_data["total_rules"]));
            output.push_str(&format!("\x1b[33m├─ Total Applications: {}\x1b[0m\n", performance_data["total_applications"]));
            output.push_str(&format!("\x1b[33m├─ Avg Success Rate: {:.1}%\x1b[0m\n", performance_data["avg_success_rate"].as_f64().unwrap_or(0.0) * 100.0));
            output.push_str(&format!("\x1b[33m└─ Avg Execution Time: {:.2}μs\x1b[0m\n", performance_data["avg_execution_time"].as_f64().unwrap_or(0.0)));
        } else {
            output.push_str("📈 Summary Statistics\n");
            output.push_str(&format!("├─ Total Rules: {}\n", performance_data["total_rules"]));
            output.push_str(&format!("├─ Total Applications: {}\n", performance_data["total_applications"]));
            output.push_str(&format!("├─ Avg Success Rate: {:.1}%\n", performance_data["avg_success_rate"].as_f64().unwrap_or(0.0) * 100.0));
            output.push_str(&format!("└─ Avg Execution Time: {:.2}μs\n", performance_data["avg_execution_time"].as_f64().unwrap_or(0.0)));
        }
        
        output.push_str("\n");
        
        // Top performers
        if let Some(top_performers) = performance_data["top_performers"].as_array() {
            if !top_performers.is_empty() {
                if use_colors {
                    output.push_str("\x1b[1;32m🏆 Top Performers\x1b[0m\n");
                } else {
                    output.push_str("🏆 Top Performers\n");
                }
                
                for (i, performer) in top_performers.iter().enumerate() {
                    let connector = if i == top_performers.len() - 1 { "└─" } else { "├─" };
                    let name = performer["name"].as_str().unwrap_or("Unknown");
                    let impact_score = performer["impact_score"].as_f64().unwrap_or(0.0);
                    
                    if use_colors {
                        output.push_str(&format!("\x1b[32m{} {} (Impact: {:.3})\x1b[0m\n", connector, name, impact_score));
                    } else {
                        output.push_str(&format!("{} {} (Impact: {:.3})\n", connector, name, impact_score));
                    }
                }
            }
        }
        
        output.push_str("\n");
        
        // Most used rules
        if let Some(most_used) = performance_data["most_used"].as_array() {
            if !most_used.is_empty() {
                if use_colors {
                    output.push_str("\x1b[1;36m🔥 Most Used Rules\x1b[0m\n");
                } else {
                    output.push_str("🔥 Most Used Rules\n");
                }
                
                for (i, rule) in most_used.iter().enumerate() {
                    let connector = if i == most_used.len() - 1 { "└─" } else { "├─" };
                    let name = rule["name"].as_str().unwrap_or("Unknown");
                    let apply_count = rule["apply_count"].as_u64().unwrap_or(0);
                    
                    if use_colors {
                        output.push_str(&format!("\x1b[36m{} {} ({} applications)\x1b[0m\n", connector, name, apply_count));
                    } else {
                        output.push_str(&format!("{} {} ({} applications)\n", connector, name, apply_count));
                    }
                }
            }
        }
        
        output
    }
    
    /// Get impact indicator emoji based on impact score
    fn get_impact_indicator(&self, impact_score: f64) -> &'static str {
        if impact_score >= 0.8 {
            "🔥" // High impact
        } else if impact_score >= 0.6 {
            "⚡" // Medium-high impact
        } else if impact_score >= 0.4 {
            "💡" // Medium impact
        } else if impact_score >= 0.2 {
            "📝" // Low-medium impact
        } else {
            "💤" // Low impact
        }
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
                        "rule_dependency_graph" => {
                            if let Some(impacts_data) = content.get("rule_impacts") {
                                let impacts: HashMap<String, RuleImpact> = serde_json::from_value(impacts_data.clone())
                                    .map_err(|e| ContextError::VisualizationError(format!("Failed to parse rule impacts: {}", e)))?;
                                Ok(self.render_dependency_graph_text(&impacts, self.config.terminal_colors))
                            } else {
                                Err(ContextError::VisualizationError("Missing rule impacts data".to_string()))
                            }
                        }
                        "rule_impact_analysis" => {
                            if let Some(impacts_data) = content.get("rule_impacts") {
                                let impacts: HashMap<String, RuleImpact> = serde_json::from_value(impacts_data.clone())
                                    .map_err(|e| ContextError::VisualizationError(format!("Failed to parse rule impacts: {}", e)))?;
                                Ok(self.render_impact_analysis_text(&impacts, self.config.terminal_colors))
                            } else {
                                Err(ContextError::VisualizationError("Missing rule impacts data".to_string()))
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
                // Basic HTML rendering
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
                // SVG rendering - placeholder for now
                Ok(format!("<svg><text>{}</text></svg>", serde_json::to_string(&content).unwrap_or_default()))
            }
        }
    }
}

#[async_trait]
impl Visualizable for RuleVisualizer {
    async fn render(&self, format: VisualizationFormat) -> Result<String> {
        // Default rule visualization
        let default_impacts = HashMap::new();
        self.render_dependency_graph(&default_impacts, format).await
    }
    
    async fn get_data(&self) -> Result<VisualizationData> {
        // Return default visualization data
        Ok(VisualizationData {
            current_state: crate::ContextState {
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
        // Rule visualizer doesn't maintain its own state
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
    async fn test_rule_visualizer_creation() {
        let config = VisualizationConfig::default();
        let visualizer = RuleVisualizer::new(config);
        
        assert!(visualizer.supports_format(&VisualizationFormat::Json));
        assert!(visualizer.supports_format(&VisualizationFormat::Terminal));
        assert!(visualizer.supports_format(&VisualizationFormat::Html));
    }
    
    #[tokio::test]
    async fn test_dependency_graph_rendering() {
        let config = VisualizationConfig::default();
        let visualizer = RuleVisualizer::new(config);
        
        let mut rule_impacts = HashMap::new();
        rule_impacts.insert("rule1".to_string(), RuleImpact {
            rule_id: "rule1".to_string(),
            rule_name: "Test Rule 1".to_string(),
            apply_count: 10,
            avg_execution_time_us: 1000,
            success_rate: 0.95,
            last_applied: 1640995200,
            impact_score: 0.8,
        });
        
        let result = visualizer.render_dependency_graph(&rule_impacts, VisualizationFormat::Json).await;
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        assert!(rendered.contains("rule_dependency_graph"));
        assert!(rendered.contains("Test Rule 1"));
    }
    
    #[tokio::test]
    async fn test_impact_analysis_rendering() {
        let config = VisualizationConfig::default();
        let visualizer = RuleVisualizer::new(config);
        
        let mut rule_impacts = HashMap::new();
        rule_impacts.insert("rule1".to_string(), RuleImpact {
            rule_id: "rule1".to_string(),
            rule_name: "High Impact Rule".to_string(),
            apply_count: 100,
            avg_execution_time_us: 500,
            success_rate: 0.99,
            last_applied: 1640995200,
            impact_score: 0.9,
        });
        
        rule_impacts.insert("rule2".to_string(), RuleImpact {
            rule_id: "rule2".to_string(),
            rule_name: "Low Impact Rule".to_string(),
            apply_count: 5,
            avg_execution_time_us: 2000,
            success_rate: 0.6,
            last_applied: 1640995200,
            impact_score: 0.2,
        });
        
        let result = visualizer.render_impact_analysis(&rule_impacts, VisualizationFormat::Json).await;
        assert!(result.is_ok());
        
        let rendered = result.unwrap();
        assert!(rendered.contains("rule_impact_analysis"));
        assert!(rendered.contains("High Impact Rule"));
        assert!(rendered.contains("Low Impact Rule"));
    }
    
    #[test]
    fn test_performance_metrics_calculation() {
        let config = VisualizationConfig::default();
        let visualizer = RuleVisualizer::new(config);
        
        let mut rule_impacts = HashMap::new();
        rule_impacts.insert("rule1".to_string(), RuleImpact {
            rule_id: "rule1".to_string(),
            rule_name: "Rule 1".to_string(),
            apply_count: 100,
            avg_execution_time_us: 1000,
            success_rate: 0.95,
            last_applied: 1640995200,
            impact_score: 0.8,
        });
        
        rule_impacts.insert("rule2".to_string(), RuleImpact {
            rule_id: "rule2".to_string(),
            rule_name: "Rule 2".to_string(),
            apply_count: 50,
            avg_execution_time_us: 2000,
            success_rate: 0.85,
            last_applied: 1640995200,
            impact_score: 0.6,
        });
        
        let metrics = visualizer.calculate_performance_metrics(&rule_impacts);
        
        assert_eq!(metrics["total_rules"], 2);
        assert_eq!(metrics["total_applications"], 150);
        assert_eq!(metrics["avg_success_rate"], 0.9);
        assert_eq!(metrics["avg_execution_time"], 1500.0);
        
        let top_performers = metrics["top_performers"].as_array().unwrap();
        assert_eq!(top_performers.len(), 2);
        assert_eq!(top_performers[0]["name"], "Rule 1");
    }
    
    #[test]
    fn test_text_rendering() {
        let config = VisualizationConfig::default();
        let visualizer = RuleVisualizer::new(config);
        
        let mut rule_impacts = HashMap::new();
        rule_impacts.insert("rule1".to_string(), RuleImpact {
            rule_id: "rule1".to_string(),
            rule_name: "Test Rule".to_string(),
            apply_count: 10,
            avg_execution_time_us: 1000,
            success_rate: 0.95,
            last_applied: 1640995200,
            impact_score: 0.8,
        });
        
        let rendered = visualizer.render_dependency_graph_text(&rule_impacts, false);
        assert!(rendered.contains("Rule Dependency Graph"));
        assert!(rendered.contains("Test Rule"));
        assert!(rendered.contains("Impact: 0.800"));
        
        let rendered_with_colors = visualizer.render_dependency_graph_text(&rule_impacts, true);
        assert!(rendered_with_colors.contains("\x1b["));
    }
    
    #[test]
    fn test_impact_indicator() {
        let config = VisualizationConfig::default();
        let visualizer = RuleVisualizer::new(config);
        
        assert_eq!(visualizer.get_impact_indicator(0.9), "🔥");
        assert_eq!(visualizer.get_impact_indicator(0.7), "⚡");
        assert_eq!(visualizer.get_impact_indicator(0.5), "💡");
        assert_eq!(visualizer.get_impact_indicator(0.3), "📝");
        assert_eq!(visualizer.get_impact_indicator(0.1), "💤");
    }
} 