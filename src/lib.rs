// Squirrel AI system
//
// This crate provides the main integration for the Squirrel AI system.
// It includes various subsystems including context management, plugins, and MCP.

// Re-exports from crates
pub use squirrel_context;
pub use squirrel_interfaces;
pub use squirrel_core;
pub use squirrel_mcp;

// Add direct re-exports from MCP
pub use squirrel_mcp::{
    security::{
        types::{Action, Permission, PermissionContext, PermissionScope, Role},
        SecurityManager, SecurityManagerImpl,
        Session
    },
    types::{EncryptionFormat, SecurityLevel},
    error::{MCPError, Result},
    context_manager::Context,
    plugins::interfaces::{Plugin, PluginMetadata, PluginStatus, McpPlugin, PluginManagerInterface},
    plugins::lifecycle::{PluginLifecycleHook, CompositePluginLifecycleHook},
    plugins::adapter::{ToolPluginAdapter, ToolPluginFactory},
    plugins::discovery::{PluginProxyExecutor, PluginDiscoveryManager},
    config::McpConfig as MCPConfig
};

// Export Credentials from our adapter
pub use crate::adapter::Credentials;

/// Adapter for MCP operations
pub mod adapter;
pub use adapter::{MCPAdapter, MCPInterface};

/// Prelude for convenient imports
pub mod prelude {
    // Core types and interfaces from MCP
    pub use squirrel_mcp::context_manager::{Context, ContextManager};
    pub use squirrel_mcp::types::{MessageType, ProtocolState, SecurityLevel};
    pub use squirrel_mcp::protocol::{MCPProtocolBase, MCPProtocol};
    pub use squirrel_mcp::protocol::adapter::MCPProtocolAdapter;

    // Security features
    pub use squirrel_mcp::security::SecurityManager;
    
    // Tool management
    pub use squirrel_mcp::tool::{Tool, ToolManager, ToolState};
    pub use squirrel_mcp::tool::lifecycle::{LifecycleEvent, BasicLifecycleHook};
    
    // Monitoring system
    pub use squirrel_mcp::monitoring::{MetricsCollector, AlertManager, MonitoringSystem};
    pub use squirrel_mcp::monitoring::alerts::{Alert, AlertSeverity, AlertState};
    
    // Error handling
    pub use squirrel_mcp::error::{Result as MCPResult, MCPError};
    
    // RBAC system
    pub use squirrel_mcp::security::rbac::{
        RBACManager, 
        ValidationResult, 
        ValidationRule, 
        InheritanceType,
        ValidationAuditRecord
    };
    pub use squirrel_mcp::security::{Action, Permission, PermissionContext, PermissionScope, Role};

    // Plugin integration
    pub use squirrel_mcp::plugins::{ToolPluginAdapter, ToolPluginFactory, PluginDiscoveryManager, PluginProxyExecutor};
    pub use squirrel_mcp::plugins::lifecycle::{PluginLifecycleHook, CompositePluginLifecycleHook};
    pub use squirrel_mcp::plugins::interfaces::{Plugin, PluginMetadata, PluginStatus, McpPlugin, PluginManagerInterface};
} 