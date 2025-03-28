use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::Mutex;
use async_trait::async_trait;

use dashboard_core::data::{
    DashboardData,
    Metrics, ProtocolData, CpuMetrics, 
    MemoryMetrics, NetworkMetrics, DiskMetrics,
    NetworkInterface, DiskUsage, MetricsHistory
};
use dashboard_core::DashboardService;

use crate::adapter::{
    ConnectionEvent, ConnectionEventType, ConnectionHealth, ConnectionStatus, McpMetrics, PerformanceMetrics,
    PerformanceOptions, McpMetricsProviderTrait, MessageStats,
    TransactionStats, ErrorStats, LatencyStats, MonitoringToDashboardAdapter
};

/// A simplified mock adapter for testing the UI
#[derive(Debug)]
pub struct MockAdapter {
    connection_status: Mutex<ConnectionStatus>,
    connection_health: Mutex<ConnectionHealth>,
    connection_history: Mutex<Vec<ConnectionEvent>>,
    error_log: Mutex<Vec<String>>,
    message_log: Mutex<Vec<String>>,
    performance_metrics: Mutex<PerformanceMetrics>,
    performance_options: Mutex<PerformanceOptions>,
    protocol_metrics: Mutex<HashMap<String, f64>>,
    should_fail: Mutex<bool>,
}

impl MockAdapter {
    pub fn new() -> Self {
        Self {
            connection_status: Mutex::new(ConnectionStatus::Connected),
            connection_health: Mutex::new(ConnectionHealth {
                status: ConnectionStatus::Connected,
                last_successful: Some(Utc::now()),
                failure_count: 0,
                latency_ms: Some(15),
                error_details: None,
            }),
            connection_history: Mutex::new(vec![
                ConnectionEvent {
                    event_type: ConnectionEventType::Connected,
                    details: "Initial connection".to_string(),
                    timestamp: Utc::now(),
                }
            ]),
            error_log: Mutex::new(Vec::new()),
            message_log: Mutex::new(Vec::new()),
            performance_metrics: Mutex::new(PerformanceMetrics::new()),
            performance_options: Mutex::new(PerformanceOptions::default()),
            protocol_metrics: Mutex::new(HashMap::new()),
            should_fail: Mutex::new(false),
        }
    }
    
    pub async fn set_connection_status(&self, status: ConnectionStatus) {
        let mut current = self.connection_status.lock().await;
        *current = status.clone();
        
        // Also update connection health
        let mut health = self.connection_health.lock().await;
        health.status = status;
        
        // Add to connection history
        let mut history = self.connection_history.lock().await;
        history.push(ConnectionEvent {
            event_type: ConnectionEventType::Connected, // Using Connected instead of StatusChanged
            details: format!("Status changed to {:?}", current),
            timestamp: Utc::now(),
        });
    }
    
    pub async fn get_dashboard_data(&self) -> Result<DashboardData, String> {
        // Check if we should fail
        let should_fail = *self.should_fail.lock().await;
        if should_fail {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        // Create a mock dashboard data
        let metrics = self.generate_mock_metrics().await;
        let protocol = self.generate_mock_protocol_data().await;
        
        let result = DashboardData {
            metrics,
            protocol,
            alerts: Vec::new(),
            timestamp: Utc::now(),
        };
        
        Ok(result)
    }
    
    async fn generate_mock_metrics(&self) -> Metrics {
        // Create mock CPU metrics
        let cpu = CpuMetrics {
            usage: 45.0,
            cores: vec![40.0, 50.0, 45.0, 50.0],
            temperature: Some(65.0),
            load: [1.0, 1.5, 2.0],
        };
        
        // Create mock memory metrics
        let memory = MemoryMetrics {
            total: 16_000_000_000,  // ~16GB
            used: 4_000_000_000,    // ~4GB
            available: 12_000_000_000,
            free: 12_000_000_000,
            swap_used: 500_000_000,
            swap_total: 8_000_000_000,
        };
        
        // Create mock network interfaces
        let interfaces = vec![
            NetworkInterface {
                name: "eth0".to_string(),
                is_up: true,
                rx_bytes: 1_000_000,
                tx_bytes: 400_000,
                rx_packets: 800,
                tx_packets: 400,
                rx_errors: 0,
                tx_errors: 0,
            }
        ];
        
        // Create mock network metrics
        let network = NetworkMetrics {
            interfaces,
            total_rx_bytes: 1_500_000,
            total_tx_bytes: 500_000,
            total_rx_packets: 1000,
            total_tx_packets: 500,
        };
        
        // Create mock disk usage
        let mut usage = HashMap::new();
        usage.insert("root".to_string(), DiskUsage {
            mount_point: "/".to_string(),
            total: 1_000_000_000_000,  // ~1TB
            used: 500_000_000_000,     // ~500GB
            free: 500_000_000_000,
            used_percentage: 50.0,
        });
        
        // Create mock disk metrics
        let disk = DiskMetrics {
            usage,
            total_reads: 1000,
            total_writes: 500,
            read_bytes: 2_000_000,
            written_bytes: 1_000_000,
        };
        
        // Create mock metrics history
        let history = MetricsHistory::default();
        
        // Return properly populated metrics
        Metrics {
            cpu,
            memory,
            network,
            disk,
            history,
        }
    }
    
    async fn generate_mock_protocol_data(&self) -> ProtocolData {
        let connection_status = self.connection_status.lock().await;
        
        let connected = matches!(connection_status.clone(), ConnectionStatus::Connected);
        let status_string = connection_status.to_string();
        
        // Release the lock before other operations
        drop(connection_status);
        
        let metrics = self.protocol_metrics.lock().await.clone();
        
        ProtocolData {
            name: "MCP".to_string(),
            protocol_type: "TCP".to_string(),
            version: "1.0".to_string(),
            connected,
            last_connected: Some(Utc::now()),
            status: status_string,
            error: None,
            retry_count: 0,
            metrics,
        }
    }
}

impl Default for MockAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Function to create a mock adapter for testing
pub fn create_mock_adapter() -> Arc<MockAdapter> {
    Arc::new(MockAdapter::new())
}

#[async_trait]
impl McpMetricsProviderTrait for MockAdapter {
    async fn get_metrics(&self) -> Result<McpMetrics, String> {
        // Check if we should fail
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        // Generate mock metrics
        Ok(McpMetrics {
            message_stats: MessageStats {
                total_requests: 1000,
                total_responses: 980,
                request_rate: 10.5,
                response_rate: 10.2,
                request_types: {
                    let mut types = HashMap::new();
                    types.insert("GET".to_string(), 600);
                    types.insert("POST".to_string(), 300);
                    types.insert("PUT".to_string(), 100);
                    types
                },
            },
            transaction_stats: TransactionStats {
                total_transactions: 500,
                successful_transactions: 490,
                failed_transactions: 10,
                transaction_rate: 5.0,
                success_rate: 98.0,
            },
            error_stats: ErrorStats {
                total_errors: 20,
                connection_errors: 5,
                protocol_errors: 10,
                timeout_errors: 5,
                error_rate: 2.0,
                error_types: {
                    let mut errors = HashMap::new();
                    errors.insert("Timeout".to_string(), 5);
                    errors.insert("Connection".to_string(), 5);
                    errors.insert("Protocol".to_string(), 10);
                    errors
                },
            },
            latency_stats: LatencyStats {
                average_latency_ms: 15.5,
                median_latency_ms: 12.0,
                p95_latency_ms: 30.0,
                p99_latency_ms: 50.0,
                min_latency_ms: 5.0,
                max_latency_ms: 100.0,
                latency_histogram: vec![5.0, 10.0, 15.0, 20.0, 30.0, 50.0, 100.0],
            },
            timestamp: Utc::now(),
        })
    }
    
    async fn get_connection_status(&self) -> Result<ConnectionStatus, String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        Ok(self.connection_status.lock().await.clone())
    }
    
    async fn get_connection_health(&self) -> Result<ConnectionHealth, String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        Ok(self.connection_health.lock().await.clone())
    }
    
    async fn get_protocol_metrics(&self) -> Result<HashMap<String, f64>, String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        Ok(self.protocol_metrics.lock().await.clone())
    }
    
    async fn get_connection_history(&self) -> Result<Vec<ConnectionEvent>, String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        Ok(self.connection_history.lock().await.clone())
    }
    
    async fn get_message_log(&self) -> Result<Vec<String>, String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        Ok(self.message_log.lock().await.clone())
    }
    
    async fn get_recent_errors(&self, _limit: usize) -> Result<Vec<String>, String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        Ok(self.error_log.lock().await.clone())
    }
    
    async fn get_error_log(&self) -> Result<Vec<String>, String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        Ok(self.error_log.lock().await.clone())
    }
    
    async fn set_performance_options(&self, options: PerformanceOptions) -> Result<(), String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        *self.performance_options.lock().await = options;
        
        Ok(())
    }
    
    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        Ok(self.performance_metrics.lock().await.clone())
    }
    
    async fn set_should_fail(&self, should_fail: bool) {
        let mut current = self.should_fail.lock().await;
        *current = should_fail;
        
        if should_fail {
            // Update connection status and health
            let mut status = self.connection_status.lock().await;
            *status = ConnectionStatus::Error("Mock failure".to_string());
            
            let mut health = self.connection_health.lock().await;
            health.status = ConnectionStatus::Error("Mock failure".to_string());
            health.failure_count += 1;
            health.error_details = Some("Mock failure".to_string());
            
            // Add to connection history
            let mut history = self.connection_history.lock().await;
            history.push(ConnectionEvent {
                event_type: ConnectionEventType::Error,
                details: "Mock failure".to_string(),
                timestamp: Utc::now(),
            });
        }
    }
    
    async fn reconnect(&self) -> Result<bool, String> {
        if *self.should_fail.lock().await {
            return Err("Mock adapter configured to fail".to_string());
        }
        
        // Simulate reconnection by updating the connection status
        let mut health = self.connection_health.lock().await;
        health.status = ConnectionStatus::Connected;
        health.last_successful = Some(Utc::now());
        
        // Add to connection history
        let mut history = self.connection_history.lock().await;
        history.push(ConnectionEvent {
            event_type: ConnectionEventType::Connected,
            details: "Reconnected".to_string(),
            timestamp: Utc::now(),
        });
        
        Ok(true)
    }
}

#[async_trait]
impl MonitoringToDashboardAdapter for MockAdapter {
    async fn get_connection_status(&self) -> Result<ConnectionStatus, std::io::Error> {
        let status = self.connection_status.lock().await.clone();
        Ok(status)
    }
    
    async fn get_dashboard_data(&self) -> Result<DashboardData, std::io::Error> {
        match self.get_dashboard_data().await {
            Ok(data) => Ok(data),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }
    
    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics, std::io::Error> {
        match McpMetricsProviderTrait::get_performance_metrics(self).await {
            Ok(metrics) => Ok(metrics),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }
    
    async fn start(&self, dashboard_service: Arc<dyn DashboardService>) -> Result<(), std::io::Error> {
        // Create a clone of the adapter to move into the async task
        let adapter_clone = MockAdapter::new();
        let update_interval = self.performance_options.lock().await.polling_min_interval_ms;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(update_interval));
            
            loop {
                interval.tick().await;
                
                // Get the latest data (use the moved clone)
                if let Ok(data) = adapter_clone.get_dashboard_data().await {
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