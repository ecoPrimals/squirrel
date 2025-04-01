use squirrel_mcp::error::Result;
use squirrel_mcp::monitoring::MonitoringSystem;
use squirrel_mcp::monitoring::metrics::{Metric, MetricType, MetricValue};
use tokio::main;
use std::time::Duration;
use rand::random;

/// Example demonstrating observability features in MCP
#[main]
async fn main() -> Result<()> {
    // Create a new monitoring system
    let monitoring_system = MonitoringSystem::new();
    let metrics_collector = monitoring_system.metrics_collector();
    
    // Register metrics
    metrics_collector.register_metric(Metric::new(
        "requests_total", 
        "Total number of requests", 
        MetricType::Counter, 
        MetricValue::Integer(0)
    ));
    
    metrics_collector.register_metric(Metric::new(
        "active_connections", 
        "Number of active connections", 
        MetricType::Gauge, 
        MetricValue::Integer(0)
    ));
    
    metrics_collector.register_metric(Metric::new(
        "response_time_ms", 
        "Response time in milliseconds", 
        MetricType::Histogram, 
        MetricValue::Histogram(Vec::new())
    ));
    
    // Simulate application activity
    println!("Simulating application activity...");
    for i in 1..=5 {
        // Increment request counter
        metrics_collector.increment_counter("requests_total");
        
        // Set active connections (simulate fluctuation)
        let connections = i * 2;
        metrics_collector.update_metric("active_connections", MetricValue::Integer(connections as i64));
        println!("Active connections: {}", connections);
        
        // Record response time (simulate fluctuation)
        let response_time = 50.0 + (i as f64 * 10.0) + (random::<f64>() * 20.0);
        metrics_collector.observe_histogram("response_time_ms", response_time);
        println!("Request {} completed in {:.2}ms", i, response_time);
        
        // Sleep to simulate time passing
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Get final metrics
    let metrics = metrics_collector.get_all_metrics();
    
    // Display metrics
    println!("\nFinal Metrics:");
    for (name, metric) in metrics {
        let value_str = match &metric.value {
            MetricValue::Integer(i) => i.to_string(),
            MetricValue::Float(f) => format!("{:.2}", f),
            MetricValue::Histogram(h) => format!("{:?}", h),
            _ => format!("{:?}", metric.value),
        };
        println!("{}: {} = {}", name, metric.description, value_str);
    }
    
    println!("\nObservability example completed successfully!");
    Ok(())
} 