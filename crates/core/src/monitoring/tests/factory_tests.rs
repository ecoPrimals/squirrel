#![allow(unused_imports)]
use crate::error::Result;
use crate::monitoring::{
    MonitoringConfig, MonitoringIntervals, MonitoringServiceFactory,
    alerts::AlertConfig,
    health::HealthConfig,
    metrics::MetricConfig,
    network::NetworkConfig,
};

#[tokio::test]
async fn test_monitoring_service_factory_creates_service() -> Result<()> {
    // Create a monitoring service factory with default config
    let factory = MonitoringServiceFactory::new();
    
    // Create a service
    let service = factory.create_service();
    
    // Start the service
    service.start().await?;
    
    // Verify service is running by checking health
    let _health = service.health_status().await?;
    
    // Stop the service
    service.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_monitoring_service_factory_with_custom_config() -> Result<()> {
    // Create a custom config
    let config = MonitoringConfig {
        intervals: MonitoringIntervals {
            health_check: 2,
            metric_collection: 5,
            network_monitoring: 10,
        },
        health: HealthConfig::default(),
        metrics: MetricConfig::default(),
        alerts: AlertConfig::default(),
        network: NetworkConfig::default(),
    };
    
    // Create a monitoring service factory with custom config
    let factory = MonitoringServiceFactory::with_config(config.clone());
    
    // Create a service
    let service = factory.create_service_with_config(config);
    
    // Start the service
    service.start().await?;
    
    // Verify service is running by checking health
    let _health = service.health_status().await?;
    
    // Stop the service
    service.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_start_service_method() -> Result<()> {
    // Create a monitoring service factory
    let factory = MonitoringServiceFactory::new();
    
    // Start a service
    let service = factory.start_service().await?;
    
    // Verify service is running by checking health
    let _health = service.health_status().await?;
    
    // Stop the service
    service.stop().await?;
    
    Ok(())
} 