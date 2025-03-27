use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::plugin::resource_monitor::ResourceMonitor;
use crate::plugin::security::ResourceLimits;

#[tokio::test]
async fn test_resource_monitor_basics() {
    // Create a new resource monitor
    let monitor = ResourceMonitor::new();
    
    // Register a process
    let plugin_id = Uuid::new_v4();
    let process_id = std::process::id(); // Use the current process for testing
    
    // Register the process
    monitor.register_process(
        plugin_id,
        process_id,
        "test-plugin",
        None
    ).await.unwrap();
    
    // Set resource limits
    let limits = ResourceLimits {
        max_memory_bytes: 1024 * 1024 * 1024, // 1GB
        max_cpu_percent: 90,
        max_storage_bytes: 1024 * 1024 * 1024, // 1GB
        max_network_bytes: 1024 * 1024 * 1024, // 1GB
        max_file_handles: 1000,
        max_threads: 100,
    };
    
    monitor.set_resource_limits(plugin_id, limits).await.unwrap();
    
    // Measure resources
    let results = monitor.measure_all_resources().await.unwrap();
    assert!(results.contains_key(&plugin_id));
    
    // Check that resource usage data was stored
    let usage = monitor.get_resource_usage(plugin_id).await;
    assert!(usage.is_some());
    
    // Verify the resource usage has valid values
    let usage = usage.unwrap();
    assert!(usage.memory_bytes > 0);
    assert!(usage.cpu_percent >= 0);
    assert!(usage.threads > 0);
    
    // Unregister the process
    monitor.unregister_process(plugin_id).await.unwrap();
    
    // Verify the process is no longer tracked
    let usage = monitor.get_resource_usage(plugin_id).await;
    assert!(usage.is_none());
}

#[tokio::test]
async fn test_resource_monitor_startup() {
    // Create a resource monitor with a short interval
    let monitor = ResourceMonitor::new()
        .with_interval(Duration::from_millis(100));
    
    // Start monitoring and verify it doesn't cause errors
    monitor.start_monitoring().unwrap();
    
    // Register a process
    let plugin_id = Uuid::new_v4();
    let process_id = std::process::id();
    
    monitor.register_process(
        plugin_id,
        process_id,
        "test-process",
        None
    ).await.unwrap();
    
    // Wait for monitoring to run a few cycles
    sleep(Duration::from_millis(300)).await;
    
    // Check that usage data is being collected
    let usage = monitor.get_resource_usage(plugin_id).await;
    assert!(usage.is_some());
    
    // Clean up
    monitor.unregister_process(plugin_id).await.unwrap();
}

#[tokio::test]
async fn test_monitor_with_limits() {
    // Create a resource monitor
    let monitor = ResourceMonitor::new();
    
    // Register a process
    let plugin_id = Uuid::new_v4();
    let process_id = std::process::id();
    
    monitor.register_process(
        plugin_id,
        process_id,
        "test-limits",
        None
    ).await.unwrap();
    
    // Set very low resource limits (to trigger warnings)
    // These won't cause errors since the monitor only logs warnings
    let limits = ResourceLimits {
        max_memory_bytes: 1,             // 1 byte (will definitely be exceeded)
        max_cpu_percent: 1,              // 1% CPU (likely to be exceeded)
        max_storage_bytes: 1,            // 1 byte (will definitely be exceeded)
        max_network_bytes: 1,            // 1 byte (will definitely be exceeded)
        max_file_handles: 1,             // 1 file handle (likely to be exceeded)
        max_threads: 1,                  // 1 thread (likely to be exceeded)
    };
    
    monitor.set_resource_limits(plugin_id, limits).await.unwrap();
    
    // Measure resources
    monitor.measure_all_resources().await.unwrap();
    
    // Get the resource usage data
    let usage = monitor.get_resource_usage(plugin_id).await;
    assert!(usage.is_some());
    
    // Clean up
    monitor.unregister_process(plugin_id).await.unwrap();
}

#[cfg(all(unix, not(target_os = "macos"), not(feature = "ci")))]
#[tokio::test]
async fn test_monitor_on_linux() {
    use std::process::Command;
    
    // Skip if not running on Linux
    if !cfg!(target_os = "linux") {
        return;
    }
    
    // Spawn a process to monitor (sleep command)
    let child = Command::new("sleep")
        .arg("10")
        .spawn()
        .unwrap();
    
    let pid = child.id();
    
    // Create a resource monitor
    let monitor = ResourceMonitor::new();
    
    // Register the process
    let plugin_id = Uuid::new_v4();
    
    monitor.register_process(
        plugin_id,
        pid,
        "sleep-process",
        None
    ).await.unwrap();
    
    // Measure resources
    let results = monitor.measure_all_resources().await.unwrap();
    assert!(results.contains_key(&plugin_id));
    
    // Get the resource usage data
    let usage = monitor.get_resource_usage(plugin_id).await;
    assert!(usage.is_some());
    
    let usage = usage.unwrap();
    
    // Verify the resource usage values are reasonable for a sleep process
    assert!(usage.memory_bytes > 0);
    assert!(usage.cpu_percent < 10); // sleep shouldn't use much CPU
    assert!(usage.threads > 0);
    
    // Clean up
    monitor.unregister_process(plugin_id).await.unwrap();
}

#[cfg(all(windows, not(feature = "ci")))]
#[tokio::test]
async fn test_monitor_on_windows() {
    use std::process::Command;
    
    // Skip if not running on Windows
    if !cfg!(target_os = "windows") {
        return;
    }
    
    // Spawn a process to monitor (timeout command)
    let child = Command::new("powershell.exe")
        .args(&["-Command", "Start-Sleep -Seconds 10"])
        .spawn()
        .unwrap();
    
    let pid = child.id();
    
    // Create a resource monitor
    let monitor = ResourceMonitor::new();
    
    // Register the process
    let plugin_id = Uuid::new_v4();
    
    monitor.register_process(
        plugin_id,
        pid,
        "sleep-process",
        None
    ).await.unwrap();
    
    // Measure resources
    let results = monitor.measure_all_resources().await.unwrap();
    assert!(results.contains_key(&plugin_id));
    
    // Get the resource usage data
    let usage = monitor.get_resource_usage(plugin_id).await;
    assert!(usage.is_some());
    
    let usage = usage.unwrap();
    
    // Verify the resource usage values are reasonable
    assert!(usage.memory_bytes > 0);
    assert!(usage.cpu_percent < 10); // sleep shouldn't use much CPU
    assert!(usage.threads > 0);
    
    // Clean up
    monitor.unregister_process(plugin_id).await.unwrap();
} 