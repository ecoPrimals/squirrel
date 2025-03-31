use chrono::{DateTime, Duration, Utc};
use dashboard_core::data::MetricsHistory;
use std::collections::HashMap;

use crate::widgets::chart::{ChartWidget, ChartType, NetworkDataType};

#[test]
fn test_chart_widget_new() {
    // Create test data
    let data = create_test_data();
    
    // Create chart widget
    let widget = ChartWidget::new(&data, "Test Chart");
    
    // Verify default properties
    assert_eq!("Test Chart", widget.title);
    assert_eq!(ChartType::Line, widget.chart_type);
    assert_eq!("", widget.y_label);
    assert_eq!(300, widget.time_range);
    assert_eq!(None, widget.min_y);
    assert_eq!(None, widget.max_y);
}

#[test]
fn test_chart_widget_builders() {
    // Create test data
    let data = create_test_data();
    
    // Create chart widget with all builder methods
    let widget = ChartWidget::new(&data, "Test Chart")
        .chart_type(ChartType::Scatter)
        .y_label("CPU Usage (%)")
        .time_range(600)
        .min_y(0.0)
        .max_y(100.0);
    
    // Verify properties were set correctly
    assert_eq!("Test Chart", widget.title);
    assert_eq!(ChartType::Scatter, widget.chart_type);
    assert_eq!("CPU Usage (%)", widget.y_label);
    assert_eq!(600, widget.time_range);
    assert_eq!(Some(0.0), widget.min_y);
    assert_eq!(Some(100.0), widget.max_y);
}

#[test]
fn test_chart_widget_with_empty_data() {
    // Create empty data
    let data: Vec<(DateTime<Utc>, f64)> = Vec::new();
    
    // Create chart widget
    let widget = ChartWidget::new(&data, "Empty Chart");
    
    // No assertion needed - we just verify it doesn't panic
    // A real terminal render test would verify it shows "No data available"
}

#[test]
fn test_chart_widget_from_dashboard_cpu() {
    // Create test metrics history
    let history = create_test_history();
    
    // Create chart widget from dashboard CPU history
    let widget = ChartWidget::from_dashboard_cpu(&history, "CPU Chart");
    
    // Verify it was created with CPU data
    assert_eq!("CPU Chart", widget.title);
}

#[test]
fn test_chart_widget_from_dashboard_memory() {
    // Create test metrics history
    let history = create_test_history();
    
    // Create chart widget from dashboard memory history
    let widget = ChartWidget::from_dashboard_memory(&history, "Memory Chart");
    
    // Verify it was created with memory data
    assert_eq!("Memory Chart", widget.title);
}

#[test]
fn test_chart_widget_from_dashboard_network() {
    // Create test metrics history
    let history = create_test_history();
    
    // Create chart widget from dashboard network history with RX data
    let rx_widget = ChartWidget::from_dashboard_network(
        &history, 
        NetworkDataType::Rx, 
        "Network RX Chart"
    );
    
    // Verify it was created with correct title and data type
    assert_eq!("Network RX Chart", rx_widget.title);
    
    // Create chart widget from dashboard network history with TX data
    let tx_widget = ChartWidget::from_dashboard_network(
        &history, 
        NetworkDataType::Tx, 
        "Network TX Chart"
    );
    
    // Verify it was created with correct title and data type
    assert_eq!("Network TX Chart", tx_widget.title);
}

#[test]
fn test_chart_widget_calculate_y_range() {
    // Create test data with known min/max values: 10, 20, 15, 25, 5
    let data = vec![
        (Utc::now() - Duration::seconds(40), 10.0),
        (Utc::now() - Duration::seconds(30), 20.0),
        (Utc::now() - Duration::seconds(20), 15.0),
        (Utc::now() - Duration::seconds(10), 25.0),
        (Utc::now(), 5.0),
    ];
    
    // Create chart widget without min/max specified
    let widget_default = ChartWidget::new(&data, "Test Chart");
    
    // Call calculate_y_range directly using the exposed method for testing
    let (min_y, max_y) = widget_default.calculate_y_range();
    
    // Verify calculated range (min should be 0.0, max should be slightly more than 25.0)
    assert!(min_y <= 0.0, "Minimum Y should be 0.0 or less");
    assert!(max_y >= 25.0, "Maximum Y should be 25.0 or more");
    
    // Create chart widget with min/max specified
    let widget_custom = ChartWidget::new(&data, "Test Chart")
        .min_y(-10.0)
        .max_y(50.0);
    
    // Call calculate_y_range directly
    let (min_y, max_y) = widget_custom.calculate_y_range();
    
    // Verify calculated range uses specified min/max
    assert_eq!(-10.0, min_y);
    assert_eq!(50.0, max_y);
}

#[test]
fn test_chart_widget_with_network_data() {
    // Create test network data
    let now = Utc::now();
    let network_data = vec![
        (now - Duration::seconds(40), (1000u64, 500u64)),
        (now - Duration::seconds(30), (2000u64, 1000u64)),
        (now - Duration::seconds(20), (3000u64, 1500u64)),
        (now - Duration::seconds(10), (4000u64, 2000u64)),
        (now, (5000u64, 2500u64)),
    ];
    
    // Create chart widget for RX data
    let rx_widget = ChartWidget::new_network(
        &network_data, 
        NetworkDataType::Rx, 
        "Network RX Chart"
    );
    
    // Verify it was created correctly
    assert_eq!("Network RX Chart", rx_widget.title);
    assert_eq!("RX Bytes", rx_widget.y_label);
    
    // Create chart widget for TX data
    let tx_widget = ChartWidget::new_network(
        &network_data, 
        NetworkDataType::Tx, 
        "Network TX Chart"
    );
    
    // Verify it was created correctly
    assert_eq!("Network TX Chart", tx_widget.title);
    assert_eq!("TX Bytes", tx_widget.y_label);
}

/// Create test time series data
fn create_test_data() -> Vec<(DateTime<Utc>, f64)> {
    let now = Utc::now();
    vec![
        (now - Duration::seconds(50), 10.0),
        (now - Duration::seconds(40), 20.0),
        (now - Duration::seconds(30), 15.0),
        (now - Duration::seconds(20), 25.0),
        (now - Duration::seconds(10), 18.0),
        (now, 22.0),
    ]
}

/// Create test metrics history
fn create_test_history() -> MetricsHistory {
    let now = Utc::now();
    
    let mut history = MetricsHistory::default();
    
    // Add CPU data
    history.cpu.push((now - Duration::seconds(50), 10.0));
    history.cpu.push((now - Duration::seconds(40), 20.0));
    history.cpu.push((now - Duration::seconds(30), 15.0));
    history.cpu.push((now - Duration::seconds(20), 25.0));
    history.cpu.push((now - Duration::seconds(10), 18.0));
    history.cpu.push((now, 22.0));
    
    // Add memory data
    history.memory.push((now - Duration::seconds(50), 30.0));
    history.memory.push((now - Duration::seconds(40), 35.0));
    history.memory.push((now - Duration::seconds(30), 40.0));
    history.memory.push((now - Duration::seconds(20), 38.0));
    history.memory.push((now - Duration::seconds(10), 42.0));
    history.memory.push((now, 45.0));
    
    // Add network data
    history.network.push((now - Duration::seconds(50), (1000u64, 500u64)));
    history.network.push((now - Duration::seconds(40), (2000u64, 1000u64)));
    history.network.push((now - Duration::seconds(30), (3000u64, 1500u64)));
    history.network.push((now - Duration::seconds(20), (4000u64, 2000u64)));
    history.network.push((now - Duration::seconds(10), (5000u64, 2500u64)));
    history.network.push((now, (6000u64, 3000u64)));
    
    history
} 