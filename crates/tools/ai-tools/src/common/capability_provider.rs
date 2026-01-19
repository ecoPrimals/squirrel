//! Capability-based AI Provider (TRUE PRIMAL!)
//!
//! This provider delegates AI operations to discovered capability providers
//! via Unix sockets (NO HTTP, NO reqwest, NO ring!).
//!
//! **Architecture**: Squirrel → Unix Socket → Songbird → AI Vendors

use crate::capability_ai::{
    AiClient, AiClientConfig, ChatMessage as CapabilityChatMessage, ChatOptions,
    ChatResponse as CapabilityChatResponse,
};
use crate::common::providers::{AICapability, AIProvider};
use crate::common::{ChatChoice, ChatMessage, ChatRequest, ChatResponse, MessageRole, UsageInfo};
use crate::Result;
use async_trait::async_trait;
use tracing::{debug, info, warn};

/// Capability-based AI provider (Pure Rust!)
///
/// Delegates all AI operations to a discovered capability provider
/// (e.g., Songbird) via Unix socket JSON-RPC.
///
/// **NO reqwest, NO ring!**
#[derive(Debug)]
pub struct CapabilityAIProvider {
    client: AiClient,
    name: String,
}

impl CapabilityAIProvider {
    /// Create a new capability-based AI provider
    ///
    /// # Arguments
    /// * `socket_path` - Path to AI capability Unix socket (from discovery!)
    /// * `name` - Provider name for logging/debugging
    pub fn new(socket_path: std::path::PathBuf, name: String) -> Result<Self> {
        let config = AiClientConfig {
            socket_path,
            timeout_secs: 30,
            max_retries: 3,
            retry_delay_ms: 100,
        };

        let client = AiClient::new(config).map_err(|e| {
            crate::Error::Configuration(format!("Failed to create capability AI client: {}", e))
        })?;

        info!("Initialized capability-based AI provider: {}", name);

        Ok(Self { client, name })
    }

    /// Create from environment (reads AI_CAPABILITY_SOCKET)
    pub fn from_env() -> Result<Self> {
        let client = AiClient::from_env().map_err(|e| {
            crate::Error::Configuration(format!(
                "Failed to create capability AI client from env: {}",
                e
            ))
        })?;

        Ok(Self {
            client,
            name: "capability-ai".to_string(),
        })
    }

    /// Convert our ChatMessage to capability ChatMessage
    fn convert_messages(messages: &[ChatMessage]) -> Vec<CapabilityChatMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                    MessageRole::Function => "function",
                    MessageRole::Tool => "tool",
                }
                .to_string();

                CapabilityChatMessage {
                    role,
                    content: msg.content.clone().unwrap_or_default(),
                }
            })
            .collect()
    }

    /// Convert capability ChatResponse to our ChatResponse
    fn convert_response(capability_response: CapabilityChatResponse) -> ChatResponse {
        let choice = ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content: Some(capability_response.content),
            finish_reason: capability_response.finish_reason,
            tool_calls: None,
        };

        let usage = capability_response.usage.map(|u| UsageInfo {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        ChatResponse {
            id: format!("capability-{}", uuid::Uuid::new_v4()),
            model: capability_response.model,
            choices: vec![choice],
            usage,
        }
    }
}

#[async_trait]
impl AIProvider for CapabilityAIProvider {
    async fn process_chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        debug!(
            "Processing chat via capability: model={:?}, message_count={}",
            request.model,
            request.messages.len()
        );

        let model = request.model.as_deref().unwrap_or("gpt-3.5-turbo");
        let messages = Self::convert_messages(&request.messages);

        let options = request.parameters.as_ref().map(|params| ChatOptions {
            temperature: params.temperature,
            max_tokens: params.max_tokens,
            stream: params.stream,
        });

        let capability_response = self
            .client
            .chat_completion(model, messages, options)
            .await
            .map_err(|e| {
                warn!("Capability AI chat completion failed: {}", e);
                crate::Error::Network(format!("Capability AI error: {}", e))
            })?;

        let response = Self::convert_response(capability_response);

        debug!("Capability chat completion successful");

        Ok(response)
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> bool {
        // Try a simple operation to check if capability is available
        // For now, just return true if client was created successfully
        // In a full implementation, could do a ping/health JSON-RPC call
        true
    }

    fn capabilities(&self) -> &[AICapability] {
        // Capability-based provider supports all capabilities
        // (delegated to the actual provider via Songbird)
        &[
            AICapability::Chat,
            AICapability::Completion,
            AICapability::Embedding,
            AICapability::CodeGeneration,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_messages() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: Some("Hello!".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: Some("Hi there!".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let converted = CapabilityAIProvider::convert_messages(&messages);

        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].role, "user");
        assert_eq!(converted[0].content, "Hello!");
        assert_eq!(converted[1].role, "assistant");
        assert_eq!(converted[1].content, "Hi there!");
    }
}
