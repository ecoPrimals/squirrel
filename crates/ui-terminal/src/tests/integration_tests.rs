use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use dashboard_core::{
    DashboardService,
    DashboardUpdate,
    service::DefaultDashboardService,
    config::DashboardConfig,
    data::{DashboardData, SystemSnapshot, NetworkSnapshot, AlertsSnapshot, MetricsSnapshot},
};

use crate::{
    TuiDashboard,
    app::App,
    adapter::MonitoringToDashboardAdapter,
};

/// Test that the terminal UI can be created with a dashboard service
#[tokio::test]
async fn test_tui_dashboard_can_be_created_with_service() {
    // Create a dashboard service
    let config = DashboardConfig::default()
        .with_update_interval(5)
        .with_max_history_points(100);
    
    let (dashboard_service, _rx) = DefaultDashboardService::new(config);
    
    // Create a terminal UI
    let tui = TuiDashboard::new(dashboard_service);
    
    // Check that the terminal UI was created
    assert!(tui.dashboard_service.is_some());
}

/// Test that the dashboard data can be updated and retrieved
#[tokio::test]
async fn test_dashboard_data_can_be_updated_and_retrieved() {
    // Create a dashboard service
    let config = DashboardConfig::default()
        .with_update_interval(5)
        .with_max_history_points(100);
    
    let (dashboard_service, _rx) = DefaultDashboardService::new(config);
    
    // Create test data
    let test_data = create_test_dashboard_data();
    
    // Update the dashboard with test data
    dashboard_service.update_dashboard_data(test_data.clone()).await.unwrap();
    
    // Retrieve the dashboard data
    let retrieved_data = dashboard_service.get_dashboard_data().await.unwrap();
    
    // Check that the data was updated correctly
    assert_eq!(retrieved_data.system.cpu_usage, test_data.system.cpu_usage);
    assert_eq!(retrieved_data.network.rx_bytes, test_data.network.rx_bytes);
    assert_eq!(retrieved_data.metrics.counters.get("protocol.messages"), 
              test_data.metrics.counters.get("protocol.messages"));
}

/// Test that the app state can be updated with dashboard data
#[test]
fn test_app_state_can_be_updated_with_dashboard_data() {
    // Create an app
    let mut app = App::new();
    
    // Create test data
    let test_data = create_test_dashboard_data();
    
    // Update the app with test data
    app.update_dashboard_data(test_data.clone());
    
    // Check that the data was updated correctly
    assert!(app.dashboard_data().is_some());
    let app_data = app.dashboard_data().unwrap();
    assert_eq!(app_data.system.cpu_usage, test_data.system.cpu_usage);
}

/// Test that the monitoring adapter can collect data
#[test]
fn test_monitoring_adapter_can_collect_data() {
    // Create a monitoring adapter
    let mut adapter = MonitoringToDashboardAdapter::new();
    
    // Collect data
    let data = adapter.collect_dashboard_data();
    
    // Check that data was collected
    assert!(data.system.cpu_usage >= 0.0 && data.system.cpu_usage <= 100.0);
    assert!(data.system.memory_used <= data.system.memory_total);
    assert!(data.metrics.counters.contains_key("protocol.messages"));
    assert!(data.metrics.counters.contains_key("mcp.requests"));
}

/// Test that the app can handle dashboard updates
#[test]
fn test_app_can_handle_dashboard_updates() {
    // Create an app
    let mut app = App::new();
    
    // Create test data
    let test_data = create_test_dashboard_data();
    
    // Create an update
    let update = DashboardUpdate::FullUpdate(test_data.clone());
    
    // Handle the update
    app.handle_update(update);
    
    // Check that the data was updated correctly
    assert!(app.dashboard_data().is_some());
    let app_data = app.dashboard_data().unwrap();
    assert_eq!(app_data.system.cpu_usage, test_data.system.cpu_usage);
}

/// Test that the TuiDashboard can process updates through channels
#[tokio::test]
async fn test_tui_dashboard_can_process_updates() {
    // Create a dashboard service
    let config = DashboardConfig::default()
        .with_update_interval(5)
        .with_max_history_points(100);
    
    let (dashboard_service, mut rx) = DefaultDashboardService::new(config);
    
    // Create a terminal UI
    let mut tui = TuiDashboard::new_from_default_service((dashboard_service.clone(), rx));
    
    // Create test data
    let test_data = create_test_dashboard_data();
    
    // Update the dashboard with test data
    dashboard_service.update_dashboard_data(test_data.clone()).await.unwrap();
    
    // Allow time for the update to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check that the app state was updated
    assert!(tui.app.dashboard_data().is_some());
}

/// Create test dashboard data
fn create_test_dashboard_data() -> DashboardData {
    use std::collections::HashMap;
    use chrono::Utc;
    
    // Create system snapshot
    let system = SystemSnapshot {
        cpu_usage: 45.0,
        memory_used: 4_000_000_000,
        memory_total: 16_000_000_000,
        disk_used: 500_000_000_000,
        disk_total: 1_000_000_000_000,
        load_average: [1.0, 1.5, 2.0],
        uptime: 3600,
    };
    
    // Create network snapshot
    let mut interfaces = HashMap::new();
    interfaces.insert("eth0".to_string(), crate::dashboard_core::data::InterfaceStats {
        name: "eth0".to_string(),
        rx_bytes: 1_000_000,
        tx_bytes: 400_000,
        rx_packets: 800,
        tx_packets: 400,
        is_up: true,
    });
    
    let network = NetworkSnapshot {
        rx_bytes: 1_500_000,
        tx_bytes: 500_000,
        rx_packets: 1000,
        tx_packets: 500,
        interfaces,
    };
    
    // Create alerts snapshot
    let alerts = AlertsSnapshot {
        active: Vec::new(),
        recent: Vec::new(),
        counts: HashMap::new(),
    };
    
    // Create metrics snapshot
    let mut counters = HashMap::new();
    counters.insert("protocol.messages".to_string(), 1000u64);
    counters.insert("protocol.transactions".to_string(), 500u64);
    counters.insert("protocol.errors".to_string(), 10u64);
    counters.insert("mcp.requests".to_string(), 600u64);
    counters.insert("mcp.responses".to_string(), 400u64);
    counters.insert("mcp.transactions".to_string(), 500u64);
    counters.insert("mcp.connection_errors".to_string(), 5u64);
    counters.insert("mcp.protocol_errors".to_string(), 5u64);
    
    let mut gauges = HashMap::new();
    gauges.insert("protocol.message_rate".to_string(), 10.0);
    gauges.insert("protocol.transaction_rate".to_string(), 5.0);
    gauges.insert("protocol.error_rate".to_string(), 1.0);
    gauges.insert("mcp.success_rate".to_string(), 99.0);
    
    let mut latency_histogram = Vec::new();
    for i in 0..20 {
        latency_histogram.push(i as f64 * 5.0);
    }
    
    let mut histograms = HashMap::new();
    histograms.insert("protocol.latency".to_string(), latency_histogram);
    
    let metrics = MetricsSnapshot {
        values: HashMap::new(),
        counters,
        gauges,
        histograms,
    };
    
    // Create dashboard data
    DashboardData {
        system,
        network,
        alerts,
        metrics,
        timestamp: Utc::now(),
    }
} 