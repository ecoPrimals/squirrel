//! Application core functionality for the Squirrel project
//!
//! This module provides the main application structure and core functionality.
//! It serves as the central coordination point for the application, managing
//! configuration, state, and providing access to other core components.

use std::{fmt::Debug, sync::Arc, path::PathBuf};
use serde::{Serialize, Deserialize};
use crate::monitoring::{
    MonitoringService,
    MonitoringConfig,
    MonitoringServiceFactory,
};
use crate::app::context::AppContext;
use crate::app::events::DefaultEventEmitter;
use crate::error::Result;

pub use metrics::Metrics;

/// Command processing functionality including handlers, hooks and processors
pub mod command;
/// Application context management for storing state and configuration
pub mod context;
/// Event system for application-wide messaging and notifications 
pub mod events;
/// Metrics collection and reporting functionality
pub mod metrics;
/// Application-specific monitoring components
pub mod monitoring;
/// Application module
pub mod core;

/// Application core functionality
#[derive(Debug, Clone)]
pub struct App {
    /// Application configuration
    pub config: AppConfig,
    /// Application context
    context: Arc<AppContext>,
    /// Event emitter
    event_emitter: Arc<DefaultEventEmitter>,
    /// Monitoring service (optional)
    monitoring_factory: Option<Arc<MonitoringServiceFactory>>,
    /// Active monitoring service (optional)
    monitoring: Option<Arc<MonitoringService>>,
}

impl App {
    /// Create a new application instance from config
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The monitoring service fails to start
    /// - Required resources are unavailable
    pub async fn from_config(config: AppConfig) -> Result<Self> {
        let event_emitter = Arc::new(DefaultEventEmitter::new());
        let context = Arc::new(AppContext::new(config.clone(), event_emitter.clone()));
        
        let (monitoring_factory, monitoring) = if let Some(ref monitoring_config) = config.monitoring {
            // Create the factory
            let factory = Arc::new(MonitoringServiceFactory::new(monitoring_config.clone()));
            
            // Create and start a service
            let service = factory.create_service_with_config(monitoring_config.clone());
            service.start().await?;
            
            (Some(factory), Some(service))
        } else {
            (None, None)
        };

        Ok(Self {
            config,
            context,
            event_emitter,
            monitoring_factory,
            monitoring,
        })
    }

    /// Get the application context
    #[must_use]
    pub fn context(&self) -> Arc<AppContext> {
        self.context.clone()
    }

    /// Get the event emitter
    #[must_use]
    pub fn event_emitter(&self) -> Arc<DefaultEventEmitter> {
        self.event_emitter.clone()
    }

    /// Get the monitoring service if available
    #[must_use]
    pub fn monitoring(&self) -> Option<Arc<MonitoringService>> {
        self.monitoring.clone()
    }
    
    /// Get the monitoring service factory if available
    #[must_use]
    pub fn monitoring_factory(&self) -> Option<Arc<MonitoringServiceFactory>> {
        self.monitoring_factory.clone()
    }

    /// Start the application
    ///
    /// # Errors
    ///
    /// Returns an error if the monitoring service fails to start
    pub async fn start(&self) -> Result<()> {
        if let Some(monitoring) = &self.monitoring {
            monitoring.start().await?;
        }
        Ok(())
    }

    /// Stop the application
    ///
    /// # Errors
    ///
    /// Returns an error if the monitoring service fails to stop
    pub async fn stop(&self) -> Result<()> {
        if let Some(monitoring) = &self.monitoring {
            monitoring.stop().await?;
        }
        Ok(())
    }
}

/// Application configuration settings
///
/// Contains settings that control the behavior of the application,
/// including data storage location and monitoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application data directory
    pub data_dir: PathBuf,
    /// Monitoring configuration (optional)
    pub monitoring: Option<MonitoringConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data"),
            monitoring: Some(MonitoringConfig::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // No additional imports needed

    #[tokio::test]
    async fn test_app_lifecycle() {
        // Make sure any existing monitoring service is shut down
        let _ = crate::monitoring::shutdown().await;
        
        // Create app config with monitoring enabled
        let temp_dir = std::env::temp_dir().join("squirrel_test_app");
        let _ = std::fs::create_dir_all(&temp_dir);
        
        let config = AppConfig {
            data_dir: temp_dir.clone(),
            monitoring: Some(MonitoringConfig::default()),
        };
        
        // Create and start the app
        let app = App::from_config(config).await.expect("Failed to create app");
        app.start().await.expect("Failed to start app");
        
        // Verify monitoring is available and operational
        let monitoring = app.monitoring();
        assert!(monitoring.is_some(), "Monitoring should be enabled");
        
        let factory = app.monitoring_factory();
        assert!(factory.is_some(), "Monitoring factory should be available");
        
        if let Some(monitoring) = monitoring {
            // Test health checking
            let health = monitoring.get_health().await;
            assert!(health.is_ok(), "Health check should succeed");
            
            // Test metric collection
            let metrics = monitoring.get_metrics().await;
            assert!(metrics.is_ok(), "Metric collection should succeed");
        }
        
        // Stop the app
        app.stop().await.expect("Failed to stop app");
        
        // Make sure monitoring is shut down for other tests
        let _ = crate::monitoring::shutdown().await;
        
        // Clean up test directory
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}

// Re-export commonly used types
pub use command::CommandHandler;
pub use context::Context;
// pub use error::Error as AppError;
// pub use event::Event;
// pub use monitoring::Monitor; 