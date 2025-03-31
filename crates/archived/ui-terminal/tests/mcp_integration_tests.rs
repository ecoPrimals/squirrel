use std::collections::HashMap;
use chrono::Utc;
use async_trait::async_trait;
use tokio::sync::Mutex;

use ui_terminal::adapter::{
    ConnectionEvent, ConnectionEventType, ConnectionHealth, ConnectionStatus, 
    McpMetrics, McpMetricsProviderTrait, PerformanceMetrics, PerformanceOptions
};

// Test implementation of the McpMetricsProviderTrait
struct TestProvider {
    metrics: Mutex<McpMetrics>,
    connection_status: Mutex<ConnectionStatus>,
    connection_health: Mutex<ConnectionHealth>,
    connection_history: Mutex<Vec<ConnectionEvent>>,
    message_log: Mutex<Vec<String>>,
    error_log: Mutex<Vec<String>>,
    protocol_metrics: Mutex<HashMap<String, f64>>,
    performance_metrics: Mutex<PerformanceMetrics>,
    should_fail: Mutex<bool>,
}

impl TestProvider {
    fn new() -> Self {
        Self {
            metrics: Mutex::new(McpMetrics::default()),
            connection_status: Mutex::new(ConnectionStatus::Connected),
            connection_health: Mutex::new(ConnectionHealth {
                latency_ms: 25.0,
                packet_loss: 0.0,
                stability: 100.0,
                signal_strength: 100.0,
                last_checked: Utc::now(),
            }),
            connection_history: Mutex::new(vec![
                ConnectionEvent {
                    event_type: ConnectionEventType::Connected,
                    details: "Initial connection".to_string(),
                    timestamp: Utc::now(),
                }
            ]),
            message_log: Mutex::new(vec![
                "Connected to MCP server".to_string(),
                "Sent handshake".to_string(),
                "Received handshake response".to_string(),
            ]),
            error_log: Mutex::new(Vec::new()),
            protocol_metrics: Mutex::new(HashMap::new()),
            performance_metrics: Mutex::new(PerformanceMetrics::default()),
            should_fail: Mutex::new(false),
        }
    }
}

#[async_trait]
impl McpMetricsProviderTrait for TestProvider {
    async fn get_metrics(&self) -> Result<McpMetrics, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        let metrics = self.metrics.lock().await.clone();
        Ok(metrics)
    }
    
    async fn get_connection_status(&self) -> Result<ConnectionStatus, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        let status = self.connection_status.lock().await.clone();
        Ok(status)
    }
    
    async fn get_connection_health(&self) -> Result<ConnectionHealth, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        let health = self.connection_health.lock().await.clone();
        Ok(health)
    }
    
    async fn get_protocol_metrics(&self) -> Result<HashMap<String, f64>, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        let metrics = self.protocol_metrics.lock().await.clone();
        Ok(metrics)
    }
    
    async fn reconnect(&self) -> Result<bool, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        // Simulate reconnection by updating the connection status
        let mut status = self.connection_status.lock().await;
        *status = ConnectionStatus::Connected;
        
        let mut health = self.connection_health.lock().await;
        health.stability = 100.0;
        health.signal_strength = 100.0;
        health.packet_loss = 0.0;
        health.last_checked = Utc::now();
        
        // Add to connection history
        let mut history = self.connection_history.lock().await;
        history.push(ConnectionEvent {
            event_type: ConnectionEventType::Connected,
            details: "Reconnected".to_string(),
            timestamp: Utc::now(),
        });
        
        // Add to message log
        let mut log = self.message_log.lock().await;
        log.push("Reconnected to MCP server".to_string());
        
        Ok(true)
    }
    
    async fn get_connection_history(&self) -> Result<Vec<ConnectionEvent>, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        let history = self.connection_history.lock().await.clone();
        Ok(history)
    }
    
    async fn get_message_log(&self) -> Result<Vec<String>, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        let log = self.message_log.lock().await.clone();
        Ok(log)
    }
    
    async fn get_recent_errors(&self, limit: usize) -> Result<Vec<String>, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        let log = self.error_log.lock().await.clone();
        let start = if log.len() > limit { log.len() - limit } else { 0 };
        Ok(log[start..].to_vec())
    }
    
    async fn get_error_log(&self) -> Result<Vec<String>, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        let log = self.error_log.lock().await.clone();
        Ok(log)
    }
    
    async fn set_performance_options(&self, _options: PerformanceOptions) -> Result<(), String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        // Nothing to do for the test
        Ok(())
    }
    
    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, String> {
        if *self.should_fail.lock().await {
            return Err("Provider configured to fail".to_string());
        }
        
        let metrics = self.performance_metrics.lock().await.clone();
        Ok(metrics)
    }
    
    async fn set_should_fail(&self, should_fail: bool) {
        let mut fail = self.should_fail.lock().await;
        *fail = should_fail;
        
        if should_fail {
            // Update status
            let mut status = self.connection_status.lock().await;
            *status = ConnectionStatus::Error("Provider configured to fail".to_string());
            
            // Update health
            let mut health = self.connection_health.lock().await;
            health.stability = 0.0;
            health.signal_strength = 0.0;
            health.packet_loss = 100.0;
            health.last_checked = Utc::now();
        } else {
            // Update status back to connected
            let mut status = self.connection_status.lock().await;
            *status = ConnectionStatus::Connected;
            
            // Update health
            let mut health = self.connection_health.lock().await;
            health.stability = 100.0;
            health.signal_strength = 100.0;
            health.packet_loss = 0.0;
            health.last_checked = Utc::now();
        }
    }
}

#[tokio::test]
async fn test_metrics_provider_trait() {
    let provider = TestProvider::new();
    
    // Test metrics
    let metrics = provider.get_metrics().await.unwrap();
    assert_eq!(metrics.message_stats.total_requests, 0);
    
    // Test connection status
    let status = provider.get_connection_status().await.unwrap();
    assert!(matches!(status, ConnectionStatus::Connected));
    
    // Test connection health
    let health = provider.get_connection_health().await.unwrap();
    assert_eq!(health.latency_ms, 25.0);
    assert_eq!(health.stability, 100.0);
    assert_eq!(health.signal_strength, 100.0);
    assert_eq!(health.packet_loss, 0.0);
    
    // Test message log
    let log = provider.get_message_log().await.unwrap();
    assert_eq!(log.len(), 3);
    assert_eq!(log[0], "Connected to MCP server");
    
    // Test error log
    let errors = provider.get_error_log().await.unwrap();
    assert_eq!(errors.len(), 0);
    
    // Test reconnect
    let reconnect_result = provider.reconnect().await.unwrap();
    assert!(reconnect_result);
    
    // Test connection history
    let history = provider.get_connection_history().await.unwrap();
    assert_eq!(history.len(), 2);
    assert!(matches!(history[0].event_type, ConnectionEventType::Connected));
    
    // Test performance metrics
    let perf = provider.get_performance_metrics().await.unwrap();
    assert_eq!(perf.metrics_requests, 0);
    
    // Test failure mode
    provider.set_should_fail(true).await;
    let status_result = provider.get_connection_status().await;
    assert!(status_result.is_err());
    
    // Reset failure mode
    provider.set_should_fail(false).await;
    let status = provider.get_connection_status().await.unwrap();
    assert!(matches!(status, ConnectionStatus::Connected));
} 