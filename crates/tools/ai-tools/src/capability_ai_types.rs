// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Chat-completion types and JSON-RPC wire format for `capability_ai`.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Chat message for chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role (e.g., "system", "user", "assistant")
    pub role: String,
    /// Message content text
    pub content: String,
}

impl ChatMessage {
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// Optional parameters for chat completion
#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    /// Sampling temperature (0.0–2.0). Higher values increase randomness.
    pub temperature: Option<f32>,
    /// Maximum number of tokens to generate.
    pub max_tokens: Option<u32>,
    /// Whether to stream the response incrementally.
    pub stream: Option<bool>,
    /// Nucleus sampling parameter (alternative to temperature).
    pub top_p: Option<f32>,
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Generated text content from the model.
    pub content: String,
    /// Model identifier that produced the response.
    pub model: String,
    /// Reason generation stopped (e.g., "stop", "length").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    /// Token usage statistics if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Number of tokens in the input prompt.
    pub prompt_tokens: u32,
    /// Number of tokens in the generated completion.
    pub completion_tokens: u32,
    /// Total tokens (prompt + completion).
    pub total_tokens: u32,
}

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize)]
pub(crate) struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: JsonValue,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize)]
pub(crate) struct JsonRpcResponse {
    #[expect(dead_code, reason = "deserialized from JSON-RPC at runtime")]
    pub jsonrpc: String,
    #[expect(dead_code, reason = "deserialized from JSON-RPC at runtime")]
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Deserialize)]
pub(crate) struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[expect(dead_code, reason = "deserialized from JSON-RPC at runtime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}
