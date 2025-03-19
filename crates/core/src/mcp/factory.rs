use std::sync::Arc;
use super::{MCP, MCPConfig};

/// Factory for creating MCP instances
#[derive(Debug)]
pub struct MCPFactory {
    /// Configuration for creating MCP instances
    config: MCPConfig,
}

impl MCPFactory {
    /// Create a new MCPFactory with default configuration
    pub fn new() -> Self {
        Self {
            config: MCPConfig::default(),
        }
    }
    
    /// Create a new MCPFactory with the given configuration
    pub const fn with_config(config: MCPConfig) -> Self {
        Self { config }
    }
    
    /// Create a new MCP instance
    pub fn create_mcp(&self) -> Arc<MCP> {
        Arc::new(MCP::new(self.config.clone()))
    }
}

impl Default for MCPFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new MCP factory with default configuration
pub fn create_mcp_factory() -> MCPFactory {
    MCPFactory::new()
}

/// Create a new MCP instance with default configuration
pub fn create_mcp() -> Arc<MCP> {
    let factory = create_mcp_factory();
    factory.create_mcp()
} 