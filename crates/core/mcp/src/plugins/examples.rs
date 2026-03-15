// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

// Plugin system examples
//
// This file provides examples of using the plugin system within MCP.
// The examples here are for demonstration purposes only.

use std::sync::Arc;
use anyhow::Result;
use serde_json::json;
use tracing::info;
use uuid::Uuid;
use async_trait::async_trait;

use crate::tool::{Tool, ToolManager};
use crate::tool::lifecycle::BasicLifecycleHook;
use crate::plugins::interfaces::{Plugin, PluginMetadata, PluginStatus, McpPlugin, PluginCapability};
use crate::plugins::versioning::{ProtocolVersion, VersionRequirement, ProtocolVersionManager};
use crate::plugins::integration::MockPluginManager;
use crate::tool::CoreToolManager;

/// Example function that sets up a basic plugin environment
///
/// # Errors
///
/// Returns an error if:
/// - The tool manager fails to register the tool
/// - There are issues with tool configuration or initialization
pub async fn setup_basic_plugin_environment() -> Result<()> {
    // Create a tool manager
    let tool_manager = Arc::new(CoreToolManager::new());
        
    // Create a simple tool
    let tool = Tool::builder()
        .id("sample-tool")
        .name("Sample Tool")
        .version("1.0.0")
        .description("A sample tool for testing")
        .security_level(1)
        .build();
        
    let tool_name = match &tool {
        Ok(t) => t.name.clone(),
        Err(_) => "unknown".to_string(),
    };
    
    // Simplified tool registration (just log for now)
    info!("Registering example tool: {}", tool_name);
    
    // In a real implementation, this would register the tool
    // For now, we'll just return success
    Ok(())
}

/// Example function that demonstrates protocol versioning with plugins
///
/// # Errors
///
/// Returns an error if:
/// - The version compatibility check fails
/// - The protocol version conversion to semver format fails
/// - The message compatibility check fails
pub async fn protocol_versioning_example() -> Result<()> {
    // Create protocol version manager
    let version_manager = ProtocolVersionManager::new(
        ProtocolVersion::new(1, 2, 0),  // Current version
        vec![
            ProtocolVersion::new(1, 0, 0),  // Minimum supported version
            ProtocolVersion::new(1, 1, 0),
            ProtocolVersion::new(1, 2, 0),
        ],
    );
    
    // Create a tool manager
    let tool_manager = Arc::new(CoreToolManager::new());
    
    // Create a plugin manager
    let plugin_manager = Arc::new(MockPluginManager::new());
    
    // Set up the integration
    let integration = crate::plugins::PluginSystemIntegration::with_version_manager(
        tool_manager,
        plugin_manager,
        version_manager,
    );
    
    // We would normally register tools as plugins here, but we'll just check some version information
    let plugin_version = ProtocolVersion::new(1, 1, 0);
    let req = VersionRequirement::new(">=1.0.0, <2.0.0");
    
    if plugin_version.is_compatible_with(&req)? {
        println!("Plugin version {} is compatible with requirement {}", 
                 plugin_version, req.requirement);
    }
    
    // Example of creating a message with version information
    let _message = json!({
        "capability": "test",
        "parameters": {}
    });
    
    // Create a versioned message
    let versioned_message = json!({
        "capability": "test",
        "parameters": {},
        "protocol_version": {
            "major": 1,
            "minor": 1,
            "patch": 0
        }
    });
    
    // Check message compatibility
    if integration.version_manager().check_message_compatibility(&versioned_message)? {
        println!("Message is compatible with the current protocol version");
    }
    
    // Create a non-versioned message and check compatibility
    let _message = json!({
        "capability": "test",
        "parameters": {}
    });
    
    Ok(())
}

/// Example implementation of a plugin
#[derive(Debug)]
pub struct ExamplePlugin {
    /// Metadata describing the plugin
    metadata: PluginMetadata,
    /// Version requirements for compatibility
    version_requirements: VersionRequirement,
}

impl ExamplePlugin {
    #[must_use] pub fn new(name: &str, version: &str, description: &str) -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                version: version.to_string(),
                description: description.to_string(),
                status: PluginStatus::Registered,
                capabilities: vec![PluginCapability::Tool],
            },
            version_requirements: VersionRequirement::new(">=1.0.0, <2.0.0"),
        }
    }
    
    #[must_use] pub fn with_version_requirements(
        name: &str, 
        version: &str, 
        description: &str,
        requirements: &str
    ) -> Self {
        Self {
            metadata: PluginMetadata {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                version: version.to_string(),
                description: description.to_string(),
                status: PluginStatus::Registered,
                capabilities: vec![PluginCapability::Tool],
            },
            version_requirements: VersionRequirement::new(requirements),
        }
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    async fn initialize(&self) -> Result<()> {
        println!("Initializing plugin: {}", self.metadata.name);
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        println!("Shutting down plugin: {}", self.metadata.name);
        Ok(())
    }
}

#[async_trait]
impl McpPlugin for ExamplePlugin {
    async fn handle_message(&self, message: serde_json::Value) -> Result<serde_json::Value> {
        // Check capability
        let capability = message.get("capability")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing capability in message"))?;
            
        println!("Handling capability: {}", capability);
        
        // Default success response
        Ok(json!({
            "success": true,
            "result": {
                "message": "Executed capability successfully",
                "capability": capability,
            }
        }))
    }
    
    fn validate_message_schema(&self, message: &serde_json::Value) -> Result<()> {
        if !message.is_object() {
            return Err(anyhow::anyhow!("Message must be an object"));
        }
        
        if message.get("capability").is_none() {
            return Err(anyhow::anyhow!("Message must contain 'capability' field"));
        }
        
        Ok(())
    }
    
    fn protocol_version_requirements(&self) -> VersionRequirement {
        self.version_requirements.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_example_plugin() {
        let plugin = ExamplePlugin::new("Test Plugin", "1.0.0", "A test plugin");
        
        // Verify metadata
        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "Test Plugin");
        assert_eq!(metadata.version, "1.0.0");
        
        // Test initialization
        assert!(plugin.initialize().await.is_ok());
        
        // Test message handling
        let message = json!({
            "capability": "test_capability",
            "parameters": {
                "input": "test value"
            }
        });
        
        let result = plugin.handle_message(message).await.unwrap();
        assert!(result.get("success").and_then(|v| v.as_bool()).unwrap_or(false));
        
        // Test version requirements
        let req = plugin.protocol_version_requirements();
        assert_eq!(req.requirement, ">=1.0.0, <2.0.0");
        
        // Test shutdown
        assert!(plugin.shutdown().await.is_ok());
    }
} 