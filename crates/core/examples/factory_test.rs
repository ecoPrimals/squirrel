// This example has been commented out because it depends on the squirrel-monitoring crate
// which may have changed its API or structure.
//
// To enable this example:
// 1. Update the imports and usages according to the current monitoring module
// 2. Add the appropriate dependency in Cargo.toml
// 3. Uncomment the code

/*
use std::error::Error;
use squirrel_monitoring::{
    MonitoringConfig, MonitoringServiceFactory,
    alerts::AlertConfig,
    health::HealthConfig,
    metrics::MetricConfig,
    network::NetworkConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Testing MonitoringServiceFactory...");
    
    // Create a factory with default configuration
    // Explicitly specify () as the generic parameter for NotificationManagerTrait
    let factory = MonitoringServiceFactory::<()>::new();
    println!("✅ Successfully created factory with default config");
    
    // Create a service (note: create_service doesn't return a Future)
    let service = factory.create_service();
    println!("✅ Successfully created service");
    
    // Start the service
    service.start().await?;
    println!("✅ Successfully started service");
    
    // Check health status
    let health_status = service.health_status().await?;
    println!("✅ Health status: {:?}", health_status);
    
    // Stop service
    service.stop().await?;
    println!("✅ Successfully stopped service");
    
    // Create custom configuration
    let custom_config = MonitoringConfig {
        health: HealthConfig {
            interval: 5, // 5 seconds interval
            ..Default::default()
        },
        metrics: MetricConfig {
            interval: 10, // 10 seconds interval
            ..Default::default()
        },
        alerts: AlertConfig::default(),
        network: NetworkConfig {
            interval: 15, // 15 seconds interval
            ..Default::default()
        },
        intervals: Default::default(),
    };
    
    // Create factory with custom configuration
    let factory = MonitoringServiceFactory::<()>::with_config(custom_config);
    println!("✅ Successfully created factory with custom config");
    
    // Create and start a service with custom config
    let service = factory.create_service();
    service.start().await?;
    println!("✅ Successfully started service with custom config");
    
    // Check health status
    let health_status = service.health_status().await?;
    println!("✅ Health status with custom config: {:?}", health_status);
    
    // Stop service
    service.stop().await?;
    println!("✅ Successfully stopped service with custom config");
    
    println!("All tests completed successfully!");
    
    Ok(())
}
*/

// Placeholder main function
fn main() {
    println!("This example has been commented out as it needs updating.");
} 