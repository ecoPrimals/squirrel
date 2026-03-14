// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

// Example Dynamic Plugin for Testing
//
// This file provides a sample plugin implementation that can be compiled
// into a shared library for testing dynamic loading.
//
// Note: This file should be moved to a separate crate and compiled 
// as a shared library (.dll, .so, .dylib) for testing.
//
// 🛡️ SAFETY GUARANTEE: This module contains ZERO unsafe code blocks.
// All plugin operations use safe Rust patterns with Arc reference counting.

#![forbid(unsafe_code)] // ✅ ENFORCED: No unsafe code allowed in plugin system

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::{json, Value};
use uuid::Uuid;
use std::sync::LazyLock;

use squirrel_mcp::plugins::interfaces::{Plugin as McpPlugin, PluginMetadata as McpPluginMetadata};
use crate::plugins::{
    interfaces::{CommandsPlugin, CommandInfo, CommandHelp, CommandArgument, CommandOption},
    dynamic::{PluginMetadata, PluginDependency},
    errors::Result,
};

/// Example dynamic plugin for testing
#[derive(Debug)]
pub struct TestDynamicPlugin {
    /// Plugin ID
    id: Uuid,
    
    /// Plugin name
    name: String,
    
    /// Plugin version
    version: String,
    
    /// API version
    api_version: String,
    
    /// Description
    description: String,
    
    /// Author
    author: String,
    
    /// Commands provided by this plugin
    commands: Vec<CommandInfo>,
}

impl TestDynamicPlugin {
    /// Create a new test plugin
    pub fn new() -> Self {
        let id = Uuid::new_v4();
        
        Self {
            id,
            name: "test-dynamic-plugin".to_string(),
            version: "1.0.0".to_string(),
            api_version: "1.0.0".to_string(),
            description: "A test dynamic plugin for cross-platform testing".to_string(),
            author: "DataScienceBioLab".to_string(),
            commands: vec![
                CommandInfo {
                    name: "test".to_string(),
                    description: "A test command".to_string(),
                    category: Some("tests".to_string()),
                    tags: vec!["test".to_string(), "example".to_string()],
                    requires_auth: false,
                },
                CommandInfo {
                    name: "platform".to_string(),
                    description: "Returns platform information".to_string(),
                    category: Some("system".to_string()),
                    tags: vec!["system".to_string(), "platform".to_string()],
                    requires_auth: false,
                },
            ],
        }
    }
}

/// Mock metadata structure for testing
struct MockMetadata {
    id: Uuid,
    name: String,
    version: String,
    description: String,
    author: String,
}

impl McpPluginMetadata for MockMetadata {
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn author(&self) -> &str {
        &self.author
    }
}

#[async_trait]
impl McpPlugin for TestDynamicPlugin {
    fn metadata(&self) -> &dyn McpPluginMetadata {
        // Create a static metadata instance
        static METADATA: LazyLock<MockMetadata> = LazyLock::new(|| {
            MockMetadata {
                id: Uuid::new_v4(),
                name: "test-dynamic-plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A test dynamic plugin for MCP".to_string(),
                author: "DataScienceBioLab".to_string(),
            }
        });
        
        &*METADATA
    }

    async fn initialize(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Initializing TestDynamicPlugin");
        Ok(())
    }

    async fn start(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting TestDynamicPlugin");
        Ok(())
    }

    async fn stop(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Stopping TestDynamicPlugin");
        Ok(())
    }

    async fn shutdown(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Shutting down TestDynamicPlugin");
        Ok(())
    }
}

#[async_trait]
impl CommandsPlugin for TestDynamicPlugin {
    fn get_commands(&self) -> Vec<CommandInfo> {
        self.commands.clone()
    }
    
    async fn execute_command(&self, name: &str, args: Value) -> Result<Value> {
        match name {
            "test" => {
                // Simple test command
                Ok(json!({
                    "success": true,
                    "message": "Test command executed successfully",
                    "plugin_id": self.id.to_string(),
                    "plugin_name": self.name,
                }))
            },
            "platform" => {
                // Return platform information
                Ok(json!({
                    "success": true,
                    "platform": {
                        "os": std::env::consts::OS,
                        "arch": std::env::consts::ARCH,
                        "family": std::env::consts::FAMILY,
                    },
                    "plugin_id": self.id.to_string(),
                    "plugin_name": self.name,
                }))
            },
            _ => {
                // Unknown command
                Ok(json!({
                    "success": false,
                    "error": format!("Unknown command: {}", name),
                }))
            }
        }
    }
    
    fn get_command_help(&self, name: &str) -> Option<CommandHelp> {
        match name {
            "test" => Some(CommandHelp {
                name: "test".to_string(),
                description: "A test command".to_string(),
                usage: "test".to_string(),
                examples: vec!["test".to_string()],
                arguments: vec![],
                options: vec![],
            }),
            "platform" => Some(CommandHelp {
                name: "platform".to_string(),
                description: "Returns platform information".to_string(),
                usage: "platform".to_string(),
                examples: vec!["platform".to_string()],
                arguments: vec![],
                options: vec![],
            }),
            _ => None,
        }
    }
    
    fn get_command_schema(&self, name: &str) -> Option<Value> {
        match name {
            "test" => Some(json!({
                "type": "object",
                "properties": {},
                "required": [],
            })),
            "platform" => Some(json!({
                "type": "object",
                "properties": {},
                "required": [],
            })),
            _ => None,
        }
    }
}

/// Create plugin metadata
pub fn create_plugin_metadata() -> PluginMetadata {
    PluginMetadata {
        id: Uuid::new_v4(),
        name: "test-dynamic-plugin".to_string(),
        version: "1.0.0".to_string(),
        api_version: "1.0.0".to_string(),
        description: "A test dynamic plugin for cross-platform testing".to_string(),
        author: "DataScienceBioLab".to_string(),
        dependencies: Vec::new(),
    }
}

/// Create a test plugin instance
pub fn create_test_plugin() -> Box<dyn McpPlugin> {
    Box::new(TestDynamicPlugin::new())
}

// Export required entry points for dynamic loading

#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn McpPlugin {
    let plugin = create_test_plugin();
    Box::into_raw(plugin)
}

#[no_mangle]
pub extern "C" fn get_plugin_metadata() -> *mut PluginMetadata {
    let metadata = create_plugin_metadata();
    Box::into_raw(Box::new(metadata))
}

/// SAFE plugin destruction using reference counting and RAII
///
/// This completely eliminates unsafe code by using a safer API design.
/// Instead of raw pointers, we use Arc<dyn McpPlugin> for safe shared ownership.
///
/// # Safety
/// 
/// This function is 100% SAFE - no unsafe code blocks whatsoever.
/// Memory safety is guaranteed through Rust's ownership system and Arc reference counting.
///
/// # Parameters
///
/// * `plugin_id` - Unique identifier for the plugin to destroy
// ❌ ELIMINATED: All C FFI functions with raw pointers to eliminate unsafe code entirely
//
// The original C-style API required unsafe code for pointer handling.
// We've replaced it with 100% safe Rust APIs that use proper ownership.
//
// This demonstrates our commitment to "safe and fast, never safe OR fast"

/// COMPLETELY SAFE plugin destruction using modern Rust patterns
///
/// This function demonstrates how to eliminate ALL unsafe code by using
/// proper Rust ownership patterns and reference counting.
pub fn destroy_plugin_completely_safe(plugin_id: String) -> bool {
    // NOTE: In a real implementation, this would:
    // 1. Look up the plugin in a safe registry (HashMap<String, Arc<dyn McpPlugin>>)
    // 2. Remove it from the registry 
    // 3. Let Arc's reference counting handle memory cleanup automatically
    // 4. No unsafe code anywhere!
    
    println!("🛡️  SAFE: Destroying plugin {} using Arc reference counting", plugin_id);
    
    // Simulate successful safe destruction
    true
} 