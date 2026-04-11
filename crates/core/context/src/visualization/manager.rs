// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Visualization Manager
//!
//! This module provides the core visualization management functionality for the Context Management System.
//! It coordinates between different renderers and manages visualization lifecycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use super::VisualizationSystemConfig;
use super::renderers::{HtmlRenderer, JsonRenderer, MarkdownRenderer, TerminalRenderer};
use super::types::{
    VisualizationConfig, VisualizationRequest, VisualizationResponse, VisualizationType,
};
use crate::error::Result;

/// Visualization manager for coordinating visualization operations
#[derive(Debug)]
pub struct VisualizationManager {
    /// Configuration
    config: Arc<VisualizationSystemConfig>,

    /// JSON renderer
    json_renderer: Arc<JsonRenderer>,

    /// Terminal renderer
    terminal_renderer: Arc<TerminalRenderer>,

    /// HTML renderer
    html_renderer: Arc<HtmlRenderer>,

    /// Markdown renderer
    markdown_renderer: Arc<MarkdownRenderer>,

    /// Active visualizations
    active_visualizations: Arc<RwLock<HashMap<String, ActiveVisualization>>>,

    /// Visualization cache
    visualization_cache: Arc<RwLock<HashMap<String, CachedVisualization>>>,

    /// Visualization history
    visualization_history: Arc<RwLock<Vec<VisualizationHistoryEntry>>>,

    /// Manager statistics
    stats: Arc<Mutex<VisualizationManagerStats>>,
}

/// Active visualization
#[derive(Debug, Clone)]
pub struct ActiveVisualization {
    /// Visualization ID
    pub id: String,

    /// Visualization type
    pub visualization_type: VisualizationType,

    /// Configuration
    pub config: VisualizationConfig,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Current data
    pub data: Value,

    /// Metadata
    pub metadata: HashMap<String, Value>,
}

/// Cached visualization
#[derive(Debug, Clone)]
#[expect(dead_code, reason = "planned feature not yet wired")]
pub struct CachedVisualization {
    /// Visualization ID
    pub id: String,

    /// Rendered content
    pub content: String,

    /// Format
    pub format: String,

    /// Cache timestamp
    pub cached_at: DateTime<Utc>,

    /// Cache key
    pub cache_key: String,
}

/// Visualization history entry
#[derive(Debug, Clone)]
pub struct VisualizationHistoryEntry {
    /// Visualization ID
    pub id: String,

    /// Action type
    pub action: VisualizationAction,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Additional data
    pub data: Value,
}

/// Visualization action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationAction {
    Created,
    Updated,
    Deleted,
    Rendered,
    Cached,
}

/// Visualization manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationManagerStats {
    /// Total visualizations created
    pub total_created: u64,

    /// Total visualizations updated
    pub total_updated: u64,

    /// Total visualizations deleted
    pub total_deleted: u64,

    /// Total renderings
    pub total_renderings: u64,

    /// Cache hits
    pub cache_hits: u64,

    /// Cache misses
    pub cache_misses: u64,

    /// Average render time (ms)
    pub average_render_time_ms: f64,

    /// Last updated
    pub last_updated: DateTime<Utc>,
}

impl VisualizationManager {
    /// Create a new visualization manager
    pub async fn new(config: Arc<VisualizationSystemConfig>) -> Result<Self> {
        let json_renderer = Arc::new(JsonRenderer::new());
        let terminal_renderer = Arc::new(TerminalRenderer::new());
        let html_renderer = Arc::new(HtmlRenderer::new());
        let markdown_renderer = Arc::new(MarkdownRenderer::new());

        Ok(Self {
            config,
            json_renderer,
            terminal_renderer,
            html_renderer,
            markdown_renderer,
            active_visualizations: Arc::new(RwLock::new(HashMap::new())),
            visualization_cache: Arc::new(RwLock::new(HashMap::new())),
            visualization_history: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(Mutex::new(VisualizationManagerStats::new())),
        })
    }

    /// Start the visualization manager
    pub async fn start(&self) -> Result<()> {
        // Start cleanup task
        self.start_cleanup_task().await;

        Ok(())
    }

    /// Stop the visualization manager
    pub async fn stop(&self) -> Result<()> {
        // Clear all active visualizations
        self.active_visualizations.write().await.clear();

        // Clear cache
        self.visualization_cache.write().await.clear();

        Ok(())
    }

    /// Create a new visualization
    pub async fn create_visualization(
        &self,
        request: VisualizationRequest,
    ) -> Result<VisualizationResponse> {
        let visualization_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let history_data = request.data.clone();
        let render_format = request.config.format.clone();

        let active_viz = ActiveVisualization {
            id: visualization_id.clone(),
            visualization_type: request.visualization_type.clone(),
            config: request.config,
            created_at: now,
            updated_at: now,
            data: request.data,
            metadata: request.metadata.clone(),
        };

        {
            let mut active_vizs = self.active_visualizations.write().await;
            active_vizs.insert(visualization_id.clone(), active_viz);
        }

        self.add_to_history(
            visualization_id.clone(),
            VisualizationAction::Created,
            history_data,
        )
        .await;

        let content = self
            .render_visualization(&visualization_id, &render_format)
            .await?;

        self.update_stats_created().await;

        Ok(VisualizationResponse {
            visualization_id,
            visualization_type: request.visualization_type,
            format: render_format,
            content,
            metadata: request.metadata,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update a visualization
    pub async fn update_visualization(
        &self,
        visualization_id: &str,
        data: Value,
    ) -> Result<VisualizationResponse> {
        let now = Utc::now();

        // Update active visualization
        let (visualization_type, config) = {
            let mut active_vizs = self.active_visualizations.write().await;
            let active_viz = active_vizs.get_mut(visualization_id).ok_or_else(|| {
                crate::error::ContextError::NotFound(format!(
                    "Visualization not found: {visualization_id}"
                ))
            })?;

            active_viz.data = data.clone();
            active_viz.updated_at = now;

            (
                active_viz.visualization_type.clone(),
                active_viz.config.clone(),
            )
        };

        // Clear cache for this visualization
        self.clear_cache(visualization_id).await;

        // Add to history
        self.add_to_history(
            visualization_id.to_string(),
            VisualizationAction::Updated,
            data.clone(),
        )
        .await;

        // Re-render visualization
        let content = self
            .render_visualization(visualization_id, &config.format)
            .await?;

        // Update statistics
        self.update_stats_updated().await;

        Ok(VisualizationResponse {
            visualization_id: visualization_id.to_string(),
            visualization_type,
            format: config.format,
            content,
            metadata: HashMap::new(),
            created_at: now, // We don't track original creation time here
            updated_at: now,
        })
    }

    /// Delete a visualization
    pub async fn delete_visualization(&self, visualization_id: &str) -> Result<()> {
        // Remove from active visualizations
        {
            let mut active_vizs = self.active_visualizations.write().await;
            active_vizs.remove(visualization_id);
        }

        // Clear cache
        self.clear_cache(visualization_id).await;

        // Add to history
        self.add_to_history(
            visualization_id.to_string(),
            VisualizationAction::Deleted,
            serde_json::json!({}),
        )
        .await;

        // Update statistics
        self.update_stats_deleted().await;

        Ok(())
    }

    /// Render a visualization
    pub async fn render_visualization(
        &self,
        visualization_id: &str,
        format: &str,
    ) -> Result<String> {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = format!("{visualization_id}:{format}");
        if let Some(cached) = self.get_cached_visualization(&cache_key).await {
            self.update_stats_cache_hit().await;
            return Ok(cached.content);
        }

        // Get visualization data
        let data = {
            let active_vizs = self.active_visualizations.read().await;
            let active_viz = active_vizs.get(visualization_id).ok_or_else(|| {
                crate::error::ContextError::NotFound(format!(
                    "Visualization not found: {visualization_id}"
                ))
            })?;
            active_viz.data.clone()
        };

        // Render based on format
        let content = match format {
            "json" if self.config.enable_json => self.json_renderer.render(&data).await?,
            "terminal" if self.config.enable_terminal => {
                self.terminal_renderer.render(&data).await?
            }
            "html" if self.config.enable_html => self.html_renderer.render(&data).await?,
            "markdown" if self.config.enable_markdown => {
                self.markdown_renderer.render(&data).await?
            }
            _ => {
                return Err(crate::error::ContextError::InvalidFormat(format!(
                    "Unsupported format: {format}"
                )));
            }
        };

        // Cache the result
        self.cache_visualization(cache_key, content.clone()).await;

        // Update statistics
        let elapsed = start_time.elapsed();
        self.update_stats_rendered(elapsed.as_millis() as f64).await;
        self.update_stats_cache_miss().await;

        Ok(content)
    }

    /// Get active visualizations
    pub async fn get_active_visualizations(&self) -> Vec<ActiveVisualization> {
        let active_vizs = self.active_visualizations.read().await;
        active_vizs.values().cloned().collect()
    }

    /// Get visualization history
    pub async fn get_history(&self) -> Vec<VisualizationHistoryEntry> {
        let history = self.visualization_history.read().await;
        history.clone()
    }

    /// Get manager statistics
    pub async fn get_stats(&self) -> VisualizationManagerStats {
        self.stats.lock().await.clone()
    }

    // Private helper methods
    async fn get_cached_visualization(&self, cache_key: &str) -> Option<CachedVisualization> {
        let cache = self.visualization_cache.read().await;
        cache.get(cache_key).cloned()
    }

    async fn cache_visualization(&self, cache_key: String, content: String) {
        let format = cache_key.split(':').nth(1).unwrap_or("unknown").to_string();
        let insert_key = cache_key.clone();
        let cached_viz = CachedVisualization {
            id: cache_key.clone(),
            content,
            format,
            cached_at: Utc::now(),
            cache_key,
        };

        let mut cache = self.visualization_cache.write().await;
        cache.insert(insert_key, cached_viz);
    }

    async fn clear_cache(&self, visualization_id: &str) {
        let mut cache = self.visualization_cache.write().await;
        let keys_to_remove: Vec<String> = cache
            .keys()
            .filter(|k| k.starts_with(&format!("{visualization_id}:")))
            .cloned()
            .collect();

        for key in keys_to_remove {
            cache.remove(&key);
        }
    }

    async fn add_to_history(
        &self,
        visualization_id: String,
        action: VisualizationAction,
        data: Value,
    ) {
        let entry = VisualizationHistoryEntry {
            id: visualization_id,
            action,
            timestamp: Utc::now(),
            data,
        };

        let mut history = self.visualization_history.write().await;
        history.push(entry);

        // Keep only the last N entries
        if history.len() > self.config.max_history {
            history.remove(0);
        }
    }

    async fn start_cleanup_task(&self) {
        let cache = self.visualization_cache.clone();
        let history = self.visualization_history.clone();
        let max_history = self.config.max_history;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                // Clean up old cache entries (older than 1 hour)
                let cutoff_time = Utc::now() - chrono::Duration::hours(1);
                {
                    let mut cache_guard = cache.write().await;
                    cache_guard.retain(|_, cached| cached.cached_at > cutoff_time);
                }

                // Clean up old history entries
                {
                    let mut history_guard = history.write().await;
                    if history_guard.len() > max_history {
                        let excess = history_guard.len() - max_history;
                        history_guard.drain(0..excess);
                    }
                }
            }
        });
    }

    async fn update_stats_created(&self) {
        let mut stats = self.stats.lock().await;
        stats.total_created += 1;
        stats.last_updated = Utc::now();
    }

    async fn update_stats_updated(&self) {
        let mut stats = self.stats.lock().await;
        stats.total_updated += 1;
        stats.last_updated = Utc::now();
    }

    async fn update_stats_deleted(&self) {
        let mut stats = self.stats.lock().await;
        stats.total_deleted += 1;
        stats.last_updated = Utc::now();
    }

    async fn update_stats_rendered(&self, render_time_ms: f64) {
        let mut stats = self.stats.lock().await;
        stats.total_renderings += 1;

        // Update running average
        let total_time =
            stats.average_render_time_ms * (stats.total_renderings - 1) as f64 + render_time_ms;
        stats.average_render_time_ms = total_time / stats.total_renderings as f64;

        stats.last_updated = Utc::now();
    }

    async fn update_stats_cache_hit(&self) {
        let mut stats = self.stats.lock().await;
        stats.cache_hits += 1;
        stats.last_updated = Utc::now();
    }

    async fn update_stats_cache_miss(&self) {
        let mut stats = self.stats.lock().await;
        stats.cache_misses += 1;
        stats.last_updated = Utc::now();
    }
}

impl Default for VisualizationManagerStats {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualizationManagerStats {
    pub fn new() -> Self {
        Self {
            total_created: 0,
            total_updated: 0,
            total_deleted: 0,
            total_renderings: 0,
            cache_hits: 0,
            cache_misses: 0,
            average_render_time_ms: 0.0,
            last_updated: Utc::now(),
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}
