//! Core-Monitoring Integration Example
//!
//! This example demonstrates how to integrate the Squirrel Core system with the
//! Monitoring system for comprehensive observability and metrics collection.
//! It shows how core components can expose metrics and health data to the monitoring
//! system while maintaining separation of concerns.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use async_trait::async_trait;

// Using our own simplified types instead of importing from monitoring crate
// to avoid dependency issues

// Simple metric value enum
#[derive(Debug, Clone)]
enum MetricValue {
    Integer(i64),
    Float(f64),
    Counter(u64),
    Gauge(f64),
    Text(String),
}

// Simple metric type enum
#[derive(Debug, Clone, Copy)]
enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

// Simple metric struct
#[derive(Debug, Clone)]
struct Metric {
    name: String,
    description: String,
    metric_type: MetricType,
    value: MetricValue,
}

impl Metric {
    fn new(name: String, description: String, metric_type: MetricType, value: MetricValue) -> Self {
        Self {
            name,
            description,
            metric_type,
            value,
        }
    }
}

// Simple alert severity enum
#[derive(Debug, Clone, Copy, PartialEq)]
enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

// Simple alert configuration struct
#[derive(Debug, Clone)]
struct AlertConfiguration {
    name: String,
    description: String,
    severity: AlertSeverity,
    enabled: bool,
}

// Simple alert struct
#[derive(Debug, Clone)]
struct Alert {
    id: String,
    config: AlertConfiguration,
    status: String,
}

use tracing::{info, warn, error};

// Example CoreComponent that we want to monitor
struct ExampleCoreComponent {
    name: String,
    state: Mutex<ComponentState>,
    config: ComponentConfig,
}

#[derive(Debug, Clone)]
struct ComponentState {
    active: bool,
    last_operation_time_ms: u64,
    error_rate: f64,
    operations_count: u64,
}

#[derive(Debug, Clone)]
struct ComponentConfig {
    update_interval: Duration,
    threshold_limit: u64,
}

impl ExampleCoreComponent {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            state: Mutex::new(ComponentState {
                active: true,
                last_operation_time_ms: 50,
                error_rate: 0.01,
                operations_count: 0,
            }),
            config: ComponentConfig {
                update_interval: Duration::from_secs(5),
                threshold_limit: 1000,
            },
        }
    }

    fn perform_operation(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        
        // Simulating some work
        state.operations_count += 1;
        
        // Every 10 operations, increase error rate and operation time
        if state.operations_count % 10 == 0 {
            state.error_rate += 0.01;
            state.last_operation_time_ms += 10;
        }
        
        // If we reach 30 operations, simulate degradation
        if state.operations_count >= 30 {
            state.active = false;
            return Err("Component exceeded operation threshold".to_string());
        }
        
        Ok(())
    }
    
    fn get_health_status(&self) -> ComponentHealth {
        let state = self.state.lock().unwrap();
        
        let status = if !state.active {
            HealthStatus::Unhealthy
        } else if state.error_rate > 0.05 || state.last_operation_time_ms > 100 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        
        // Clone status for use in the message
        let status_for_message = status.clone();
        
        ComponentHealth {
            component_name: self.name.clone(),
            status,
            message: format!("Component status: {:?}", status_for_message),
            metrics: HashMap::from([
                ("error_rate".to_string(), state.error_rate),
                ("operation_time_ms".to_string(), state.last_operation_time_ms as f64),
                ("operations_count".to_string(), state.operations_count as f64),
            ]),
        }
    }
    
    fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.active = true;
        state.error_rate = 0.01;
        state.last_operation_time_ms = 50;
        info!("Component '{}' has been reset", self.name);
    }
}

// Health-related structures
#[derive(Debug, Clone, PartialEq)]
enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone)]
struct ComponentHealth {
    component_name: String,
    status: HealthStatus,
    message: String,
    metrics: HashMap<String, f64>,
}

// Adapter connecting Core and Monitoring
struct CoreComponentMonitoringAdapter {
    core_component: Arc<ExampleCoreComponent>,
    metrics_collector: Arc<SimpleMetricsCollector>,
    alert_manager: Arc<SimpleAlertManager>,
    check_interval: Duration,
}

impl CoreComponentMonitoringAdapter {
    fn new(
        core_component: Arc<ExampleCoreComponent>,
        metrics_collector: Arc<SimpleMetricsCollector>,
        alert_manager: Arc<SimpleAlertManager>,
        check_interval: Duration,
    ) -> Self {
        Self {
            core_component,
            metrics_collector,
            alert_manager,
            check_interval,
        }
    }
    
    async fn start_monitoring(&self) {
        info!("Starting monitoring for component '{}'", self.core_component.name);
        
        loop {
            // Check component health
            let health = self.core_component.get_health_status();
            
            // Register metrics
            self.register_metrics(&health).await;
            
            // Check for alerts
            if health.status != HealthStatus::Healthy {
                self.trigger_alert(&health).await;
            }
            
            // Sleep for the check interval
            tokio::time::sleep(self.check_interval).await;
        }
    }
    
    async fn register_metrics(&self, health: &ComponentHealth) {
        let component_name = &health.component_name;
        
        // Register health status metric
        let status_value = match health.status {
            HealthStatus::Healthy => 0,
            HealthStatus::Degraded => 1,
            HealthStatus::Unhealthy => 2,
        };
        
        let status_metric = Metric::new(
            format!("{}_health_status", component_name),
            "Component health status".to_string(),
            MetricType::Gauge,
            MetricValue::Integer(status_value),
        );
        
        if let Err(err) = self.metrics_collector.register_metric(status_metric) {
            warn!("Failed to register health status metric: {}", err);
        }
        
        // Register component-specific metrics
        for (key, value) in &health.metrics {
            let metric = Metric::new(
                format!("{}_{}", component_name, key),
                format!("{} for {}", key, component_name),
                MetricType::Gauge,
                MetricValue::Float(*value),
            );
            
            if let Err(err) = self.metrics_collector.register_metric(metric) {
                warn!("Failed to register metric {}: {}", key, err);
            }
        }
    }
    
    async fn trigger_alert(&self, health: &ComponentHealth) {
        let severity = match health.status {
            HealthStatus::Degraded => AlertSeverity::Warning,
            HealthStatus::Unhealthy => AlertSeverity::Error,
            _ => AlertSeverity::Info,
        };
        
        let alert_config = AlertConfiguration {
            name: format!("{}_health_alert", health.component_name),
            description: format!("Health check for {} is in status: {:?} - {}", 
                health.component_name, health.status, health.message),
            severity,
            enabled: true,
        };
        
        if let Err(err) = self.alert_manager.register_alert(alert_config) {
            warn!("Failed to register alert: {}", err);
        }
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
        }
    }
    
    fn register_metric(&self, metric: Metric) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.push(metric);
        Ok(())
    }
    
    fn get_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
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
                alert.status
            );
            println!("  Severity: {:?}", alert.config.severity);
            println!("  Description: {}", alert.config.description);
        }
    }
    
    fn register_alert(&self, config: AlertConfiguration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let alert = Alert {
            id: format!("alert-{}", uuid::Uuid::new_v4()),
            config,
            status: "OK".to_string(),
        };
        
        let mut alerts = self.alerts.lock().unwrap();
        alerts.push(alert);
        Ok(())
    }
    
    fn get_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.lock().unwrap();
        alerts.clone()
    }
}

// Example recovery handler that can reset components when alerted
struct RecoveryHandler {
    core_component: Arc<ExampleCoreComponent>,
    alert_manager: Arc<SimpleAlertManager>,
    check_interval: Duration,
}

impl RecoveryHandler {
    fn new(
        core_component: Arc<ExampleCoreComponent>,
        alert_manager: Arc<SimpleAlertManager>,
        check_interval: Duration,
    ) -> Self {
        Self {
            core_component,
            alert_manager,
            check_interval,
        }
    }
    
    async fn start(&self) {
        info!("Starting recovery handler for component '{}'", self.core_component.name);
        
        loop {
            // Check for alerts that require recovery
            let alerts = self.alert_manager.get_alerts();
            for alert in alerts {
                if alert.config.severity == AlertSeverity::Error && 
                   alert.config.name.contains(&self.core_component.name) {
                    info!("Triggering recovery for component '{}'", self.core_component.name);
                    self.core_component.reset();
                }
            }
            
            // Sleep for the check interval
            tokio::time::sleep(self.check_interval).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logs
    tracing_subscriber::fmt::init();
    
    info!("Starting core-monitoring integration example");
    
    // Create the components
    let metrics_collector = Arc::new(SimpleMetricsCollector::new());
    let alert_manager = Arc::new(SimpleAlertManager::new());
    let core_component = Arc::new(ExampleCoreComponent::new("example-service"));
    
    // Create the monitoring adapter
    let adapter = Arc::new(CoreComponentMonitoringAdapter::new(
        core_component.clone(),
        metrics_collector.clone(),
        alert_manager.clone(),
        Duration::from_secs(1),
    ));
    
    // Create the recovery handler
    let recovery_handler = Arc::new(RecoveryHandler::new(
        core_component.clone(),
        alert_manager.clone(),
        Duration::from_secs(1),
    ));
    
    // Start monitoring in the background
    let adapter_clone = adapter.clone();
    let monitoring_task = tokio::spawn(async move {
        adapter_clone.start_monitoring().await;
    });
    
    // Start recovery handler in the background
    let recovery_clone = recovery_handler.clone();
    let recovery_task = tokio::spawn(async move {
        recovery_clone.start().await;
    });
    
    // Run the example workflow
    info!("Running example workflow");
    
    // Start with healthy component
    println!("\n=== Initial Status ===");
    println!("Component Status: {:?}", core_component.get_health_status().status);
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Perform operations until degradation
    info!("Performing operations until degradation");
    for i in 0..20 {
        if let Err(e) = core_component.perform_operation() {
            error!("Operation failed: {}", e);
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Check degraded status
    println!("\n=== After Operations (Degraded) ===");
    println!("Component Status: {:?}", core_component.get_health_status().status);
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Perform more operations until failure
    info!("Performing more operations until failure");
    for i in 0..20 {
        if let Err(e) = core_component.perform_operation() {
            error!("Operation failed: {}", e);
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Check unhealthy status
    println!("\n=== After More Operations (Unhealthy) ===");
    println!("Component Status: {:?}", core_component.get_health_status().status);
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Wait for recovery to trigger
    info!("Waiting for recovery to trigger");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Check recovered status
    println!("\n=== After Recovery ===");
    println!("Component Status: {:?}", core_component.get_health_status().status);
    metrics_collector.print_metrics();
    alert_manager.print_alerts();
    
    // Clean shutdown (in a real app you would properly shut down the tasks)
    info!("Example completed");
    
    Ok(())
} 