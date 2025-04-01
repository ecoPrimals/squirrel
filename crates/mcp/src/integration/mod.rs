//! Integration adapters for connecting MCP with other components
//! 
//! This module contains adapter implementations that connect MCP with various other
//! components in the system. These adapters follow the adapter pattern described in
//! the MCP Integration Guide.

/// Core system adapter for MCP integration
///
/// Provides integration between the MCP protocol and core system components.
/// This adapter handles message routing, state management, authentication,
/// and other core integration points between MCP and the main application.
// pub mod core_adapter; <-- Remove this

// New module declarations
pub mod types;
pub mod helpers;
pub mod adapter;
pub mod alert_recovery_adapter;
pub mod health_check_adapter;
pub mod monitoring_bridge_impl;

// Re-export the main adapter
pub use self::adapter::CoreMCPAdapter;

// Tests module
#[cfg(test)]
pub mod tests;

/// Re-exports of key integration traits and types
pub mod prelude {
    pub use crate::protocol::MCPProtocol;
    pub use crate::integration::types::MessageHandler; // Updated path
    pub use crate::protocol::types::MCPMessage; // Use MCPMessage from protocol::types
    pub use crate::types::MCPResponse; // Use MCPResponse from crate::types
    pub use crate::security::{AuthCredentials, Action}; // Import from security directly
    pub use crate::error::{MCPError, Result as MCPResult};
    
    pub use super::adapter::CoreMCPAdapter; // Updated path
}

// Re-export core components needed by external crates using the adapter
// Clean up imports to avoid circular dependencies
pub use crate::error::{MCPError, Result as MCPResult}; // Use MCPError and Result from error module
pub use crate::security::{AuthCredentials, SecurityLevel}; // Import from security directly

// Re-export transport-related types
// ... existing code ... 

// Module exports
pub use self::types::{CoreState, StateUpdate};
pub use self::alert_recovery_adapter::AlertToRecoveryAdapter;
pub use self::health_check_adapter::{ResilienceHealthCheckAdapter, create_metrics_from_health_result};
pub use self::monitoring_bridge_impl::{HealthMonitoringBridge, HealthMonitoringBridgeConfig}; 