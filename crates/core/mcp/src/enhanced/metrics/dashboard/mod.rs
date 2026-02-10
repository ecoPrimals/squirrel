// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Metrics Dashboard
//!
//! This module provides a web-based dashboard for visualizing metrics,
//! trends, and system health status with real-time updates via WebSocket.

pub mod data;
pub mod api;
pub mod websocket;
pub mod ui;
pub mod summary;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, error, instrument};

use crate::error::{Result, types::MCPError};
use super::aggregator::AggregatedMetrics;
use super::{MetricsConfig, AlertSeverity};

pub use data::*;
pub use api::*;
pub use websocket::*;
pub use ui::*;
pub use summary::*;

/// Main metrics dashboard
#[derive(Debug)]
pub struct MetricsDashboard {
    /// Dashboard configuration
    config: DashboardConfig,
    
    /// Dashboard state
    state: Arc<RwLock<DashboardState>>,
    
    /// Data cache
    cache: Arc<Mutex<DashboardDataCache>>,
    
    /// WebSocket connections
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    
    /// Update channel sender
    update_sender: Option<tokio::sync::broadcast::Sender<DashboardUpdate>>,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Server address
    pub address: String,
    
    /// Server port
    pub port: u16,
    
    /// Update interval
    pub update_interval: Duration,
    
    /// Cache size
    pub cache_size: usize,
    
    /// Theme configuration
    pub theme: DashboardTheme,
    
    /// Authentication enabled
    pub auth_enabled: bool,
    
    /// Max connections
    pub max_connections: usize,
    
    /// CORS origins
    pub cors_origins: Vec<String>,
}

/// Dashboard update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardUpdate {
    /// Update type
    pub update_type: String,
    
    /// Update payload
    pub payload: serde_json::Value,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl MetricsDashboard {
    /// Create a new metrics dashboard
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(DashboardState::default())),
            cache: Arc::new(Mutex::new(DashboardDataCache::new(1000))),
            connections: Arc::new(RwLock::new(HashMap::new())),
            update_sender: None,
        }
    }
    
    /// Start the dashboard server
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting metrics dashboard on {}:{}", self.config.address, self.config.port);
        
        // Initialize update channel
        let (tx, _rx) = tokio::sync::broadcast::channel(100);
        self.update_sender = Some(tx);
        
        // Start background tasks
        self.start_background_tasks().await?;
        
        // Start web server
        self.start_web_server().await?;
        
        Ok(())
    }
    
    /// Start background tasks
    async fn start_background_tasks(&self) -> Result<()> {
        // Start data update task
        let state = Arc::clone(&self.state);
        let cache = Arc::clone(&self.cache);
        let update_interval = self.config.update_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(update_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::update_dashboard_data(Arc::clone(&state), Arc::clone(&cache)).await {
                    error!("Failed to update dashboard data: {:?}", e);
                }
            }
        });
        
        // Start connection cleanup task
        let connections = Arc::clone(&self.connections);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                Self::cleanup_connections(Arc::clone(&connections)).await;
            }
        });
        
        Ok(())
    }
    
    /// Start the web server
    async fn start_web_server(&self) -> Result<()> {
        // Implementation placeholder for web server
        info!("Web server would start here on {}:{}", self.config.address, self.config.port);
        Ok(())
    }
    
    /// Update dashboard data
    async fn update_dashboard_data(
        state: Arc<RwLock<DashboardState>>, 
        cache: Arc<Mutex<DashboardDataCache>>
    ) -> Result<()> {
        // Implementation placeholder for data updates
        debug!("Updating dashboard data");
        Ok(())
    }
    
    /// Cleanup inactive connections
    async fn cleanup_connections(connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>) {
        let mut conns = connections.write().await;
        conns.retain(|_, conn| conn.is_active());
        debug!("Cleaned up inactive connections, {} remaining", conns.len());
    }
    
    /// Process aggregated metrics
    pub async fn process_metrics(&self, metrics: &AggregatedMetrics) -> Result<()> {
        // Update cache
        {
            let mut cache = self.cache.lock().await;
            cache.update_metrics(metrics)?;
        }
        
        // Update state
        {
            let mut state = self.state.write().await;
            state.last_update = Some(Utc::now());
            state.status = DashboardStatus::Active;
        }
        
        // Broadcast update
        if let Some(sender) = &self.update_sender {
            let update = DashboardUpdate {
                update_type: "metrics".to_string(),
                payload: serde_json::to_value(metrics)
                    .map_err(|e| MCPError::SerializationError(e.to_string()))?,
                timestamp: Utc::now(),
            };
            
            if let Err(e) = sender.send(update) {
                debug!("No active subscribers for dashboard updates: {:?}", e);
            }
        }
        
        Ok(())
    }
    
    /// Get dashboard overview
    pub async fn get_overview(&self) -> Result<DashboardOverview> {
        let cache = self.cache.lock().await;
        cache.get_overview()
    }
    
    /// Get system status
    pub async fn get_system_status(&self) -> Result<SystemOverview> {
        let cache = self.cache.lock().await;
        cache.get_system_status()
    }
    
    /// Get performance summary
    pub async fn get_performance_summary(&self) -> Result<PerformanceSummary> {
        let cache = self.cache.lock().await;
        cache.get_performance_summary()
    }
    
    /// Stop the dashboard
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping metrics dashboard");
        
        // Close all connections
        {
            let mut connections = self.connections.write().await;
            for (_, conn) in connections.drain() {
                conn.close().await;
            }
        }
        
        // Update state
        {
            let mut state = self.state.write().await;
            state.status = DashboardStatus::Stopped;
        }
        
        Ok(())
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 8080,
            update_interval: Duration::from_secs(5),
            cache_size: 1000,
            theme: DashboardTheme::default(),
            auth_enabled: false,
            max_connections: 100,
            cors_origins: vec!["*".to_string()],
        }
    }
} 