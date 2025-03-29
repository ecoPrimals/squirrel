use super::*;
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, Utc};
use dashboard_core::data::{
    Alert, AlertSeverity, CpuMetrics, DiskMetrics, DiskUsage, MemoryMetrics, Metrics,
    MetricsHistory, NetworkInterface, NetworkMetrics, Protocol, ProtocolData, ProtocolStatus
};
use dashboard_core::health::{HealthCheck, HealthStatus};

/// A mock monitoring adapter that generates realistic test data
/// for development and testing purposes.
pub struct MockMonitoringAdapter {
    /// Update interval in seconds
    update_interval: Duration,
    /// Last update time
    last_update: Mutex<Option<Instant>>,
    /// Metrics history
    metrics_history: Arc<Mutex<MetricsHistory>>,
    /// Protocol type
    protocol_type: Protocol,
    /// Whether the connection is established
    connected: bool,
}

impl Default for MockMonitoringAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MockMonitoringAdapter {
    /// Create a new mock monitoring adapter
    pub fn new() -> Self {
        let adapter = Self {
            update_interval: Duration::from_secs(5),
            last_update: Mutex::new(None),
            metrics_history: Arc::new(Mutex::new(MetricsHistory::default())),
            protocol_type: Protocol::Http,
            connected: true,
        };
        
        // Initialize with some data points for history
        {
            let mut history = adapter.metrics_history.lock().unwrap();
            
            // Generate a few random CPU data points
            for _ in 0..20 {
                history.cpu.push((Utc::now(), thread_rng().gen_range(5.0..80.0)));
            }
            
            // Generate a few random memory data points
            for _ in 0..20 {
                history.memory.push((Utc::now(), thread_rng().gen_range(20.0..85.0)));
            }
            
            // Generate a few random network data points
            for _ in 0..20 {
                history.network.push((
                    Utc::now(), 
                    (
                        thread_rng().gen_range(1000..100000), 
                        thread_rng().gen_range(500..50000)
                    )
                ));
            }
        }
        
        adapter
    }
    
    /// Set the update interval
    pub fn with_update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }
    
    /// Set the protocol type
    pub fn with_protocol_type(mut self, protocol_type: Protocol) -> Self {
        self.protocol_type = protocol_type;
        self
    }
    
    /// Update mock data based on the interval
    fn update_mock_data(&self) {
        let mut last_update = self.last_update.lock().unwrap();
        let now = Instant::now();
        
        if let Some(last) = *last_update {
            if now.duration_since(last) < self.update_interval {
                return;
            }
        }
        
        *last_update = Some(now);
        
        // Update metrics history with new random values
        let mut history = self.metrics_history.lock().unwrap();
        
        // Add new CPU data point
        history.cpu.push((Utc::now(), thread_rng().gen_range(5.0..80.0)));
        
        // Add new memory data point
        history.memory.push((Utc::now(), thread_rng().gen_range(20.0..85.0)));
        
        // Add new network data point
        history.network.push((
            Utc::now(), 
            (
                thread_rng().gen_range(1000..100000), 
                thread_rng().gen_range(500..50000)
            )
        ));
        
        // Keep only the last 100 data points
        if history.cpu.len() > 100 {
            history.cpu.remove(0);
        }
        
        if history.memory.len() > 100 {
            history.memory.remove(0);
        }
        
        if history.network.len() > 100 {
            history.network.remove(0);
        }
    }
}

impl fmt::Debug for MockMonitoringAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockMonitoringAdapter")
            .field("update_interval", &self.update_interval)
            .field("connected", &self.connected)
            .finish()
    }
}

impl MonitoringAdapter for MockMonitoringAdapter {
    fn get_metrics(&self) -> Metrics {
        self.update_mock_data();
        
        let history = self.metrics_history.lock().unwrap();
        let cpu_usage = if let Some((_, usage)) = history.cpu.last() {
            *usage
        } else {
            50.0 // Default value if no history
        };
        
        let memory_usage_pct = if let Some((_, usage)) = history.memory.last() {
            *usage
        } else {
            50.0 // Default value if no history
        };
        
        let mut rng = thread_rng();
        
        // Create CPU metrics
        let cpu = CpuMetrics {
            usage: cpu_usage,
            cores: vec![
                rng.gen_range(0.0..100.0),
                rng.gen_range(0.0..100.0),
                rng.gen_range(0.0..100.0),
                rng.gen_range(0.0..100.0),
            ],
            temperature: Some(rng.gen_range(30.0..70.0)),
            load: [
                rng.gen_range(0.1..3.0),
                rng.gen_range(0.1..2.5),
                rng.gen_range(0.1..2.0),
            ],
        };
        
        // Create memory metrics
        let memory_total = 16 * 1024 * 1024 * 1024; // 16 GB
        let memory_used = (memory_total as f64 * memory_usage_pct / 100.0) as u64;
        let memory = MemoryMetrics {
            total: memory_total,
            used: memory_used,
            available: memory_total - memory_used,
            free: memory_total - memory_used,
            swap_used: rng.gen_range(0..512) * 1024 * 1024,
            swap_total: 4 * 1024 * 1024 * 1024, // 4 GB swap
        };
        
        // Create network interfaces
        let mut interfaces = Vec::new();
        interfaces.push(NetworkInterface {
            name: "eth0".to_string(),
            is_up: true,
            rx_bytes: rng.gen_range(1000..100000),
            tx_bytes: rng.gen_range(500..50000),
            rx_packets: rng.gen_range(100..1000),
            tx_packets: rng.gen_range(50..500),
            rx_errors: rng.gen_range(0..5),
            tx_errors: rng.gen_range(0..3),
        });
        
        // Create network metrics
        let network = NetworkMetrics {
            interfaces: interfaces.clone(),
            total_rx_bytes: interfaces.iter().map(|i| i.rx_bytes).sum(),
            total_tx_bytes: interfaces.iter().map(|i| i.tx_bytes).sum(),
            total_rx_packets: interfaces.iter().map(|i| i.rx_packets).sum(),
            total_tx_packets: interfaces.iter().map(|i| i.tx_packets).sum(),
        };
        
        // Create disk usage map
        let mut disk_usage = HashMap::new();
        
        // Root volume
        disk_usage.insert("/".to_string(), DiskUsage {
            mount_point: "/".to_string(),
            total: 500 * 1024 * 1024 * 1024, // 500 GB
            used: rng.gen_range(50..400) * 1024 * 1024 * 1024,
            free: rng.gen_range(50..200) * 1024 * 1024 * 1024,
            used_percentage: rng.gen_range(10.0..80.0),
        });
        
        // Home volume
        disk_usage.insert("/home".to_string(), DiskUsage {
            mount_point: "/home".to_string(),
            total: 1000 * 1024 * 1024 * 1024, // 1 TB
            used: rng.gen_range(100..800) * 1024 * 1024 * 1024,
            free: rng.gen_range(100..400) * 1024 * 1024 * 1024,
            used_percentage: rng.gen_range(10.0..80.0),
        });
        
        // Create disk metrics
        let disk = DiskMetrics {
            usage: disk_usage,
            total_reads: rng.gen_range(1000..10000),
            total_writes: rng.gen_range(500..5000),
            read_bytes: rng.gen_range(10..100) * 1024 * 1024,
            written_bytes: rng.gen_range(5..50) * 1024 * 1024,
        };
        
        // Return complete metrics
        Metrics {
            cpu,
            memory,
            network,
            disk,
            history: history.clone(),
        }
    }
    
    fn health_checks(&self) -> Vec<HealthCheck> {
        self.update_mock_data();
        
        let mut checks = Vec::new();
        let mut rng = thread_rng();
        
        // System health
        checks.push(HealthCheck::new(
            "System",
            HealthStatus::Ok,
            "All systems operational"
        ));
        
        // Database health
        let db_status = if rng.gen_bool(0.9) {
            HealthStatus::Ok
        } else {
            HealthStatus::Warning
        };
        
        checks.push(HealthCheck::new(
            "Database",
            db_status,
            match db_status {
                HealthStatus::Ok => "Connected, normal operations",
                _ => "High latency detected",
            }
        ));
        
        // Network health
        let net_status = if rng.gen_bool(0.95) {
            HealthStatus::Ok
        } else if rng.gen_bool(0.5) {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };
        
        checks.push(HealthCheck::new(
            "Network",
            net_status,
            match net_status {
                HealthStatus::Ok => "All connections stable",
                HealthStatus::Warning => "Increased packet loss detected",
                HealthStatus::Critical => "Connection failures detected",
                _ => "Unknown status",
            }
        ));
        
        // Protocol health based on the protocol type
        let protocol_status = if rng.gen_bool(0.8) {
            HealthStatus::Ok
        } else if rng.gen_bool(0.5) {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };
        
        checks.push(HealthCheck::new(
            format!("{:?} Protocol", self.protocol_type),
            protocol_status,
            match protocol_status {
                HealthStatus::Ok => format!("{:?} protocol operating normally", self.protocol_type),
                HealthStatus::Warning => format!("{:?} protocol experiencing delays", self.protocol_type),
                HealthStatus::Critical => format!("{:?} protocol connection failures", self.protocol_type),
                _ => "Unknown status".to_string(),
            }
        ));
        
        checks
    }
    
    fn alerts(&self) -> Vec<Alert> {
        self.update_mock_data();
        
        let mut alerts = Vec::new();
        let mut rng = thread_rng();
        
        // Randomly generate a few alerts
        let alert_count = rng.gen_range(0..3);
        
        for i in 0..alert_count {
            // Generate random alert properties
            let severity = match rng.gen_range(0..4) {
                0 => AlertSeverity::Info,
                1 => AlertSeverity::Warning,
                2 => AlertSeverity::Error,
                _ => AlertSeverity::Critical,
            };
            
            let source = match rng.gen_range(0..3) {
                0 => "System",
                1 => "Application",
                _ => "User",
            };
            
            let messages = vec![
                "CPU usage exceeded threshold",
                "Memory usage is high",
                "Disk space running low",
                "Network connection unstable",
                "Database connection latency high",
                "Protocol message rate dropping",
                "Failed authentication attempt",
                "System update available",
            ];
            
            let title = messages[rng.gen_range(0..messages.len())];
            let message = format!("{}: Additional details", title);
            let timestamp = Utc::now() - chrono::Duration::seconds(rng.gen_range(0..3600));
            let acknowledged = rng.gen_bool(0.3);
            
            let alert = Alert {
                id: format!("alert-{}", i),
                title: title.to_string(),
                message,
                severity,
                source: source.to_string(),
                timestamp,
                acknowledged,
                acknowledged_by: None,
                acknowledged_at: None,
            };
            
            alerts.push(alert);
        }
        
        alerts
    }
    
    fn protocol_status(&self) -> Option<ProtocolData> {
        self.update_mock_data();
        
        if !self.connected {
            return None;
        }
        
        let mut rng = thread_rng();
        let connected = self.connected;
        
        Some(ProtocolData {
            name: format!("{:?}", self.protocol_type),
            protocol_type: format!("{:?}", self.protocol_type),
            version: "1.0".to_string(),
            connected,
            last_connected: Some(Utc::now() - chrono::Duration::seconds(if connected { 0 } else { rng.gen_range(60..300) })),
            status: if connected { "Connected".to_string() } else { "Disconnected".to_string() },
            error: if connected { None } else { Some("Connection lost, reconnecting...".to_string()) },
            retry_count: if connected { 0 } else { rng.gen_range(1..5) },
            metrics: HashMap::new(),
        })
    }
} 