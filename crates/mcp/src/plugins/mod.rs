// Copyright DataScienceBioLab 2024
// For MCP Plugin System Infrastructure
//
// This module implements plugin system integration for MCP

/// Local plugin interfaces to avoid circular dependencies
///
/// Contains core plugin traits and interface definitions used throughout the plugin system.
/// These interfaces are defined separately to prevent circular dependencies between modules.
pub mod interfaces;

/// Plugin adapter implementations
///
/// Provides adapters for connecting different types of plugins to the MCP system.
/// This includes tool plugins, protocol adapters, and other extension mechanisms.
pub mod adapter;

/// Plugin lifecycle management
///
/// Handles the lifecycle of plugins including initialization, activation, deactivation, 
/// and shutdown. Provides hooks for monitoring and controlling plugin state changes.
pub mod lifecycle;

/// Plugin discovery and loading
///
/// Implements mechanisms for discovering, loading, and registering plugins.
/// Supports dynamic loading from various sources including local filesystem and remote repositories.
pub mod discovery;

/// Plugin integration with core MCP components
///
/// Provides integration points between plugins and core MCP functionality.
/// Handles message routing, command execution, and other integration aspects.
pub mod integration;

/// Plugin versioning and compatibility
///
/// Manages plugin version compatibility, requirements, and dependencies.
/// Ensures plugins are compatible with the current MCP protocol version.
pub mod versioning;

/// Example plugins and implementations
///
/// Contains example plugin implementations for testing and demonstration purposes.
/// These examples show how to create and use different types of MCP plugins.
pub mod examples;

// Re-export key types/interfaces
pub use interfaces::{Plugin, PluginMetadata, PluginStatus, McpPlugin, PluginManagerInterface};
pub use adapter::{ToolPluginAdapter, ToolPluginFactory};
pub use lifecycle::{PluginLifecycleHook, CompositePluginLifecycleHook};

// Re-export discovery manager and executor
pub use self::discovery::{PluginDiscoveryManager, PluginProxyExecutor};

// Re-export integration system
pub use self::integration::{PluginSystemIntegration, PluginToolExecutor};

// Re-export versioning
pub use self::versioning::{ProtocolVersion, VersionRequirement, ProtocolVersionManager};

#[cfg(test)]
mod tests; 