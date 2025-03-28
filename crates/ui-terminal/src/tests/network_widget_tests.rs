use std::collections::HashMap;
use dashboard_core::data::{NetworkMetrics, NetworkInterface};
use crate::widgets::network::NetworkWidget;

#[test]
fn test_network_widget_creation() {
    // Create test network metrics
    let interfaces = vec![
        NetworkInterface {
            name: "eth0".to_string(),
            is_up: true,
            rx_bytes: 1_000_000,
            tx_bytes: 400_000,
            rx_packets: 800,
            tx_packets: 400,
            rx_errors: 0,
            tx_errors: 0,
        },
        NetworkInterface {
            name: "wlan0".to_string(),
            is_up: false,
            rx_bytes: 500_000,
            tx_bytes: 200_000,
            rx_packets: 400,
            tx_packets: 200,
            rx_errors: 2,
            tx_errors: 1,
        },
    ];

    let metrics = NetworkMetrics {
        interfaces,
        total_rx_bytes: 1_500_000,
        total_tx_bytes: 600_000,
        total_rx_packets: 1200,
        total_tx_packets: 600,
    };

    // Create network widget - this will panic if there's an issue with creation
    let _widget = NetworkWidget::new(&metrics, "Network Statistics");
    
    // Just verify the widget can be created without error
    assert!(true);
}

// This test would ideally test rendering, but we'd need to mock the Frame
// Without introducing additional dependencies, we can do a limited test
#[test]
fn test_network_widget_with_empty_interfaces() {
    // Create network metrics with no interfaces
    let metrics = NetworkMetrics {
        interfaces: Vec::new(),
        total_rx_bytes: 0,
        total_tx_bytes: 0,
        total_rx_packets: 0,
        total_tx_packets: 0,
    };

    // Create network widget - this will panic if there's an issue with creation
    let _widget = NetworkWidget::new(&metrics, "Empty Network");
    
    // Just verify the widget can be created without error
    assert!(true);
}

#[test]
fn test_network_widget_with_multiple_interfaces() {
    // Create test network metrics with multiple interfaces
    let mut interfaces = Vec::new();
    
    // Add multiple interfaces
    for i in 0..5 {
        interfaces.push(NetworkInterface {
            name: format!("eth{}", i),
            is_up: i % 2 == 0, // Alternate up/down status
            rx_bytes: 1_000_000 * (i as u64 + 1),
            tx_bytes: 500_000 * (i as u64 + 1),
            rx_packets: 1000 * (i as u64 + 1),
            tx_packets: 500 * (i as u64 + 1),
            rx_errors: i as u64,
            tx_errors: i as u64,
        });
    }

    let total_rx = interfaces.iter().map(|i| i.rx_bytes).sum();
    let total_tx = interfaces.iter().map(|i| i.tx_bytes).sum();
    let total_rx_packets = interfaces.iter().map(|i| i.rx_packets).sum();
    let total_tx_packets = interfaces.iter().map(|i| i.tx_packets).sum();

    let metrics = NetworkMetrics {
        interfaces,
        total_rx_bytes: total_rx,
        total_tx_bytes: total_tx,
        total_rx_packets: total_rx_packets,
        total_tx_packets: total_tx_packets,
    };

    // Create network widget
    let _widget = NetworkWidget::new(&metrics, "Multiple Interfaces");

    // Verify the metrics data
    assert_eq!(5, metrics.interfaces.len());
    assert_eq!(15_000_000, metrics.total_rx_bytes);
    assert_eq!(7_500_000, metrics.total_tx_bytes);   // Sum of 0.5,1,1.5,2,2.5 million
} 