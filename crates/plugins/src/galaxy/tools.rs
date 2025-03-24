//! Galaxy tools plugin extension
//!
//! This module provides an extension trait for Galaxy plugins that adds tool-related functionality.

use std::collections::HashMap;
use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::galaxy::GalaxyPlugin;
use crate::tools::ToolPlugin;

/// Galaxy tool plugin extension
#[async_trait]
pub trait GalaxyToolPlugin: GalaxyPlugin + ToolPlugin {
    /// List available Galaxy tools
    async fn list_galaxy_tools(&self) -> Result<Vec<GalaxyToolInfo>>;
    
    /// Get details for a specific Galaxy tool
    async fn get_galaxy_tool(&self, tool_id: &str) -> Result<GalaxyToolInfo>;
    
    /// Execute a Galaxy tool
    async fn execute_galaxy_tool(&self, tool_id: &str, parameters: HashMap<String, Value>) -> Result<String>;
    
    /// Get the status of a Galaxy job
    async fn get_galaxy_job_status(&self, job_id: &str) -> Result<GalaxyJobStatus>;
    
    /// Get the results of a Galaxy job
    async fn get_galaxy_job_results(&self, job_id: &str) -> Result<Vec<GalaxyToolOutput>>;
}

/// Galaxy tool information
#[derive(Debug, Clone)]
pub struct GalaxyToolInfo {
    /// Tool ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool version
    pub version: String,
    /// Tool parameters
    pub parameters: Vec<GalaxyToolParameter>,
}

/// Galaxy tool parameter
#[derive(Debug, Clone)]
pub struct GalaxyToolParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub type_name: String,
    /// Whether the parameter is required
    pub required: bool,
    /// Parameter default value
    pub default: Option<Value>,
    /// Parameter description
    pub description: String,
}

/// Galaxy job status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalaxyJobStatus {
    /// Job is waiting to run
    Waiting,
    /// Job is running
    Running,
    /// Job has completed successfully
    Completed,
    /// Job has failed
    Failed,
    /// Job has been cancelled
    Cancelled,
    /// Job status is unknown
    Unknown,
}

/// Galaxy tool output
#[derive(Debug, Clone)]
pub struct GalaxyToolOutput {
    /// Output name
    pub name: String,
    /// Output ID
    pub id: String,
    /// Output format
    pub format: String,
    /// URL to download the output
    pub url: Option<String>,
}

/// Extension implementations for GalaxyAdapterPlugin
#[cfg(test)]
mod tests {
    use super::*;
    use crate::galaxy::adapter_plugin::GalaxyAdapterPlugin;
    
    // Sample implementation for a Galaxy tool plugin
    #[derive(Debug)]
    struct TestGalaxyToolPlugin {
        galaxy_plugin: GalaxyAdapterPlugin,
    }
    
    #[async_trait]
    impl GalaxyPlugin for TestGalaxyToolPlugin {
        async fn connect(&self, connection_info: Value) -> Result<()> {
            self.galaxy_plugin.connect(connection_info).await
        }
        
        async fn send_data(&self, data: Value) -> Result<Value> {
            self.galaxy_plugin.send_data(data).await
        }
        
        async fn receive_data(&self) -> Result<Value> {
            self.galaxy_plugin.receive_data().await
        }
        
        fn get_integration_types(&self) -> Vec<String> {
            self.galaxy_plugin.get_integration_types()
        }
    }
    
    // Implementation of the core Plugin trait
    #[async_trait]
    impl crate::plugin::Plugin for TestGalaxyToolPlugin {
        fn metadata(&self) -> &crate::plugin::PluginMetadata {
            self.galaxy_plugin.metadata()
        }
        
        async fn initialize(&self) -> Result<()> {
            self.galaxy_plugin.initialize().await
        }
        
        async fn shutdown(&self) -> Result<()> {
            self.galaxy_plugin.shutdown().await
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    
    // Implementation of the ToolPlugin trait
    #[async_trait]
    impl ToolPlugin for TestGalaxyToolPlugin {
        async fn execute_command(&self, command: &str, args: Value) -> Result<crate::tools::CommandResult> {
            match command {
                "list_tools" => {
                    let tools = self.list_galaxy_tools().await?;
                    let tool_json = tools.iter().map(|t| {
                        serde_json::json!({
                            "id": t.id,
                            "name": t.name,
                            "description": t.description,
                            "version": t.version,
                        })
                    }).collect::<Vec<_>>();
                    Ok(crate::tools::CommandResult::success(tool_json))
                },
                "execute_tool" => {
                    let tool_id = args.get("tool_id")
                        .and_then(|id| id.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing tool_id"))?;
                    
                    let parameters = args.get("parameters")
                        .and_then(|p| p.as_object())
                        .ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
                    
                    let mut param_map = HashMap::new();
                    for (key, value) in parameters {
                        param_map.insert(key.clone(), value.clone());
                    }
                    
                    let job_id = self.execute_galaxy_tool(tool_id, param_map).await?;
                    Ok(crate::tools::CommandResult::success(serde_json::json!({ "job_id": job_id })))
                },
                _ => Ok(crate::tools::CommandResult::error(format!("Unsupported command: {}", command))),
            }
        }
        
        fn get_commands(&self) -> Vec<crate::tools::CommandMetadata> {
            vec![
                crate::tools::CommandMetadata {
                    name: "list_tools".to_string(),
                    description: "List all Galaxy tools".to_string(),
                    permissions: vec!["galaxy.tools.list".to_string()],
                    usage: "list_tools".to_string(),
                    examples: vec!["list_tools".to_string()],
                    arguments: vec![],
                    flags: vec![],
                },
                crate::tools::CommandMetadata {
                    name: "execute_tool".to_string(),
                    description: "Execute a Galaxy tool".to_string(),
                    permissions: vec!["galaxy.tools.execute".to_string()],
                    usage: "execute_tool --tool_id <tool_id> --parameters <parameters>".to_string(),
                    examples: vec!["execute_tool --tool_id test_tool --parameters {}".to_string()],
                    arguments: vec![],
                    flags: vec![],
                },
            ]
        }
    }
    
    // Implementation of the GalaxyToolPlugin trait
    #[async_trait]
    impl GalaxyToolPlugin for TestGalaxyToolPlugin {
        async fn list_galaxy_tools(&self) -> Result<Vec<GalaxyToolInfo>> {
            // In a real implementation, this would call the Galaxy API
            Ok(vec![
                GalaxyToolInfo {
                    id: "test_tool".to_string(),
                    name: "Test Tool".to_string(),
                    description: "A test Galaxy tool".to_string(),
                    version: "1.0.0".to_string(),
                    parameters: vec![
                        GalaxyToolParameter {
                            name: "input".to_string(),
                            type_name: "data".to_string(),
                            required: true,
                            default: None,
                            description: "Input dataset".to_string(),
                        },
                    ],
                },
            ])
        }
        
        async fn get_galaxy_tool(&self, tool_id: &str) -> Result<GalaxyToolInfo> {
            // In a real implementation, this would call the Galaxy API
            Ok(GalaxyToolInfo {
                id: tool_id.to_string(),
                name: "Test Tool".to_string(),
                description: "A test Galaxy tool".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![
                    GalaxyToolParameter {
                        name: "input".to_string(),
                        type_name: "data".to_string(),
                        required: true,
                        default: None,
                        description: "Input dataset".to_string(),
                    },
                ],
            })
        }
        
        async fn execute_galaxy_tool(&self, tool_id: &str, _parameters: HashMap<String, Value>) -> Result<String> {
            // In a real implementation, this would call the Galaxy API
            Ok(format!("job_{}", tool_id))
        }
        
        async fn get_galaxy_job_status(&self, _job_id: &str) -> Result<GalaxyJobStatus> {
            // In a real implementation, this would call the Galaxy API
            Ok(GalaxyJobStatus::Completed)
        }
        
        async fn get_galaxy_job_results(&self, job_id: &str) -> Result<Vec<GalaxyToolOutput>> {
            // In a real implementation, this would call the Galaxy API
            Ok(vec![
                GalaxyToolOutput {
                    name: "output".to_string(),
                    id: format!("{}_output", job_id),
                    format: "tabular".to_string(),
                    url: Some(format!("/api/datasets/{}_output/display", job_id)),
                },
            ])
        }
    }
} 