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
use crate::data::{DashboardData, CpuMetrics, MemoryMetrics, NetworkMetrics, DiskMetrics, NetworkInterface};
use crate::error::{Result, DashboardError};
use crate::update::DashboardUpdate;

/// Dashboard service interface.
///
/// This trait defines the core functionality of a dashboard service,
/// which collects and provides dashboard data and handles updates.
#[async_trait]
pub trait DashboardService: Send + Sync {
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
                    rx_per_sec: 0.0,
                    tx_per_sec: 0.0,
                    rx_total: 0,
                    tx_total: 0,
                    interfaces: HashMap::new(),
                },
                disk: DiskMetrics {
                    disks: HashMap::new(),
                    io_per_sec: 0.0,
                    read_per_sec: 0.0,
                    write_per_sec: 0.0,
                },
                history: crate::data::MetricsHistory::default(),
            },
            protocol: crate::data::ProtocolData::default(),
            alerts: Vec::new(),
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
        let root_disk = data.metrics.disk.disks.entry("root".to_string())
            .or_insert_with(|| crate::data::DiskInfo {
                mount_point: "/".to_string(),
                total: 1_000_000_000_000, // ~1TB
                used: 500_000_000_000, // ~500GB
                free: 500_000_000_000,
                fs_type: "ext4".to_string(),
            });
        
        root_disk.used = 500_000_000_000; // ~500GB
        root_disk.total = 1_000_000_000_000; // ~1TB
        
        // Update network metrics with dummy data
        data.metrics.network.interfaces.clear();
        data.metrics.network.rx_total = 1_500_000; // 1.5MB received
        data.metrics.network.tx_total = 500_000; // 500KB sent
        data.metrics.network.rx_per_sec = 1000.0;
        data.metrics.network.tx_per_sec = 500.0;
        
        // Add some dummy network interfaces
        let interface1 = NetworkInterface {
            name: "eth0".to_string(),
            rx_per_sec: 800.0,
            tx_per_sec: 400.0,
            rx_total: 1_000_000, // 1MB
            tx_total: 400_000, // 400KB
            is_up: true,
        };
        
        let interface2 = NetworkInterface {
            name: "wlan0".to_string(),
            rx_per_sec: 200.0,
            tx_per_sec: 100.0,
            rx_total: 500_000, // 500KB
            tx_total: 100_000, // 100KB
            is_up: true,
        };
        
        data.metrics.network.interfaces.insert("eth0".to_string(), interface1);
        data.metrics.network.interfaces.insert("wlan0".to_string(), interface2);
        
        // Add timestamp to metrics history
        let timestamp = Utc::now();
        data.metrics.history.timestamps.push(timestamp);
        
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
            
        network_rx_history.push((timestamp, data.metrics.network.rx_total as f64));
        
        let network_tx_history = history.entry("network.tx_bytes".to_string())
            .or_insert_with(Vec::new);
            
        network_tx_history.push((timestamp, data.metrics.network.tx_total as f64));
        
        // Trim history if needed
        let config = self.config.read().await;
        let max_history_points = config.max_history_points;
        
        for (_, points) in history.iter_mut() {
            if points.len() > max_history_points {
                *points = points.drain(points.len() - max_history_points..).collect();
            }
        }
        
        // Create a clone for sending update
        let data_clone = data.clone();
        drop(data);
        
        // Send update to subscribers
        if let Err(e) = self.update_sender.send(DashboardUpdate::FullUpdate(data_clone)).await {
            return Err(DashboardError::Update(format!("Failed to send update: {}", e)));
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
        let (tx, rx) = mpsc::channel(100);
        
        // Send current data as an initial update
        let data = self.data.read().await.clone();
        if let Err(e) = tx.send(DashboardUpdate::FullUpdate(data)).await {
            log::error!("Failed to send initial update: {}", e);
        }
        
        // Return the receiver
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