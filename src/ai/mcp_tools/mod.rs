//! Machine Context Protocol (MCP) tools module
//! 
//! This module provides tools for implementing the Machine Context Protocol,
//! which enables AI tools to understand and interact with the development environment.

pub mod context;
pub mod protocol;
pub mod types;
pub mod security;
pub mod persistence;
pub mod sync;
pub mod port_manager;
pub mod registry;
pub mod monitoring;
pub mod llm;

pub use context::MachineContext;
pub use protocol::MCPProtocol;
pub use types::{MCPCommand, MCPResponse, MCPError};
pub use security::{SecurityManager, SecurityContext, SecurityLevel, AuthToken};
pub use persistence::{ContextSnapshot, PersistenceManager};
pub use sync::{SyncManager, SyncConfig, SyncEvent};
pub use port_manager::{PortManager, PortStatus, PortSecurity};
pub use registry::{RegistryService, ToolRegistration, ToolCapability, ToolParameter};
pub use monitoring::{
    MonitoringService,
    MonitoringConfig,
    HealthStatus,
    HealthCheck,
    SystemMetrics,
    PerformanceMetric,
};
pub use llm::{
    LLMService,
    LLMConfig,
    PromptTemplate,
    LLMResponse,
    LLMUsage,
};

/// Re-export commonly used types
pub mod prelude {
    pub use super::{
        MachineContext,
        MCPProtocol,
        MCPCommand,
        MCPResponse,
        MCPError,
        SecurityManager,
        SecurityContext,
        SecurityLevel,
        AuthToken,
        ContextSnapshot,
        PersistenceManager,
        SyncManager,
        SyncConfig,
        SyncEvent,
        PortManager,
        PortStatus,
        PortSecurity,
        RegistryService,
        ToolRegistration,
        ToolCapability,
        ToolParameter,
        MonitoringService,
        MonitoringConfig,
        HealthStatus,
        HealthCheck,
        SystemMetrics,
        PerformanceMetric,
        LLMService,
        LLMConfig,
        PromptTemplate,
        LLMResponse,
        LLMUsage,
    };
} 