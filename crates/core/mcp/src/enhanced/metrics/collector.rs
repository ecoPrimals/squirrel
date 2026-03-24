// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Unified Metrics Collector
//!
//! This module collects metrics from all enhanced MCP components including
//! workflows, service compositions, connections, transports, serialization,
//! streaming, and coordination systems.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, error, instrument};

use crate::error::{Result, types::MCPError};

// Import metrics from all enhanced MCP components
use crate::enhanced::connection_pool::metrics::ConnectionPoolMetrics;
use crate::enhanced::serialization::SerializationMetrics;

/// Unified metrics collector that gathers metrics from all components
#[derive(Debug)]
pub struct UnifiedMetricsCollector {
    /// Component collectors
    component_collectors: Arc<RwLock<HashMap<String, Box<dyn ComponentMetricsCollector>>>>,
    
    /// System metrics collector
    system_collector: Arc<SystemMetricsCollector>,
    
    /// Collection state
    state: Arc<RwLock<CollectorState>>,
    
    /// Configuration
    config: CollectorConfig,
}

/// Component metrics collector trait
#[async_trait::async_trait]
pub trait ComponentMetricsCollector: Send + Sync + std::fmt::Debug {
    /// Component name
    fn component_name(&self) -> &str;
    
    /// Collect metrics from this component
    async fn collect_metrics(&self) -> Result<ComponentMetrics>;
    
    /// Component health status
    async fn health_check(&self) -> Result<ComponentHealth>;
    
    /// Component capabilities
    fn capabilities(&self) -> Vec<MetricCapability>;
}

/// Configuration for the collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Enable system metrics collection
    pub enable_system_metrics: bool,
    
    /// System metrics collection interval
    pub system_metrics_interval_secs: u64,
    
    /// Component timeout for metrics collection
    pub component_timeout_secs: u64,
    
    /// Retry attempts for failed collections
    pub retry_attempts: u32,
    
    /// Enable detailed performance tracking
    pub enable_performance_tracking: bool,
}

/// Collector state
#[derive(Debug, Clone)]
pub struct CollectorState {
    /// Collector status
    pub status: CollectorStatus,
    
    /// Last collection time
    pub last_collection: Option<DateTime<Utc>>,
    
    /// Total collections performed
    pub total_collections: u64,
    
    /// Failed collections
    pub failed_collections: u64,
    
    /// Registered components count
    pub registered_components: u32,
    
    /// Collection performance metrics
    pub collection_performance: CollectionPerformance,
}

/// Collector status
#[derive(Debug, Clone, PartialEq)]
pub enum CollectorStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

/// Collection performance metrics
#[derive(Debug, Clone, Default)]
pub struct CollectionPerformance {
    /// Average collection time
    pub avg_collection_time: Duration,
    
    /// Last collection time
    pub last_collection_time: Duration,
    
    /// Minimum collection time
    pub min_collection_time: Duration,
    
    /// Maximum collection time
    pub max_collection_time: Duration,
    
    /// Total collection time
    pub total_collection_time: Duration,
}

/// Unified metrics from all components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMetrics {
    /// Collection timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Workflow management metrics
    pub workflow_metrics: Option<WorkflowMetrics>,
    
    /// Service composition metrics
    pub service_composition_metrics: Option<ServiceCompositionMetrics>,
    
    /// Connection pool metrics
    pub connection_pool_metrics: Option<ConnectionPoolMetrics>,
    
    /// Serialization metrics
    pub serialization_metrics: Option<SerializationMetrics>,
    
    /// Transport metrics
    pub transport_metrics: Option<TransportMetrics>,
    
    /// Streaming metrics
    pub streaming_metrics: Option<StreamingMetrics>,
    
    /// Coordinator metrics
    pub coordinator_metrics: Option<CoordinatorMetrics>,
    
    /// WebSocket transport metrics
    pub websocket_metrics: Option<WebSocketMetrics>,
    
    /// Event system metrics
    pub event_system_metrics: Option<EventSystemMetrics>,
    
    /// Multi-agent metrics
    pub multi_agent_metrics: Option<MultiAgentMetrics>,
    
    /// System metrics
    pub system_metrics: Option<SystemMetrics>,
    
    /// Component health statuses
    pub component_health: HashMap<String, ComponentHealth>,
    
    /// Collection metadata
    pub collection_metadata: CollectionMetadata,
}

/// Individual component metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    /// Component name
    pub component_name: String,
    
    /// Metrics data
    pub metrics: HashMap<String, MetricValue>,
    
    /// Collection timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Collection duration
    pub collection_duration: Duration,
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram { buckets: Vec<f64>, counts: Vec<u64> },
    Summary { percentiles: HashMap<String, f64> },
    Duration(Duration),
    String(String),
    Boolean(bool),
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub component_name: String,
    
    /// Health status
    pub status: HealthStatus,
    
    /// Health score (0.0 to 1.0)
    pub score: f64,
    
    /// Last health check
    pub last_check: DateTime<Utc>,
    
    /// Health details
    pub details: HashMap<String, String>,
    
    /// Error message if unhealthy
    pub error_message: Option<String>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
    Unavailable,
}

/// Metric collection capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricCapability {
    RealTime,
    Historical,
    Aggregation,
    Alerting,
    Export,
    Dashboard,
    Custom(String),
}

/// Collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    /// Collection ID
    pub collection_id: String,
    
    /// Collection duration
    pub collection_duration: Duration,
    
    /// Number of components collected
    pub components_collected: u32,
    
    /// Collection errors
    pub errors: Vec<CollectionError>,
    
    /// Performance metrics
    pub performance: CollectionPerformanceData,
}

/// Collection error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionError {
    /// Component name where error occurred
    pub component: String,
    
    /// Error message
    pub error_message: String,
    
    /// Error timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Error severity
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}

/// Collection performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionPerformanceData {
    /// Time per component
    pub component_times: HashMap<String, Duration>,
    
    /// System metrics collection time
    pub system_metrics_time: Duration,
    
    /// Total collection overhead
    pub collection_overhead: Duration,
}

// Specific metrics structures for each component

/// Workflow management metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub total_workflows: u64,
    pub active_workflows: u64,
    pub completed_workflows: u64,
    pub failed_workflows: u64,
    pub avg_execution_time: Duration,
    pub workflow_success_rate: f64,
    pub queued_workflows: u64,
    pub workflow_types: HashMap<String, u64>,
}

/// Service composition metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCompositionMetrics {
    pub total_compositions: u64,
    pub active_compositions: u64,
    pub completed_compositions: u64,
    pub failed_compositions: u64,
    pub avg_execution_time: Duration,
    pub composition_success_rate: f64,
    pub composition_types: HashMap<String, u64>,
}

/// Transport metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMetrics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_errors: u64,
    pub message_errors: u64,
    pub avg_message_latency: Duration,
}

/// Streaming metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingMetrics {
    pub total_streams: u64,
    pub active_streams: u64,
    pub total_messages: u64,
    pub total_bytes: u64,
    pub average_stream_lifetime: f64,
    pub system_throughput: f64,
    pub system_latency: f64,
    pub stream_types: HashMap<String, u64>,
}

/// Coordinator metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub active_sessions: u64,
    pub total_models: u64,
    pub avg_response_time: Duration,
    pub total_cost: f64,
    pub provider_usage: HashMap<String, u64>,
}

/// WebSocket transport metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMetrics {
    pub active_websocket_connections: u64,
    pub websocket_messages_sent: u64,
    pub websocket_messages_received: u64,
    pub websocket_connection_errors: u64,
    pub websocket_ping_latency: Duration,
}

/// Event system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSystemMetrics {
    pub total_events: u64,
    pub events_per_second: f64,
    pub active_subscriptions: u64,
    pub event_types: HashMap<String, u64>,
    pub event_processing_latency: Duration,
}

/// Multi-agent metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentMetrics {
    pub total_agents: u64,
    pub active_agents: u64,
    pub agent_collaborations: u64,
    pub messages_between_agents: u64,
    pub workflow_executions: u64,
    pub agent_types: HashMap<String, u64>,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f64,
    pub disk_usage_bytes: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub open_file_descriptors: u64,
    pub thread_count: u64,
    pub uptime_seconds: u64,
}

/// System metrics collector
#[derive(Debug)]
pub struct SystemMetricsCollector {
    /// Collection state
    state: Arc<Mutex<SystemCollectorState>>,
    
    /// Configuration
    config: SystemCollectorConfig,
}

#[derive(Debug, Clone)]
pub struct SystemCollectorState {
    pub last_collection: Option<DateTime<Utc>>,
    pub collection_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCollectorConfig {
    pub enabled: bool,
    pub collection_interval_secs: u64,
}

impl UnifiedMetricsCollector {
    /// Create a new unified metrics collector
    pub async fn new() -> Result<Self> {
        let config = CollectorConfig::default();
        let system_collector = Arc::new(SystemMetricsCollector::new(SystemCollectorConfig::default()));
        
        let state = Arc::new(RwLock::new(CollectorState {
            status: CollectorStatus::Stopped,
            last_collection: None,
            total_collections: 0,
            failed_collections: 0,
            registered_components: 0,
            collection_performance: CollectionPerformance::default(),
        }));
        
        Ok(Self {
            component_collectors: Arc::new(RwLock::new(HashMap::new())),
            system_collector,
            state,
            config,
        })
    }
    
    /// Start the metrics collector
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting Unified Metrics Collector");
        
        // Update state
        {
            let mut state = self.state.write().await;
            state.status = CollectorStatus::Starting;
        }
        
        // Start system metrics collection if enabled
        if self.config.enable_system_metrics {
            self.start_system_metrics_collection().await;
        }
        
        // Update state to running
        {
            let mut state = self.state.write().await;
            state.status = CollectorStatus::Running;
        }
        
        info!("Unified Metrics Collector started");
        Ok(())
    }
    
    /// Stop the metrics collector
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Unified Metrics Collector");
        
        let mut state = self.state.write().await;
        state.status = CollectorStatus::Stopped;
        
        info!("Unified Metrics Collector stopped");
        Ok(())
    }
    
    /// Register a component metrics collector
    pub async fn register_component_collector(&self, collector: Box<dyn ComponentMetricsCollector>) -> Result<()> {
        let component_name = collector.component_name().to_string();
        
        {
            let mut collectors = self.component_collectors.write().await;
            collectors.insert(component_name.clone(), collector);
        }
        
        {
            let mut state = self.state.write().await;
            state.registered_components += 1;
        }
        
        info!("Registered component collector: {}", component_name);
        Ok(())
    }
    
    /// Collect metrics from all components
    #[instrument(skip(self))]
    pub async fn collect_all_metrics(&self) -> Result<UnifiedMetrics> {
        let start_time = Instant::now();
        let collection_id = uuid::Uuid::new_v4().to_string();
        
        let mut errors = Vec::new();
        let mut component_times = HashMap::new();
        let mut component_health = HashMap::new();
        
        // Collect from all registered components
        let collectors = self.component_collectors.read().await;
        let mut component_metrics_map = HashMap::new();
        
        for (name, collector) in collectors.iter() {
            let component_start = Instant::now();
            
            match tokio::time::timeout(
                Duration::from_secs(self.config.component_timeout_secs),
                collector.collect_metrics()
            ).await {
                Ok(Ok(metrics)) => {
                    component_metrics_map.insert(name.clone(), metrics);
                    
                    // Collect health status
                    if let Ok(health) = collector.health_check().await {
                        component_health.insert(name.clone(), health);
                    }
                }
                Ok(Err(e)) => {
                    errors.push(CollectionError {
                        component: name.clone(),
                        error_message: e.to_string(),
                        timestamp: Utc::now(),
                        severity: ErrorSeverity::Error,
                    });
                }
                Err(_) => {
                    errors.push(CollectionError {
                        component: name.clone(),
                        error_message: "Collection timeout".to_string(),
                        timestamp: Utc::now(),
                        severity: ErrorSeverity::Warning,
                    });
                }
            }
            
            component_times.insert(name.clone(), component_start.elapsed());
        }
        
        // Collect system metrics
        let system_metrics_start = Instant::now();
        let system_metrics = if self.config.enable_system_metrics {
            match self.system_collector.collect_system_metrics().await {
                Ok(metrics) => Some(metrics),
                Err(e) => {
                    errors.push(CollectionError {
                        component: "system".to_string(),
                        error_message: e.to_string(),
                        timestamp: Utc::now(),
                        severity: ErrorSeverity::Warning,
                    });
                    None
                }
            }
        } else {
            None
        };
        let system_metrics_time = system_metrics_start.elapsed();
        
        // Build unified metrics
        let unified_metrics = UnifiedMetrics {
            timestamp: Utc::now(),
            workflow_metrics: self.extract_workflow_metrics(&component_metrics_map),
            service_composition_metrics: self.extract_service_composition_metrics(&component_metrics_map),
            connection_pool_metrics: self.extract_connection_pool_metrics(&component_metrics_map),
            serialization_metrics: self.extract_serialization_metrics(&component_metrics_map),
            transport_metrics: self.extract_transport_metrics(&component_metrics_map),
            streaming_metrics: self.extract_streaming_metrics(&component_metrics_map),
            coordinator_metrics: self.extract_coordinator_metrics(&component_metrics_map),
            websocket_metrics: self.extract_websocket_metrics(&component_metrics_map),
            event_system_metrics: self.extract_event_system_metrics(&component_metrics_map),
            multi_agent_metrics: self.extract_multi_agent_metrics(&component_metrics_map),
            system_metrics,
            component_health,
            collection_metadata: CollectionMetadata {
                collection_id,
                collection_duration: start_time.elapsed(),
                components_collected: collectors.len() as u32,
                errors,
                performance: CollectionPerformanceData {
                    component_times,
                    system_metrics_time,
                    collection_overhead: start_time.elapsed(),
                },
            },
        };
        
        // Update state
        {
            let mut state = self.state.write().await;
            state.last_collection = Some(Utc::now());
            state.total_collections += 1;
            
            if !unified_metrics.collection_metadata.errors.is_empty() {
                state.failed_collections += 1;
            }
            
            // Update performance metrics
            let collection_time = start_time.elapsed();
            state.collection_performance.last_collection_time = collection_time;
            state.collection_performance.total_collection_time += collection_time;
            
            if state.total_collections == 1 {
                state.collection_performance.avg_collection_time = collection_time;
                state.collection_performance.min_collection_time = collection_time;
                state.collection_performance.max_collection_time = collection_time;
            } else {
                // Update average
                state.collection_performance.avg_collection_time = 
                    state.collection_performance.total_collection_time / state.total_collections as u32;
                
                // Update min/max
                if collection_time < state.collection_performance.min_collection_time {
                    state.collection_performance.min_collection_time = collection_time;
                }
                if collection_time > state.collection_performance.max_collection_time {
                    state.collection_performance.max_collection_time = collection_time;
                }
            }
        }
        
        debug!(
            "Collected metrics from {} components in {:?}",
            collectors.len(),
            start_time.elapsed()
        );
        
        Ok(unified_metrics)
    }
    
    /// Get collector state
    pub async fn get_state(&self) -> CollectorState {
        self.state.read().await.clone()
    }
    
    // Private helper methods
    
    /// Start system metrics collection
    async fn start_system_metrics_collection(&self) {
        let system_collector = self.system_collector.clone();
        let interval_secs = self.config.system_metrics_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = system_collector.update_metrics().await {
                    error!("Error updating system metrics: {}", e);
                }
            }
        });
    }
    
    // Metric extraction methods
    
    fn extract_workflow_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<WorkflowMetrics> {
        // Extract workflow metrics from component metrics
        // Implementation would parse metrics from workflow management component
        None // Placeholder
    }
    
    fn extract_service_composition_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<ServiceCompositionMetrics> {
        // Extract service composition metrics
        None // Placeholder
    }
    
    fn extract_connection_pool_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<ConnectionPoolMetrics> {
        // Extract connection pool metrics
        if let Some(component) = component_metrics.get("connection_pool") {
            // Parse connection pool metrics
            // This would convert from ComponentMetrics to ConnectionPoolMetrics
        }
        None // Placeholder
    }
    
    fn extract_serialization_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<SerializationMetrics> {
        // Extract serialization metrics
        None // Placeholder
    }
    
    fn extract_transport_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<TransportMetrics> {
        // Extract transport metrics
        None // Placeholder
    }
    
    fn extract_streaming_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<StreamingMetrics> {
        // Extract streaming metrics
        None // Placeholder
    }
    
    fn extract_coordinator_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<CoordinatorMetrics> {
        // Extract coordinator metrics
        None // Placeholder
    }
    
    fn extract_websocket_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<WebSocketMetrics> {
        // Extract WebSocket metrics
        None // Placeholder
    }
    
    fn extract_event_system_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<EventSystemMetrics> {
        // Extract event system metrics
        None // Placeholder
    }
    
    fn extract_multi_agent_metrics(&self, component_metrics: &HashMap<String, ComponentMetrics>) -> Option<MultiAgentMetrics> {
        // Extract multi-agent metrics
        None // Placeholder
    }
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new(config: SystemCollectorConfig) -> Self {
        let state = Arc::new(Mutex::new(SystemCollectorState {
            last_collection: None,
            collection_count: 0,
            error_count: 0,
        }));
        
        Self { state, config }
    }
    
    /// Collect current system metrics
    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        // In a real implementation, this would use system APIs
        // For now, we'll return mock data
        Ok(SystemMetrics {
            cpu_usage_percent: 25.0,
            memory_usage_bytes: 512 * 1024 * 1024, // 512 MB
            memory_usage_percent: 30.0,
            disk_usage_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            network_bytes_sent: 1024 * 1024,
            network_bytes_received: 2 * 1024 * 1024,
            open_file_descriptors: 256,
            thread_count: 32,
            uptime_seconds: 3600,
        })
    }
    
    /// Update metrics
    pub async fn update_metrics(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        state.last_collection = Some(Utc::now());
        state.collection_count += 1;
        Ok(())
    }
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            enable_system_metrics: true,
            system_metrics_interval_secs: 30,
            component_timeout_secs: 10,
            retry_attempts: 3,
            enable_performance_tracking: true,
        }
    }
}

impl Default for SystemCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_secs: 30,
        }
    }
} 