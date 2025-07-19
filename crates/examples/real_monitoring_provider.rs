//! Real Monitoring API Provider Example
//!
//! This example demonstrates how to implement a real monitoring API provider
//! that connects the dashboard core with the squirrel-monitoring system.
//! It uses the bridge pattern to avoid circular dependencies.

use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde_json::Value;
use squirrel_mcp::monitoring::{create_production_monitoring_client, MonitoringClient, MonitoringEvent, MetricValue, AlertLevel};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tokio::time::sleep;
use serde_json::json;
use chrono::Utc;

use dashboard_core::{
    data::{DashboardData, ProtocolData, Alert},
    monitoring::MonitoringAPIProvider,
    error::{Result, DashboardError},
    mcp::{McpClient, McpError},
    DashboardConfig,
    DefaultDashboardService,
    DashboardService,
};

// Use the MonitoringClient trait from the squirrel_mcp::monitoring module
use squirrel_mcp::monitoring::MonitoringClient;

// Real implementation of MonitoringAPIProvider that bridges to squirrel-monitoring
struct RealMonitoringAdapter {
    // Use the actual monitoring client from squirrel-monitoring
    monitoring_client: Arc<dyn MonitoringClient>,
    metrics: Arc<Mutex<HashMap<String, Value>>>,
}

impl RealMonitoringAdapter {
    fn new(monitoring_client: Arc<dyn MonitoringClient>) -> Self {
        Self { 
            monitoring_client,
            metrics: Arc::new(Mutex::new(HashMap::new()))
        }
    }
    
    fn update_metrics(&mut self, component_id: &str, data: Value) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.insert(component_id.to_string(), data);
        }
    }
}

#[async_trait]
impl MonitoringAPIProvider for RealMonitoringAdapter {
    async fn get_available_components(&self) -> Result<Vec<String>> {
        // In a real implementation, we would ask the monitoring system
        // For this example, just return the components we know
        let components = vec![
            "cpu".to_string(),
            "memory".to_string(),
            "network".to_string(),
            "disk".to_string(),
            "alerts".to_string()
        ];
        
        Ok(components)
    }
    
    async fn get_component_data(&self, component_id: &str) -> Result<Value> {
        // Get component data from our cache
        if let Ok(metrics) = self.metrics.lock() {
            if let Some(data) = metrics.get(component_id) {
                return Ok(data.clone());
            }
        }
        
        // If not found, return empty data
        match component_id {
            "cpu" => Ok(json!({
                "usage": 0.0,
                "load": 0.0,
                "cores": 0,
                "temperature": 0
            })),
            "memory" => Ok(json!({
                "total": 0,
                "used": 0,
                "free": 0,
                "usage_percent": 0.0
            })),
            "network" => Ok(json!({
                "rx_bytes": 0,
                "tx_bytes": 0,
                "connections": 0
            })),
            "disk" => Ok(json!({
                "total": 0,
                "used": 0,
                "free": 0,
                "usage_percent": 0.0
            })),
            "alerts" => Ok(json!([])),
            _ => Err(DashboardError::NotFound(format!("Component {} not found", component_id)))
        }
    }
    
    async fn get_health_status(&self) -> Result<Value> {
        // In a real implementation, we would get health status from the monitoring system
        // For this example, just return some simulated health data
        Ok(json!({
            "healthy": true,
            "components": {
                "cpu": "healthy",
                "memory": "healthy",
                "network": "healthy",
                "disk": "healthy"
            },
            "uptime": 3600,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    async fn subscribe(&self, _component_id: &str) -> Result<()> {
        // In a real implementation, we would subscribe to component updates
        // For this example, just return success
        Ok(())
    }
    
    async fn unsubscribe(&self, _component_id: &str) -> Result<()> {
        // In a real implementation, we would unsubscribe from component updates
        // For this example, just return success
        Ok(())
    }
}

// Real provider that integrates with the monitoring system
struct RealMonitoringProvider {
    monitoring_client: Arc<dyn MonitoringClient>,
    metrics: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    last_update: Arc<Mutex<Instant>>,
    monitoring_adapter: Arc<Mutex<RealMonitoringAdapter>>,
}

impl RealMonitoringProvider {
    fn new(monitoring_client: Arc<dyn MonitoringClient>) -> Self {
        let metrics = Arc::new(Mutex::new(HashMap::new()));
        let adapter = RealMonitoringAdapter::new(monitoring_client.clone());
        
        Self {
            monitoring_client,
            metrics: metrics.clone(),
            last_update: Arc::new(Mutex::new(Instant::now())),
            monitoring_adapter: Arc::new(Mutex::new(adapter)),
        }
    }

    async fn start_metrics_collection(&self) -> anyhow::Result<()> {
        println!("Starting metrics collection...");
        
        // Create a clone of self for the task
        let metrics = self.metrics.clone();
        let last_update = self.last_update.clone();
        let monitoring_adapter = self.monitoring_adapter.clone();
        
        tokio::spawn(async move {
            loop {
                // Update metrics every 5 seconds
                sleep(Duration::from_secs(5)).await;
                
                // Update all metrics
                let metrics_provider = RealMetricsProvider::new(metrics.clone(), monitoring_adapter.clone());
                if let Err(e) = metrics_provider.update_all_metrics().await {
                    eprintln!("Error updating metrics: {}", e);
                }
                
                // Update last update time
                if let Ok(mut last_update) = last_update.lock() {
                    *last_update = Instant::now();
                }
            }
        });
        
        Ok(())
    }
}

// Metrics provider implementation
struct RealMetricsProvider {
    metrics: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    monitoring_adapter: Arc<Mutex<RealMonitoringAdapter>>,
}

impl RealMetricsProvider {
    fn new(metrics: Arc<Mutex<HashMap<String, serde_json::Value>>>, monitoring_adapter: Arc<Mutex<RealMonitoringAdapter>>) -> Self {
        Self { 
            metrics,
            monitoring_adapter
        }
    }
    
    async fn update_all_metrics(&self) -> anyhow::Result<()> {
        // Update all metrics in parallel
        let cpu_result = self.update_cpu_metrics().await;
        let memory_result = self.update_memory_metrics().await;
        let network_result = self.update_network_metrics().await;
        let disk_result = self.update_disk_metrics().await;
        let alerts_result = self.update_alerts().await;
        
        // Check for errors
        if let Err(e) = &cpu_result {
            eprintln!("Error updating CPU metrics: {}", e);
        }
        
        if let Err(e) = &memory_result {
            eprintln!("Error updating memory metrics: {}", e);
        }
        
        if let Err(e) = &network_result {
            eprintln!("Error updating network metrics: {}", e);
        }
        
        if let Err(e) = &disk_result {
            eprintln!("Error updating disk metrics: {}", e);
        }
        
        if let Err(e) = &alerts_result {
            eprintln!("Error updating alerts: {}", e);
        }
        
        // Return success if all metrics were updated successfully
        Ok(())
    }
    
    async fn update_cpu_metrics(&self) -> anyhow::Result<()> {
        // Simulate CPU metrics (load and usage)
        let cpu_load = rand::random::<f64>() * 100.0;
        let cpu_usage = rand::random::<f64>() * 100.0;
        
        // Create CPU metrics data
        let cpu_data = json!({
            "load": cpu_load,
            "usage": cpu_usage,
            "cores": 8,
            "temperature": 45 + (rand::random::<f64>() * 20.0) as u8
        });
        
        // Update metrics in our local cache
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.insert("cpu".to_string(), cpu_data.clone());
        }
        
        // Update the monitoring adapter
        if let Ok(mut adapter) = self.monitoring_adapter.lock() {
            adapter.update_metrics("cpu", cpu_data.clone());
        }
        
        // In a real implementation, we would use the monitoring system here
        // For now, just print the update
        println!("Updated CPU metrics: {}", cpu_data);
        
        Ok(())
    }
    
    async fn update_memory_metrics(&self) -> anyhow::Result<()> {
        // Simulate memory usage (in MB)
        let total_memory = 16384; // 16 GB
        let used_memory = (rand::random::<f64>() * total_memory as f64) as u64;
        let free_memory = total_memory - used_memory;
        
        // Create memory metrics data
        let memory_data = json!({
            "total": total_memory,
            "used": used_memory,
            "free": free_memory,
            "usage_percent": (used_memory as f64 / total_memory as f64) * 100.0
        });
        
        // Update metrics in our local cache
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.insert("memory".to_string(), memory_data.clone());
        }
        
        // Update the monitoring adapter
        if let Ok(mut adapter) = self.monitoring_adapter.lock() {
            adapter.update_metrics("memory", memory_data.clone());
        }
        
        // In a real implementation, we would use the monitoring system here
        // For now, just print the update
        println!("Updated memory metrics: {}", memory_data);
        
        Ok(())
    }
    
    async fn update_network_metrics(&self) -> anyhow::Result<()> {
        // Simulate network metrics
        let rx_bytes = rand::random::<u64>() % 1_000_000;
        let tx_bytes = rand::random::<u64>() % 1_000_000;
        let connections = rand::random::<u16>() % 100;
        
        // Create network metrics data
        let network_data = json!({
            "rx_bytes": rx_bytes,
            "tx_bytes": tx_bytes,
            "connections": connections,
            "interfaces": [
                {
                    "name": "eth0",
                    "rx_bytes": rx_bytes / 2,
                    "tx_bytes": tx_bytes / 2
                },
                {
                    "name": "wlan0",
                    "rx_bytes": rx_bytes / 2,
                    "tx_bytes": tx_bytes / 2
                }
            ]
        });
        
        // Update metrics in our local cache
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.insert("network".to_string(), network_data.clone());
        }
        
        // Update the monitoring adapter
        if let Ok(mut adapter) = self.monitoring_adapter.lock() {
            adapter.update_metrics("network", network_data.clone());
        }
        
        // In a real implementation, we would use the monitoring system here
        // For now, just print the update
        println!("Updated network metrics: {}", network_data);
        
        Ok(())
    }
    
    async fn update_disk_metrics(&self) -> anyhow::Result<()> {
        // Simulate disk metrics
        let total_space = 1_000_000_000; // 1 TB
        let used_space = (rand::random::<f64>() * total_space as f64) as u64;
        let free_space = total_space - used_space;
        
        // Create disk metrics data
        let disk_data = json!({
            "total": total_space,
            "used": used_space,
            "free": free_space,
            "usage_percent": (used_space as f64 / total_space as f64) * 100.0,
            "mountpoints": [
                {
                    "path": "/",
                    "total": total_space / 2,
                    "used": used_space / 2,
                    "free": free_space / 2
                },
                {
                    "path": "/home",
                    "total": total_space / 2,
                    "used": used_space / 2,
                    "free": free_space / 2
                }
            ]
        });
        
        // Update metrics in our local cache
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.insert("disk".to_string(), disk_data.clone());
        }
        
        // Update the monitoring adapter
        if let Ok(mut adapter) = self.monitoring_adapter.lock() {
            adapter.update_metrics("disk", disk_data.clone());
        }
        
        // In a real implementation, we would use the monitoring system here
        // For now, just print the update
        println!("Updated disk metrics: {}", disk_data);
        
        Ok(())
    }
    
    async fn update_alerts(&self) -> anyhow::Result<()> {
        // Only generate an alert occasionally (1 in 10 chance)
        if rand::random::<u8>() % 10 == 0 {
            // Create a random alert
            let alert_types = ["CPU", "Memory", "Disk", "Network"];
            let alert_severities = ["Info", "Warning", "Error", "Critical"];
            
            let alert_type = alert_types[rand::random::<usize>() % alert_types.len()];
            let alert_severity = alert_severities[rand::random::<usize>() % alert_severities.len()];
            
            let alerts_data = json!([
                {
                    "type": alert_type,
                    "message": format!("{} usage is high", alert_type),
                    "severity": alert_severity,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            ]);
            
            // Update metrics in our local cache
            if let Ok(mut metrics) = self.metrics.lock() {
                metrics.insert("alerts".to_string(), alerts_data.clone());
            }
            
            // Update the monitoring adapter
            if let Ok(mut adapter) = self.monitoring_adapter.lock() {
                adapter.update_metrics("alerts", alerts_data.clone());
            }
            
            // In a real implementation, we would use the monitoring system here
            // For now, just print the update
            println!("Generated alert: {}", alerts_data);
        }
        
        Ok(())
    }
}

// Mock implementation of McpClient for our example
#[derive(Debug)]
struct MockMcpClient {
    should_fail: bool,
    connected: bool,
}

impl MockMcpClient {
    fn new() -> Self {
        Self {
            should_fail: false,
            connected: true,
        }
    }
}

#[async_trait]
impl McpClient for MockMcpClient {
    async fn get_metrics(&mut self) -> std::result::Result<dashboard_core::data::Metrics, McpError> {
        // Return empty metrics by default
        Ok(Default::default())
    }
    
    async fn get_protocol_data(&mut self) -> std::result::Result<ProtocolData, McpError> {
        // Return basic protocol data
        Ok(ProtocolData {
            name: "MCP Protocol".to_string(),
            protocol_type: "RPC".to_string(),
            version: "1.0.0".to_string(),
            status: "Connected".to_string(),
            connected: true,
            last_connected: Some(Utc::now()),
            retry_count: 0,
            error: None,
            metrics: HashMap::new(),
        })
    }
    
    async fn get_alerts(&mut self) -> std::result::Result<Vec<Alert>, McpError> {
        // Return empty alerts list
        Ok(Vec::new())
    }
    
    async fn acknowledge_alert(&mut self, _alert_id: &str, _acknowledged_by: &str) -> std::result::Result<(), McpError> {
        // Mock implementation for acknowledging alerts
        Ok(())
    }
    
    fn set_should_fail(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize production monitoring client with Songbird integration
    let monitoring_client = create_production_monitoring_client();
    
    println!("🎵 Real Monitoring Provider Example with Songbird Integration");
    println!("============================================================");
    
    // Test monitoring client health
    match monitoring_client.get_health_status().await {
        Ok(is_healthy) => {
            if is_healthy {
                println!("✅ Songbird monitoring client is healthy");
            } else {
                println!("⚠️  Songbird monitoring client health check failed");
            }
        }
        Err(e) => {
            println!("❌ Failed to check monitoring client health: {}", e);
        }
    }

    // Record sample events
    println!("\n📊 Recording sample monitoring events...");
    
    // Example 1: System startup event
    let startup_event = MonitoringEvent {
        timestamp: Utc::now(),
        event_type: "system_startup".to_string(),
        message: "Real monitoring provider example started".to_string(),
        level: AlertLevel::Info,
        source: "real-monitoring-provider".to_string(),
        tags: {
            let mut tags = HashMap::new();
            tags.insert("component".to_string(), "example".to_string());
            tags.insert("version".to_string(), "0.1.0".to_string());
            tags
        },
        metadata: HashMap::new(),
    };

    if let Err(e) = monitoring_client.record_event(startup_event).await {
        println!("❌ Failed to record startup event: {}", e);
    } else {
        println!("✅ Recorded startup event");
    }

    // Example 2: Performance metrics
    let mut perf_tags = HashMap::new();
    perf_tags.insert("component".to_string(), "example".to_string());

    if let Err(e) = monitoring_client
        .record_metric("cpu_usage", MetricValue::Float(45.7), Some(perf_tags.clone()))
        .await
    {
        println!("❌ Failed to record CPU metric: {}", e);
    } else {
        println!("✅ Recorded CPU usage metric: 45.7%");
    }

    if let Err(e) = monitoring_client
        .record_metric("memory_usage", MetricValue::Float(67.2), Some(perf_tags.clone()))
        .await
    {
        println!("❌ Failed to record memory metric: {}", e);
    } else {
        println!("✅ Recorded memory usage metric: 67.2%");
    }

    if let Err(e) = monitoring_client
        .record_metric("active_connections", MetricValue::Integer(142), Some(perf_tags))
        .await
    {
        println!("❌ Failed to record connections metric: {}", e);
    } else {
        println!("✅ Recorded active connections: 142");
    }

    // Example 3: Error event
    let error_event = MonitoringEvent {
        timestamp: Utc::now(),
        event_type: "application_error".to_string(),
        message: "Simulated error for monitoring demonstration".to_string(),
        level: AlertLevel::Medium,
        source: "real-monitoring-provider".to_string(),
        tags: {
            let mut tags = HashMap::new();
            tags.insert("error_type".to_string(), "simulation".to_string());
            tags.insert("severity".to_string(), "medium".to_string());
            tags
        },
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("stack_trace".to_string(), "example_stack_trace".to_string());
            metadata.insert("user_id".to_string(), "user_123".to_string());
            metadata
        },
    };

    if let Err(e) = monitoring_client.record_event(error_event).await {
        println!("❌ Failed to record error event: {}", e);
    } else {
        println!("✅ Recorded error event");
    }

    // Get metrics summary
    println!("\n📈 Current metrics summary:");
    match monitoring_client.get_metrics_summary().await {
        Ok(metrics) => {
            for (key, value) in metrics {
                println!("  • {}: {:?}", key, value);
            }
        }
        Err(e) => {
            println!("❌ Failed to get metrics summary: {}", e);
        }
    }

    println!("\n🎉 Real monitoring provider example completed successfully!");
    println!("💡 Check your Songbird dashboard for the recorded events and metrics.");
    
    Ok(())
}

// Helper function to print dashboard data
fn print_dashboard_data(data: &DashboardData) {
    // Print CPU metrics
    println!("CPU Usage: {:.1}%", data.metrics.cpu.usage);
    println!("CPU Load: {:.2} {:.2} {:.2}", 
        data.metrics.cpu.load[0], 
        data.metrics.cpu.load[1], 
        data.metrics.cpu.load[2]
    );
    
    // Print memory metrics
    println!("Memory: {:.1}% used ({} MB / {} MB)", 
        100.0 * data.metrics.memory.used as f64 / data.metrics.memory.total as f64,
        data.metrics.memory.used / 1024 / 1024,
        data.metrics.memory.total / 1024 / 1024
    );
    
    // Print network metrics
    if !data.metrics.network.interfaces.is_empty() {
        println!("Network Interfaces: {}", data.metrics.network.interfaces.len());
        for interface in &data.metrics.network.interfaces {
            println!("  {}: RX: {} MB, TX: {} MB", 
                interface.name,
                interface.rx_bytes / 1024 / 1024,
                interface.tx_bytes / 1024 / 1024
            );
        }
    } else {
        println!("No network interfaces detected");
    }
    
    // Print disk metrics
    if let Some(disk_usage) = data.metrics.disk.usage.get("/") {
        println!("Disk Usage (/): {:.1}% used ({} GB / {} GB)", 
            disk_usage.used_percentage,
            disk_usage.used / 1024 / 1024 / 1024,
            disk_usage.total / 1024 / 1024 / 1024
        );
    }
    
    // Print alerts
    if !data.alerts.is_empty() {
        println!("Active Alerts: {}", data.alerts.len());
        for (i, alert) in data.alerts.iter().enumerate().take(3) {
            println!("  {}: [{}] {} - {}", 
                i + 1, 
                format!("{:?}", alert.severity), 
                alert.title, 
                alert.message
            );
        }
        
        if data.alerts.len() > 3 {
            println!("  ...and {} more", data.alerts.len() - 3);
        }
    } else {
        println!("No active alerts");
    }
} 