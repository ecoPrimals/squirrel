//! MCP Monitoring Integration Example
//!
//! This example demonstrates the integration between the MCP resilience health monitoring
//! system and a monitoring system. It shows how health checks, metrics, and alerts are
//! integrated using the adapters and bridge.

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
use squirrel_mcp::error::Result;
use tracing::{info, warn};

// Custom HealthMonitoringBridge implementation
struct HealthMonitoringBridge {
    health_checks: RwLock<Vec<Box<dyn HealthCheck>>>,
    metrics_collector: Arc<SimpleMetricsCollector>,
    alert_manager: Arc<SimpleAlertManager>,
    recovery_strategy: Arc<Mutex<RecoveryStrategy>>,
}

impl HealthMonitoringBridge {
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
                    info!("Recovery action would be triggered for {}", check.id());
                    // In a real implementation, this would trigger the recovery action
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

#[derive(Debug)]
struct SimpleAlertManager {
    alerts: Arc<Mutex<Vec<Alert>>>,
}

impl SimpleAlertManager {
    fn new() -> Self {
        Self {
            alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn print_alerts(&self) {
        let alerts = self.alerts.lock().unwrap();
        println!("\n=== Alerts ===");
        if alerts.is_empty() {
            println!("No alerts triggered");
            return;
        }
        
        for alert in alerts.iter() {
            println!("Alert: {} [{}]", 
                alert.config.name, 
                alert.state
            );
            println!("  Severity: {:?}", alert.config.severity);
            println!("  Description: {}", alert.config.description);
        }
    }
    
    // Implement required alert manager methods directly
    fn register_alert(&self, config: AlertConfiguration) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let alert = create_alert(config);
        let mut alerts = self.alerts.lock().unwrap();
        alerts.push(alert);
        Ok(())
    }
    
    fn get_alert(&self, id: &str) -> Option<Alert> {
        let alerts = self.alerts.lock().unwrap();
        alerts.iter().find(|a| a.id == id).cloned()
    }
    
    fn update_alert(&self, alert: Alert) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut alerts = self.alerts.lock().unwrap();
        if let Some(idx) = alerts.iter().position(|a| a.id == alert.id) {
            alerts[idx] = alert;
        }
        Ok(())
    }
}

// Custom function to create an alert
fn create_alert(config: AlertConfiguration) -> Alert {
    Alert {
        id: config.name.clone(),
        config,
        state: AlertState::Ok,
        first_fired_at: None,
        last_fired_at: None,
        last_checked_at: Some(chrono::Utc::now()),
        triggered_value: None,
        firing_count: 0,
        acknowledged_by: None,
        acknowledged_at: None,
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logs
    tracing_subscriber::fmt::init();
    
    info!("Starting MCP monitoring integration example");
    
    // Create the monitoring components
    let metrics_collector = Arc::new(SimpleMetricsCollector::new());
    let alert_manager = Arc::new(SimpleAlertManager::new());
    
    // Create the resilience components
    let recovery_strategy = Arc::new(Mutex::new(RecoveryStrategy::default()));
    
    // Create the integration bridge
    let bridge = HealthMonitoringBridge::new(
        metrics_collector.clone(),
        alert_manager.clone(),
        recovery_strategy.clone(),
    );
    
    // Create health checks for different components
    let api_health = ApiHealthCheck::new("api-service");
    let db_health = ApiHealthCheck::new("database");
    
    // Register health checks with the bridge
    bridge.register_health_check(Box::new(api_health.clone())).await?;
    bridge.register_health_check(Box::new(db_health.clone())).await?;
    
    // Run the initial health checks (all healthy)
    info!("Running initial health checks (all healthy)");
    bridge.check_all().await?;
    
    // Print the collected metrics and alerts
    print_component_status("Initial Status", &api_health, &db_health).await;
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Simulate API degradation
    tokio::time::sleep(Duration::from_secs(1)).await;
    info!("Simulating API degradation");
    api_health.set_status(HealthStatus::Degraded);
    
    // Run health checks again
    bridge.check_all().await?;
    
    // Print the updated metrics and alerts
    print_component_status("After API Degradation", &api_health, &db_health).await;
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Simulate database failure
    tokio::time::sleep(Duration::from_secs(1)).await;
    info!("Simulating database failure");
    db_health.set_status(HealthStatus::Unhealthy);
    
    // Run health checks again
    bridge.check_all().await?;
    
    // Print the updated metrics and alerts
    print_component_status("After Database Failure", &api_health, &db_health).await;
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Restore all services to healthy
    tokio::time::sleep(Duration::from_secs(1)).await;
    info!("Restoring all services to healthy status");
    api_health.set_status(HealthStatus::Healthy);
    db_health.set_status(HealthStatus::Healthy);
    
    // Run health checks again
    bridge.check_all().await?;
    
    // Print the final metrics and alerts
    print_component_status("Final Status (Recovered)", &api_health, &db_health).await;
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    info!("MCP monitoring integration example completed");
    Ok(())
}

async fn print_component_status(label: &str, api: &ApiHealthCheck, db: &ApiHealthCheck) {
    println!("\n=== {} ===", label);
    println!("API Status: {:?}", *api.status.lock().unwrap());
    println!("DB Status: {:?}", *db.status.lock().unwrap());
} 