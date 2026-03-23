// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP Adapter for AI Router
#![allow(dead_code, reason = "MCP adapter module awaiting activation")]
//!
//! This module provides an adapter between the AIRouter and MCP for
//! remote AI capabilities.

use async_trait::async_trait;
use std::collections::HashMap;

use super::{MCPInterface, NodeId, RemoteAIRequest, RemoteAIResponse, RemoteAIResponseStream};
use crate::Result;
use crate::common::capability::AICapabilities;
use crate::common::{ChatChoice, ChatResponse, MessageRole, UsageInfo};

/// Configuration for the MCP adapter
#[derive(Debug, Clone)]
pub struct MCPAdapterConfig {
    /// Endpoint for MCP server
    pub endpoint: String,

    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,

    /// Authentication token for MCP server
    pub auth_token: Option<String>,

    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

impl Default for MCPAdapterConfig {
    fn default() -> Self {
        Self {
            endpoint: crate::config::DefaultEndpoints::mcp_server_endpoint(),
            connection_timeout_ms: 5000,
            auth_token: None,
            verify_ssl: true,
        }
    }
}

/// MCP adapter for remote AI capabilities
#[derive(Debug)]
pub struct MCPAdapter {
    /// Configuration
    config: MCPAdapterConfig,

    #[cfg(test)]
    mock_responses: std::sync::RwLock<HashMap<String, ChatResponse>>,

    #[cfg(test)]
    mock_capabilities: std::sync::RwLock<HashMap<NodeId, HashMap<String, AICapabilities>>>,
}

impl MCPAdapter {
    /// Create a new MCP adapter
    pub fn new(config: MCPAdapterConfig) -> Self {
        Self {
            config,
            #[cfg(test)]
            mock_responses: std::sync::RwLock::new(HashMap::new()),
            #[cfg(test)]
            mock_capabilities: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// For testing: add a mock response
    #[cfg(test)]
    pub fn add_mock_response(&self, request_id: String, response: ChatResponse) {
        let responses_result = self.mock_responses.write();

        let mut responses = match responses_result {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to acquire mock responses write lock: {}", e);
                return; // Gracefully fail for test setup
            }
        };

        responses.insert(request_id, response);
    }

    /// For testing: add mock capabilities
    #[cfg(test)]
    pub fn add_mock_capabilities(
        &self,
        node_id: NodeId,
        capabilities: HashMap<String, AICapabilities>,
    ) {
        let caps_result = self.mock_capabilities.write();

        let mut caps = match caps_result {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to acquire mock capabilities write lock: {}", e);
                return; // Gracefully fail for test setup
            }
        };

        caps.insert(node_id, capabilities);
    }
}

#[async_trait]
impl MCPInterface for MCPAdapter {
    async fn send_request(
        &self,
        node_id: &NodeId,
        request: RemoteAIRequest,
    ) -> Result<RemoteAIResponse> {
        // For testing: return mock response if available
        #[cfg(test)]
        {
            let responses_result = self.mock_responses.read();

            let responses = match responses_result {
                Ok(guard) => guard,
                Err(e) => {
                    tracing::error!("Failed to acquire mock responses read lock: {}", e);
                    // Continue with default response if lock fails
                    return Ok(RemoteAIResponse {
                        response_id: uuid::Uuid::new_v4(),
                        request_id: request.request_id,
                        provider_id: request.provider_id,
                        chat_response: ChatResponse {
                            id: uuid::Uuid::new_v4().to_string(),
                            model: "remote".to_string(),
                            choices: vec![ChatChoice {
                                index: 0,
                                role: MessageRole::Assistant,
                                content: Some(format!(
                                    "Error accessing mock responses for node {node_id:?}"
                                )),
                                finish_reason: Some("stop".to_string()),
                                tool_calls: None,
                            }],
                            usage: Some(UsageInfo {
                                prompt_tokens: 10,
                                completion_tokens: 20,
                                total_tokens: 30,
                            }),
                        },
                    });
                }
            };

            if let Some(response) = responses.get(&request.request_id.to_string()) {
                return Ok(RemoteAIResponse {
                    response_id: uuid::Uuid::new_v4(),
                    request_id: request.request_id,
                    provider_id: request.provider_id,
                    chat_response: response.clone(),
                });
            }
        }

        // Default response when MCP protocol is not yet wired (Phase 2).
        // In production this is a placeholder until full MCP transport integration.
        Ok(RemoteAIResponse {
            response_id: uuid::Uuid::new_v4(),
            request_id: request.request_id,
            provider_id: request.provider_id,
            chat_response: ChatResponse {
                id: uuid::Uuid::new_v4().to_string(),
                model: "remote".to_string(),
                choices: vec![ChatChoice {
                    index: 0,
                    role: MessageRole::Assistant,
                    content: Some(format!("Remote response from node {node_id:?} via MCP")),
                    finish_reason: Some("stop".to_string()),
                    tool_calls: None,
                }],
                usage: Some(UsageInfo {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                }),
            },
        })
    }

    async fn stream_request(
        &self,
        _node_id: &NodeId,
        _request: RemoteAIRequest,
    ) -> Result<RemoteAIResponseStream> {
        // MCP streaming requires a real MCP transport connection.
        // Until MCP protocol integration is wired (Phase 2), return
        // an error rather than silently returning fake data.
        Err(crate::error::AIError::Network(
            "MCP streaming not yet wired — use send_request for non-streaming".to_string(),
        ))
    }

    async fn discover_capabilities(
        &self,
    ) -> Result<HashMap<NodeId, HashMap<String, AICapabilities>>> {
        #[cfg(test)]
        let mut all_capabilities = HashMap::new();
        #[cfg(not(test))]
        let all_capabilities = HashMap::new(); // Placeholder until MCP protocol wired (Phase 2)

        // Include mock capabilities for testing
        #[cfg(test)]
        {
            let mock_caps_result = self.mock_capabilities.read();

            let mock_caps = match mock_caps_result {
                Ok(guard) => guard,
                Err(e) => {
                    tracing::error!("Failed to acquire mock capabilities read lock: {}", e);
                    // Return empty capabilities if lock fails
                    return Ok(all_capabilities);
                }
            };

            all_capabilities.extend(mock_caps.clone());
        }

        Ok(all_capabilities)
    }
}

// Placeholder types for MCP protocol
// These would be replaced with actual MCP protocol types
// NOTE(phase2): MCP adapter implementation - pending full MCP protocol integration
#[derive(Debug)]
struct McpClient;

#[derive(Debug)]
struct McpRequest {
    id: String,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug)]
struct McpResponse {
    id: String,
    result: serde_json::Value,
}

#[derive(Debug)]
struct McpChunk {
    id: String,
    data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::ChatRequest;
    use crate::common::capability::{ModelType, TaskType};

    #[tokio::test]
    async fn test_mock_response() {
        // Create adapter
        let adapter = MCPAdapter::new(MCPAdapterConfig::default());

        // Create a mock response
        let response = ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            model: "test-model".to_string(),
            choices: vec![crate::common::ChatChoice {
                index: 0,
                role: crate::common::MessageRole::Assistant,
                content: Some("Custom mock response".to_string()),
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: None,
        };

        // Add mock response
        let request_id = uuid::Uuid::new_v4().to_string();
        adapter.add_mock_response(request_id.clone(), response);

        // Create request
        let node_id = NodeId("test-node".to_string());
        let remote_request = RemoteAIRequest {
            request_id: uuid::Uuid::parse_str(&request_id).unwrap(),
            session_id: None,
            provider_id: "test-provider".to_string(),
            chat_request: ChatRequest::new(),
            task: crate::common::capability::AITask::default(),
        };

        // Send request
        let result = adapter
            .send_request(&node_id, remote_request)
            .await
            .unwrap();

        // Verify response
        assert_eq!(
            result.chat_response.choices[0].content.as_ref().unwrap(),
            "Custom mock response"
        );
    }

    #[tokio::test]
    async fn test_capability_discovery() {
        // Create adapter
        let adapter = MCPAdapter::new(MCPAdapterConfig::default());

        // Create mock capabilities
        let node_id = NodeId("test-node".to_string());
        let mut capabilities = HashMap::new();

        let mut ai_cap = AICapabilities::default();
        ai_cap.add_model_type(ModelType::LargeLanguageModel);
        ai_cap.add_task_type(TaskType::TextGeneration);
        ai_cap.with_max_context_size(8192);

        capabilities.insert("test-provider".to_string(), ai_cap);

        // Add mock capabilities
        adapter.add_mock_capabilities(node_id.clone(), capabilities);

        // Discover capabilities
        let result = adapter.discover_capabilities().await.unwrap();

        // Verify capabilities
        assert!(result.contains_key(&node_id));
        assert!(result[&node_id].contains_key("test-provider"));
        assert_eq!(result[&node_id]["test-provider"].max_context_size, 8192);
    }

    #[tokio::test]
    async fn test_default_response() {
        // Create adapter
        let adapter = MCPAdapter::new(MCPAdapterConfig::default());

        // Create request with unknown ID
        let node_id = NodeId("test-node".to_string());
        let remote_request = RemoteAIRequest {
            request_id: uuid::Uuid::new_v4(),
            session_id: None,
            provider_id: "test-provider".to_string(),
            chat_request: ChatRequest::new(),
            task: crate::common::capability::AITask::default(),
        };

        // Send request
        let result = adapter
            .send_request(&node_id, remote_request)
            .await
            .unwrap();

        // Verify default response contains expected text
        let content = result.chat_response.choices[0].content.as_ref().unwrap();

        // Response format: "Remote response from node NodeId("test-node") via MCP"
        assert!(content.contains("Remote response from node"));
        assert!(content.contains("test-node"));
        assert!(content.contains("via MCP"));
    }
}
