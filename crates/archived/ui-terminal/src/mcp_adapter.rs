use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration as StdDuration; // Alias to avoid conflict

use tokio::sync::{Mutex, mpsc};
use tokio::time::{Instant, Duration}; // Use tokio's time types

use chrono::{DateTime, Utc};
use squirrel_mcp::client::{MCPClient as SquirrelMcpClient, ClientState as SquirrelClientState, Message as SquirrelMcpMessage, MessageType as SquirrelMessageType};
use crate::adapter::{ConnectionEvent, ConnectionEventType, ConnectionHealth, ConnectionStatus, McpMetricsProviderTrait, PerformanceOptions};
use dashboard_core::data::McpMetrics;
use dashboard_core::metrics::PerformanceMetrics;
use crate::config::McpMetricsConfig;
use log::{debug, error, info, warn};
use thiserror::Error;
use tokio::task;

// Import core data types
use dashboard_core::{\
    DashboardData, Metrics, CpuMetrics, MemoryMetrics, NetworkMetrics, DiskMetrics, Alert, ProtocolData,\
};
use dashboard_core::adapter; // Use alias for dashboard_core::adapter
use crate::adapter::config::AdapterConfig;
use crate::adapter::metrics_provider::{McpMetricsProviderTrait, MetricProviderError};
use crate::mcp_client_wrapper::MCPClient;
use dashboard_core::mcp_models::{McpActionResponse, McpApiError, McpClientError, McpConfig, McpConnection, McpEvent, McpMessage, McpOperationResult, McpQueryResponse, McpTool}; // Keep needed imports
use serde::de::DeserializeOwned;
use serde_json::Value; // Add Value
use tokio::sync::{RwLock};

/// Configuration for MCP Metrics Provider - Moved here or ensure imported correctly
// pub struct McpMetricsConfig { ... } // Definition should be elsewhere, ensure import

/// Caching utility for metrics or status data
#[derive(Debug, Clone)] // Added Clone derive
struct CachedMetrics<T: Clone> {
    value: Option<T>,
    last_updated: Option<Instant>,
    ttl: Duration, // Use tokio::time::Duration
}

impl<T: Clone> CachedMetrics<T> {
    /// Create a new metrics cache with the given TTL in milliseconds
    fn new(ttl_ms: u64) -> Self {
        Self {
            value: None,
            last_updated: None,
            ttl: Duration::from_millis(ttl_ms), // Use tokio Duration
        }
    }

    /// Create a new metrics cache with a default TTL (5 seconds)
    fn new_default() -> Self {
        Self::new(5000) // 5 seconds default TTL
    }

    /// Get the cached value if it's still valid
    fn get(&self) -> Option<T> {
        if let (Some(value), Some(last_updated)) = (&self.value, self.last_updated) {
            if last_updated.elapsed() < self.ttl {
                return Some(value.clone());
            }
        }
        None
    }

    /// Update the cached value
    fn update(&mut self, value: T) {
        self.value = Some(value);
        self.last_updated = Some(Instant::now());
    }

    /// Invalidate the cache
    fn invalidate(&mut self) {
        self.value = None;
        self.last_updated = None;
    }
}

/// Define McpClientType alias using the actual client
type McpClientType = SquirrelMcpClient;

/// Provides metrics and connection status information from an MCP source.
/// Uses tokio::sync::Mutex for asynchronous locking.
#[derive(Debug)] // Added derive Debug
pub struct RealMcpMetricsProvider {
    /// MCP client wrapped in Arc<Mutex> for safe concurrent access
    mcp_client: Option<Arc<Mutex<McpClientType>>>,

    /// Configuration for the provider
    config: McpMetricsConfig,

    /// Metrics cache (using McpMetrics from dashboard_core)
    metrics_cache: Mutex<CachedMetrics<McpMetrics>>,

    /// Protocol metrics cache (Assuming HashMap<String, f64> is correct)
    protocol_metrics_cache: Mutex<CachedMetrics<HashMap<String, f64>>>,

    /// Connection health cache
    connection_health_cache: Mutex<CachedMetrics<ConnectionHealth>>,

    /// Connection status cache
    connection_status_cache: Mutex<CachedMetrics<ConnectionStatus>>,

    /// Message log (using VecDeque)
    message_log: Arc<Mutex<VecDeque<String>>>, // Use VecDeque and Arc<Mutex>

    /// Error log (using VecDeque)
    error_log: Arc<Mutex<VecDeque<String>>>, // Use VecDeque and Arc<Mutex>

    /// Connection history (using VecDeque<adapter::ConnectionEvent>)
    connection_history: Arc<Mutex<VecDeque<adapter::ConnectionEvent>>>, // Use adapter::ConnectionEvent

    /// Performance metrics
    performance_metrics: Mutex<PerformanceMetrics>,

    /// Performance options
    performance_options: Mutex<PerformanceOptions>,

    /// Flag to simulate failures for testing
    should_fail: Arc<Mutex<bool>>,
}

impl RealMcpMetricsProvider {
    /// Creates a new `RealMcpMetricsProvider` with default configuration.
    /// The MCP client needs to be set separately using `set_client`.
    pub fn new(config: McpMetricsConfig) -> Self {
        // Default TTLs (adjust as needed)
        let metrics_ttl = config.update_interval_ms.max(1000); // e.g., 1 second minimum
        let status_ttl = metrics_ttl / 2; // Check status more frequently

        Self {
            mcp_client: None, // Client is set later
            // Use config values for capacities and TTLs
            metrics_cache: Mutex::new(CachedMetrics::new(metrics_ttl)),
            protocol_metrics_cache: Mutex::new(CachedMetrics::new(metrics_ttl)),
            connection_health_cache: Mutex::new(CachedMetrics::new(status_ttl)),
            connection_status_cache: Mutex::new(CachedMetrics::new(status_ttl)),
            message_log: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_message_log_size))),
            error_log: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_error_log_size))),
            connection_history: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_history_size))),
            performance_metrics: Mutex::new(PerformanceMetrics::default()),
            performance_options: Mutex::new(PerformanceOptions::default()), // Initialize performance options
            should_fail: Arc::new(Mutex::new(false)),
            config, // Store the config
        }
    }

    /// Sets the MCP client for the provider.
    pub async fn set_client(&mut self, client: Option<McpClientType>) {
        let new_client_arc = client.map(|c| Arc::new(Mutex::new(c)));
        self.mcp_client = new_client_arc;
        self.invalidate_all_caches().await;
    }
    
    /// Helper to invalidate all caches (e.g., on reconnect or client change)
    async fn invalidate_all_caches(&self) {
        self.metrics_cache.lock().await.invalidate();
        self.protocol_metrics_cache.lock().await.invalidate();
        self.connection_health_cache.lock().await.invalidate();
        self.connection_status_cache.lock().await.invalidate();
        // Logs and history are usually not invalidated this way
    }

    /// Initializes the connection by calling the MCPClient's connect method.
    pub async fn initialize(&self) -> Result<(), String> {
        self.add_connection_event(adapter::ConnectionEventType::Connecting, "Attempting to connect...".to_string()).await;

        if let Some(client_arc) = &self.mcp_client {
            let connect_result = {
                let mut client_guard = client_arc.lock().await;
                 // Assuming connect() takes &mut self might be incorrect, adjust if needed
                 // If connect takes &self, it's simpler: client_guard.connect().await
                 // Let's proceed assuming it needs &mut for state changes
                 client_guard.connect().await // Assuming connect signature is async fn connect(&mut self) -> Result<(), Error>
            };

            match connect_result {
                Ok(_) => {
                    self.add_connection_event(adapter::ConnectionEventType::Connected, "Connection successful.".to_string()).await;
                    // Invalidate caches on successful connect
                    self.invalidate_all_caches().await;
                    Ok(())
                }
                Err(e) => {
                    let err_msg = format!("Connection failed: {}", e);
                    error!("{}", err_msg);
                    // Use adapter::ConnectionEventType::Error
                    self.add_connection_event(adapter::ConnectionEventType::Error, err_msg.clone()).await;
                    Err(err_msg)
                }
            }
        } else {
            let err_msg = "MCP client not set. Cannot initialize connection.".to_string();
            warn!("{}", err_msg);
             // Use adapter::ConnectionEventType::Error
            self.add_connection_event(adapter::ConnectionEventType::Error, err_msg.clone()).await;
            Err(err_msg)
        }
    }

    /// Collects metrics directly from the MCP client by sending a command.
    /// This bypasses the cache.
    async fn collect_mcp_metrics(&self) -> Result<McpMetrics, String> {
        const COMMAND_NAME: &str = "get_mcp_metrics";

        if let Some(client_arc) = &self.mcp_client {
            let client_guard = client_arc.lock().await;
            match client_guard.send_command_with_content(COMMAND_NAME, serde_json::Value::Null).await {
                Ok(response) => {
                    // Use the imported SquirrelMessageType
                    if response.message_type == SquirrelMessageType::Response {
                        // Use from_str for String content
                        match serde_json::from_str::<McpMetrics>(&response.content) {
                            Ok(metrics) => Ok(metrics),
                            Err(e) => {
                                let err_msg = format!("Failed to deserialize McpMetrics from response content: {}", e);
                                error!("{}", err_msg);
                                Err(err_msg)
                            }
                        }
                    } else {
                        let err_msg = format!("Received unexpected message type or error response for {}: {:?}", COMMAND_NAME, response);
                         error!("{}", err_msg);
                         Err(err_msg)
                    }
                }
                Err(e) => {
                    let err_msg = format!("Failed to send command '{}': {}", COMMAND_NAME, e);
                    error!("{}", err_msg);
                    Err(err_msg)
                }
            }
        } else {
            Err("MCP client not set. Cannot collect MCP metrics.".to_string())
        }
    }
    
    /// Collects protocol-specific metrics directly from the MCP client.
    /// This bypasses the cache.
    async fn collect_protocol_metrics(&self) -> Result<HashMap<String, f64>, String> {
        const COMMAND_NAME: &str = "get_protocol_metrics";

         if let Some(client_arc) = &self.mcp_client {
            let client_guard = client_arc.lock().await;
            match client_guard.send_command_with_content(COMMAND_NAME, serde_json::Value::Null).await {
                Ok(response) => {
                    // Use the imported SquirrelMessageType
                     if response.message_type == SquirrelMessageType::Response {
                        // Use from_str for String content
                        match serde_json::from_str::<HashMap<String, f64>>(&response.content) {
                            Ok(metrics) => Ok(metrics),
                            Err(e) => {
                                let err_msg = format!("Failed to deserialize Protocol Metrics from response content: {}", e);
                                error!("{}", err_msg);
                                Err(err_msg)
                            }
                        }
                    } else {
                        let err_msg = format!("Received unexpected message type or error response for {}: {:?}", COMMAND_NAME, response);
                         error!("{}", err_msg);
                         Err(err_msg)
                    }
                }
                Err(e) => {
                    let err_msg = format!("Failed to send command '{}': {}", COMMAND_NAME, e);
                    error!("{}", err_msg);
                    Err(err_msg)
                }
            }
        } else {
            Err("MCP client not set. Cannot collect protocol metrics.".to_string())
        }
    }

    /// Subscribes to metric updates at a specified interval.
    /// Returns a channel receiver for `McpMetrics`.
    /// Note: This implementation polls; a real implementation might use MCP client's subscription features if available.
    pub fn subscribe(&self, interval_ms: u64) -> mpsc::Receiver<McpMetrics> {
        let (tx, rx) = mpsc::channel(100); // Buffer size 100
        let zelf = self.clone(); // Clone Arc<Self> for the task
        let interval = Duration::from_millis(interval_ms);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                // Fetch metrics (using the trait method which handles caching)
                match zelf.get_metrics().await {
                    Ok(metrics) => {
                        if tx.send(metrics).await.is_err() {
                            // Receiver dropped, stop the task
                           // info!("Metrics subscriber disconnected.");
                            break;
                        }
                    }
                    Err(_e) => {
                        //error!("Failed to get metrics for subscriber: {}", e);
                        // Decide if the task should stop or continue trying
                    }
                }
            }
        });

        rx
    }
    
    /// Adds an event to the connection history, maintaining max size.
    /// Uses adapter::ConnectionEventType and adapter::ConnectionEvent
    async fn add_connection_event(&self, event_type: adapter::ConnectionEventType, details: String) {
        let event = adapter::ConnectionEvent { // Use adapter::ConnectionEvent
            timestamp: Utc::now(),
            event_type: event_type.clone(), // Clone event_type if needed later
            details,
        };
        
        let mut history = self.connection_history.lock().await;
        if history.len() >= self.config.max_history_size {
            history.pop_front(); // Remove oldest event
        }
        history.push_back(event); // Push adapter::ConnectionEvent

        // Also add errors to the error log
        if let adapter::ConnectionEventType::Error = event_type { // Match on adapter::Error variant
             let mut errors = self.error_log.lock().await;
             if errors.len() >= self.config.max_error_log_size {
                 errors.pop_front();
             }
             // Log the detail associated with the error event
             errors.push_back(format!("{}: {}", Utc::now(), details)); // Use details from the input event
         }
    }

    /// Checks the connection status directly by querying the MCPClient state.
    /// Returns adapter::ConnectionStatus
    async fn check_connection_status_direct(&self) -> ConnectionStatus { // Return adapter::ConnectionStatus
        if let Some(client_arc) = &self.mcp_client {
            let client_state = {
                let client_guard = client_arc.lock().await;
                // Assuming client_guard is MutexGuard<SquirrelMcpClient>
                client_guard.get_state().await // Call get_state on the client
            };

            // Map MCPClient::ClientState to adapter::ConnectionStatus
            // Use imported SquirrelClientState
            match client_state {
                SquirrelClientState::Connected => ConnectionStatus::Connected,
                SquirrelClientState::Connecting => ConnectionStatus::Connecting,
                SquirrelClientState::Disconnected => ConnectionStatus::Disconnected,
                SquirrelClientState::Disconnecting => ConnectionStatus::Connecting, // Map Disconnecting->Connecting?
                SquirrelClientState::Failed => ConnectionStatus::Error("Connection failed".to_string()),
                // Add default case or ensure all variants are handled if ClientState enum changes
            }
        } else {
            ConnectionStatus::Disconnected // Return adapter::ConnectionStatus variant
        }
    }
    
    /// Gets the connection status, using the cache if possible.
    async fn get_connection_status_cached(&self) -> Result<ConnectionStatus, String> {
        if let Some(status) = self.connection_status_cache.lock().await.get() {
            return Ok(status);
        }

        let status = self.check_connection_status_direct().await;
        self.connection_status_cache.lock().await.update(status.clone());
        Ok(status) 
    }

    /// Attempts to reconnect the MCP client.
    /// Uses adapter::ConnectionEventType
    pub async fn reconnect_client(&self) -> Result<bool, String> {
        info!("Attempting to reconnect MCP client...");
        // Use adapter::ConnectionEventType::Reconnecting
        self.add_connection_event(adapter::ConnectionEventType::Reconnecting, "Attempting to reconnect...".to_string()).await;

        if let Some(client_arc) = &self.mcp_client {
            let mut client_guard = client_arc.lock().await;

            // 1. Attempt to disconnect first (ignore error)
            match client_guard.disconnect().await {
                Ok(_) => info!("MCP Client disconnected successfully before reconnecting."),
                Err(e) => warn!("Error during disconnect before reconnect: {} (proceeding with connect attempt)", e),
            }

            // 2. Attempt to connect
            match client_guard.connect().await {
                Ok(_) => {
                    info!("MCP Client reconnected successfully.");
                    self.invalidate_all_caches().await;
                    // Use adapter::ConnectionEventType::ReconnectSuccess
                    self.add_connection_event(adapter::ConnectionEventType::ReconnectSuccess, "Reconnection successful.".to_string()).await;
                    Ok(true)
                }
                Err(e) => {
                    let err_msg = format!("Reconnection failed: {}", e);
                    error!("{}", err_msg);
                    // Use adapter::ConnectionEventType::ReconnectFailure
                    self.add_connection_event(adapter::ConnectionEventType::ReconnectFailure, err_msg.clone()).await;
                    Err(err_msg)
                }
            }
        } else {
            let err_msg = "MCP client not set. Cannot reconnect.".to_string();
            warn!("{}", err_msg);
            // Use adapter::ConnectionEventType::ReconnectFailure
            self.add_connection_event(adapter::ConnectionEventType::ReconnectFailure, err_msg.clone()).await;
            Err(err_msg)
        }
    }

    // --- Methods for accessing logs and history ---

    /// Gets the connection history. Returns Vec<adapter::ConnectionEvent>
    async fn get_connection_history_internal(&self) -> Result<Vec<adapter::ConnectionEvent>, String> {
        let history_arc = Arc::clone(&self.connection_history);
        let history_guard = history_arc.lock().await;
        // Collects the VecDeque<adapter::ConnectionEvent> into Vec<adapter::ConnectionEvent>
        Ok(history_guard.iter().cloned().collect())
    }

    /// Gets the message log.
    async fn get_message_log_internal(&self) -> Result<Vec<String>, String> {
        let log_arc = Arc::clone(&self.message_log);
        let log_guard = log_arc.lock().await;
        Ok(log_guard.iter().cloned().collect())
    }

    /// Gets recent errors from the error log.
    async fn get_recent_errors_internal(&self, limit: usize) -> Result<Vec<String>, String> {
        let errors_arc = Arc::clone(&self.error_log);
        let errors_guard = errors_arc.lock().await;
        let start_index = errors_guard.len().saturating_sub(limit);
        Ok(errors_guard.range(start_index..).cloned().collect())
    }

    /// Gets the full error log.
    async fn get_error_log_internal(&self) -> Result<Vec<String>, String> {
        let errors_arc = Arc::clone(&self.error_log);
        let errors_guard = errors_arc.lock().await;
        Ok(errors_guard.iter().cloned().collect())
    }

    /// Sets performance monitoring options.
    async fn set_performance_options_internal(&self, options: PerformanceOptions) -> Result<(), String> {
        let mut perf_opts = self.performance_options.lock().await;
        *perf_opts = options;
        Ok(())
    }

    /// Gets collected performance metrics.
    async fn get_performance_metrics_internal(&self) -> Result<PerformanceMetrics, String> {
        let perf_metrics = self.performance_metrics.lock().await;
        Ok(perf_metrics.clone()) // Assuming PerformanceMetrics is Clone
    }

    /// Sets the flag to simulate failures.
    async fn set_should_fail_internal(&self, should_fail: bool) {
        let mut fail_flag = self.should_fail.lock().await;
        *fail_flag = should_fail;
    }

    /// Calculates connection health based on status and history.
    /// Returns adapter::ConnectionHealth
    async fn calculate_connection_health(&self) -> ConnectionHealth { // Return adapter::ConnectionHealth
        let current_status = self.check_connection_status_direct().await;
        let now = Utc::now();
        let history = self.connection_history.lock().await; // VecDeque<adapter::ConnectionEvent>

        let last_connect_time = history.iter()
            // Use adapter::ConnectionEventType variants
            .filter(|e| matches!(e.event_type, adapter::ConnectionEventType::Connected | adapter::ConnectionEventType::ReconnectSuccess))
            .last()
            .map(|e| e.timestamp);

        let last_change_time = history.back().map(|e| e.timestamp);

        // Initialize adapter::ConnectionHealth correctly
        ConnectionHealth {
            status: current_status.clone(), // Use adapter::ConnectionStatus
            connected_since: if current_status == ConnectionStatus::Connected { last_connect_time } else { None },
            last_status_change: last_change_time, // Store Option<DateTime> directly
            last_checked: now, // Store DateTime directly
            // Add missing fields from adapter::ConnectionHealth definition with reasonable defaults/calculations
            latency_ms: 50.0, // TODO: Calculate or fetch actual latency
            packet_loss: 0.0, // TODO: Calculate or fetch actual packet loss
            stability: if current_status == ConnectionStatus::Connected { 100.0 } else { 0.0 }, // Simple stability logic
            signal_strength: 100.0, // TODO: Fetch if applicable
            health_score: if current_status == ConnectionStatus::Connected { 1.0 } else { 0.0 }, // Simple score (0.0-1.0)
        }
    }

    /// Gets connection health, using cache if possible.
    async fn get_connection_health_cached(&self) -> Result<ConnectionHealth, String> {
        if let Some(health) = self.connection_health_cache.lock().await.get() {
            return Ok(health);
        }

        let health = self.calculate_connection_health().await;
        self.connection_health_cache.lock().await.update(health.clone());
        Ok(health)
    }

    /// Starts a background task to periodically check the connection status
    /// and send events through the returned channel.
    /// Uses adapter::ConnectionEventType and adapter::ConnectionEvent
    pub fn start_connection_check(&self, interval_ms: u64) -> mpsc::Receiver<adapter::ConnectionEvent> { // Return adapter::ConnectionEvent
        let (tx, rx) = mpsc::channel(100);
        let zelf = self.clone(); // Clone Arc<Self>
        let interval = Duration::from_millis(interval_ms);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            let mut last_known_status = ConnectionStatus::Connecting; // Initial assumption

            loop {
                interval_timer.tick().await;
                let current_status = zelf.check_connection_status_direct().await;

                // Detect status changes and send events
                if current_status != last_known_status {
                    // Use adapter::ConnectionStatus and adapter::ConnectionEventType
                    let event_type: Option<adapter::ConnectionEventType> = match (&last_known_status, &current_status) {
                        (ConnectionStatus::Connecting, ConnectionStatus::Connected) => Some(adapter::ConnectionEventType::Connected),
                        (ConnectionStatus::Connecting, ConnectionStatus::Disconnected) => Some(adapter::ConnectionEventType::Disconnected),
                        (ConnectionStatus::Connecting, ConnectionStatus::Error(_)) => Some(adapter::ConnectionEventType::Error),
                        (ConnectionStatus::Connected, ConnectionStatus::Disconnected) => Some(adapter::ConnectionEventType::Disconnected),
                        (ConnectionStatus::Connected, ConnectionStatus::Error(_)) => Some(adapter::ConnectionEventType::Error),
                        (ConnectionStatus::Disconnected, ConnectionStatus::Connecting) => Some(adapter::ConnectionEventType::Reconnecting),
                        (ConnectionStatus::Disconnected, ConnectionStatus::Connected) => Some(adapter::ConnectionEventType::ReconnectSuccess),
                        (ConnectionStatus::Disconnected, ConnectionStatus::Error(_)) => Some(adapter::ConnectionEventType::Error),
                        (ConnectionStatus::Error(_), ConnectionStatus::Connecting) => Some(adapter::ConnectionEventType::Reconnecting),
                        (ConnectionStatus::Error(_), ConnectionStatus::Connected) => Some(adapter::ConnectionEventType::ReconnectSuccess),
                        (ConnectionStatus::Error(old_e), ConnectionStatus::Error(new_e)) if old_e != new_e => Some(adapter::ConnectionEventType::Error),
                        _ => None,
                    };

                    if let Some(ev_type) = event_type {
                        let details = match &current_status {
                             ConnectionStatus::Connected => "Connection established".to_string(),
                             ConnectionStatus::Disconnected => "Connection lost".to_string(),
                             ConnectionStatus::Connecting => "Attempting connection...".to_string(),
                             ConnectionStatus::Error(e) => format!("Connection error: {}", e),
                             ConnectionStatus::Degraded => "Connection degraded".to_string(), // Handle Degraded case if needed
                             ConnectionStatus::Unknown => "Connection status unknown".to_string(), // Handle Unknown case
                         };

                        // Add to internal history *and* send to subscriber
                        // Pass the determined event type and details
                        zelf.add_connection_event(ev_type.clone(), details.clone()).await;

                        // Create adapter::ConnectionEvent
                        let event = adapter::ConnectionEvent {
                            timestamp: Utc::now(),
                            event_type: ev_type,
                            details,
                        };

                        if tx.send(event).await.is_err() {
                           // info!("Connection status subscriber disconnected.");
                            break; // Stop task if receiver is dropped
                        }
                    }
                    last_known_status = current_status; // Update last known status
                }
                 // Also update cache regardless of change notification
                 zelf.connection_status_cache.lock().await.update(last_known_status.clone());
            }
        });

        rx
    }

    /// Checks if there are any errors in the error log.
    pub async fn has_errors(&self) -> bool {
        !self.error_log.lock().await.is_empty()
    }
}

// --- Clone Implementation ---

impl Clone for RealMcpMetricsProvider {
    fn clone(&self) -> Self {
        Self {
            mcp_client: self.mcp_client.clone(),
            config: self.config.clone(),
            metrics_cache: Mutex::new(self.metrics_cache.blocking_lock().clone()),
            protocol_metrics_cache: Mutex::new(self.protocol_metrics_cache.blocking_lock().clone()),
            connection_health_cache: Mutex::new(self.connection_health_cache.blocking_lock().clone()),
            connection_status_cache: Mutex::new(self.connection_status_cache.blocking_lock().clone()),
            message_log: Arc::clone(&self.message_log),
            error_log: Arc::clone(&self.error_log),
            connection_history: Arc::clone(&self.connection_history),
            performance_metrics: Mutex::new(self.performance_metrics.blocking_lock().clone()),
            performance_options: Mutex::new(self.performance_options.blocking_lock().clone()),
            should_fail: Arc::clone(&self.should_fail),
        }
    }
}

// --- Trait Implementation ---

#[async_trait::async_trait]
impl McpMetricsProviderTrait for RealMcpMetricsProvider {
    /// Gets MCP metrics, using the cache if available and valid.
    async fn get_metrics(&self) -> Result<McpMetrics, String> {
        if let Some(metrics) = self.metrics_cache.lock().await.get() {
            return Ok(metrics);
        }
        let metrics = self.collect_mcp_metrics().await?;
        self.metrics_cache.lock().await.update(metrics.clone());
        Ok(metrics)
    }

    /// Gets the connection status, using the cache.
     async fn get_connection_status(&self) -> Result<ConnectionStatus, String> {
         self.get_connection_status_cached().await
     }

    /// Gets the connection health, using the cache.
     async fn get_connection_health(&self) -> Result<ConnectionHealth, String> {
         self.get_connection_health_cached().await
     }

    /// Gets protocol metrics, using the cache.
    async fn get_protocol_metrics(&self) -> Result<HashMap<String, f64>, String> {
        if let Some(metrics) = self.protocol_metrics_cache.lock().await.get() {
            return Ok(metrics);
        }
        let metrics = self.collect_protocol_metrics().await?;
        self.protocol_metrics_cache.lock().await.update(metrics.clone());
        Ok(metrics)
    }

    /// Attempts to reconnect the client.
    async fn reconnect(&self) -> Result<bool, String> {
        info!("Attempting MCP reconnection via trait...");
        // Use the internal reconnect_client method
        self.reconnect_client().await
    }

    /// Gets the connection history. Returns Vec<adapter::ConnectionEvent>.
    async fn get_connection_history(&self) -> Result<Vec<adapter::ConnectionEvent>, String> { // Correct signature
        self.get_connection_history_internal().await // Call internal helper
    }

    /// Gets the message log.
    async fn get_message_log(&self) -> Result<Vec<String>, String> {
        self.get_message_log_internal().await
    }

    /// Gets recent errors.
    async fn get_recent_errors(&self, limit: usize) -> Result<Vec<String>, String> {
        self.get_recent_errors_internal(limit).await
    }

    /// Gets the full error log.
    async fn get_error_log(&self) -> Result<Vec<String>, String> {
        self.get_error_log_internal().await
    }

    /// Sets performance options.
    async fn set_performance_options(&self, options: PerformanceOptions) -> Result<(), String> {
        self.set_performance_options_internal(options).await
    }

    /// Gets performance metrics.
    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, String> {
        self.get_performance_metrics_internal().await
    }

    /// Sets the failure simulation flag.
    async fn set_should_fail(&self, should_fail: bool) {
        self.set_should_fail_internal(should_fail).await;
        // The trait doesn't specify a return value, so we don't return Ok(())
    }
}

// --- Helper Functions / Factory ---

/// Factory function to create a RealMcpMetricsProvider instance wrapped in Arc.
pub fn create_mcp_metrics_provider(config: McpMetricsConfig) -> Arc<dyn McpMetricsProviderTrait> { // Return Arc<dyn Trait>
    Arc::new(RealMcpMetricsProvider::new(config))
}

/// Calculates time since a given DateTime<Utc>.
fn time_since(timestamp: &DateTime<Utc>) -> StdDuration {
    Utc::now().signed_duration_since(*timestamp).to_std().unwrap_or_default()
}

// // --- Old MCPAdapter (Commented Out/Removed) ---
// /// MCP adapter for the UI
// // pub struct MCPAdapter {
// //     /// MCP client
// //     client: Arc<Mutex<MCPClient>>, // Needs correct MCPClient type
// //     /// Connection history
// //     connection_history: Arc<Mutex<Vec<ConnectionEvent>>>, // Needs adapter::ConnectionEvent
// // }

// // impl MCPAdapter {
// //     /// Create a new MCP adapter
// //     pub fn new(client: Arc<Mutex<MCPClient>>) -> Self { // Needs correct MCPClient type
// //         Self {
// //             client,
// //             connection_history: Arc::new(Mutex::new(Vec::new())),
// //         }
// //     }

// //     /// Get the current metrics
// //     pub async fn get_metrics(&self) -> Option<DashboardData> {
// //         // Removed incorrect lock match
// //         let client_guard = self.client.lock().await; // Lock needs error handling or expect
// //         // Removed dummy data logic
// //         None // Placeholder
// //     }

// //     /// Get the connection history
// //     pub async fn get_connection_history(&self) -> Vec<ConnectionEvent> { // Needs adapter::ConnectionEvent
// //         let history = self.connection_history.lock().await;
// //         history.clone()
// //     }
// // }

// // --- Redundant Local Event Types (Commented Out/Removed) ---
// // /// Connection event
// // #[derive(Clone, Debug)]
// // pub struct ConnectionEvent { ... }

// // /// Connection event type
// // #[derive(Clone, Debug, PartialEq)]
// // pub enum ConnectionEventType { ... }

// // pub struct ConnectionEvent { ... }
// // pub enum ConnectionEventType { ... }

/// MCP adapter for the UI
pub struct MCPAdapter {
    /// MCP client
    client: Arc<Mutex<MCPClient>>,
    /// Connection history
    connection_history: Arc<Mutex<Vec<ConnectionEvent>>>,
}

impl MCPAdapter {
    /// Create a new MCP adapter
    pub fn new(client: Arc<Mutex<MCPClient>>) -> Self {
        Self {
            client,
            connection_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the current metrics
    pub async fn get_metrics(&self) -> Option<DashboardData> {
        let client_guard = match self.client.lock().await {
             Ok(guard) => guard,
             Err(e) => {
                 error!("Failed to lock MCPClient mutex: {}", e);
                 return None; // Return None if lock fails
             }
        };
        
        // FIXME: This still uses dummy data. Needs call to actual metrics provider.
        //        Also, client_guard is MCPClient, not the provider trait.
        let _ = client_guard; // Use guard to avoid unused warning for now

        // Create default metrics
        let metrics = Metrics {
            cpu: CpuMetrics {
                usage: 0.0,
                cores: vec![],
                temperature: None,
                load: [0.0, 0.0, 0.0],
            },
            memory: MemoryMetrics {
                total: 0,
                used: 0,
                available: 0,
                free: 0,
                swap_used: 0,
                swap_total: 0,
            },
            network: NetworkMetrics {
                interfaces: vec![],
                total_rx_bytes: 0,
                total_tx_bytes: 0,
                total_rx_packets: 0,
                total_tx_packets: 0,
            },
            disk: DiskMetrics {
                usage: Default::default(),
                total_reads: 0,
                total_writes: 0,
                read_bytes: 0,
                written_bytes: 0,
            },
            history: Default::default(),
        };

        Some(DashboardData {
            metrics,
            protocol: Default::default(),
            alerts: vec![],
            timestamp: Utc::now(),
        })
    }

    /// Get the connection history
    pub async fn get_connection_history(&self) -> Vec<ConnectionEvent> {
        let history = self.connection_history.lock().await;
        history.clone()
    }
}

/// Connection event
#[derive(Clone, Debug)]
pub struct ConnectionEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: ConnectionEventType,
    /// Event details
    pub details: String,
}

/// Connection event type
#[derive(Clone, Debug, PartialEq)] // Added PartialEq
pub enum ConnectionEventType {
    /// Connected to server
    Connected,
    /// Disconnected from server
    Disconnected,
    /// Reconnecting to server
    Reconnecting,
    /// Reconnection successful
    ReconnectSuccess,
    /// Reconnection failed
    ReconnectFailure,
    /// Connection error
    Error(String),
    Connecting, // Added missing variant
} 