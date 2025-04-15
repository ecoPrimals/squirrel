//! Dashboard service interface and implementation.
//!
//! This module defines the DashboardService trait and its implementation.

use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::collections::HashMap;
use crate::config::DashboardConfig;
use crate::data::{DashboardData, CpuMetrics, MemoryMetrics, NetworkMetrics, 
                 DiskMetrics, NetworkInterface, DiskUsage, Alert, AlertSeverity,
                 MetricsHistory, ProtocolData, Metrics};
use crate::error::{Result, DashboardError};
use crate::update::DashboardUpdate;

/// Dashboard service interface.
///
/// This trait defines the core functionality of a dashboard service,
/// which collects and provides dashboard data and handles updates.
#[async_trait]
pub trait DashboardService: Send + Sync + std::fmt::Debug {
    /// Get the current dashboard data
    async fn get_dashboard_data(&self) -> Result<DashboardData>;
    
    /// Get historical metric values
    async fn get_metric_history(&self, metric_name: &str, time_period: Duration) -> Result<Vec<(DateTime<Utc>, f64)>>;
    
    /// Acknowledge an alert
    async fn acknowledge_alert(&self, alert_id: &str, acknowledged_by: &str) -> Result<()>;
    
    /// Subscribe to dashboard updates
    async fn subscribe(&self) -> mpsc::Receiver<DashboardUpdate>;
    
    /// Update dashboard configuration
    async fn update_config(&self, config: DashboardConfig) -> Result<()>;
    
    /// Update dashboard data
    async fn update_dashboard_data(&self, data: DashboardData) -> Result<()>;
    
    /// Start the dashboard service
    async fn start(&self) -> Result<()>;
    
    /// Stop the dashboard service
    async fn stop(&self) -> Result<()>;
}

/// Default implementation of DashboardService
#[derive(Debug)]
pub struct DefaultDashboardService {
    /// Configuration
    config: Arc<RwLock<DashboardConfig>>,
    /// Current dashboard data
    data: Arc<RwLock<DashboardData>>,
    /// Metrics history
    metric_history: Arc<RwLock<HashMap<String, Vec<(DateTime<Utc>, f64)>>>>,
    /// Update channel sender
    update_sender: mpsc::Sender<DashboardUpdate>,
    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl DefaultDashboardService {
    /// Create a new DefaultDashboardService with default configuration
    pub fn default() -> Arc<Self> {
        let config = DashboardConfig::default();
        let (service, _) = Self::new(config);
        service
    }

    /// Create a new dashboard service with the given configuration
    pub fn new(config: DashboardConfig) -> (Arc<Self>, mpsc::Receiver<DashboardUpdate>) {
        let (tx, rx) = mpsc::channel(100);
        
        // Create default empty dashboard data
        let data = DashboardData {
            metrics: crate::data::Metrics {
                cpu: CpuMetrics {
                    usage: 0.0,
                    cores: Vec::new(),
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
                    interfaces: Vec::new(),
                    total_rx_bytes: 0,
                    total_tx_bytes: 0,
                    total_rx_packets: 0,
                    total_tx_packets: 0,
                },
                disk: DiskMetrics {
                    usage: HashMap::new(),
                    total_reads: 0,
                    total_writes: 0,
                    read_bytes: 0,
                    written_bytes: 0,
                },
                history: crate::data::MetricsHistory::default(),
            },
            protocol: crate::data::ProtocolData::default(),
            alerts: Vec::new(),
            timestamp: Utc::now(),
        };
        
        let service = Arc::new(Self {
            config: Arc::new(RwLock::new(config)),
            data: Arc::new(RwLock::new(data)),
            metric_history: Arc::new(RwLock::new(HashMap::new())),
            update_sender: tx,
            running: Arc::new(RwLock::new(false)),
        });
        
        (service, rx)
    }
    
    /// Collect dashboard data from system
    async fn collect_dashboard_data(&self) -> Result<()> {
        // Get system information
        let mut sys_info = sysinfo::System::new_all();
        sys_info.refresh_all();
        
        // Update dashboard data
        let mut data = self.data.write().await;
        
        // Use dummy data for development
        data.metrics.cpu.usage = 45.0; // Dummy CPU usage
        data.metrics.memory.used = 4_000_000_000; // ~4GB
        data.metrics.memory.total = 16_000_000_000; // ~16GB
        
        // Use dummy disk data - assuming we have at least one disk
        let root_disk = data.metrics.disk.usage.entry("root".to_string())
            .or_insert_with(|| crate::data::DiskUsage {
                mount_point: "/".to_string(),
                total: 1_000_000_000_000, // ~1TB
                used: 500_000_000_000, // ~500GB
                free: 500_000_000_000,
                used_percentage: 50.0,
            });
        
        root_disk.used = 500_000_000_000; // ~500GB
        root_disk.total = 1_000_000_000_000; // ~1TB
        root_disk.free = root_disk.total - root_disk.used;
        root_disk.used_percentage = (root_disk.used as f64 / root_disk.total as f64) * 100.0;
        
        // Update network metrics with dummy data
        data.metrics.network.interfaces.clear();
        data.metrics.network.total_rx_bytes = 1_500_000; // 1.5MB received
        data.metrics.network.total_tx_bytes = 500_000; // 500KB sent
        data.metrics.network.total_rx_packets = 10000;
        data.metrics.network.total_tx_packets = 5000;
        
        // Add some dummy network interfaces
        let interface1 = NetworkInterface {
            name: "eth0".to_string(),
            is_up: true,
            rx_bytes: 1_000_000, // 1MB
            tx_bytes: 400_000, // 400KB
            rx_packets: 8000,
            tx_packets: 4000,
            rx_errors: 0,
            tx_errors: 0,
        };
        
        let interface2 = NetworkInterface {
            name: "wlan0".to_string(),
            is_up: true,
            rx_bytes: 500_000, // 500KB
            tx_bytes: 100_000, // 100KB
            rx_packets: 2000,
            tx_packets: 1000,
            rx_errors: 0,
            tx_errors: 0,
        };
        
        data.metrics.network.interfaces.push(interface1);
        data.metrics.network.interfaces.push(interface2);
        
        // Add timestamp
        let timestamp = Utc::now();
        data.timestamp = timestamp;
        
        // Update metric history with dummy data
        let mut history = self.metric_history.write().await;
        
        // Store CPU usage history
        let cpu_history = history.entry("system.cpu".to_string())
            .or_insert_with(Vec::new);
            
        cpu_history.push((timestamp, data.metrics.cpu.usage));
        
        // Store memory usage history
        let memory_history = history.entry("system.memory".to_string())
            .or_insert_with(Vec::new);
            
        memory_history.push((timestamp, data.metrics.memory.used as f64));
        
        // Store network history
        let network_rx_history = history.entry("network.rx_bytes".to_string())
            .or_insert_with(Vec::new);
            
        network_rx_history.push((timestamp, data.metrics.network.total_rx_bytes as f64));
        
        let network_tx_history = history.entry("network.tx_bytes".to_string())
            .or_insert_with(Vec::new);
            
        network_tx_history.push((timestamp, data.metrics.network.total_tx_bytes as f64));
        
        // Also update the metrics history in dashboard data
        let cpu_usage = data.metrics.cpu.usage;
        let mem_used = data.metrics.memory.used;
        let rx_bytes = data.metrics.network.total_rx_bytes;
        let tx_bytes = data.metrics.network.total_tx_bytes;
        
        data.metrics.history.cpu.push((timestamp, cpu_usage));
        data.metrics.history.memory.push((timestamp, mem_used as f64));
        data.metrics.history.network.push((timestamp, (rx_bytes, tx_bytes)));
        
        // Trim history if needed
        let config = self.config.read().await;
        
        // Get the max history points
        let max_points = config.max_history_points;
        
        // Trim metric_history
        for (_, values) in history.iter_mut() {
            if values.len() > max_points {
                values.drain(0..values.len() - max_points);
            }
        }
        
        // Trim dashboard data history
        let cpu_len = data.metrics.history.cpu.len();
        if cpu_len > max_points {
            data.metrics.history.cpu.drain(0..cpu_len - max_points);
        }
        
        let memory_len = data.metrics.history.memory.len();
        if memory_len > max_points {
            data.metrics.history.memory.drain(0..memory_len - max_points);
        }
        
        let network_len = data.metrics.history.network.len();
        if network_len > max_points {
            data.metrics.history.network.drain(0..network_len - max_points);
        }
        
        for (_, values) in data.metrics.history.custom.iter_mut() {
            let values_len = values.len();
            if values_len > max_points {
                values.drain(0..values_len - max_points);
            }
        }
        
        Ok(())
    }
    
    /// Update dashboard data with external data
    pub async fn update_dashboard_data(&self, data: DashboardData) -> Result<()> {
        // Update dashboard data
        *self.data.write().await = data.clone();
        
        // Send update to subscribers
        if let Err(e) = self.update_sender.send(DashboardUpdate::FullUpdate(data)).await {
            return Err(DashboardError::Update(format!("Failed to send update: {}", e)));
        }
        
        Ok(())
    }
    
    /// Update data with new values (legacy method)
    pub async fn update_data(&self, data: DashboardData) -> Result<()> {
        self.update_dashboard_data(data).await
    }
}

impl Clone for DefaultDashboardService {
    fn clone(&self) -> Self {
        // Create a new mpsc channel
        let (tx, _) = mpsc::channel(100);
        
        Self {
            config: self.config.clone(),
            data: self.data.clone(),
            metric_history: self.metric_history.clone(),
            update_sender: tx,
            running: self.running.clone(),
        }
    }
}

#[async_trait]
impl DashboardService for DefaultDashboardService {
    async fn get_dashboard_data(&self) -> Result<DashboardData> {
        let data = self.data.read().await.clone();
        Ok(data)
    }
    
    async fn get_metric_history(&self, metric_name: &str, time_period: Duration) -> Result<Vec<(DateTime<Utc>, f64)>> {
        let history = self.metric_history.read().await;
        
        if let Some(data) = history.get(metric_name) {
            // Filter data points by time period
            let cutoff = Utc::now() - chrono::Duration::from_std(time_period).unwrap_or_default();
            let filtered = data.iter()
                .filter(|(time, _)| *time >= cutoff)
                .cloned()
                .collect();
            Ok(filtered)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn acknowledge_alert(&self, alert_id: &str, acknowledged_by: &str) -> Result<()> {
        let mut data = self.data.write().await;
        
        // Find the alert by ID and acknowledge it
        let mut found = false;
        
        for alert in &mut data.alerts {
            if alert.id == alert_id {
                alert.acknowledged = true;
                alert.acknowledged_by = Some(acknowledged_by.to_string());
                alert.acknowledged_at = Some(Utc::now());
                found = true;
                break;
            }
        }
        
        if !found {
            return Err(DashboardError::NotFound(format!("Alert with ID {} not found", alert_id)));
        }
        
        // Send alert update notification
        let alert = data.alerts.iter()
            .find(|a| a.id == alert_id)
            .cloned()
            .ok_or_else(|| DashboardError::NotFound(format!("Alert with ID {} not found", alert_id)))?;
        
        self.update_sender.send(DashboardUpdate::AlertUpdate {
            alert,
            timestamp: Utc::now(),
        }).await.map_err(|e| DashboardError::Update(format!("Failed to send update: {}", e)))?;
        
        Ok(())
    }
    
    async fn subscribe(&self) -> mpsc::Receiver<DashboardUpdate> {
        // Create a dummy channel
        let (tx, rx) = mpsc::channel(10);
        // Spawn a task to send updates periodically for testing
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let _ = tx.send(DashboardUpdate::MetricsUpdate {
                    metrics: HashMap::new(),
                    timestamp: Utc::now(),
                }).await;
            }
        });
        rx
    }
    
    async fn update_config(&self, config: DashboardConfig) -> Result<()> {
        // Update the config
        *self.config.write().await = config.clone();
        
        // Send update
        let config_update = DashboardUpdate::ConfigUpdate { config };
        if let Err(e) = self.update_sender.send(config_update).await {
            return Err(DashboardError::Update(format!("Failed to send update: {}", e)));
        }
        
        Ok(())
    }
    
    async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        
        *running = true;
        drop(running);
        
        let config = self.config.read().await.clone();
        let update_interval = config.update_interval_duration();
        
        // Clone the Arc references for the tokio task
        let self_clone = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval_timer = interval(update_interval);
            
            loop {
                interval_timer.tick().await;
                
                let running = *self_clone.running.read().await;
                if !running {
                    break;
                }
                
                // Use the improved data collection method
                if let Err(e) = self_clone.collect_dashboard_data().await {
                    log::error!("Failed to collect dashboard data: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }
    
    async fn update_dashboard_data(&self, data: DashboardData) -> Result<()> {
        DefaultDashboardService::update_dashboard_data(self, data).await
    }
}

/// Mock implementation of DashboardService for testing
#[derive(Debug, Clone)]
pub struct MockDashboardService {
    data: DashboardData,
}

impl MockDashboardService {
    /// Create a new MockDashboardService
    pub fn new() -> Self {
        // Create default metrics
        let metrics = Metrics {
            cpu: CpuMetrics {
                usage: 25.0,
                cores: vec![25.0, 30.0, 20.0, 15.0],
                temperature: Some(45.0),
                load: [1.0, 0.8, 0.6],
            },
            memory: MemoryMetrics {
                total: 16 * 1024 * 1024 * 1024, // 16 GB
                used: 4 * 1024 * 1024 * 1024,   // 4 GB
                available: 12 * 1024 * 1024 * 1024, // 12 GB
                free: 10 * 1024 * 1024 * 1024,  // 10 GB
                swap_used: 512 * 1024 * 1024,   // 512 MB
                swap_total: 8 * 1024 * 1024 * 1024, // 8 GB
            },
            network: NetworkMetrics {
                interfaces: vec![
                    NetworkInterface {
                        name: "eth0".to_string(),
                        is_up: true,
                        rx_bytes: 1_000_000,
                        tx_bytes: 500_000,
                        rx_packets: 10_000,
                        tx_packets: 5_000,
                        rx_errors: 0,
                        tx_errors: 0,
                    },
                    NetworkInterface {
                        name: "wlan0".to_string(),
                        is_up: true,
                        rx_bytes: 500_000,
                        tx_bytes: 250_000,
                        rx_packets: 5_000,
                        tx_packets: 2_500,
                        rx_errors: 2,
                        tx_errors: 0,
                    },
                ],
                total_rx_bytes: 1_500_000,
                total_tx_bytes: 750_000,
                total_rx_packets: 15_000,
                total_tx_packets: 7_500,
            },
            disk: DiskMetrics {
                usage: {
                    let mut map = HashMap::new();
                    map.insert("/".to_string(), DiskUsage {
                        mount_point: "/".to_string(),
                        total: 512 * 1024 * 1024 * 1024, // 512 GB
                        used: 128 * 1024 * 1024 * 1024,  // 128 GB
                        free: 384 * 1024 * 1024 * 1024,  // 384 GB
                        used_percentage: 25.0,
                    });
                    map.insert("/home".to_string(), DiskUsage {
                        mount_point: "/home".to_string(),
                        total: 1024 * 1024 * 1024 * 1024, // 1 TB
                        used: 512 * 1024 * 1024 * 1024,   // 512 GB
                        free: 512 * 1024 * 1024 * 1024,   // 512 GB
                        used_percentage: 50.0,
                    });
                    map
                },
                total_reads: 10_000,
                total_writes: 5_000,
                read_bytes: 100_000_000,
                written_bytes: 50_000_000,
            },
            history: MetricsHistory::default(),
        };

        // Create protocol data
        let protocol = ProtocolData {
            name: "Mock Protocol".to_string(),
            protocol_type: "Custom".to_string(),
            version: "1.0.0".to_string(),
            connected: true,
            last_connected: Some(Utc::now()),
            status: "Connected".to_string(),
            error: None,
            retry_count: 0,
            metrics: {
                let mut map = HashMap::new();
                map.insert("latency_ms".to_string(), 15.5);
                map.insert("throughput_kbps".to_string(), 1024.0);
                map
            },
        };

        // Create alerts
        let alerts = vec![
            Alert {
                id: "alert-1".to_string(),
                title: "High CPU Usage".to_string(),
                message: "CPU usage above 90% for 5 minutes".to_string(),
                severity: AlertSeverity::Warning,
                source: "system_monitor".to_string(),
                timestamp: Utc::now(),
                acknowledged: false,
                acknowledged_by: None,
                acknowledged_at: None,
            },
            Alert {
                id: "alert-2".to_string(),
                title: "Service Restart".to_string(),
                message: "Service restarted automatically".to_string(),
                severity: AlertSeverity::Info,
                source: "service_monitor".to_string(),
                timestamp: Utc::now(),
                acknowledged: true,
                acknowledged_by: Some("admin".to_string()),
                acknowledged_at: Some(Utc::now()),
            },
        ];

        Self {
            data: DashboardData {
                metrics,
                protocol,
                alerts,
                timestamp: Utc::now(),
            },
        }
    }
}

#[async_trait]
impl DashboardService for MockDashboardService {
    async fn get_dashboard_data(&self) -> Result<DashboardData> {
        Ok(self.data.clone())
    }

    async fn get_metric_history(&self, _metric_name: &str, _time_period: Duration) -> Result<Vec<(DateTime<Utc>, f64)>> {
        // Return dummy history data
        let now = Utc::now();
        let mut history = Vec::new();
        for i in 0..10 {
            history.push((now - chrono::Duration::minutes(i as i64), 50.0 + (i as f64 * 2.0)));
        }
        Ok(history)
    }

    async fn acknowledge_alert(&self, _alert_id: &str, _acknowledged_by: &str) -> Result<()> {
        // For testing, just return success
        Ok(())
    }

    async fn subscribe(&self) -> mpsc::Receiver<DashboardUpdate> {
        // Create a dummy channel
        let (tx, rx) = mpsc::channel(10);
        // Spawn a task to send updates periodically for testing
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let _ = tx.send(DashboardUpdate::MetricsUpdate {
                    metrics: HashMap::new(),
                    timestamp: Utc::now(),
                }).await;
            }
        });
        rx
    }

    async fn update_config(&self, _config: DashboardConfig) -> Result<()> {
        // For testing, just return success
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        // For testing, just return success
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        // For testing, just return success
        Ok(())
    }

    async fn update_dashboard_data(&self, _data: DashboardData) -> Result<()> {
        // For testing, just return success
        Ok(())
    }
} 