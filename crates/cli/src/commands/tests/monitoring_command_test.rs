use std::sync::Arc;
use async_trait::async_trait;
use tokio::test;

use crate::commands::monitoring_command::MonitoringCommand;
use crate::commands::executor::CommandExecutor;
use crate::commands::registry::CommandRegistry;
use crate::commands::error::Result;
use squirrel_monitoring::{
    health::{HealthStatus, HealthCheck},
    metrics::{Metric, MetricType},
    alerts::{Alert, AlertSeverity},
};

/// Test for registering the monitoring command
#[test]
async fn test_monitoring_command_registration() {
    // Create a new command registry
    let mut registry = CommandRegistry::new();
    
    // Register the monitoring command
    MonitoringCommand::register(&mut registry).expect("Failed to register monitoring command");
    
    // Check if the command is registered
    assert!(registry.get_command("monitoring").is_some());
}

/// Test for health subcommand with text output
#[test]
async fn test_monitoring_health_command_text_output() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("health");
    
    // Execute the command
    let result = command.execute(vec!["health".to_string()]).await.expect("Failed to execute command");
    
    // Check the output
    assert!(result.contains("HEALTH STATUS:"));
    assert!(result.contains("test-component-1: HEALTHY"));
    assert!(result.contains("test-component-2: DEGRADED"));
    assert!(result.contains("test-component-3: UNHEALTHY"));
}

/// Test for health subcommand with JSON output
#[test]
async fn test_monitoring_health_command_json_output() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("health");
    
    // Execute the command with JSON format
    let result = command.execute(vec!["health".to_string(), "--format".to_string(), "json".to_string()]).await
        .expect("Failed to execute command");
    
    // Check the output is valid JSON and contains expected data
    let json_result: serde_json::Value = serde_json::from_str(&result).expect("Failed to parse JSON");
    assert!(json_result.is_array());
    assert_eq!(json_result.as_array().unwrap().len(), 3);
    
    // Check component IDs
    let component_ids: Vec<&str> = json_result.as_array().unwrap().iter()
        .map(|v| v["component_id"].as_str().unwrap())
        .collect();
    assert!(component_ids.contains(&"test-component-1"));
    assert!(component_ids.contains(&"test-component-2"));
    assert!(component_ids.contains(&"test-component-3"));
}

/// Test for health subcommand with component filter
#[test]
async fn test_monitoring_health_command_with_component_filter() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("health");
    
    // Execute the command with component filter
    let result = command.execute(vec!["health".to_string(), "--component".to_string(), "test-component-1".to_string()])
        .await.expect("Failed to execute command");
    
    // Check the output only contains the filtered component
    assert!(result.contains("test-component-1"));
    assert!(!result.contains("test-component-2"));
    assert!(!result.contains("test-component-3"));
}

/// Test for metrics subcommand with text output
#[test]
async fn test_monitoring_metrics_command_text_output() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("metrics");
    
    // Execute the command
    let result = command.execute(vec!["metrics".to_string()]).await.expect("Failed to execute command");
    
    // Check the output
    assert!(result.contains("METRICS:"));
    assert!(result.contains("cpu_usage"));
    assert!(result.contains("memory_usage"));
    assert!(result.contains("request_count"));
}

/// Test for metrics subcommand with JSON output
#[test]
async fn test_monitoring_metrics_command_json_output() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("metrics");
    
    // Execute the command with JSON format
    let result = command.execute(vec!["metrics".to_string(), "--format".to_string(), "json".to_string()])
        .await.expect("Failed to execute command");
    
    // Check the output is valid JSON and contains expected data
    let json_result: serde_json::Value = serde_json::from_str(&result).expect("Failed to parse JSON");
    assert!(json_result.is_array());
    assert_eq!(json_result.as_array().unwrap().len(), 3);
    
    // Check metric names
    let metric_names: Vec<&str> = json_result.as_array().unwrap().iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    assert!(metric_names.contains(&"cpu_usage"));
    assert!(metric_names.contains(&"memory_usage"));
    assert!(metric_names.contains(&"request_count"));
}

/// Test for metrics subcommand with name filter
#[test]
async fn test_monitoring_metrics_command_with_name_filter() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("metrics");
    
    // Execute the command with name filter
    let result = command.execute(vec!["metrics".to_string(), "--name".to_string(), "cpu_usage".to_string()])
        .await.expect("Failed to execute command");
    
    // Check the output only contains the filtered metric
    assert!(result.contains("cpu_usage"));
    assert!(!result.contains("memory_usage"));
    assert!(!result.contains("request_count"));
}

/// Test for alerts subcommand with text output
#[test]
async fn test_monitoring_alerts_command_text_output() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("alerts");
    
    // Execute the command
    let result = command.execute(vec!["alerts".to_string()]).await.expect("Failed to execute command");
    
    // Check the output
    assert!(result.contains("ALERTS:"));
    assert!(result.contains("[INFO]"));
    assert!(result.contains("[WARNING]"));
    assert!(result.contains("[ERROR]"));
    assert!(result.contains("[CRITICAL]"));
}

/// Test for alerts subcommand with JSON output
#[test]
async fn test_monitoring_alerts_command_json_output() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("alerts");
    
    // Execute the command with JSON format
    let result = command.execute(vec!["alerts".to_string(), "--format".to_string(), "json".to_string()])
        .await.expect("Failed to execute command");
    
    // Check the output is valid JSON and contains expected data
    let json_result: serde_json::Value = serde_json::from_str(&result).expect("Failed to parse JSON");
    assert!(json_result.is_array());
    assert_eq!(json_result.as_array().unwrap().len(), 4);
    
    // Check alert severities
    let severities: Vec<&str> = json_result.as_array().unwrap().iter()
        .map(|v| v["severity"].as_str().unwrap())
        .collect();
    assert!(severities.contains(&"Info"));
    assert!(severities.contains(&"Warning"));
    assert!(severities.contains(&"Error"));
    assert!(severities.contains(&"Critical"));
}

/// Test for alerts subcommand with severity filter
#[test]
async fn test_monitoring_alerts_command_with_severity_filter() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("alerts");
    
    // Execute the command with severity filter
    let result = command.execute(vec!["alerts".to_string(), "--severity".to_string(), "error".to_string()])
        .await.expect("Failed to execute command");
    
    // Check the output only contains the filtered severity
    assert!(result.contains("[ERROR]"));
    assert!(!result.contains("[INFO]"));
    assert!(!result.contains("[WARNING]"));
    assert!(!result.contains("[CRITICAL]"));
}

/// Test for invalid severity filter
#[test]
async fn test_monitoring_alerts_command_with_invalid_severity() {
    // Create a monitoring command with mock data
    let command = create_test_monitoring_command("alerts");
    
    // Execute the command with invalid severity filter
    let result = command.execute(vec!["alerts".to_string(), "--severity".to_string(), "invalid".to_string()]).await;
    
    // Check the command returned an error
    assert!(result.is_err());
    
    // Check the error message
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid severity"));
}

/// Test for help command
#[test]
fn test_monitoring_command_help() {
    // Create a monitoring command
    let command = create_test_monitoring_command("health");
    
    // Get the help text
    let help = command.help();
    
    // Check the help text contains expected sections
    assert!(help.contains("MONITORING COMMANDS"));
    assert!(help.contains("monitoring health"));
    assert!(help.contains("monitoring metrics"));
    assert!(help.contains("monitoring alerts"));
    assert!(help.contains("OPTIONS:"));
    assert!(help.contains("EXAMPLES:"));
}

/// Helper function to create a monitoring command with mock data
fn create_test_monitoring_command(default_cmd: &str) -> MonitoringCommand {
    use clap::Parser;
    
    // Create command args
    let args = match default_cmd {
        "health" => vec!["monitoring", "health"],
        "metrics" => vec!["monitoring", "metrics"],
        "alerts" => vec!["monitoring", "alerts"],
        _ => vec!["monitoring", "health"],
    };
    
    // Parse args
    MonitoringCommand::parse_from(args)
}

/// Helper functions to create mock data
fn create_mock_health_checks() -> Vec<HealthCheck> {
    vec![
        HealthCheck {
            component_id: "test-component-1".to_string(),
            status: HealthStatus::Healthy,
            timestamp: chrono::Utc::now(),
            message: Some("System is healthy".to_string()),
        },
        HealthCheck {
            component_id: "test-component-2".to_string(),
            status: HealthStatus::Degraded,
            timestamp: chrono::Utc::now(),
            message: Some("Performance issues detected".to_string()),
        },
        HealthCheck {
            component_id: "test-component-3".to_string(),
            status: HealthStatus::Unhealthy,
            timestamp: chrono::Utc::now(),
            message: Some("Service is unavailable".to_string()),
        },
    ]
}

fn create_mock_metrics() -> Vec<Metric> {
    use serde_json::json;
    
    vec![
        Metric {
            name: "cpu_usage".to_string(),
            component_id: "test-component-1".to_string(),
            metric_type: MetricType::Gauge,
            value: json!(45.2),
            timestamp: chrono::Utc::now(),
            labels: Some(vec![("host".to_string(), "server-1".to_string())]),
        },
        Metric {
            name: "memory_usage".to_string(),
            component_id: "test-component-1".to_string(),
            metric_type: MetricType::Gauge,
            value: json!(1024.0),
            timestamp: chrono::Utc::now(),
            labels: Some(vec![("host".to_string(), "server-1".to_string())]),
        },
        Metric {
            name: "request_count".to_string(),
            component_id: "test-component-2".to_string(),
            metric_type: MetricType::Counter,
            value: json!(1250),
            timestamp: chrono::Utc::now(),
            labels: Some(vec![("endpoint".to_string(), "/api/v1/users".to_string())]),
        },
    ]
}

fn create_mock_alerts() -> Vec<Alert> {
    vec![
        Alert {
            id: "alert-1".to_string(),
            name: "High CPU Usage".to_string(),
            component_id: "test-component-1".to_string(),
            severity: AlertSeverity::Warning,
            message: "CPU usage is above 80%".to_string(),
            timestamp: chrono::Utc::now(),
            labels: Some(vec![("host".to_string(), "server-1".to_string())]),
        },
        Alert {
            id: "alert-2".to_string(),
            name: "Low Disk Space".to_string(),
            component_id: "test-component-1".to_string(),
            severity: AlertSeverity::Error,
            message: "Disk space is below 10%".to_string(),
            timestamp: chrono::Utc::now(),
            labels: Some(vec![("host".to_string(), "server-1".to_string())]),
        },
        Alert {
            id: "alert-3".to_string(),
            name: "Service Restart".to_string(),
            component_id: "test-component-2".to_string(),
            severity: AlertSeverity::Info,
            message: "Service has been restarted".to_string(),
            timestamp: chrono::Utc::now(),
            labels: Some(vec![("service".to_string(), "api-server".to_string())]),
        },
        Alert {
            id: "alert-4".to_string(),
            name: "Database Connection Failed".to_string(),
            component_id: "test-component-3".to_string(),
            severity: AlertSeverity::Critical,
            message: "Cannot connect to database".to_string(),
            timestamp: chrono::Utc::now(),
            labels: Some(vec![("database".to_string(), "main-db".to_string())]),
        },
    ]
}

// Mock clients for testing
struct MockHealthClient {
    health_checks: Vec<HealthCheck>,
}

impl MockHealthClient {
    fn new(health_checks: Vec<HealthCheck>) -> Self {
        Self { health_checks }
    }
}

#[async_trait]
impl super::super::monitoring_command::HealthCheckClient for MockHealthClient {
    async fn get_health_checks(&self, component_id: Option<&str>) -> std::result::Result<Vec<HealthCheck>, String> {
        let filtered_checks = if let Some(component) = component_id {
            self.health_checks.iter()
                .filter(|check| check.component_id == component)
                .cloned()
                .collect()
        } else {
            self.health_checks.clone()
        };
        
        Ok(filtered_checks)
    }
}

struct MockMetricsClient {
    metrics: Vec<Metric>,
}

impl MockMetricsClient {
    fn new(metrics: Vec<Metric>) -> Self {
        Self { metrics }
    }
}

#[async_trait]
impl super::super::monitoring_command::MetricsClient for MockMetricsClient {
    async fn get_metrics(&self, name: Option<&str>, component_id: Option<&str>) -> std::result::Result<Vec<Metric>, String> {
        let mut filtered_metrics = self.metrics.clone();
        
        if let Some(name_filter) = name {
            filtered_metrics = filtered_metrics.into_iter()
                .filter(|metric| metric.name == name_filter)
                .collect();
        }
        
        if let Some(component) = component_id {
            filtered_metrics = filtered_metrics.into_iter()
                .filter(|metric| metric.component_id == component)
                .collect();
        }
        
        Ok(filtered_metrics)
    }
}

struct MockAlertsClient {
    alerts: Vec<Alert>,
}

impl MockAlertsClient {
    fn new(alerts: Vec<Alert>) -> Self {
        Self { alerts }
    }
}

#[async_trait]
impl super::super::monitoring_command::AlertsClient for MockAlertsClient {
    async fn get_alerts(&self, severity: Option<AlertSeverity>, component_id: Option<&str>) -> std::result::Result<Vec<Alert>, String> {
        let mut filtered_alerts = self.alerts.clone();
        
        if let Some(sev) = severity {
            filtered_alerts = filtered_alerts.into_iter()
                .filter(|alert| alert.severity == sev)
                .collect();
        }
        
        if let Some(component) = component_id {
            filtered_alerts = filtered_alerts.into_iter()
                .filter(|alert| alert.component_id == component)
                .collect();
        }
        
        Ok(filtered_alerts)
    }
}

// Mock the MonitoringClient functions for the tests
#[ctor::ctor]
fn setup_test_monitoring_client() {
    use std::sync::Mutex;
    
    // Override the MonitoringClient::connect function for testing
    lazy_static::lazy_static! {
        static ref MOCK_HEALTH_CHECKS: Mutex<Vec<HealthCheck>> = Mutex::new(vec![]);
        static ref MOCK_METRICS: Mutex<Vec<Metric>> = Mutex::new(vec![]);
        static ref MOCK_ALERTS: Mutex<Vec<Alert>> = Mutex::new(vec![]);
    }
    
    // Set up mock data
    *MOCK_HEALTH_CHECKS.lock().unwrap() = create_mock_health_checks();
    *MOCK_METRICS.lock().unwrap() = create_mock_metrics();
    *MOCK_ALERTS.lock().unwrap() = create_mock_alerts();
    
    // Override the MonitoringClient implementation for testing
    #[async_trait]
    impl super::super::monitoring_command::MonitoringClient {
        pub async fn connect() -> std::result::Result<Self, String> {
            let health_client = Arc::new(MockHealthClient::new(MOCK_HEALTH_CHECKS.lock().unwrap().clone()));
            let metrics_client = Arc::new(MockMetricsClient::new(MOCK_METRICS.lock().unwrap().clone()));
            let alerts_client = Arc::new(MockAlertsClient::new(MOCK_ALERTS.lock().unwrap().clone()));
            
            Ok(Self {
                health_client,
                metrics_client,
                alerts_client,
            })
        }
    }
} 