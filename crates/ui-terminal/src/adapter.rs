use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

/// Protocol message for debugging
#[derive(Debug, Clone)]
pub struct ProtocolMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub direction: MessageDirection,
    pub size_bytes: usize,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub has_error: bool,
}

/// Message type for filtering
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Heartbeat,
    Error,
    Other(String),
}

/// Message direction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageDirection {
    Incoming,
    Outgoing,
}

/// Protocol error details
#[derive(Debug, Clone)]
pub struct ProtocolError {
    pub timestamp: DateTime<Utc>,
    pub error_type: String,
    pub message: String,
    pub source: String,
    pub related_message_id: Option<String>,
    pub stack_trace: Option<String>,
    pub is_recoverable: bool,
}

/// Connection status for MCP client
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    /// Connected to MCP service
    Connected,
    /// Disconnected from MCP service
    Disconnected,
    /// Connecting to MCP service
    Connecting,
    /// Error connecting to MCP service
    Error(String),
}

/// Connection health status
#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    pub status: ConnectionStatus,
    pub last_successful: Option<DateTime<Utc>>,
    pub failure_count: u32,
    pub latency_ms: Option<u64>,
    pub error_details: Option<String>,
}

/// Connection event
#[derive(Debug, Clone)]
pub struct ConnectionEvent {
    pub event_type: ConnectionEventType,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

/// Connection event type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionEventType {
    Connected,
    Disconnected,
    Reconnecting,
    ReconnectSuccess,
    ReconnectFailure,
    Error,
}

/// Performance options
#[derive(Debug, Clone)]
pub struct PerformanceOptions {
    pub metrics_cache_ttl_ms: u64,
    pub use_compressed_timeseries: bool,
    pub history_max_points: usize,
    pub adaptive_polling: bool,
    pub polling_min_interval_ms: u64,
    pub polling_max_interval_ms: u64,
}

impl Default for PerformanceOptions {
    fn default() -> Self {
        Self {
            metrics_cache_ttl_ms: 5000,
            use_compressed_timeseries: true,
            history_max_points: 1000,
            adaptive_polling: true,
            polling_min_interval_ms: 1000,
            polling_max_interval_ms: 10000,
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub metrics_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_request_time_ms: u64,
    pub average_request_time_ms: f64,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            metrics_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            total_request_time_ms: 0,
            average_request_time_ms: 0.0,
            cpu_usage: Some(0.0),
            memory_usage: Some(0.0),
        }
    }
}

/// Message statistics
#[derive(Debug, Clone)]
pub struct MessageStats {
    pub total_requests: u64,
    pub total_responses: u64,
    pub request_rate: f64,
    pub response_rate: f64,
    pub request_types: HashMap<String, u64>,
}

/// Transaction statistics
#[derive(Debug, Clone)]
pub struct TransactionStats {
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub transaction_rate: f64,
    pub success_rate: f64,
}

/// Error statistics
#[derive(Debug, Clone)]
pub struct ErrorStats {
    pub total_errors: u64,
    pub connection_errors: u64,
    pub protocol_errors: u64,
    pub timeout_errors: u64,
    pub error_rate: f64,
    pub error_types: HashMap<String, u64>,
}

/// Latency statistics
#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub average_latency_ms: f64,
    pub median_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub latency_histogram: Vec<f64>,
}

/// MCP metrics data structure
#[derive(Debug, Clone)]
pub struct McpMetrics {
    /// Message statistics
    pub message_stats: MessageStats,
    /// Transaction statistics
    pub transaction_stats: TransactionStats,
    /// Error statistics
    pub error_stats: ErrorStats,
    /// Latency measurements
    pub latency_stats: LatencyStats,
    /// Timestamp when metrics were collected
    pub timestamp: DateTime<Utc>,
}

impl Default for McpMetrics {
    fn default() -> Self {
        Self {
            message_stats: MessageStats {
                total_requests: 0,
                total_responses: 0,
                request_rate: 0.0,
                response_rate: 0.0,
                request_types: HashMap::new(),
            },
            transaction_stats: TransactionStats {
                total_transactions: 0,
                successful_transactions: 0,
                failed_transactions: 0,
                transaction_rate: 0.0,
                success_rate: 0.0,
            },
            error_stats: ErrorStats {
                total_errors: 0,
                connection_errors: 0,
                protocol_errors: 0,
                timeout_errors: 0,
                error_rate: 0.0,
                error_types: HashMap::new(),
            },
            latency_stats: LatencyStats {
                average_latency_ms: 0.0,
                median_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                min_latency_ms: 0.0,
                max_latency_ms: 0.0,
                latency_histogram: Vec::new(),
            },
            timestamp: Utc::now(),
        }
    }
}

/// MCP metrics provider interface
#[async_trait]
pub trait McpMetricsProvider: Send + Sync {
    /// Get metrics from MCP
    async fn get_metrics(&self) -> Result<McpMetrics, String>;
    
    /// Get connection status
    async fn get_connection_status(&self) -> Result<ConnectionStatus, String>;
    
    /// Get connection health
    async fn get_connection_health(&self) -> Result<ConnectionHealth, String>;
    
    /// Get protocol metrics
    async fn get_protocol_metrics(&self) -> Result<HashMap<String, f64>, String>;
    
    /// Attempt to reconnect
    async fn reconnect(&self) -> Result<bool, String>;
    
    /// Get connection history
    async fn get_connection_history(&self) -> Result<Vec<ConnectionEvent>, String>;
    
    /// Get message log
    async fn get_message_log(&self) -> Result<Vec<String>, String>;
    
    /// Get recent errors
    async fn get_recent_errors(&self, limit: usize) -> Result<Vec<String>, String>;
    
    /// Get error log
    async fn get_error_log(&self) -> Result<Vec<String>, String>;
    
    /// Set performance options
    async fn set_performance_options(&self, options: PerformanceOptions) -> Result<(), String>;
    
    /// Get performance metrics
    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, String>;
    
    /// Set whether the provider should fail (for testing)
    async fn set_should_fail(&self, should_fail: bool);
}

/// Implementation helpers for ConnectionStatus
impl ToString for ConnectionStatus {
    fn to_string(&self) -> String {
        match self {
            ConnectionStatus::Connected => "Connected".to_string(),
            ConnectionStatus::Disconnected => "Disconnected".to_string(),
            ConnectionStatus::Connecting => "Connecting".to_string(),
            ConnectionStatus::Error(msg) => format!("Error: {}", msg),
        }
    }
}
