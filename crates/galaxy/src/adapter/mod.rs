/*!
 * Galaxy adapter implementation.
 * 
 * This module provides the main adapter that connects the MCP protocol
 * to the Galaxy API, following the adapter pattern for dependency injection.
 */

use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::client::GalaxyClient;
use crate::config::GalaxyConfig;
use crate::error::{Error, Result};
use crate::models::{GalaxyTool, ParameterValue};
use crate::security::credentials::credentials_from_config;

// Only include MCP imports when the mcp-integration feature is enabled
#[cfg(feature = "mcp-integration")]
pub mod mcp_types {
    // Re-export the MCP types needed by the adapter
    // Use a manual Debug implementation wrapper since MCPProtocolAdapter doesn't implement Debug
    use std::fmt;
    
    pub use squirrel_mcp::types::{MCPMessage as Message, MessageType};
    
    // Wrapper for ContextManager that implements Debug
    pub struct ContextManager(pub squirrel_context::ContextManagerImpl);
    
    impl fmt::Debug for ContextManager {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("ContextManager")
                .field("initialized", &"<context_manager>")
                .finish()
        }
    }
    
    impl Default for ContextManager {
        fn default() -> Self {
            Self(squirrel_context::create_default_manager())
        }
    }
    
    // Wrapper for MCPProtocolAdapter that implements Debug
    pub struct Protocol(pub squirrel_mcp::protocol::MCPProtocolAdapter);
    
    impl Default for Protocol {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl Protocol {
        pub fn new() -> Self {
            Self(squirrel_mcp::protocol::MCPProtocolAdapter::new())
        }
    }
    
    impl fmt::Debug for Protocol {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Protocol")
                .field("initialized", &"<protocol_adapter>")
                .finish()
        }
    }
}

#[cfg(feature = "mcp-integration")]
use mcp_types::*;

/// Galaxy adapter for the MCP protocol
#[derive(Debug)]
pub struct GalaxyAdapter {
    /// Configuration for the adapter
    config: GalaxyConfig,
    
    /// Galaxy client for API communication
    client: Arc<GalaxyClient>,
    
    /// MCP protocol handler (optional)
    #[cfg(feature = "mcp-integration")]
    protocol: Option<Protocol>,
    
    /// Context manager (optional)
    #[cfg(feature = "mcp-integration")]
    context: Option<ContextManager>,
}

impl GalaxyAdapter {
    /// Creates a new Galaxy adapter with the given configuration
    pub fn new(config: GalaxyConfig) -> Result<Self> {
        config.validate()?;
        
        // Create secure credentials from config
        let credentials = credentials_from_config(&config)?;
        
        let client = GalaxyClient::new(
            &config.api_url,
            credentials,
            Some(config.timeout),
        )?;
        
        Ok(Self {
            config,
            client: Arc::new(client),
            #[cfg(feature = "mcp-integration")]
            protocol: None,
            #[cfg(feature = "mcp-integration")]
            context: None,
        })
    }
    
    /// Returns the current configuration
    pub fn config(&self) -> &GalaxyConfig {
        &self.config
    }
    
    /// Initializes the adapter with MCP integration
    #[cfg(feature = "mcp-integration")]
    pub fn initialize_mcp(&mut self) -> Result<()> {
        if self.protocol.is_some() {
            return Err(Error::AlreadyInitialized);
        }
        
        self.protocol = Some(Protocol::new());
        self.context = Some(ContextManager::default());
        
        info!("Galaxy adapter initialized with MCP integration");
        Ok(())
    }
    
    /// Checks if the adapter is initialized with MCP
    #[cfg(feature = "mcp-integration")]
    pub fn is_mcp_initialized(&self) -> bool {
        self.protocol.is_some() && self.context.is_some()
    }
    
    /// Gets the Galaxy client
    pub fn client(&self) -> &GalaxyClient {
        &self.client
    }
    
    /// Lists available Galaxy tools
    pub async fn list_tools(&self) -> Result<Vec<GalaxyTool>> {
        debug!("Listing Galaxy tools");
        self.client.list_tools().await
    }
    
    /// Gets a specific Galaxy tool by ID
    pub async fn get_tool(&self, tool_id: &str) -> Result<GalaxyTool> {
        debug!("Getting Galaxy tool: {}", tool_id);
        self.client.get_tool(tool_id).await
    }
    
    /// Executes a Galaxy tool with the specified parameters
    pub async fn execute_tool(
        &self,
        tool_id: &str,
        parameters: &std::collections::HashMap<String, ParameterValue>,
    ) -> Result<String> {
        debug!("Executing Galaxy tool: {}", tool_id);
        self.client.execute_tool(tool_id, parameters).await
    }
    
    /// Gets the status of a job
    pub async fn get_job_status(&self, job_id: &str) -> Result<crate::models::tool::JobState> {
        debug!("Getting job status: {}", job_id);
        self.client.get_job_status(job_id).await
    }
    
    /// Gets the results of a completed job
    pub async fn get_job_results(&self, job_id: &str) -> Result<Vec<crate::models::tool::ToolOutput>> {
        debug!("Getting job results: {}", job_id);
        self.client.get_job_results(job_id).await
    }
    
    /// Creates a new history
    pub async fn create_history(&self, name: &str) -> Result<crate::models::history::History> {
        debug!("Creating history: {}", name);
        self.client.create_history(name).await
    }
    
    /// Uploads a dataset to Galaxy
    pub async fn upload_dataset(
        &self,
        name: &str,
        data: Vec<u8>,
        file_type: &str,
        history_id: Option<&str>,
    ) -> Result<crate::models::dataset::Dataset> {
        debug!("Uploading dataset: {}", name);
        self.client.upload_dataset(history_id.unwrap_or("default"), name, data, Some(file_type)).await
    }
    
    /// Downloads a dataset from Galaxy
    pub async fn download_dataset(&self, dataset_id: &str) -> Result<Vec<u8>> {
        debug!("Downloading dataset: {}", dataset_id);
        self.client.download_dataset(dataset_id).await
    }
}

/// MCP-specific functionality
#[cfg(feature = "mcp-integration")]
impl GalaxyAdapter {
    /// Handle incoming MCP messages
    pub async fn handle_message(&self, message: Message) -> Result<Message> {
        info!("Handling MCP message");
        
        // Extract message properties safely
        let message_type = message.message_type;
        
        match message_type {
            MessageType::Command => {
                // For now, we'll just return a simple response
                let result = self.handle_command(&message).await?;
                Ok(result)
            },
            MessageType::Response => {
                debug!("Received response message");
                Ok(Message {
                    id: message.id.clone(),
                    message_type: MessageType::Response,
                    payload: serde_json::json!({
                        "success": true,
                        "message": "Response received"
                    }),
                })
            },
            _ => {
                warn!("Unsupported message type: {:?}", message_type);
                Ok(Message {
                    id: message.id.clone(),
                    message_type: MessageType::Response,
                    payload: serde_json::json!({
                        "success": false,
                        "error": format!("Unsupported message type: {:?}", message_type)
                    }),
                })
            }
        }
    }
    
    /// Handle command messages
    async fn handle_command(&self, message: &Message) -> Result<Message> {
        // Get the command from the payload
        if let Some(command) = message.payload.get("command").and_then(|c| c.as_str()) {
            match command {
                "discover_tools" => {
                    debug!("Tool discovery request");
                    // Extract parameters from the payload
                    let tool_prefix = message.payload.get("tool_prefix").and_then(|p| p.as_str());
                    let limit = message.payload.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
                    let offset = message.payload.get("offset").and_then(|o| o.as_u64()).map(|o| o as usize);
                    
                    // Query tools from Galaxy
                    let tools = self.list_tools().await?;
                    
                    // Filter by prefix if provided
                    let filtered_tools = match tool_prefix {
                        Some(prefix) => tools.iter()
                            .filter(|t| t.metadata.name.starts_with(prefix))
                            .collect::<Vec<_>>(),
                        None => tools.iter().collect::<Vec<_>>()
                    };
                    
                    // Apply pagination
                    let start = offset.unwrap_or(0);
                    let end = start + limit.unwrap_or(filtered_tools.len());
                    let paginated_tools = filtered_tools.iter()
                        .skip(start)
                        .take(if end > filtered_tools.len() { filtered_tools.len() - start } else { end - start })
                        .map(|t| {
                            serde_json::json!({
                                "id": t.id,
                                "name": t.metadata.name,
                                "description": t.metadata.description.clone().unwrap_or_default(),
                                "parameters": t.inputs.iter().map(|p| {
                                    serde_json::json!({
                                        "name": p.name,
                                        "parameter_type": p.type_name,
                                        "required": p.required
                                    })
                                }).collect::<Vec<_>>()
                            })
                        })
                        .collect::<Vec<_>>();
                    
                    Ok(Message {
                        id: message.id.clone(),
                        message_type: MessageType::Response,
                        payload: serde_json::json!({
                            "command": "discover_tools_response",
                            "tools": paginated_tools,
                            "total": filtered_tools.len(),
                            "limit": limit,
                            "offset": offset
                        }),
                    })
                },
                "execute_tool" => {
                    debug!("Tool execution request");
                    // Extract parameters from the payload
                    let tool_id = message.payload.get("tool_id")
                        .and_then(|t| t.as_str())
                        .ok_or_else(|| Error::MissingData("tool_id".to_string()))?;
                    
                    let parameters = message.payload.get("parameters")
                        .and_then(|p| p.as_object())
                        .ok_or_else(|| Error::MissingData("parameters".to_string()))?;
                    
                    let context = message.payload.get("context")
                        .and_then(|c| c.as_object())
                        .ok_or_else(|| Error::MissingData("context".to_string()))?;
                    
                    // Extract history ID from context
                    let _history_id = context.get("history_id")
                        .and_then(|h| h.as_str())
                        .ok_or_else(|| Error::MissingData("history_id in context".to_string()))?;
                    
                    // Convert parameters to ParameterValue format
                    let mut param_values = std::collections::HashMap::new();
                    for (key, value) in parameters {
                        let param_value = match value {
                            serde_json::Value::String(s) => ParameterValue::String(s.clone()),
                            serde_json::Value::Number(n) => {
                                if n.is_f64() {
                                    ParameterValue::Number(n.as_f64().unwrap())
                                } else if n.is_i64() {
                                    ParameterValue::Number(n.as_i64().unwrap() as f64)
                                } else if n.is_u64() {
                                    ParameterValue::Number(n.as_u64().unwrap() as f64)
                                } else {
                                    ParameterValue::String(n.to_string())
                                }
                            },
                            serde_json::Value::Bool(b) => ParameterValue::Boolean(*b),
                            _ => ParameterValue::String(value.to_string()),
                        };
                        param_values.insert(key.clone(), param_value);
                    }
                    
                    // Execute the tool
                    let job = self.execute_tool(tool_id, &param_values).await?;
                    
                    Ok(Message {
                        id: message.id.clone(),
                        message_type: MessageType::Response,
                        payload: serde_json::json!({
                            "command": "execute_tool_response",
                            "job_id": job,
                            "status": "submitted"
                        }),
                    })
                },
                "get_job_status" => {
                    debug!("Job status request");
                    // Extract job ID from the payload
                    let job_id = message.payload.get("job_id")
                        .and_then(|j| j.as_str())
                        .ok_or_else(|| Error::MissingData("job_id".to_string()))?;
                    
                    // Get the job status
                    let job_status = self.client.get_job_status(job_id).await?;
                    
                    // Determine if the job is in a terminal state
                    let is_terminal = job_status.is_terminal();
                    let is_successful = job_status.is_successful();
                    
                    Ok(Message {
                        id: message.id.clone(),
                        message_type: MessageType::Response,
                        payload: serde_json::json!({
                            "command": "get_job_status_response",
                            "job_id": job_id,
                            "state": format!("{:?}", job_status).to_lowercase(),
                            "is_terminal": is_terminal,
                            "is_successful": is_successful
                        }),
                    })
                },
                "get_job_results" => {
                    debug!("Job results request");
                    // Extract job ID from the payload
                    let job_id = message.payload.get("job_id")
                        .and_then(|j| j.as_str())
                        .ok_or_else(|| Error::MissingData("job_id".to_string()))?;
                    
                    // Get the job status to check outputs
                    let _job_status = self.client.get_job_status(job_id).await?;
                    
                    // Get the job outputs - in a real implementation, we'd get real outputs
                    // For now, we create a mock output for demonstration
                    let outputs = [
                        crate::models::tool::ToolOutput {
                            name: "output1".to_string(),
                            id: format!("{}:output1", job_id),
                            format: "tabular".to_string(),
                            url: Some(format!("/api/datasets/{}/display", job_id)),
                        }
                    ];
                    
                    // Format the outputs
                    let output_values = outputs.iter().map(|output| {
                        serde_json::json!({
                            "id": output.id,
                            "name": output.name,
                            "format": output.format,
                            "url": output.url
                        })
                    }).collect::<Vec<_>>();
                    
                    Ok(Message {
                        id: message.id.clone(),
                        message_type: MessageType::Response,
                        payload: serde_json::json!({
                            "command": "get_job_results_response",
                            "job_id": job_id,
                            "outputs": output_values
                        }),
                    })
                },
                _ => {
                    warn!("Unsupported command: {}", command);
                    Ok(Message {
                        id: message.id.clone(),
                        message_type: MessageType::Response,
                        payload: serde_json::json!({
                            "success": false,
                            "error": format!("Unsupported command: {}", command)
                        }),
                    })
                }
            }
        } else {
            warn!("Missing command in payload");
            Ok(Message {
                id: message.id.clone(),
                message_type: MessageType::Response,
                payload: serde_json::json!({
                    "success": false,
                    "error": "Missing command in payload"
                }),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_adapter_initialization() {
        let config = GalaxyConfig::default()
            .with_api_key("test_key");
        
        let adapter = GalaxyAdapter::new(config).unwrap();
        assert!(adapter.client.is_initialized());
    }
    
    #[tokio::test]
    async fn test_config_accessor() {
        let config = GalaxyConfig::default()
            .with_api_key("test_key");
        
        let adapter = GalaxyAdapter::new(config.clone()).unwrap();
        assert_eq!(adapter.config().api_key, config.api_key);
    }
    
    #[cfg(feature = "mcp-integration")]
    #[tokio::test]
    async fn test_mcp_initialization() {
        let config = GalaxyConfig::default()
            .with_api_key("test_key");
        
        let mut adapter = GalaxyAdapter::new(config).unwrap();
        assert!(!adapter.is_mcp_initialized());
        
        adapter.initialize_mcp().unwrap();
        assert!(adapter.is_mcp_initialized());
        
        // Second initialization should fail
        let result = adapter.initialize_mcp();
        assert!(result.is_err());
    }
} 