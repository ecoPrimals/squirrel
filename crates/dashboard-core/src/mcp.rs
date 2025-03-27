use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::data::{Alert, Metrics, ProtocolData};

/// Error type for MCP client operations
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    /// Data error
    #[error("Data error: {0}")]
    DataError(String),
    
    /// API error
    #[error("API error: {0}")]
    ApiError(String),
    
    /// Timeout error
    #[error("Timeout error")]
    TimeoutError,
}

/// Result type for MCP client operations
pub type McpResult<T> = Result<T, McpError>;

/// MCP client interface
#[async_trait]
pub trait McpClient {
    /// Get metrics from MCP
    async fn get_metrics(&mut self) -> McpResult<Metrics>;
    
    /// Get protocol data from MCP
    async fn get_protocol_data(&mut self) -> McpResult<ProtocolData>;
    
    /// Get alerts from MCP
    async fn get_alerts(&mut self) -> McpResult<Vec<Alert>>;
    
    /// Acknowledge an alert
    async fn acknowledge_alert(&mut self, alert_id: &str, by: &str) -> McpResult<()>;
    
    /// Set whether the client should fail (for testing)
    fn set_should_fail(&mut self, should_fail: bool);
    
    /// Check if the client is connected
    fn is_connected(&self) -> bool;
}

/// Mock MCP client for testing
pub struct MockMcpClient {
    /// Whether the client should fail
    should_fail: bool,
    
    /// Whether the client is connected
    connected: bool,
    
    /// Mock metrics
    metrics: Metrics,
    
    /// Mock protocol data
    protocol_data: ProtocolData,
    
    /// Mock alerts
    alerts: Vec<Alert>,
    
    /// Counter for generating data
    counter: u64,
    
    /// Last update time
    last_update: DateTime<Utc>,
    
    /// Whether to simulate data
    simulate_data: bool,
}

impl Default for MockMcpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockMcpClient {
    /// Create a new mock MCP client
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        
        // Initialize with random metrics
        let mut metrics = Metrics::default();
        metrics.cpu.usage = rng.gen_range(5.0..25.0);
        metrics.cpu.cores = vec![
            rng.gen_range(0.0..100.0),
            rng.gen_range(0.0..100.0),
            rng.gen_range(0.0..100.0),
            rng.gen_range(0.0..100.0),
        ];
        metrics.cpu.temperature = Some(rng.gen_range(35.0..45.0));
        metrics.cpu.load = [
            rng.gen_range(0.1..2.0),
            rng.gen_range(0.1..1.5),
            rng.gen_range(0.1..1.0),
        ];
        
        // Memory metrics
        metrics.memory.total = 16 * 1024 * 1024 * 1024; // 16 GB
        metrics.memory.used = rng.gen_range(4..8) * 1024 * 1024 * 1024;
        metrics.memory.free = metrics.memory.total - metrics.memory.used;
        metrics.memory.available = metrics.memory.free + rng.gen_range(1..3) * 1024 * 1024 * 1024;
        metrics.memory.swap_total = 8 * 1024 * 1024 * 1024; // 8 GB
        metrics.memory.swap_used = rng.gen_range(0..2) * 1024 * 1024 * 1024;
        
        // Initialize protocol data
        let mut protocol_data = ProtocolData::default();
        protocol_data.status = "Connected".to_string();
        protocol_data.version = "1.0.0".to_string();
        protocol_data.connected = true;
        protocol_data.last_connected = Some(Utc::now());
        protocol_data.retry_count = 0;
        
        // Add some metrics
        protocol_data.metrics.insert("packets_sent".to_string(), "1024".to_string());
        protocol_data.metrics.insert("packets_received".to_string(), "2048".to_string());
        protocol_data.metrics.insert("errors".to_string(), "0".to_string());
        protocol_data.metrics.insert("latency_ms".to_string(), "15".to_string());
        
        // Add some simulated status
        protocol_data.metrics.insert("simulated".to_string(), "true".to_string());
        protocol_data.metrics.insert("last_real_data".to_string(), 
            (Utc::now() - chrono::Duration::hours(2)).to_rfc3339());
            
        // Add some data
        protocol_data.data.insert("mode".to_string(), "standard".to_string());
        protocol_data.data.insert("encryption".to_string(), "enabled".to_string());
        protocol_data.data.insert("compression".to_string(), "enabled".to_string());
        
        // Initialize with some mock alerts
        let alerts = vec![
            Alert::new("system-001", "warning", "System memory usage high")
                .with_details("Memory usage has exceeded 80% threshold"),
            Alert::new("network-001", "info", "Network throughput nominal")
                .with_details("Network throughput is within expected ranges"),
        ];
        
        Self {
            should_fail: false,
            connected: true,
            metrics,
            protocol_data,
            alerts,
            counter: 0,
            last_update: Utc::now(),
            simulate_data: true,
        }
    }
    
    /// Update mock data with slightly changed values
    fn update_mock_data(&mut self) {
        let mut rng = rand::thread_rng();
        self.counter += 1;
        
        // Update CPU metrics with slight variations
        self.metrics.cpu.usage += rng.gen_range(-2.0..2.0);
        self.metrics.cpu.usage = self.metrics.cpu.usage.max(0.0).min(100.0);
        
        for core in &mut self.metrics.cpu.cores {
            *core += rng.gen_range(-5.0..5.0);
            *core = core.max(0.0).min(100.0);
        }
        
        if let Some(temp) = &mut self.metrics.cpu.temperature {
            *temp += rng.gen_range(-0.5..0.5);
        }
        
        // Update memory metrics
        let memory_change = rng.gen_range(-100..100) * 1024 * 1024;
        self.metrics.memory.used = (self.metrics.memory.used as i64 + memory_change)
            .max(0)
            .min(self.metrics.memory.total as i64) as u64;
        self.metrics.memory.free = self.metrics.memory.total - self.metrics.memory.used;
        
        // Update protocol metrics
        let packets_sent = self.protocol_data.metrics.get("packets_sent")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        self.protocol_data.metrics.insert(
            "packets_sent".to_string(), 
            (packets_sent + rng.gen_range(50..150)).to_string()
        );
        
        let packets_received = self.protocol_data.metrics.get("packets_received")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        self.protocol_data.metrics.insert(
            "packets_received".to_string(), 
            (packets_received + rng.gen_range(80..200)).to_string()
        );
        
        let latency = rng.gen_range(10..30);
        self.protocol_data.metrics.insert("latency_ms".to_string(), latency.to_string());
        
        // Occasionally add a new alert
        if rng.gen_ratio(1, 10) {
            let id = format!("alert-{:03}", self.counter);
            let severity = match rng.gen_range(0..3) {
                0 => "info",
                1 => "warning",
                _ => "critical",
            };
            
            let message = match severity {
                "critical" => "Critical system resource shortage",
                "warning" => "System performance degrading",
                _ => "System information notice",
            };
            
            let details = match severity {
                "critical" => "A critical resource threshold has been crossed requiring immediate attention",
                "warning" => "System performance metrics indicate possible degradation of service",
                _ => "Routine system notification for informational purposes",
            };
            
            self.alerts.push(Alert::new(&id, severity, message).with_details(details));
            
            // Keep alert list to a reasonable size
            if self.alerts.len() > 10 {
                self.alerts.remove(0);
            }
        }
        
        // Update timestamps
        self.last_update = Utc::now();
    }
}

#[async_trait]
impl McpClient for MockMcpClient {
    async fn get_metrics(&mut self) -> McpResult<Metrics> {
        if self.should_fail {
            return Err(McpError::ConnectionError("Simulated connection error".to_string()));
        }
        
        self.update_mock_data();
        Ok(self.metrics.clone())
    }
    
    async fn get_protocol_data(&mut self) -> McpResult<ProtocolData> {
        if self.should_fail {
            return Err(McpError::ConnectionError("Simulated connection error".to_string()));
        }
        
        // Update protocol data
        self.protocol_data.last_connected = Some(Utc::now());
        
        Ok(self.protocol_data.clone())
    }
    
    async fn get_alerts(&mut self) -> McpResult<Vec<Alert>> {
        if self.should_fail {
            return Err(McpError::ConnectionError("Simulated connection error".to_string()));
        }
        
        Ok(self.alerts.clone())
    }
    
    async fn acknowledge_alert(&mut self, alert_id: &str, by: &str) -> McpResult<()> {
        if self.should_fail {
            return Err(McpError::ConnectionError("Simulated connection error".to_string()));
        }
        
        for alert in &mut self.alerts {
            if alert.id == alert_id {
                alert.acknowledge(by);
                return Ok(());
            }
        }
        
        Err(McpError::DataError(format!("Alert with ID {} not found", alert_id)))
    }
    
    fn set_should_fail(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
        
        if should_fail {
            self.connected = false;
            self.protocol_data.connected = false;
            self.protocol_data.status = "Disconnected".to_string();
            self.protocol_data.error = Some("Simulated connection error".to_string());
        } else {
            self.connected = true;
            self.protocol_data.connected = true;
            self.protocol_data.status = "Connected".to_string();
            self.protocol_data.error = None;
        }
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// Start mock client with automatic data updates for testing
pub async fn start_mock_client_with_auto_updates(update_interval: Duration) -> Arc<Mutex<MockMcpClient>> {
    let client = Arc::new(Mutex::new(MockMcpClient::new()));
    
    // Clone for the update task
    let client_clone = client.clone();
    
    // Spawn update task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(update_interval);
        
        loop {
            interval.tick().await;
            
            // Lock and update data
            let mut client = client_clone.lock().await;
            client.update_mock_data();
        }
    });
    
    client
} 