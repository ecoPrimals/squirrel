//! Resilience Monitoring Integration Example
//!
//! This example demonstrates the integration between MCP resilience health monitoring
//! system and monitoring system. It shows how health checks, metrics, and alerts are
//! used together to build a resilient system.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use async_trait::async_trait;

use squirrel_mcp::monitoring::{
    alerts::{Alert, AlertConfiguration, AlertState, AlertAction, AlertCondition, AlertSeverity},
    metrics::{Metric, MetricType, MetricValue},
};
use squirrel_mcp::resilience::{
    health::{HealthCheck, HealthCheckConfig, HealthCheckResult, HealthStatus, HealthMonitor},
    recovery::RecoveryStrategy, 
};
use squirrel_mcp::error::{Result, MCPError};
use tracing::{info, warn};

// Custom HealthMonitoringBridge implementation
#[derive(Debug)]
struct ResilienceMonitoringBridge {
    health_checks: RwLock<Vec<Box<dyn HealthCheck>>>,
    metrics_collector: Arc<SimpleMetricsCollector>,
    alert_manager: Arc<SimpleAlertManager>,
    recovery_strategy: Arc<Mutex<RecoveryStrategy>>,
}

impl ResilienceMonitoringBridge {
    fn new(
        metrics_collector: Arc<SimpleMetricsCollector>,
        alert_manager: Arc<SimpleAlertManager>,
        recovery_strategy: Arc<Mutex<RecoveryStrategy>>,
    ) -> Self {
        Self {
            health_checks: RwLock::new(Vec::new()),
            metrics_collector,
            alert_manager,
            recovery_strategy,
        }
    }
    
    async fn register_health_check(&self, health_check: Box<dyn HealthCheck>) -> Result<()> {
        let mut checks = self.health_checks.write().unwrap();
        checks.push(health_check);
        info!("Health check registered: {}", checks.len());
        Ok(())
    }
    
    async fn check_all(&self) -> Result<()> {
        // Get all health checks
        let checks = self.health_checks.read().unwrap();
        
        // Check each health check
        for check in checks.iter() {
            let result = check.check().await;
            
            // Create a metric for the health status
            let health_metric = Metric::new(
                format!("{}_health_status", check.id()),
                format!("Health status for {}", check.id()),
                MetricType::Gauge,
                match result.status {
                    HealthStatus::Healthy => MetricValue::Integer(0),
                    HealthStatus::Degraded => MetricValue::Integer(1),
                    HealthStatus::Unhealthy => MetricValue::Integer(2),
                    _ => MetricValue::Integer(3),
                },
            );
            
            // Register the metric
            if let Err(err) = self.metrics_collector.register_metric(health_metric) {
                warn!("Failed to register health metric: {}", err);
            }
            
            // Create metrics for each component-specific metric
            for (key, value) in &result.metrics {
                let component_metric = Metric::new(
                    format!("{}_{}", check.id(), key),
                    format!("{} for {}", key, check.id()),
                    MetricType::Gauge,
                    MetricValue::Float(*value),
                );
                
                // Register the component metric
                if let Err(err) = self.metrics_collector.register_metric(component_metric) {
                    warn!("Failed to register component metric: {}", err);
                }
            }
            
            // Create alerts for unhealthy components
            if result.status != HealthStatus::Healthy {
                let severity = match result.status {
                    HealthStatus::Degraded => AlertSeverity::Warning,
                    HealthStatus::Unhealthy => AlertSeverity::Error,
                    _ => AlertSeverity::Info,
                };
                
                let alert_config = AlertConfiguration {
                    name: format!("{}_health_alert", check.id()),
                    description: format!("Health check for {} is in status: {:?} - {}", 
                        check.id(), result.status, result.message),
                    condition: AlertCondition::Custom("health_check".to_string()),
                    severity,
                    actions: vec![AlertAction::Log],
                    check_interval_seconds: 60,
                    minimum_interval_seconds: 300,
                    enabled: true,
                    labels: HashMap::new(),
                };
                
                // Register the alert
                if let Err(err) = self.alert_manager.register_alert(alert_config) {
                    warn!("Failed to register alert: {}", err);
                }
                
                // Trigger recovery if needed
                if result.requires_recovery() {
                    info!("Triggering recovery for {}", check.id());
                    let failure_info = squirrel_mcp::resilience::recovery::FailureInfo {
                        message: result.message.clone(),
                        severity: match result.status {
                            HealthStatus::Degraded => squirrel_mcp::resilience::recovery::FailureSeverity::Minor,
                            HealthStatus::Unhealthy => squirrel_mcp::resilience::recovery::FailureSeverity::Critical,
                            _ => squirrel_mcp::resilience::recovery::FailureSeverity::Info,
                        },
                        context: check.id().to_string(),
                        recovery_attempts: 0,
                    };
                    
                    // In a real implementation, this would trigger the appropriate recovery action
                    // based on the component and failure info
                    if let Ok(mut strategy) = self.recovery_strategy.try_lock() {
                        // Example of what could happen in a real system:
                        // strategy.handle_failure(failure_info, || {
                        //     Ok(()) // Recovery action would go here
                        // });
                        info!("Recovery would trigger for {}: {:?}", check.id(), failure_info);
                    }
                }
            }
        }
        
        Ok(())
    }
}

// A example health check implementation
#[derive(Debug, Clone)]
struct ApiHealthCheck {
    component_name: String,
    status: Arc<Mutex<HealthStatus>>,
    config: HealthCheckConfig,
}

impl ApiHealthCheck {
    fn new(component_name: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
            status: Arc::new(Mutex::new(HealthStatus::Healthy)),
            config: HealthCheckConfig {
                check_interval: Duration::from_secs(60),
                check_timeout: Duration::from_secs(5),
                failure_threshold: 3,
                recovery_threshold: 2,
                auto_recovery: true,
            },
        }
    }
    
    // Update the health status for testing
    fn set_status(&self, status: HealthStatus) {
        let mut current = self.status.lock().unwrap();
        *current = status;
    }
}

#[async_trait]
impl HealthCheck for ApiHealthCheck {
    fn id(&self) -> &str {
        &self.component_name
    }
    
    fn config(&self) -> &HealthCheckConfig {
        &self.config
    }
    
    fn config_mut(&mut self) -> &mut HealthCheckConfig {
        &mut self.config
    }
    
    async fn check(&self) -> HealthCheckResult {
        let status = *self.status.lock().unwrap();
        let mut result = HealthCheckResult::new(
            self.component_name.clone(), 
            status,
            format!("{} health check completed with status {:?}", self.component_name, status)
        );
        
        // Add some metrics based on the status
        result = result.with_metric("response_time_ms", match status {
            HealthStatus::Healthy => 50.0,
            HealthStatus::Degraded => 150.0,
            HealthStatus::Unhealthy => 500.0,
            _ => 0.0,
        });
        
        result = result.with_metric("request_count", 1000.0);
        
        result = result.with_metric("error_rate", match status {
            HealthStatus::Healthy => 0.01,
            HealthStatus::Degraded => 0.05,
            HealthStatus::Unhealthy => 0.25,
            _ => 0.0,
        });
        
        result
    }
}

// Simple monitoring components for the example
#[derive(Debug)]
struct SimpleMetricsCollector {
    metrics: Arc<Mutex<Vec<Metric>>>,
}

impl SimpleMetricsCollector {
    fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn print_metrics(&self) {
        let metrics = self.metrics.lock().unwrap();
        println!("\n=== Metrics ===");
        if metrics.is_empty() {
            println!("No metrics collected");
            return;
        }
        
        for metric in metrics.iter() {
            println!("Metric: {} = {:?} [{:?}]", 
                metric.name, 
                metric.value, 
                metric.metric_type
            );
            if !metric.labels.is_empty() {
                println!("  Labels: {:?}", metric.labels);
            }
        }
    }
    
    // Implement required metrics collector methods directly
    fn register_metric(&self, metric: Metric) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(metric);
        Ok(())
    }
    
    fn get_metric(&self, name: &str) -> Option<Metric> {
        let metrics = self.metrics.lock().unwrap();
        metrics.iter().find(|m| m.name == name).cloned()
    }
}

// Simple alert manager for the example
#[derive(Debug)]
struct SimpleAlertManager {
    alerts: Arc<Mutex<Vec<AlertConfiguration>>>,
    triggered: Arc<Mutex<Vec<Alert>>>,
}

impl SimpleAlertManager {
    fn new() -> Self {
        Self {
            alerts: Arc::new(Mutex::new(Vec::new())),
            triggered: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn register_alert(&self, config: AlertConfiguration) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut alerts = self.alerts.lock().unwrap();
        alerts.push(config);
        Ok(())
    }
    
    fn trigger_alert(&self, alert_name: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let alerts = self.alerts.lock().unwrap();
        if let Some(config) = alerts.iter().find(|a| a.name == alert_name) {
            let alert = Alert {
                id: uuid::Uuid::new_v4().to_string(),
                config: config.clone(),
                state: AlertState::Active,
                first_fired_at: chrono::Utc::now(),
                last_fired_at: chrono::Utc::now(),
                recovery_at: None,
                times_fired: 1,
                silenced: false,
                silenced_until: None,
                last_notification: None,
                metadata: Default::default(),
            };
            
            let mut triggered = self.triggered.lock().unwrap();
            triggered.push(alert);
        }
        
        Ok(())
    }
    
    fn print_alerts(&self) {
        let alerts = self.alerts.lock().unwrap();
        let triggered = self.triggered.lock().unwrap();
        
        println!("\n=== Alert Configurations ===");
        if alerts.is_empty() {
            println!("No alert configurations");
        } else {
            for alert in alerts.iter() {
                println!("Alert: {} ({:?}) - {}", 
                    alert.name,
                    alert.severity,
                    alert.description
                );
            }
        }
        
        println!("\n=== Triggered Alerts ===");
        if triggered.is_empty() {
            println!("No alerts triggered");
        } else {
            for alert in triggered.iter() {
                println!("Triggered: {} ({:?}) - {}", 
                    alert.config.name,
                    alert.state,
                    alert.config.description
                );
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Starting resilience monitoring integration example");
    
    // Create the components
    let metrics_collector = Arc::new(SimpleMetricsCollector::new());
    let alert_manager = Arc::new(SimpleAlertManager::new());
    let recovery_strategy = Arc::new(Mutex::new(RecoveryStrategy::default()));
    
    // Create the resilience monitoring bridge
    let bridge = ResilienceMonitoringBridge::new(
        metrics_collector.clone(),
        alert_manager.clone(),
        recovery_strategy.clone(),
    );
    
    // Create some health checks
    let api_service = ApiHealthCheck::new("api_service");
    let database = ApiHealthCheck::new("database");
    
    // Register the health checks
    bridge.register_health_check(Box::new(api_service.clone())).await?;
    bridge.register_health_check(Box::new(database.clone())).await?;
    
    // Start with everything healthy
    info!("Initial check - all components healthy");
    bridge.check_all().await?;
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Simulate degradation
    info!("Simulating API service degradation");
    api_service.set_status(HealthStatus::Degraded);
    bridge.check_all().await?;
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Simulate failure
    info!("Simulating database failure");
    database.set_status(HealthStatus::Unhealthy);
    bridge.check_all().await?;
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Simulate recovery
    info!("Simulating recovery");
    api_service.set_status(HealthStatus::Healthy);
    database.set_status(HealthStatus::Healthy);
    bridge.check_all().await?;
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    info!("Resilience monitoring integration example completed");
    
    Ok(())
} 