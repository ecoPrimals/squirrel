use std::collections::HashMap;
use dashboard_core::data::{
    Metrics, CpuMetrics, MemoryMetrics, NetworkMetrics, DiskMetrics, 
    NetworkInterface, DiskUsage, MetricsHistory
};
use crate::widgets::metrics::MetricsWidget;

#[test]
fn test_metrics_widget_creation() {
    // Create test metrics
    let metrics = create_test_metrics();
    
    // Create metrics widget
    let _widget = MetricsWidget::new(&metrics, "System Metrics");
    
    // This test just verifies that the widget can be created without panicking
    assert!(true);
}

#[test]
fn test_metrics_widget_with_high_usage() {
    // Create test metrics with high CPU and memory usage
    let mut metrics = create_test_metrics();
    
    // Set high CPU usage
    metrics.cpu.usage = 95.0;
    metrics.cpu.cores = vec![90.0, 95.0, 98.0, 97.0];
    
    // Set high memory usage
    metrics.memory.used = 15_000_000_000;
    metrics.memory.available = 1_000_000_000;
    metrics.memory.free = 1_000_000_000;
    
    // Create metrics widget
    let _widget = MetricsWidget::new(&metrics, "High Usage Metrics");
    
    // This test just verifies that the widget can be created with high usage values
    assert_eq!(95.0, metrics.cpu.usage);
    assert!(metrics.memory.used > metrics.memory.free);
}

#[test]
fn test_metrics_widget_with_empty_disk_usage() {
    // Create test metrics with no disk usage
    let mut metrics = create_test_metrics();
    
    // Set empty disk usage
    metrics.disk.usage = HashMap::new();
    
    // Create metrics widget
    let _widget = MetricsWidget::new(&metrics, "No Disk Metrics");
    
    // This test verifies the widget can handle no disk usage
    assert_eq!(0, metrics.disk.usage.len());
}

#[test]
fn test_metrics_widget_with_multiple_disk_mounts() {
    // Create test metrics with multiple disk mounts
    let mut metrics = create_test_metrics();
    
    // Clear existing disk usage and add multiple mounts
    metrics.disk.usage = HashMap::new();
    
    // Add multiple mounts
    for i in 0..5 {
        let mount_point = format!("/mnt/disk{}", i);
        let total = 1_000_000_000_000; // 1 TB
        let used = (i as u64 + 1) * 100_000_000_000; // 100GB * (i+1)
        let free = total - used;
        let used_percentage = (used as f64 / total as f64) * 100.0;
        
        metrics.disk.usage.insert(mount_point.clone(), DiskUsage {
            mount_point,
            total,
            used,
            free,
            used_percentage,
        });
    }
    
    // Create metrics widget
    let _widget = MetricsWidget::new(&metrics, "Multi-Disk Metrics");
    
    // This test verifies the widget can handle multiple disk mounts
    assert_eq!(5, metrics.disk.usage.len());
}

/// Creates test metrics with reasonable defaults
fn create_test_metrics() -> Metrics {
    // Create CPU metrics
    let cpu = CpuMetrics {
        usage: 45.0,
        cores: vec![40.0, 50.0, 45.0, 50.0],
        temperature: Some(65.0),
        load: [1.0, 1.5, 2.0],
    };
    
    // Create memory metrics
    let memory = MemoryMetrics {
        total: 16_000_000_000,
        used: 8_000_000_000,
        available: 8_000_000_000,
        free: 8_000_000_000,
        swap_used: 500_000_000,
        swap_total: 8_000_000_000,
    };
    
    // Create network interfaces
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
    ];
    
    // Create network metrics
    let network = NetworkMetrics {
        interfaces,
        total_rx_bytes: 1_000_000,
        total_tx_bytes: 400_000,
        total_rx_packets: 800,
        total_tx_packets: 400,
    };
    
    // Create disk usage
    let mut usage = HashMap::new();
    usage.insert("root".to_string(), DiskUsage {
        mount_point: "/".to_string(),
        total: 1_000_000_000_000,
        used: 500_000_000_000,
        free: 500_000_000_000,
        used_percentage: 50.0,
    });
    
    // Create disk metrics
    let disk = DiskMetrics {
        usage,
        total_reads: 1000,
        total_writes: 500,
        read_bytes: 2_000_000,
        written_bytes: 1_000_000,
    };
    
    // Create metrics history
    let history = MetricsHistory::default();
    
    // Return metrics
    Metrics {
        cpu,
        memory,
        network,
        disk,
        history,
    }
} 