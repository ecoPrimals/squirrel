// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP Adapter for AI Router
//!
//! This module provides an adapter between the `AIRouter` and MCP for
//! remote AI capabilities.
//!
//! **Phase 2**: Production [`MCPAdapter::send_request`] returns a deterministic local
//! response until MCP transport is wired; [`MCPAdapter::discover_capabilities`] returns
//! an empty map outside tests. Test-only injection uses `#[cfg(test)]` helpers on [`MCPAdapter`].

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
    /// Configuration (endpoint, timeouts, TLS) — applied when the adapter calls MCP transport (Phase 2).
    #[expect(
        dead_code,
        reason = "Phase 2 — used when adapter delegates to real MCP client"
    )]
    config: MCPAdapterConfig,

    #[cfg(test)]
    mock_responses: std::sync::RwLock<HashMap<String, ChatResponse>>,

    #[cfg(test)]
    mock_capabilities: std::sync::RwLock<HashMap<NodeId, HashMap<String, AICapabilities>>>,
}

impl MCPAdapter {
    /// Create a new MCP adapter
    #[must_use]
    #[allow(
        clippy::missing_const_for_fn,
        reason = "Test-only RwLock::new in struct literal is not const"
    )]
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
        match self.mock_responses.write() {
            Ok(mut responses) => {
                responses.insert(request_id, response);
            }
            Err(e) => {
                tracing::error!("Failed to acquire mock responses write lock: {}", e);
            }
        }
    }

    /// For testing: add mock capabilities
    #[cfg(test)]
    pub fn add_mock_capabilities(
        &self,
        node_id: NodeId,
        capabilities: HashMap<String, AICapabilities>,
    ) {
        match self.mock_capabilities.write() {
            Ok(mut caps) => {
                caps.insert(node_id, capabilities);
            }
            Err(e) => {
                tracing::error!("Failed to acquire mock capabilities write lock: {}", e);
            }
        }
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
            match self.mock_responses.read() {
                Ok(responses) => {
                    if let Some(response) = responses.get(&request.request_id.to_string()) {
                        return Ok(RemoteAIResponse {
                            response_id: uuid::Uuid::new_v4(),
                            request_id: request.request_id,
                            provider_id: request.provider_id,
                            chat_response: response.clone(),
                        });
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to acquire mock responses read lock: {}", e);
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
            match self.mock_capabilities.read() {
                Ok(mock_caps) => {
                    all_capabilities.extend(mock_caps.clone());
                }
                Err(e) => {
                    tracing::error!("Failed to acquire mock capabilities read lock: {}", e);
                    return Ok(all_capabilities);
                }
            }
        }

        Ok(all_capabilities)
    }
}

// Phase 2: wire types below to the real MCP JSON-RPC / stream codec when the adapter
// delegates to `squirrel-mcp` transport instead of local fallbacks.
#[derive(Debug)]
#[expect(dead_code, reason = "Phase 2 — MCP wire protocol client/types")]
struct McpClient;

#[derive(Debug)]
#[expect(dead_code, reason = "Phase 2 — MCP wire protocol client/types")]
struct McpRequest {
    id: String,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug)]
#[expect(dead_code, reason = "Phase 2 — MCP wire protocol client/types")]
struct McpResponse {
    id: String,
    result: serde_json::Value,
}

#[derive(Debug)]
#[expect(dead_code, reason = "Phase 2 — MCP wire protocol client/types")]
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
            request_id: uuid::Uuid::parse_str(&request_id).expect("should succeed"),
            session_id: None,
            provider_id: "test-provider".to_string(),
            chat_request: ChatRequest::new(),
            task: crate::common::capability::AITask::default(),
        };

        // Send request
        let result = adapter
            .send_request(&node_id, remote_request)
            .await
            .expect("should succeed");

        // Verify response
        assert_eq!(
            result.chat_response.choices[0]
                .content
                .as_ref()
                .expect("should succeed"),
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
        let result = adapter
            .discover_capabilities()
            .await
            .expect("should succeed");

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
            .expect("should succeed");

        // Verify default response contains expected text
        let content = result.chat_response.choices[0]
            .content
            .as_ref()
            .expect("should succeed");

        // Response format: "Remote response from node NodeId("test-node") via MCP"
        assert!(content.contains("Remote response from node"));
        assert!(content.contains("test-node"));
        assert!(content.contains("via MCP"));
    }
}
