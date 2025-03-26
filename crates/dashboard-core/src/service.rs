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
use crate::data::{DashboardData, SystemSnapshot, NetworkSnapshot, AlertsSnapshot, MetricsSnapshot, Alert, AlertSeverity};
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
            system: SystemSnapshot {
                cpu_usage: 0.0,
                memory_used: 0,
                memory_total: 0,
                disk_used: 0,
                disk_total: 0,
                load_average: [0.0, 0.0, 0.0],
                uptime: 0,
            },
            network: NetworkSnapshot {
                rx_bytes: 0,
                tx_bytes: 0,
                rx_packets: 0,
                tx_packets: 0,
                interfaces: HashMap::new(),
            },
            alerts: AlertsSnapshot {
                active: Vec::new(),
                recent: Vec::new(),
                counts: HashMap::new(),
            },
            metrics: MetricsSnapshot {
                values: HashMap::new(),
                counters: HashMap::new(),
                gauges: HashMap::new(),
                histograms: HashMap::new(),
            },
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
    
    /// Collect dashboard data
    async fn collect_dashboard_data(&self) -> Result<()> {
        // This is a placeholder implementation
        // In a real implementation, this would collect data from monitoring systems
        
        let mut data = self.data.write().await;
        data.timestamp = Utc::now();
        
        // Send update to subscribers
        if let Err(e) = self.update_sender.send(DashboardUpdate::FullUpdate(data.clone())).await {
            return Err(DashboardError::Update(format!("Failed to send update: {}", e)));
        }
        
        Ok(())
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
        
        // Find and update the alert
        let mut found = false;
        for alert in &mut data.alerts.active {
            if alert.id == alert_id {
                alert.acknowledged = true;
                alert.acknowledged_at = Some(Utc::now());
                alert.acknowledged_by = Some(acknowledged_by.to_string());
                found = true;
                break;
            }
        }
        
        if !found {
            return Err(DashboardError::Generic(format!("Alert with ID {} not found", alert_id)));
        }
        
        // Send update
        let alert_update = DashboardUpdate::AlertUpdate {
            alert: data.alerts.active.iter()
                .find(|a| a.id == alert_id)
                .cloned()
                .unwrap(),
            timestamp: Utc::now(),
        };
        
        if let Err(e) = self.update_sender.send(alert_update).await {
            return Err(DashboardError::Update(format!("Failed to send update: {}", e)));
        }
        
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
        let config_clone = self.config.clone();
        let data_clone = self.data.clone();
        let running_clone = self.running.clone();
        let update_sender_clone = self.update_sender.clone();
        
        tokio::spawn(async move {
            let mut interval_timer = interval(update_interval);
            
            loop {
                interval_timer.tick().await;
                
                let running = *running_clone.read().await;
                if !running {
                    break;
                }
                
                // This is a placeholder implementation
                // In a real implementation, this would collect data from monitoring systems
                let mut data = data_clone.write().await;
                data.timestamp = Utc::now();
                
                // Send update to subscribers
                if let Err(e) = update_sender_clone.send(DashboardUpdate::FullUpdate(data.clone())).await {
                    log::error!("Failed to send update: {}", e);
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
} 