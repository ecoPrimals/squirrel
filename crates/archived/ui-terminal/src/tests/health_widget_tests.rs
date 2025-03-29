use crate::widgets::health::{HealthWidget, HealthCheck, HealthStatus};
use dashboard_core::health::{HealthCheck as DashboardHealthCheck, HealthStatus as DashboardHealthStatus};
use chrono::Utc;

#[test]
fn test_health_widget_creation() {
    // Create test health checks
    let health_checks = vec![
        HealthCheck::new("Service A", HealthStatus::Healthy),
        HealthCheck::new("Service B", HealthStatus::Warning)
            .with_details("High CPU usage"),
        HealthCheck::new("Service C", HealthStatus::Critical)
            .with_details("Service unreachable"),
    ];
    
    // Create health widget
    let _widget = HealthWidget::new(&health_checks, "System Health");
    
    // This test just verifies that the widget can be created without panicking
    assert!(true);
}

#[test]
fn test_health_widget_overall_health() {
    // Case 1: All healthy
    let all_healthy = vec![
        HealthCheck::new("Service A", HealthStatus::Healthy),
        HealthCheck::new("Service B", HealthStatus::Healthy),
    ];
    
    let widget = HealthWidget::new(&all_healthy, "All Healthy");
    assert_eq!(HealthStatus::Healthy, widget.overall_health());
    
    // Case 2: Some warnings
    let some_warnings = vec![
        HealthCheck::new("Service A", HealthStatus::Healthy),
        HealthCheck::new("Service B", HealthStatus::Warning),
        HealthCheck::new("Service C", HealthStatus::Healthy),
    ];
    
    let widget = HealthWidget::new(&some_warnings, "Some Warnings");
    assert_eq!(HealthStatus::Warning, widget.overall_health());
    
    // Case 3: Some critical
    let some_critical = vec![
        HealthCheck::new("Service A", HealthStatus::Healthy),
        HealthCheck::new("Service B", HealthStatus::Warning),
        HealthCheck::new("Service C", HealthStatus::Critical),
    ];
    
    let widget = HealthWidget::new(&some_critical, "Some Critical");
    assert_eq!(HealthStatus::Critical, widget.overall_health());
    
    // Case 4: Empty
    let empty = Vec::new();
    let widget = HealthWidget::new(&empty, "Empty");
    assert_eq!(HealthStatus::Unknown, widget.overall_health());
    
    // Case 5: Unknown
    let unknown = vec![
        HealthCheck::new("Service A", HealthStatus::Unknown),
        HealthCheck::new("Service B", HealthStatus::Unknown),
    ];
    
    let widget = HealthWidget::new(&unknown, "Unknown");
    assert_eq!(HealthStatus::Unknown, widget.overall_health());
}

#[test]
fn test_health_check_creation() {
    // Test basic creation
    let check = HealthCheck::new("Service", HealthStatus::Healthy);
    assert_eq!("Service", check.name);
    assert_eq!(HealthStatus::Healthy, check.status);
    assert!(check.details.is_none());
    assert!(check.last_check.is_some());
    
    // Test with details
    let check = HealthCheck::new("Service", HealthStatus::Warning)
        .with_details("Some details");
    assert_eq!("Service", check.name);
    assert_eq!(HealthStatus::Warning, check.status);
    assert_eq!(Some("Some details".to_string()), check.details);
    
    // Test with custom time
    let time = Utc::now() - chrono::Duration::hours(1);
    let check = HealthCheck::new("Service", HealthStatus::Critical)
        .with_last_check(time);
    assert_eq!("Service", check.name);
    assert_eq!(HealthStatus::Critical, check.status);
    assert_eq!(Some(time), check.last_check);
}

#[test]
fn test_health_status_conversion() {
    // Test status conversion from dashboard health status
    assert_eq!(
        HealthStatus::Healthy,
        HealthStatus::from_dashboard_status(DashboardHealthStatus::Ok)
    );
    
    assert_eq!(
        HealthStatus::Warning,
        HealthStatus::from_dashboard_status(DashboardHealthStatus::Warning)
    );
    
    assert_eq!(
        HealthStatus::Critical,
        HealthStatus::from_dashboard_status(DashboardHealthStatus::Critical)
    );
    
    assert_eq!(
        HealthStatus::Unknown,
        HealthStatus::from_dashboard_status(DashboardHealthStatus::Unknown)
    );
}

#[test]
fn test_health_check_from_dashboard() {
    // Create a dashboard health check
    let dashboard_check = DashboardHealthCheck {
        name: "Test Service".to_string(),
        status: DashboardHealthStatus::Warning,
        details: "Service is slow".to_string(),
    };
    
    // Convert to our health check
    let check = HealthCheck::from_dashboard(&dashboard_check);
    
    // Verify conversion
    assert_eq!("Test Service", check.name);
    assert_eq!(HealthStatus::Warning, check.status);
    assert_eq!(Some("Service is slow".to_string()), check.details);
    assert!(check.last_check.is_some());
}

#[test]
fn test_health_check_as_lines() {
    // Create a health check with details and time
    let check = HealthCheck::new("Test Service", HealthStatus::Warning)
        .with_details("Service is slow");
    
    // Get lines
    let lines = check.as_lines();
    
    // Should have 3 lines: name/status, details, and time
    assert_eq!(3, lines.len());
}

#[test]
fn test_from_dashboard_health_checks() {
    // Create dashboard health checks
    let dashboard_checks = vec![
        DashboardHealthCheck {
            name: "Service A".to_string(),
            status: DashboardHealthStatus::Ok,
            details: "Running normally".to_string(),
        },
        DashboardHealthCheck {
            name: "Service B".to_string(),
            status: DashboardHealthStatus::Warning,
            details: "High load".to_string(),
        },
    ];
    
    // Convert to our health checks
    let checks = HealthWidget::from_dashboard(&dashboard_checks, "Health");
    
    // Verify conversion
    assert_eq!(2, checks.len());
    assert_eq!("Service A", checks[0].name);
    assert_eq!(HealthStatus::Healthy, checks[0].status);
    assert_eq!(Some("Running normally".to_string()), checks[0].details);
    
    assert_eq!("Service B", checks[1].name);
    assert_eq!(HealthStatus::Warning, checks[1].status);
    assert_eq!(Some("High load".to_string()), checks[1].details);
} 