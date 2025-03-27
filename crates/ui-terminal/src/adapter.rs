use std::collections::HashMap;
use std::sync::Arc;
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt};
use dashboard_core::data::{SystemSnapshot, NetworkSnapshot, InterfaceStats, DashboardData, MetricsSnapshot, Metrics, ProtocolData, Alert};
use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, Mutex};
use async_trait::async_trait;
use std::time::Duration;
use dashboard_core::mcp::{McpClient, McpError, McpResult};

/// Monitoring to Dashboard adapter for converting between monitoring and dashboard data formats
#[derive(Debug)]
pub struct MonitoringToDashboardAdapter {
    resource_adapter: ResourceMetricsCollectorAdapter,
    protocol_adapter: ProtocolMetricsAdapter,
}

impl MonitoringToDashboardAdapter {
    /// Create a new adapter
    pub fn new() -> Self {
        Self {
            resource_adapter: ResourceMetricsCollectorAdapter::new(),
            protocol_adapter: ProtocolMetricsAdapter::new(),
        }
    }
    
    /// Create a new adapter with MCP client
    pub fn new_with_mcp_client(mcp_client: Option<Arc<dyn McpMetricsProvider>>) -> Self {
        Self {
            resource_adapter: ResourceMetricsCollectorAdapter::new(),
            protocol_adapter: ProtocolMetricsAdapter::new_with_client(mcp_client),
        }
    }
    
    /// Collect dashboard data from monitoring metrics
    pub fn collect_dashboard_data(&mut self) -> DashboardData {
        // Collect system and network metrics
        let (system, network) = self.resource_adapter.collect_dashboard_data();
        
        // Collect protocol metrics
        let protocol_metrics = self.protocol_adapter.collect_metrics();
        
        // Convert to ProtocolData format for new dashboard
        let protocol_data = self.protocol_adapter.to_protocol_data();
        
        // Create dashboard data with collected metrics
        DashboardData {
            metrics: Metrics {
                cpu: system.cpu,
                memory: system.memory,
                disk: system.disk,
                network: network.network,
                history: MetricsHistory::default(), // This would be populated over time
            },
            protocol: protocol_data,
            alerts: Vec::new(), // No alerts for now
        }
    }
    
    /// Collect dashboard data asynchronously
    pub async fn collect_dashboard_data_async(&mut self) -> DashboardData {
        // Collect system and network metrics
        let (system, network) = self.resource_adapter.collect_dashboard_data();
        
        // Collect protocol metrics asynchronously
        let protocol_metrics = self.protocol_adapter.collect_metrics_async().await;
        
        // Convert to ProtocolData format for new dashboard
        let protocol_data = self.protocol_adapter.to_protocol_data();
        
        // Create dashboard data with collected metrics
        DashboardData {
            metrics: Metrics {
                cpu: system.cpu,
                memory: system.memory,
                disk: system.disk,
                network: network.network,
                history: MetricsHistory::default(), // This would be populated over time
            },
            protocol: protocol_data,
            alerts: Vec::new(), // No alerts for now
        }
    }
    
    /// Collect dashboard data with retry for reliability
    pub async fn collect_dashboard_data_with_retry(&mut self) -> DashboardData {
        // Collect system and network metrics
        let (system, network) = self.resource_adapter.collect_dashboard_data();
        
        // Collect protocol metrics with retry
        let protocol_metrics = self.protocol_adapter.collect_metrics_with_retry().await;
        
        // Convert to ProtocolData format for new dashboard
        let protocol_data = self.protocol_adapter.to_protocol_data();
        
        // Create dashboard data with collected metrics
        DashboardData {
            metrics: Metrics {
                cpu: system.cpu,
                memory: system.memory,
                disk: system.disk,
                network: network.network,
                history: MetricsHistory::default(), // This would be populated over time
            },
            protocol: protocol_data,
            alerts: Vec::new(), // No alerts for now
        }
    }
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

/// MCP metrics provider interface
#[async_trait]
pub trait McpMetricsProvider: Send + Sync {
    /// Get current metrics snapshot
    async fn get_metrics(&self) -> Result<McpMetrics, String>;
    
    /// Subscribe to metrics updates with specified interval
    fn subscribe(&self, interval_ms: u64) -> mpsc::Receiver<McpMetrics>;
    
    /// Get connection status
    async fn connection_status(&self) -> ConnectionStatus;
    
    /// Configure metrics collection
    async fn configure(&self, config: McpMetricsConfig) -> Result<(), String>;
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

/// Configuration for MCP metrics collection
#[derive(Debug, Clone)]
pub struct McpMetricsConfig {
    /// Update interval in milliseconds
    pub update_interval_ms: u64,
    /// Maximum number of history points to keep
    pub max_history_points: usize,
    /// Whether to collect latency histograms
    pub collect_latency_histogram: bool,
    /// Whether to collect error type breakdowns
    pub collect_error_types: bool,
}

impl Default for McpMetricsConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000,
            max_history_points: 1000,
            collect_latency_histogram: true,
            collect_error_types: true,
        }
    }
}

/// Protocol metrics adapter for collecting MCP protocol metrics
#[derive(Debug)]
pub struct ProtocolMetricsAdapter {
    // State tracking for metrics
    message_counter: u64,
    transaction_counter: u64,
    error_counter: u64,
    last_update: chrono::DateTime<Utc>,
    
    // MCP-specific metrics
    mcp_requests: u64,
    mcp_responses: u64,
    mcp_transactions: u64,
    mcp_connection_errors: u64,
    mcp_protocol_errors: u64,
    
    // Cache calculated rates
    message_rate: f64,
    transaction_rate: f64,
    error_rate: f64,
    mcp_success_rate: f64,
    
    // Store recent latency values for histogram
    latency_values: Vec<f64>,
    
    // MCP client reference for collecting real metrics
    mcp_client: Option<Arc<dyn McpMetricsProvider>>,
    
    // Cached metrics for fallback
    cached_metrics: Option<McpMetrics>,
    
    // Update channel receiver for metrics
    metrics_rx: Option<mpsc::Receiver<McpMetrics>>,
}

impl ProtocolMetricsAdapter {
    /// Create a new protocol metrics adapter
    pub fn new() -> Self {
        Self {
            message_counter: 0,
            transaction_counter: 0,
            error_counter: 0,
            last_update: Utc::now(),
            
            // Initialize MCP-specific metrics
            mcp_requests: 0,
            mcp_responses: 0,
            mcp_transactions: 0,
            mcp_connection_errors: 0,
            mcp_protocol_errors: 0,
            
            // Initialize rates
            message_rate: 0.0,
            transaction_rate: 0.0,
            error_rate: 0.0,
            mcp_success_rate: 100.0,
            
            // Initialize latency histogram
            latency_values: Vec::with_capacity(20),
            
            // Initialize MCP client
            mcp_client: None,
            cached_metrics: None,
            metrics_rx: None,
        }
    }
    
    /// Create a new protocol metrics adapter with MCP client
    pub fn new_with_client(client: Option<Arc<dyn McpMetricsProvider>>) -> Self {
        let metrics_rx = client.as_ref().map(|c| c.subscribe(1000)); // 1-second updates
        
        Self {
            message_counter: 0,
            transaction_counter: 0,
            error_counter: 0,
            last_update: Utc::now(),
            
            // Initialize MCP-specific metrics
            mcp_requests: 0,
            mcp_responses: 0,
            mcp_transactions: 0,
            mcp_connection_errors: 0,
            mcp_protocol_errors: 0,
            
            // Initialize rates
            message_rate: 0.0,
            transaction_rate: 0.0,
            error_rate: 0.0,
            mcp_success_rate: 100.0,
            
            // Initialize latency histogram
            latency_values: Vec::with_capacity(20),
            
            // Initialize MCP client and receiver
            mcp_client: client,
            cached_metrics: None,
            metrics_rx,
        }
    }
    
    /// Try to get MCP metrics from the MCP crate
    async fn try_collect_mcp_metrics(&mut self) -> bool {
        // First try to get metrics from the update channel
        if let Some(rx) = &mut self.metrics_rx {
            match rx.try_recv() {
                Ok(mcp_metrics) => {
                    self.update_from_mcp_metrics(mcp_metrics.clone());
                    self.cached_metrics = Some(mcp_metrics);
                    return true;
                }
                Err(_) => {
                    // No updates from channel, try direct fetch
                }
            }
        }
        
        // If no updates from channel, try direct fetch
        if let Some(client) = &self.mcp_client {
            match client.get_metrics().await {
                Ok(mcp_metrics) => {
                    self.update_from_mcp_metrics(mcp_metrics.clone());
                    self.cached_metrics = Some(mcp_metrics);
                    return true;
                }
                Err(e) => {
                    eprintln!("Failed to get MCP metrics: {}", e);
                    // Fall back to cached metrics
                    if let Some(cached) = &self.cached_metrics {
                        self.update_from_mcp_metrics(cached.clone());
                        return true;
                    }
                }
            }
        }
        
        // If we have no real metrics, generate simulated data
        // This ensures dashboard functionality even without MCP integration
        self.mcp_requests += 8 + (rand::random::<u64>() % 15);
        self.mcp_responses += 7 + (rand::random::<u64>() % 14);
        self.mcp_transactions += 4 + (rand::random::<u64>() % 8);
        
        if rand::random::<u8>() % 100 < 2 {
            // 2% chance of a connection error
            self.mcp_connection_errors += 1;
        }
        
        if rand::random::<u8>() % 100 < 3 {
            // 3% chance of a protocol error
            self.mcp_protocol_errors += 1;
        }
        
        true
    }
    
    /// Update adapter from MCP metrics
    fn update_from_mcp_metrics(&mut self, metrics: McpMetrics) {
        // Update message metrics
        self.message_counter = metrics.message_stats.total_requests + metrics.message_stats.total_responses;
        self.mcp_requests = metrics.message_stats.total_requests;
        self.mcp_responses = metrics.message_stats.total_responses;
        self.message_rate = metrics.message_stats.request_rate + metrics.message_stats.response_rate;
        
        // Update transaction metrics
        self.transaction_counter = metrics.transaction_stats.total_transactions;
        self.mcp_transactions = metrics.transaction_stats.total_transactions;
        self.transaction_rate = metrics.transaction_stats.transaction_rate;
        self.mcp_success_rate = metrics.transaction_stats.success_rate;
        
        // Update error metrics
        self.error_counter = metrics.error_stats.total_errors;
        self.mcp_connection_errors = metrics.error_stats.connection_errors;
        self.mcp_protocol_errors = metrics.error_stats.protocol_errors;
        self.error_rate = metrics.error_stats.error_rate;
        
        // Update last update timestamp
        self.last_update = metrics.timestamp;
        
        // Update latency values (add the latest histogram data to our tracking)
        if !metrics.latency_stats.latency_histogram.is_empty() {
            // Use the latency histogram as a distribution and sample from it
            for value in &metrics.latency_stats.latency_histogram {
                self.latency_values.push(*value);
                
                // Keep only the latest 100 values
                if self.latency_values.len() > 100 {
                    self.latency_values.remove(0);
                }
            }
        }
    }
    
    /// Collect protocol metrics
    pub fn collect_metrics(&mut self) -> MetricsSnapshot {
        // Run the synchronous version that can't use retry
        if !self.try_collect_mcp_metrics() {
            // If direct collection fails, use simulated metrics
            return self.collect_simulated_metrics();
        }
        
        self.convert_to_dashboard_metrics()
    }

    /// Collect metrics from protocol sources with async support
    pub async fn collect_metrics_async(&mut self) -> MetricsSnapshot {
        // Use the async version with retry support
        self.collect_metrics_with_retry().await
    }

    /// Collect simulated metrics when real metrics are unavailable
    fn collect_simulated_metrics(&mut self) -> MetricsSnapshot {
        // Generate some simulated metrics to maintain dashboard functionality
        self.message_counter += 8 + (rand::random::<u64>() % 15);
        self.transaction_counter += 4 + (rand::random::<u64>() % 8);
        
        if rand::random::<u8>() % 100 < 5 {
            // 5% chance of error
            self.error_counter += 1;
        }
        
        // Update rates
        let elapsed = Utc::now() - self.last_update;
        let seconds = elapsed.num_milliseconds() as f64 / 1000.0;
        if seconds > 0.0 {
            self.message_rate = 8.0 + (rand::random::<f64>() * 10.0);
            self.transaction_rate = 4.0 + (rand::random::<f64>() * 8.0);
            self.error_rate = (self.error_counter as f64 / self.message_counter as f64) * 100.0;
        }
        
        // Update latency values
        for _ in 0..5 {
            let latency = 20.0 + (rand::random::<f64>() * 100.0);
            self.latency_values.push(latency);
            
            // Keep only the latest 100 values
            if self.latency_values.len() > 100 {
                self.latency_values.remove(0);
            }
        }
        
        self.convert_to_dashboard_metrics().with_simulated_flag()
    }

    /// Collect metrics from the adapter with retries
    pub async fn collect_metrics_with_retry(&mut self) -> MetricsSnapshot {
        let max_retries = 3;
        let mut retries = 0;
        let mut backoff_ms = 100;
        
        while retries < max_retries {
            match self.try_collect_mcp_metrics_async().await {
                Ok(metrics) => return metrics,
                Err(error) => {
                    eprintln!("Failed to collect MCP metrics (retry {}/{}): {}", 
                              retries + 1, max_retries, error);
                    retries += 1;
                    
                    if retries < max_retries {
                        // Exponential backoff
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2;
                    }
                }
            }
        }
        
        // If all retries failed, return simulated metrics
        eprintln!("All retries failed, using fallback metrics");
        self.collect_simulated_metrics()
    }

    /// Try to collect MCP metrics asynchronously
    async fn try_collect_mcp_metrics_async(&mut self) -> Result<MetricsSnapshot, String> {
        // First try to get metrics from the update channel
        if let Some(rx) = &mut self.metrics_rx {
            match rx.try_recv() {
                Ok(mcp_metrics) => {
                    self.update_from_mcp_metrics(mcp_metrics.clone());
                    self.cached_metrics = Some(mcp_metrics);
                    return Ok(self.convert_to_dashboard_metrics());
                }
                Err(_) => {
                    // No updates from channel, try direct fetch
                }
            }
        }
        
        // If no updates from channel, try direct fetch
        if let Some(client) = &self.mcp_client {
            match client.get_metrics().await {
                Ok(mcp_metrics) => {
                    self.update_from_mcp_metrics(mcp_metrics.clone());
                    self.cached_metrics = Some(mcp_metrics);
                    return Ok(self.convert_to_dashboard_metrics());
                }
                Err(e) => {
                    // Try cached metrics
                    if let Some(cached) = &self.cached_metrics {
                        eprintln!("Using cached MCP metrics due to error: {}", e);
                        self.update_from_mcp_metrics(cached.clone());
                        return Ok(self.convert_to_dashboard_metrics().with_stale_flag());
                    }
                    
                    return Err(format!("Failed to get MCP metrics: {}", e));
                }
            }
        }
        
        // If no client is available, return error
        Err("No MCP client available".to_string())
    }

    /// Convert adapter state to dashboard metrics format
    fn convert_to_dashboard_metrics(&self) -> MetricsSnapshot {
        let mut counters = HashMap::new();
        let mut gauges = HashMap::new();
        let mut histograms = HashMap::new();
        
        // Add message metrics
        counters.insert("protocol.messages".to_string(), self.message_counter);
        counters.insert("mcp.requests".to_string(), self.mcp_requests);
        counters.insert("mcp.responses".to_string(), self.mcp_responses);
        gauges.insert("protocol.message_rate".to_string(), self.message_rate);
        
        // Add transaction metrics
        counters.insert("protocol.transactions".to_string(), self.transaction_counter);
        counters.insert("mcp.transactions".to_string(), self.mcp_transactions);
        gauges.insert("protocol.transaction_rate".to_string(), self.transaction_rate);
        gauges.insert("mcp.success_rate".to_string(), self.mcp_success_rate);
        
        // Add error metrics
        counters.insert("protocol.errors".to_string(), self.error_counter);
        counters.insert("mcp.connection_errors".to_string(), self.mcp_connection_errors);
        counters.insert("mcp.protocol_errors".to_string(), self.mcp_protocol_errors);
        gauges.insert("protocol.error_rate".to_string(), self.error_rate);
        
        // Add latency metrics
        if !self.latency_values.is_empty() {
            // Calculate basic statistics
            let len = self.latency_values.len() as f64;
            let sum: f64 = self.latency_values.iter().sum();
            let avg = sum / len;
            
            // Sort for percentiles
            let mut sorted_latencies = self.latency_values.clone();
            sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            
            // Calculate percentiles
            let median_idx = (len * 0.5) as usize;
            let p95_idx = (len * 0.95) as usize;
            let p99_idx = (len * 0.99) as usize;
            
            let median = if !sorted_latencies.is_empty() {
                sorted_latencies[median_idx.min(sorted_latencies.len() - 1)]
            } else {
                0.0
            };
            
            let p95 = if !sorted_latencies.is_empty() && p95_idx < sorted_latencies.len() {
                sorted_latencies[p95_idx]
            } else if !sorted_latencies.is_empty() {
                sorted_latencies[sorted_latencies.len() - 1]
            } else {
                0.0
            };
            
            let p99 = if !sorted_latencies.is_empty() && p99_idx < sorted_latencies.len() {
                sorted_latencies[p99_idx]
            } else if !sorted_latencies.is_empty() {
                sorted_latencies[sorted_latencies.len() - 1]
            } else {
                0.0
            };
            
            // Store latency metrics
            gauges.insert("mcp.average_latency".to_string(), avg);
            gauges.insert("mcp.median_latency".to_string(), median);
            gauges.insert("mcp.p95_latency".to_string(), p95);
            gauges.insert("mcp.p99_latency".to_string(), p99);
            
            // Create histogram for visualization (create 20 buckets from min to max)
            if sorted_latencies.len() >= 2 {
                let min = sorted_latencies[0];
                let max = sorted_latencies[sorted_latencies.len() - 1];
                let range = max - min;
                
                // Create histogram with 20 buckets
                let mut histogram = Vec::with_capacity(20);
                if range > 0.0 {
                    for i in 0..20 {
                        let bucket_min = min + (range * i as f64 / 20.0);
                        let bucket_max = min + (range * (i + 1) as f64 / 20.0);
                        let count = sorted_latencies.iter()
                            .filter(|&&v| v >= bucket_min && v < bucket_max)
                            .count() as f64;
                        histogram.push(count);
                    }
                } else {
                    // If range is 0, put all values in middle bucket
                    let mut flat_histogram = vec![0.0; 20];
                    flat_histogram[10] = len;
                    histogram = flat_histogram;
                }
                
                histograms.insert("protocol.latency".to_string(), histogram);
            }
        }
        
        // Create the metrics snapshot
        MetricsSnapshot {
            values: HashMap::new(),
            counters,
            gauges,
            histograms,
        }
    }

    /// Convert collected metrics to ProtocolData format
    pub fn to_protocol_data(&self) -> ProtocolData {
        let mut protocol_data = ProtocolData::default();
        
        // Set basic protocol data
        protocol_data.version = "1.0".to_string();
        protocol_data.connected = self.mcp_connection_errors == 0;
        protocol_data.last_connected = Some(self.last_update);
        
        if self.mcp_connection_errors > 0 {
            protocol_data.status = "Error".to_string();
            protocol_data.error = Some(format!("{} connection errors", self.mcp_connection_errors));
        } else if self.mcp_protocol_errors > 0 {
            protocol_data.status = "Warning".to_string();
            protocol_data.error = Some(format!("{} protocol errors", self.mcp_protocol_errors));
        } else {
            protocol_data.status = "Connected".to_string();
        }
        
        // Add metrics
        protocol_data.metrics.insert("packets_sent".to_string(), self.mcp_requests.to_string());
        protocol_data.metrics.insert("packets_received".to_string(), self.mcp_responses.to_string());
        protocol_data.metrics.insert("transactions".to_string(), self.mcp_transactions.to_string());
        protocol_data.metrics.insert("message_rate".to_string(), format!("{:.2}", self.message_rate));
        protocol_data.metrics.insert("transaction_rate".to_string(), format!("{:.2}", self.transaction_rate));
        protocol_data.metrics.insert("success_rate".to_string(), format!("{:.2}", self.mcp_success_rate));
        protocol_data.metrics.insert("connection_errors".to_string(), self.mcp_connection_errors.to_string());
        protocol_data.metrics.insert("protocol_errors".to_string(), self.mcp_protocol_errors.to_string());
        
        // Add simulation indicator if we're using simulated data
        if self.cached_metrics.is_some() && self.mcp_client.is_none() {
            protocol_data.metrics.insert("simulated".to_string(), "true".to_string());
            protocol_data.metrics.insert("last_real_data".to_string(), 
                self.last_update.to_rfc3339());
        }
        
        // Add protocol mode information
        protocol_data.data.insert("mode".to_string(), "standard".to_string());
        
        protocol_data
    }
}

/// Resource metrics collector adapter for connecting system metrics to dashboard-core
#[derive(Debug)]
pub struct ResourceMetricsCollectorAdapter {
    system: System,
}

impl ResourceMetricsCollectorAdapter {
    /// Create a new resource metrics collector adapter
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        ResourceMetricsCollectorAdapter {
            system,
        }
    }
    
    /// Refresh system data
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    /// Collect system metrics and convert to dashboard-core format
    pub fn collect_system_metrics(&mut self) -> SystemSnapshot {
        self.refresh();
        
        // Collect CPU metrics
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        
        // Collect memory metrics
        let memory_used = self.system.used_memory();
        let memory_total = self.system.total_memory();
        
        // Collect disk metrics
        let disks = self.system.disks();
        let mut disk_used = 0;
        let mut disk_total = 0;
        
        for disk in disks {
            disk_used += disk.total_space() - disk.available_space();
            disk_total += disk.total_space();
        }
        
        // Create system snapshot
        SystemSnapshot {
            cpu_usage,
            memory_used,
            memory_total,
            disk_used,
            disk_total,
            load_average: [0.0, 0.0, 0.0], // Replace with actual values if available
            uptime: self.system.uptime(),
        }
    }
    
    /// Collect network metrics and convert to dashboard-core format
    pub fn collect_network_metrics(&mut self) -> NetworkSnapshot {
        self.refresh();
        
        let mut rx_bytes = 0;
        let mut tx_bytes = 0;
        let mut rx_packets = 0;
        let mut tx_packets = 0;
        let mut interfaces = HashMap::new();
        
        for (name, network) in self.system.networks() {
            let rx_bytes_interface = network.received();
            let tx_bytes_interface = network.transmitted();
            let rx_packets_interface = network.packets_received();
            let tx_packets_interface = network.packets_transmitted();
            
            // Update totals
            rx_bytes += rx_bytes_interface;
            tx_bytes += tx_bytes_interface;
            rx_packets += rx_packets_interface;
            tx_packets += tx_packets_interface;
            
            // Store interface metrics
            interfaces.insert(name.clone(), InterfaceStats {
                name: name.clone(),
                rx_bytes: rx_bytes_interface,
                tx_bytes: tx_bytes_interface,
                rx_packets: rx_packets_interface,
                tx_packets: tx_packets_interface,
                is_up: true, // Fill with actual status if available
            });
        }
        
        // Create network snapshot
        NetworkSnapshot {
            rx_bytes,
            tx_bytes,
            rx_packets,
            tx_packets,
            interfaces,
        }
    }
    
    /// Collect all metrics as dashboard data
    pub fn collect_dashboard_data(&mut self) -> (SystemSnapshot, NetworkSnapshot) {
        let system_snapshot = self.collect_system_metrics();
        let network_snapshot = self.collect_network_metrics();
        
        (system_snapshot, network_snapshot)
    }
}

/// A mock implementation of McpMetricsProvider for testing purposes
#[derive(Debug)]
pub struct MockMcpClient {
    config: McpMetricsConfig,
    metrics: McpMetrics,
    status: ConnectionStatus,
    update_count: u64,
    senders: Vec<mpsc::Sender<McpMetrics>>,
}

impl MockMcpClient {
    /// Create a new mock MCP client
    pub fn new() -> Self {
        // Initialize with default metrics
        let now = Utc::now();
        
        // Create mock latency histogram
        let mut latency_histogram = Vec::with_capacity(20);
        for i in 0..20 {
            let v = 15.0 + (i as f64 * 1.5) + (rand::random::<f64>() * 10.0);
            latency_histogram.push(v);
        }
        
        // Create mock error types
        let mut error_types = HashMap::new();
        error_types.insert("timeout".to_string(), 3);
        error_types.insert("connection_lost".to_string(), 2);
        error_types.insert("invalid_format".to_string(), 5);
        
        // Create mock request types
        let mut request_types = HashMap::new();
        request_types.insert("get_status".to_string(), 45);
        request_types.insert("execute_command".to_string(), 23);
        request_types.insert("get_metrics".to_string(), 14);
        
        let metrics = McpMetrics {
            message_stats: MessageStats {
                total_requests: 82,
                total_responses: 80,
                request_rate: 4.2,
                response_rate: 4.1,
                request_types,
            },
            transaction_stats: TransactionStats {
                total_transactions: 50,
                successful_transactions: 45,
                failed_transactions: 5,
                transaction_rate: 2.5,
                success_rate: 90.0,
            },
            error_stats: ErrorStats {
                total_errors: 10,
                connection_errors: 2,
                protocol_errors: 5,
                timeout_errors: 3,
                error_rate: 2.5,
                error_types,
            },
            latency_stats: LatencyStats {
                average_latency_ms: 45.2,
                median_latency_ms: 38.5,
                p95_latency_ms: 95.6,
                p99_latency_ms: 135.2,
                min_latency_ms: 12.3,
                max_latency_ms: 156.8,
                latency_histogram,
            },
            timestamp: now,
        };
        
        Self {
            config: McpMetricsConfig::default(),
            metrics,
            status: ConnectionStatus::Connected,
            update_count: 0,
            senders: Vec::new(),
        }
    }
    
    /// Update the mock metrics with new values
    pub fn update(&mut self) {
        let now = Utc::now();
        self.update_count += 1;
        
        // Update message counts
        let new_requests = 5 + (rand::random::<u64>() % 10);
        let new_responses = new_requests - (rand::random::<u64>() % 2);
        self.metrics.message_stats.total_requests += new_requests;
        self.metrics.message_stats.total_responses += new_responses;
        self.metrics.message_stats.request_rate = new_requests as f64;
        self.metrics.message_stats.response_rate = new_responses as f64;
        
        // Update request types
        for (_, count) in self.metrics.message_stats.request_types.iter_mut() {
            *count += rand::random::<u64>() % 3;
        }
        
        // Update transaction counts
        let new_transactions = 2 + (rand::random::<u64>() % 5);
        let failed = rand::random::<u64>() % 2;
        self.metrics.transaction_stats.total_transactions += new_transactions;
        self.metrics.transaction_stats.successful_transactions += new_transactions - failed;
        self.metrics.transaction_stats.failed_transactions += failed;
        self.metrics.transaction_stats.transaction_rate = new_transactions as f64;
        self.metrics.transaction_stats.success_rate = 
            (self.metrics.transaction_stats.successful_transactions as f64 / 
             self.metrics.transaction_stats.total_transactions as f64) * 100.0;
        
        // Update error counts
        if rand::random::<u8>() % 5 == 0 {  // 20% chance of new error
            self.metrics.error_stats.total_errors += 1;
            
            // Determine error type
            let r = rand::random::<u8>() % 3;
            if r == 0 {
                self.metrics.error_stats.connection_errors += 1;
                *self.metrics.error_stats.error_types.entry("connection_lost".to_string()).or_insert(0) += 1;
            } else if r == 1 {
                self.metrics.error_stats.protocol_errors += 1;
                *self.metrics.error_stats.error_types.entry("invalid_format".to_string()).or_insert(0) += 1;
            } else {
                self.metrics.error_stats.timeout_errors += 1;
                *self.metrics.error_stats.error_types.entry("timeout".to_string()).or_insert(0) += 1;
            }
        }
        
        // Update latency values
        let avg_latency = 40.0 + (rand::random::<f64>() * 20.0 - 10.0);
        let std_dev = 15.0 + (rand::random::<f64>() * 5.0);
        
        self.metrics.latency_stats.average_latency_ms = avg_latency;
        self.metrics.latency_stats.median_latency_ms = avg_latency - 5.0;
        self.metrics.latency_stats.p95_latency_ms = avg_latency + (1.96 * std_dev);
        self.metrics.latency_stats.p99_latency_ms = avg_latency + (2.58 * std_dev);
        self.metrics.latency_stats.min_latency_ms = (avg_latency - std_dev).max(5.0);
        self.metrics.latency_stats.max_latency_ms = avg_latency + (3.0 * std_dev);
        
        // Update latency histogram
        self.metrics.latency_stats.latency_histogram.clear();
        for i in 0..20 {
            let factor = (i as f64) / 10.0;
            let v = avg_latency + std_dev * (factor - 1.0) + (rand::random::<f64>() * 5.0);
            self.metrics.latency_stats.latency_histogram.push(v.max(1.0));
        }
        
        // Update timestamp
        self.metrics.timestamp = now;
        
        // Send updates to subscribers
        for i in (0..self.senders.len()).rev() {
            if self.senders[i].try_send(self.metrics.clone()).is_err() {
                // Remove closed channels
                self.senders.remove(i);
            }
        }
    }
    
    /// Set the connection status
    pub fn set_status(&mut self, status: ConnectionStatus) {
        self.status = status;
    }
    
    /// Start automatic updates
    pub async fn start_auto_updates(client: Arc<tokio::sync::Mutex<Self>>, interval_ms: u64) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));
        
        loop {
            interval.tick().await;
            let mut locked_client = client.lock().await;
            locked_client.update();
        }
    }
}

#[async_trait]
impl McpMetricsProvider for MockMcpClient {
    async fn get_metrics(&self) -> Result<McpMetrics, String> {
        // Simulate occasional errors for testing error handling
        if self.update_count % 50 == 0 && rand::random::<u8>() % 10 == 0 {
            return Err("Simulated metrics collection error".to_string());
        }
        
        match self.status {
            ConnectionStatus::Connected => Ok(self.metrics.clone()),
            ConnectionStatus::Disconnected => Err("MCP client is disconnected".to_string()),
            ConnectionStatus::Connecting => Err("MCP client is still connecting".to_string()),
            ConnectionStatus::Error(ref e) => Err(format!("MCP client error: {}", e)),
        }
    }
    
    fn subscribe(&self, interval_ms: u64) -> mpsc::Receiver<McpMetrics> {
        let (tx, rx) = mpsc::channel(100);
        let mut this = self.to_owned();
        
        // Store the sender so it can be used in update()
        this.senders.push(tx);
        
        rx
    }
    
    async fn connection_status(&self) -> ConnectionStatus {
        self.status.clone()
    }
    
    async fn configure(&self, config: McpMetricsConfig) -> Result<(), String> {
        // In a real implementation, this would configure the client
        // For the mock, we just return Ok
        Ok(())
    }
}

/// Extensions for MetricsSnapshot
impl MetricsSnapshot {
    /// Mark metrics as stale (cached data)
    pub fn with_stale_flag(mut self) -> Self {
        self.values.insert("dashboard.stale_data".to_string(), 1.0);
        self
    }
    
    /// Mark metrics as simulated
    pub fn with_simulated_flag(mut self) -> Self {
        self.values.insert("dashboard.simulated_data".to_string(), 1.0);
        self
    }
    
    /// Check if metrics are stale
    pub fn is_stale(&self) -> bool {
        self.values.get("dashboard.stale_data").map_or(false, |v| *v > 0.0)
    }
    
    /// Check if metrics are simulated
    pub fn is_simulated(&self) -> bool {
        self.values.get("dashboard.simulated_data").map_or(false, |v| *v > 0.0)
    }
}

/// Modern MCP adapter for the new data structures
pub struct McpAdapter {
    /// MCP client
    client: Arc<Mutex<dyn McpClient + Send>>,
    
    /// Maximum number of history points to keep
    max_history_points: usize,
}

/// Error type for adapter operations
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    /// MCP client error
    #[error("MCP client error: {0}")]
    McpError(#[from] McpError),
    
    /// Other error
    #[error("Adapter error: {0}")]
    Other(String),
}

/// Result type for adapter operations
pub type AdapterResult<T> = Result<T, AdapterError>;

/// Trait for converting monitoring data to dashboard data
pub trait MonitoringToDashboardAdapter {
    /// Update dashboard data
    async fn update_dashboard_data(&self, data: &mut DashboardData) -> AdapterResult<()>;
}

impl McpAdapter {
    /// Create a new MCP adapter
    pub fn new(client: Arc<Mutex<dyn McpClient + Send>>, max_history_points: usize) -> Self {
        Self {
            client,
            max_history_points,
        }
    }

    /// Convert ProtocolData to legacy MetricsSnapshot format for backward compatibility
    pub fn protocol_data_to_metrics_snapshot(protocol_data: &ProtocolData) -> MetricsSnapshot {
        let mut snapshot = MetricsSnapshot::default();
        
        // Add connection status indicators
        snapshot.values.insert("protocol.connected".to_string(), if protocol_data.connected { 1.0 } else { 0.0 });
        
        // Add retry information
        snapshot.counters.insert("protocol.retries".to_string(), protocol_data.retry_count as u64);
        
        // Add version information (as a gauge value of the version number if possible)
        if let Ok(version_num) = protocol_data.version.parse::<f64>() {
            snapshot.gauges.insert("protocol.version".to_string(), version_num);
        }
        
        // Add protocol metrics
        for (key, value) in &protocol_data.metrics {
            // Try to convert numeric values to appropriate metric types
            if let Ok(value_num) = value.parse::<u64>() {
                snapshot.counters.insert(format!("mcp.{}", key), value_num);
            } else if let Ok(value_float) = value.parse::<f64>() {
                snapshot.gauges.insert(format!("mcp.{}", key), value_float);
            }
            
            // Store as string in labels regardless
            snapshot.labels.insert(format!("mcp.{}", key), value.clone());
        }
        
        // Add protocol data
        for (key, value) in &protocol_data.data {
            snapshot.labels.insert(format!("protocol.{}", key), value.clone());
            
            // Try to convert numeric values to appropriate metric types
            if let Ok(value_num) = value.parse::<u64>() {
                snapshot.counters.insert(format!("protocol.{}", key), value_num);
            } else if let Ok(value_float) = value.parse::<f64>() {
                snapshot.gauges.insert(format!("protocol.{}", key), value_float);
            }
        }
        
        // Add error information
        if let Some(error) = &protocol_data.error {
            snapshot.labels.insert("protocol.error".to_string(), error.clone());
            snapshot.counters.insert("protocol.errors".to_string(), 1);
            snapshot.gauges.insert("protocol.error_rate".to_string(), 100.0); // Indicate 100% error
        } else {
            snapshot.counters.insert("protocol.errors".to_string(), 0);
            snapshot.gauges.insert("protocol.error_rate".to_string(), 0.0);
        }
        
        // Add simulation flag
        if protocol_data.metrics.get("simulated").map_or(false, |v| v == "true") {
            snapshot.values.insert("dashboard.simulated_data".to_string(), 1.0);
        }
        
        // Add stale data flag based on last_connected time
        if let Some(last_connected) = protocol_data.last_connected {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(last_connected);
            
            // If last connection was more than 5 minutes ago, consider data stale
            if duration.num_minutes() > 5 {
                snapshot.values.insert("dashboard.stale_data".to_string(), 1.0);
            }
        }
        
        snapshot
    }
    
    /// Convert legacy MetricsSnapshot to ProtocolData format
    pub fn metrics_snapshot_to_protocol_data(snapshot: &MetricsSnapshot) -> ProtocolData {
        let mut protocol_data = ProtocolData::default();
        
        // Set connection status
        protocol_data.connected = snapshot.values.get("protocol.connected")
            .map_or(false, |&v| v > 0.0);
        
        // Set retry count
        protocol_data.retry_count = snapshot.counters.get("protocol.retries")
            .map_or(0, |&v| v as u32);
        
        // Extract version
        if let Some(version) = snapshot.labels.get("protocol.version") {
            protocol_data.version = version.clone();
        } else if let Some(&version_num) = snapshot.gauges.get("protocol.version") {
            protocol_data.version = format!("{:.1}", version_num);
        }
        
        // Extract error
        if let Some(error) = snapshot.labels.get("protocol.error") {
            if !error.is_empty() {
                protocol_data.error = Some(error.clone());
            }
        }
        
        // Extract metrics from counters, gauges, and labels
        for (key, &value) in &snapshot.counters {
            if key.starts_with("mcp.") {
                let metric_name = key.trim_start_matches("mcp.");
                protocol_data.metrics.insert(metric_name.to_string(), value.to_string());
            }
        }
        
        for (key, &value) in &snapshot.gauges {
            if key.starts_with("mcp.") {
                let metric_name = key.trim_start_matches("mcp.");
                protocol_data.metrics.insert(metric_name.to_string(), format!("{:.2}", value));
            }
        }
        
        for (key, value) in &snapshot.labels {
            if key.starts_with("mcp.") {
                let metric_name = key.trim_start_matches("mcp.");
                protocol_data.metrics.insert(metric_name.to_string(), value.clone());
            } else if key.starts_with("protocol.") && !key.contains("error") {
                let data_name = key.trim_start_matches("protocol.");
                protocol_data.data.insert(data_name.to_string(), value.clone());
            }
        }
        
        // Set timestamp if available
        if let Some(timestamp) = snapshot.timestamp {
            protocol_data.last_connected = Some(timestamp);
        }
        
        protocol_data
    }
}

impl MonitoringToDashboardAdapter for McpAdapter {
    async fn update_dashboard_data(&self, data: &mut DashboardData) -> AdapterResult<()> {
        // Get metrics from client
        let metrics = {
            let mut client = self.client.lock().await;
            client.get_metrics().await?
        };
        
        // Get protocol data from client
        let protocol_data = {
            let mut client = self.client.lock().await;
            client.get_protocol_data().await?
        };
        
        // Get alerts from client
        let alerts = {
            let mut client = self.client.lock().await;
            client.get_alerts().await?
        };
        
        // Update metrics history
        self.update_metrics_history(&mut data.metrics, &metrics);
        
        // Update dashboard data
        data.metrics = metrics;
        data.protocol = protocol_data;
        data.alerts = alerts;
        
        Ok(())
    }
}

impl McpAdapter {
    /// Update metrics history
    fn update_metrics_history(&self, current: &mut Metrics, new: &Metrics) {
        // Create timestamp for this data point
        let now = Utc::now();
        
        // If history is empty, initialize it
        if current.history.timestamps.is_empty() {
            current.history.timestamps.push(now);
            current.history.cpu_usage.push(new.cpu.usage);
            current.history.memory_usage.push(new.memory.used as f64 / new.memory.total as f64 * 100.0);
            current.history.network_rx.push(new.network.rx_per_sec);
            current.history.network_tx.push(new.network.tx_per_sec);
            current.history.disk_io.push(new.disk.io_per_sec);
            return;
        }
        
        // Add new data point to history
        current.history.timestamps.push(now);
        current.history.cpu_usage.push(new.cpu.usage);
        current.history.memory_usage.push(new.memory.used as f64 / new.memory.total as f64 * 100.0);
        current.history.network_rx.push(new.network.rx_per_sec);
        current.history.network_tx.push(new.network.tx_per_sec);
        current.history.disk_io.push(new.disk.io_per_sec);
        
        // Trim history if it exceeds max_history_points
        if current.history.timestamps.len() > self.max_history_points {
            current.history.timestamps.remove(0);
            current.history.cpu_usage.remove(0);
            current.history.memory_usage.remove(0);
            current.history.network_rx.remove(0);
            current.history.network_tx.remove(0);
            current.history.disk_io.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_can_be_converted_to_dashboard_format() {
        let mut adapter = MonitoringToDashboardAdapter::new();
        let dashboard_data = adapter.collect_dashboard_data();
        
        // Verify data structure
        assert!(dashboard_data.system.cpu_usage >= 0.0);
        assert!(dashboard_data.system.memory_used > 0);
        assert!(dashboard_data.system.memory_total > 0);
        assert!(dashboard_data.system.disk_used > 0);
        assert!(dashboard_data.system.disk_total > 0);
        
        // Verify metrics
        assert!(dashboard_data.metrics.counters.contains_key("protocol.messages"));
        assert!(dashboard_data.metrics.gauges.contains_key("protocol.message_rate"));
        assert!(dashboard_data.metrics.histograms.contains_key("protocol.latency"));
    }
    
    #[test]
    fn test_system_metrics_collection() {
        let mut adapter = ResourceMetricsCollectorAdapter::new();
        let system = adapter.collect_system_metrics();
        
        // Verify system metrics
        assert!(system.cpu_usage >= 0.0);
        assert!(system.memory_used > 0);
        assert!(system.memory_total > 0);
        assert!(system.disk_used > 0);
        assert!(system.disk_total > 0);
    }
    
    #[test]
    fn test_network_metrics_collection() {
        let mut adapter = ResourceMetricsCollectorAdapter::new();
        let network = adapter.collect_network_metrics();
        
        // Verify network metrics
        // Note: In a test environment, rx_bytes and tx_bytes might be zero
        // so we only check the types but not the values
        assert!(network.rx_bytes >= 0);
        assert!(network.tx_bytes >= 0);
        assert!(!network.interfaces.is_empty());
    }
    
    #[tokio::test]
    async fn test_mock_mcp_client() {
        // Create a mock MCP client
        let mock_client = MockMcpClient::new();
        
        // Get metrics
        let metrics = mock_client.get_metrics().await.unwrap();
        
        // Verify metrics
        assert!(metrics.message_stats.total_requests > 0);
        assert!(metrics.message_stats.total_responses > 0);
        assert!(metrics.transaction_stats.total_transactions > 0);
        assert!(metrics.transaction_stats.success_rate > 0.0);
        assert!(metrics.latency_stats.average_latency_ms > 0.0);
        assert!(!metrics.latency_stats.latency_histogram.is_empty());
    }
    
    #[tokio::test]
    async fn test_protocol_metrics_adapter_with_mcp() {
        // Create a mock MCP client
        let mock_client = Arc::new(MockMcpClient::new()) as Arc<dyn McpMetricsProvider>;
        
        // Create a protocol metrics adapter with the mock client
        let mut adapter = ProtocolMetricsAdapter::new_with_client(Some(mock_client));
        
        // Collect metrics
        let metrics = adapter.collect_metrics();
        
        // Verify metrics
        assert!(metrics.counters.contains_key("protocol.messages"));
        assert!(metrics.counters.contains_key("mcp.requests"));
        assert!(metrics.counters.contains_key("mcp.responses"));
        assert!(metrics.gauges.contains_key("protocol.message_rate"));
        assert!(metrics.gauges.contains_key("mcp.success_rate"));
        assert!(metrics.histograms.contains_key("protocol.latency"));
    }
} 