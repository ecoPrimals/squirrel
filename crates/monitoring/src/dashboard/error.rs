use std::fmt::Debug;
use thiserror::Error;
use squirrel_core::error::SquirrelError;

/// Dashboard errors
#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("Server error: {0}")]
    ServerError(String),
    
    #[error("Component error: {0}")]
    ComponentError(String),
    
    #[error("Client error: {0}")]
    ClientError(String),
    
    #[error("Security error: {0}")]
    SecurityError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Plugin error: {0}")]
    PluginError(String),
}

impl From<DashboardError> for SquirrelError {
    fn from(err: DashboardError) -> Self {
        match err {
            DashboardError::ServerError(msg) => SquirrelError::dashboard(format!("Server error: {}", msg)),
            DashboardError::ComponentError(msg) => SquirrelError::dashboard(format!("Component error: {}", msg)),
            DashboardError::ClientError(msg) => SquirrelError::dashboard(format!("Client error: {}", msg)),
            DashboardError::SecurityError(msg) => SquirrelError::security(format!("Dashboard security error: {}", msg)),
            DashboardError::ConfigError(msg) => SquirrelError::dashboard(format!("Config error: {}", msg)),
            DashboardError::InternalError(msg) => SquirrelError::dashboard(format!("Internal error: {}", msg)),
            DashboardError::PluginError(msg) => SquirrelError::dashboard(format!("Plugin error: {}", msg)),
        }
    }
} 