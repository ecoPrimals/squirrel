// Copyright DataScienceBioLab 2024
// For MCP Plugin System Infrastructure
//
// This module implements plugin system integration for MCP

// Local plugin interfaces to avoid circular dependencies
pub mod interfaces;

// Module implementations
pub mod adapter;
pub mod lifecycle;
pub mod discovery;
pub mod examples;

// Re-export key types/interfaces
pub use interfaces::{Plugin, PluginMetadata, PluginStatus, McpPlugin, PluginManagerInterface};
pub use adapter::{ToolPluginAdapter, ToolPluginFactory};
pub use lifecycle::{PluginLifecycleHook, CompositePluginLifecycleHook};

// Re-export discovery manager and executor
pub use self::discovery::{PluginDiscoveryManager, PluginProxyExecutor};

#[cfg(test)]
mod tests; 