use crate::error::Result;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    /// Current status of the system (e.g., "healthy", "degraded", "critical")
    pub status: String,
    /// System uptime in seconds
    pub uptime: u64,
    /// Last update timestamp (seconds since UNIX epoch)
    pub last_update: u64,
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
            uptime: 0,
            last_update: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
        }
    }
}

/// Monitoring service trait
#[async_trait::async_trait]
pub trait MonitoringServiceTrait: Send + Sync {
    /// Start the monitoring service
    async fn start(&mut self) -> Result<()>;
    
    /// Stop the monitoring service
    async fn stop(&mut self) -> Result<()>;
    
    /// Get the current system status
    async fn get_system_status(&self) -> Result<HashMap<String, String>>;
    
    /// Get all recorded metrics
    async fn get_metrics(&self) -> Result<Vec<HashMap<String, String>>>;
    
    /// Get all active alerts
    async fn get_alerts(&self) -> Result<Vec<HashMap<String, String>>>;
}

/// Factory for creating monitoring services
pub trait MonitoringServiceFactoryTrait: Send + Sync {
    /// Create a new monitoring service
    fn create_service(&self) -> std::sync::Arc<dyn MonitoringServiceTrait>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::error::Result;
    use std::collections::HashMap;
    use std::sync::Arc;

    /// Test implementation of MonitoringService
    #[derive(Debug)]
    pub struct TestMonitoringService {
        started: bool,
        stopped: bool,
    }

    impl TestMonitoringService {
        pub fn new() -> Self {
            Self {
                started: false,
                stopped: false,
            }
        }
    }

    #[async_trait::async_trait]
    impl MonitoringServiceTrait for TestMonitoringService {
        async fn start(&mut self) -> Result<()> {
            self.started = true;
            Ok(())
        }

        async fn stop(&mut self) -> Result<()> {
            self.stopped = true;
            Ok(())
        }

        async fn get_system_status(&self) -> Result<HashMap<String, String>> {
            let mut status = HashMap::new();
            status.insert("status".to_string(), "healthy".to_string());
            status.insert("uptime".to_string(), "3600".to_string());
            status.insert("timestamp".to_string(), "123456789".to_string());
            Ok(status)
        }

        async fn get_metrics(&self) -> Result<Vec<HashMap<String, String>>> {
            let mut metric = HashMap::new();
            metric.insert("name".to_string(), "test_metric".to_string());
            metric.insert("value".to_string(), "42.0".to_string());
            Ok(vec![metric])
        }

        async fn get_alerts(&self) -> Result<Vec<HashMap<String, String>>> {
            let mut alert = HashMap::new();
            alert.insert("name".to_string(), "test_alert".to_string());
            alert.insert("message".to_string(), "Test alert".to_string());
            Ok(vec![alert])
        }
    }
} 