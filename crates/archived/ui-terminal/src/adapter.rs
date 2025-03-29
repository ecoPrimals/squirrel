use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use dashboard_core::data::DashboardData;
use dashboard_core::DashboardService;
use serde::{Serialize, Deserialize};
use std::fmt;

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Connected to MCP service
    Connected,
    /// Disconnected from MCP service
    Disconnected,
    /// Connecting to MCP service
    Connecting,
    /// Connection is degraded (connected but with issues)
    Degraded,
    /// Unknown status
    Unknown,
    /// Error connecting to MCP service
    Error(String),
}

/// Connection health status
#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    /// Latency in milliseconds
    pub latency_ms: f64,
    /// Packet loss percentage (0-100)
    pub packet_loss: f64,
    /// Connection stability percentage (0-100)
    pub stability: f64,
    /// Signal strength percentage (0-100)
    pub signal_strength: f64,
    /// Connection health score (0.0-1.0)
    pub health_score: f64,
    /// Current connection status
    pub status: ConnectionStatus,
    /// When the connection was established
    pub connected_since: Option<DateTime<Utc>>,
    /// When the last status change occurred
    pub last_status_change: Option<DateTime<Utc>>,
    /// Last checked timestamp
    pub last_checked: DateTime<Utc>,
}

impl Default for ConnectionHealth {
    fn default() -> Self {
        Self {
            latency_ms: 0.0,
            packet_loss: 0.0,
            stability: 100.0,
            signal_strength: 100.0,
            health_score: 1.0,
            status: ConnectionStatus::Disconnected,
            connected_since: None,
            last_status_change: Some(Utc::now()),
            last_checked: Utc::now(),
        }
    }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub metrics_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_request_time_ms: u64,
    pub average_request_time_ms: f64,
    /// CPU usage percentage (0-100)
    pub cpu_usage: Option<f64>,
    /// Memory usage in MB
    pub memory_usage: Option<f64>,
    /// Disk usage percentage (0-100)
    pub disk_usage: Option<f64>,
    /// Network receive rate in bytes/sec
    pub network_rx_rate: Option<f64>,
    /// Network transmit rate in bytes/sec
    pub network_tx_rate: Option<f64>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            metrics_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            total_request_time_ms: 0,
            average_request_time_ms: 0.0,
            cpu_usage: None,
            memory_usage: None,
            disk_usage: None,
            network_rx_rate: None,
            network_tx_rate: None,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
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

/// McpMetricsProvider is an adapter that connects the monitoring system metrics 
/// to the dashboard, bridging the two systems.
#[derive(Debug)]
pub struct McpMetricsProvider {
    update_interval_secs: u64,
}

impl McpMetricsProvider {
    /// Create a new McpMetricsProvider with default settings
    pub fn new() -> Self {
        Self {
            update_interval_secs: 5,
        }
    }
    
    /// Create a new McpMetricsProvider with specified update interval
    pub fn with_update_interval(update_interval_secs: u64) -> Self {
        Self {
            update_interval_secs,
        }
    }
    
    /// Get the current connection status
    pub async fn get_connection_status(&self) -> Result<ConnectionStatus, std::io::Error> {
        // In a real implementation, this would check the actual connection status
        // For now, we'll just return Connected
        Ok(ConnectionStatus::Connected)
    }
    
    /// Get the current dashboard data
    pub async fn get_dashboard_data(&self) -> Result<DashboardData, std::io::Error> {
        // Create a DashboardData object with mock data
        let mut data = DashboardData::default();
        
        // Set CPU metrics
        data.metrics.cpu.usage = 45.0; // Example value
        data.metrics.cpu.cores = vec![40.0, 30.0, 50.0, 60.0]; // Example values
        
        // Set memory metrics
        data.metrics.memory.total = 16_000_000_000; // 16 GB
        data.metrics.memory.used = 8_000_000_000;   // 8 GB
        data.metrics.memory.available = 8_000_000_000;
        data.metrics.memory.free = 8_000_000_000;
        
        // Set network metrics
        data.metrics.network.total_rx_bytes = 1_500_000; // 1.5 MB
        data.metrics.network.total_tx_bytes = 500_000;   // 0.5 MB
        
        // Update timestamp
        data.timestamp = Utc::now();
        
        // Set protocol data
        data.protocol.name = "System Monitor".to_string();
        data.protocol.protocol_type = "Local".to_string();
        data.protocol.version = "1.0".to_string();
        data.protocol.connected = true;
        data.protocol.last_connected = Some(Utc::now());
        data.protocol.status = "Connected".to_string();
        
        Ok(data)
    }
    
    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, std::io::Error> {
        Ok(PerformanceMetrics {
            metrics_requests: 100,
            cache_hits: 90,
            cache_misses: 10,
            total_request_time_ms: 500,
            average_request_time_ms: 5.0,
            cpu_usage: Some(45.0),
            memory_usage: Some(8000.0), // 8000 MB
            disk_usage: Some(75.0),     // 75%
            network_rx_rate: Some(1_500_000.0),
            network_tx_rate: Some(500_000.0),
        })
    }
    
    /// Start the metrics collection service
    pub async fn start(&self, dashboard_service: Arc<dyn DashboardService>) -> Result<(), std::io::Error> {
        let provider = self.clone();
        let update_interval_secs = self.update_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(update_interval_secs));
            
            loop {
                interval.tick().await;
                
                // Get the latest data
                if let Ok(data) = provider.get_dashboard_data().await {
                    // Update the dashboard service
                    if let Err(e) = dashboard_service.update_dashboard_data(data).await {
                        eprintln!("Error updating dashboard data: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
}

impl Clone for McpMetricsProvider {
    fn clone(&self) -> Self {
        Self {
            update_interval_secs: self.update_interval_secs,
        }
    }
}

impl Default for McpMetricsProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// MonitoringToDashboardAdapter trait for connecting monitoring to dashboard
#[async_trait]
pub trait MonitoringToDashboardAdapter: Send + Sync + std::fmt::Debug {
    /// Get connection status
    async fn get_connection_status(&self) -> Result<ConnectionStatus, std::io::Error>;
    
    /// Get dashboard data
    async fn get_dashboard_data(&self) -> Result<DashboardData, std::io::Error>;
    
    /// Get performance metrics
    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, std::io::Error>;
    
    /// Start adapter
    async fn start(&self, dashboard_service: Arc<dyn DashboardService>) -> Result<(), std::io::Error>;
}

#[async_trait]
impl MonitoringToDashboardAdapter for McpMetricsProvider {
    async fn get_connection_status(&self) -> Result<ConnectionStatus, std::io::Error> {
        self.get_connection_status().await
    }
    
    async fn get_dashboard_data(&self) -> Result<DashboardData, std::io::Error> {
        self.get_dashboard_data().await
    }
    
    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, std::io::Error> {
        self.get_performance_metrics().await
    }
    
    async fn start(&self, dashboard_service: Arc<dyn DashboardService>) -> Result<(), std::io::Error> {
        self.start(dashboard_service).await
    }
}

/// Dashboard monitor implementation
#[derive(Debug)]
pub struct DashboardMonitor {
    adapter: Arc<dyn MonitoringToDashboardAdapter>,
    dashboard_service: Arc<dyn DashboardService>,
}

impl DashboardMonitor {
    /// Create a new dashboard monitor
    pub fn new(
        adapter: Arc<dyn MonitoringToDashboardAdapter>,
        dashboard_service: Arc<dyn DashboardService>,
    ) -> Self {
        Self {
            adapter,
            dashboard_service,
        }
    }
    
    /// Start the dashboard monitor
    pub async fn start(&self) -> Result<(), std::io::Error> {
        self.adapter.start(self.dashboard_service.clone()).await
    }
    
    /// Get the current dashboard data
    pub async fn get_dashboard_data(&self) -> Result<DashboardData, std::io::Error> {
        self.adapter.get_dashboard_data().await
    }
    
    /// Get the current connection status
    pub async fn get_connection_status(&self) -> Result<ConnectionStatus, std::io::Error> {
        self.adapter.get_connection_status().await
    }
    
    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, std::io::Error> {
        self.adapter.get_performance_metrics().await
    }
}

/// MCP metrics provider interface
#[async_trait]
pub trait McpMetricsProviderTrait: Send + Sync {
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
impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "Connected"),
            ConnectionStatus::Disconnected => write!(f, "Disconnected"),
            ConnectionStatus::Connecting => write!(f, "Connecting"),
            ConnectionStatus::Degraded => write!(f, "Degraded"),
            ConnectionStatus::Unknown => write!(f, "Unknown"),
            ConnectionStatus::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}
