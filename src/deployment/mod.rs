//! Deployment module for Squirrel
//!
//! This module provides deployment functionality for managing containers,
//! orchestrating services, and discovering services in the deployment environment.

pub mod container;
pub mod orchestration;
pub mod discovery;

pub use container::{Container, ContainerConfig, ContainerStatus, ContainerError};
pub use orchestration::{Orchestrator, ServiceConfig, ServiceStatus, OrchestrationError};
pub use discovery::{ServiceDiscovery, ServiceRegistration, Endpoint, DiscoveryError};

/// Deployment error types
#[derive(Debug, thiserror::Error)]
pub enum DeploymentError {
    #[error("Container error: {0}")]
    Container(#[from] ContainerError),
    
    #[error("Orchestration error: {0}")]
    Orchestration(#[from] OrchestrationError),
    
    #[error("Discovery error: {0}")]
    Discovery(#[from] DiscoveryError),
    
    #[error("Initialization error: {0}")]
    Initialization(String),
    
    #[error("Shutdown error: {0}")]
    Shutdown(String),
}

/// Deployment configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeploymentConfig {
    /// Container runtime configuration
    pub container_runtime: String,
    
    /// Orchestration platform configuration
    pub orchestration_platform: String,
    
    /// Service discovery configuration
    pub service_discovery: String,
    
    /// Default resource limits
    pub default_resources: container::ResourceLimits,
    
    /// Default health check configuration
    pub default_health_check: Option<container::HealthCheck>,
}

/// Initialize the deployment system
pub async fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize container system
    container::initialize().await?;
    
    // Initialize orchestration system
    orchestration::initialize().await?;
    
    // Initialize service discovery system
    discovery::initialize().await?;
    
    Ok(())
}

/// Shutdown the deployment system
pub async fn shutdown() -> Result<(), Box<dyn std::error::Error>> {
    // Shutdown service discovery system
    discovery::shutdown().await?;
    
    // Shutdown orchestration system
    orchestration::shutdown().await?;
    
    // Shutdown container system
    container::shutdown().await?;
    
    Ok(())
}

/// Get the current deployment configuration
pub fn get_config() -> DeploymentConfig {
    DeploymentConfig {
        container_runtime: "docker".to_string(),
        orchestration_platform: "kubernetes".to_string(),
        service_discovery: "consul".to_string(),
        default_resources: container::ResourceLimits {
            cpu: 1.0,
            memory: 1024 * 1024 * 512, // 512MB
            storage: 1024 * 1024 * 1024, // 1GB
        },
        default_health_check: Some(container::HealthCheck {
            command: vec!["curl".to_string(), "-f".to_string(), "http://localhost:8080/health".to_string()],
            interval: chrono::Duration::seconds(30),
            timeout: chrono::Duration::seconds(5),
            retries: 3,
        }),
    }
} 