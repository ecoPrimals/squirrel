// Dashboard module for monitoring system
//
// This module provides functionality for:
// - Real-time metrics visualization
// - Health status display
// - Alert management interface
// - Performance graphs
// - Resource usage charts
// - Custom dashboards
// - Data visualization
// - Interactive controls

use std::fmt::Debug;
use thiserror::Error;
use tracing::error;

/// Module for adapter implementations of dashboard functionality
pub mod adapter;
/// Module for server implementations
pub mod server;
/// Module for security implementations
pub mod security;
/// Module for configuration
pub mod config;
/// Module for secure server implementations
pub mod secure_server;
/// Module for dashboard manager
pub mod manager;

// Re-exports for common types
pub use adapter::{DashboardManagerAdapter, create_dashboard_manager_adapter, create_dashboard_manager_adapter_with_manager};
pub use config::{DashboardConfig, MetricCategory, PanelConfig, PanelType, AlertDisplaySettings};
pub use security::{TlsConfig, AuthConfig, RateLimitConfig, SecurityLoggingConfig, MaskingRule, AuditConfig, OriginVerifier, AuthManager, RateLimiter, DataMaskingManager, AuditLogger};
pub use manager::{DashboardManager, Component, DashboardStats, ClientInfo};

// Dashboard errors
#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("Server error: {0}")]
    Server(String),
    
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
    
    #[error("Client not found: {0}")]
    ClientNotFound(String),
    
    #[error("Security error: {0}")]
    Security(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
} 