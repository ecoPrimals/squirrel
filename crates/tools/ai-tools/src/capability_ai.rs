// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability-Based AI Client (TRUE PRIMAL!)
//!
//! **Evolution**: Direct HTTP → Capability Discovery
//! - OLD: Direct `reqwest` to OpenAI/Anthropic (brings `ring`!)
//! - NEW: Discover "ai.chat" capability (Pure Rust via Songbird!)
//!
//! **Philosophy**: Deploy like an infant - knows nothing, discovers everything!
//! - Squirrel doesn't know "Songbird" exists
//! - Squirrel asks: "Who provides ai.chat.completion?"
//! - Runtime answers: a path from capability discovery (often under `$XDG_RUNTIME_DIR/biomeos/`)
//! - Could be Songbird, could be any AI proxy primal!

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};
use tracing::{debug, info, warn};
use universal_constants::network::resolve_capability_unix_socket;

/// AI client configuration (capability-based!)
#[derive(Debug, Clone)]
pub struct AiClientConfig {
    /// Path to AI capability provider's Unix socket
    /// (Discovered at runtime, NOT hardcoded!)
    pub socket_path: PathBuf,

    /// Timeout for socket operations (default: 30 seconds for AI calls)
    pub timeout_secs: u64,

    /// Number of connection retries (default: 3)
    pub max_retries: usize,

    /// Delay between retries in milliseconds (default: 100ms)
    pub retry_delay_ms: u64,
}

impl Default for AiClientConfig {
    fn default() -> Self {
        Self {
            // Tiered resolution: AI_CAPABILITY_SOCKET → SQUIRREL_SOCKET → … → XDG/biomeos fallback
            socket_path: resolve_capability_unix_socket("AI_CAPABILITY_SOCKET", "ai-provider"),
            timeout_secs: 30, // AI calls can take longer
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

/// Capability-based AI client (TRUE PRIMAL!)
///
/// **NO hardcoded primal names!**
/// - Discovers "ai.chat.completion" capability at runtime
/// - Connects to whichever primal provides it
/// - Currently: Might be Songbird, might be any AI proxy primal
/// - Future: Could be multiple providers with failover
///
/// # Examples
///
/// ```no_run
/// use squirrel_ai_tools::capability_ai::{AiClient, AiClientConfig, ChatMessage};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Socket path comes from capability discovery (NOT hardcoded!)
///     let config = AiClientConfig {
///         socket_path: "/run/user/1000/biomeos/ai-provider.sock".into(),  // From discovery!
///         ..Default::default()
///     };
///     
///     let client = AiClient::new(config)?;
///     
///     let messages = vec![
///         ChatMessage {
///             role: "user".to_string(),
///             content: "Hello, ecoPrimals!".to_string(),
///         }
///     ];
///     
///     let response = client.chat_completion("gpt-4", messages, None).await?;
///     println!("AI response: {}", response.content);
///     
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct AiClient {
    config: AiClientConfig,
    request_id: std::sync::atomic::AtomicU64,
}

impl AiClient {
    /// Create a new AI client from capability discovery
    ///
    /// **IMPORTANT**: Socket path should come from capability discovery!
    /// Do NOT hardcode primal names like "songbird-nat0.sock"!
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok`; reserved for future configuration validation.
    pub fn new(config: AiClientConfig) -> Result<Self> {
        info!(
            "Initializing capability-based AI client: {}",
            config.socket_path.display()
        );

        // Don't validate socket exists (may not be started yet)
        // Discovery system handles availability

        Ok(Self {
            config,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Create from environment (for bootstrapping)
    ///
    /// Reads `AI_CAPABILITY_SOCKET` from environment.
    /// This should be set by capability discovery at startup!
    ///
    /// # Errors
    ///
    /// Propagates errors from [`Self::new`].
    pub fn from_env() -> Result<Self> {
        let config = AiClientConfig::default();
        info!(
            "AI client from env: AI_CAPABILITY_SOCKET={}",
            config.socket_path.display()
        );
        Self::new(config)
    }

    /// Chat completion via discovered AI capability
    ///
    /// # Arguments
    /// * `model` - Model name (e.g., "gpt-4", "claude-3-opus")
    /// * `messages` - Chat messages
    /// * `options` - Optional parameters (temperature, `max_tokens`, etc.)
    ///
    /// # Returns
    /// Chat completion response
    ///
    /// # Errors
    ///
    /// Returns an error when the JSON-RPC request fails or the response body cannot be parsed.
    pub async fn chat_completion(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        options: Option<ChatOptions>,
    ) -> Result<ChatResponse> {
        debug!(
            "Chat completion request via capability: model={}, message_count={}",
            model,
            messages.len()
        );

        let mut params = serde_json::json!({
            "model": model,
            "messages": messages
        });

        if let Some(opts) = options {
            if let Some(temp) = opts.temperature {
                params["temperature"] = serde_json::json!(temp);
            }
            if let Some(max_tokens) = opts.max_tokens {
                params["max_tokens"] = serde_json::json!(max_tokens);
            }
            if let Some(stream) = opts.stream {
                params["stream"] = serde_json::json!(stream);
            }
        }

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "ai.chat.completion".to_string(),
            params,
        };

        let response = self.send_request(request).await?;

        let chat_response: ChatResponse = serde_json::from_value(response["result"].clone())
            .context("Failed to parse chat completion response from AI capability")?;

        debug!(
            "Chat completion successful: content_len={}",
            chat_response.content.len()
        );

        Ok(chat_response)
    }

    /// Create text embedding via discovered AI capability
    ///
    /// # Arguments
    /// * `model` - Embedding model name (e.g., "text-embedding-3-small")
    /// * `input` - Text to embed
    ///
    /// # Returns
    /// Embedding vector
    ///
    /// # Errors
    ///
    /// Returns an error when the JSON-RPC request fails or the embedding vector cannot be parsed.
    pub async fn create_embedding(&self, model: &str, input: &str) -> Result<Vec<f32>> {
        debug!(
            "Embedding request via capability: model={}, input_len={}",
            model,
            input.len()
        );

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "ai.embedding.create".to_string(),
            params: serde_json::json!({
                "model": model,
                "input": input
            }),
        };

        let response = self.send_request(request).await?;

        let embedding: Vec<f32> =
            serde_json::from_value(response["result"]["embedding"].clone())
                .context("Failed to parse embedding from AI capability response")?;

        debug!("Embedding successful: dimension={}", embedding.len());

        Ok(embedding)
    }

    /// Simple text generation via discovered AI capability
    ///
    /// # Arguments
    /// * `model` - Model name
    /// * `prompt` - Text prompt
    /// * `max_tokens` - Optional max tokens to generate
    ///
    /// # Returns
    /// Generated text
    ///
    /// # Errors
    ///
    /// Returns an error when the JSON-RPC request fails or the generated text cannot be read.
    pub async fn text_generation(
        &self,
        model: &str,
        prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        debug!(
            "Text generation request via capability: model={}, prompt_len={}",
            model,
            prompt.len()
        );

        let mut params = serde_json::json!({
            "model": model,
            "prompt": prompt
        });

        if let Some(max) = max_tokens {
            params["max_tokens"] = serde_json::json!(max);
        }

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "ai.text.generation".to_string(),
            params,
        };

        let response = self.send_request(request).await?;

        let text = response["result"]["text"]
            .as_str()
            .context("Missing 'text' in AI capability response")?
            .to_string();

        debug!("Text generation successful: text_len={}", text.len());

        Ok(text)
    }

    /// Send JSON-RPC request to AI capability with retry logic
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonValue> {
        let mut last_error = None;

        for attempt in 1..=self.config.max_retries {
            match self.send_request_once(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!(
                        "AI capability request failed (attempt {}/{}): {}",
                        attempt, self.config.max_retries, e
                    );
                    last_error = Some(e);

                    if attempt < self.config.max_retries {
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!(
                "AI capability request failed after {} retries",
                self.config.max_retries
            )
        }))
    }

    /// Send a single JSON-RPC request to AI capability provider
    async fn send_request_once(&self, request: &JsonRpcRequest) -> Result<JsonValue> {
        // Connect to Unix socket with timeout
        let stream = timeout(
            Duration::from_secs(self.config.timeout_secs),
            UnixStream::connect(&self.config.socket_path),
        )
        .await
        .context("Timeout connecting to AI capability socket")?
        .context("Failed to connect to AI capability socket")?;

        let (read_half, mut write_half) = stream.into_split();

        // Serialize request
        let request_json =
            serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

        // Send request (with newline delimiter)
        timeout(Duration::from_secs(self.config.timeout_secs), async {
            write_half.write_all(request_json.as_bytes()).await?;
            write_half.write_all(b"\n").await?;
            write_half.flush().await?;
            Ok::<(), std::io::Error>(())
        })
        .await
        .context("Timeout sending request to AI capability")?
        .context("Failed to send request to AI capability")?;

        // Read response (newline-delimited JSON)
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();

        timeout(
            Duration::from_secs(self.config.timeout_secs),
            reader.read_line(&mut response_line),
        )
        .await
        .context("Timeout reading response from AI capability")?
        .context("Failed to read response from AI capability")?;

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .context("Failed to parse JSON-RPC response from AI capability")?;

        // Check for JSON-RPC error
        if let Some(error) = response.error {
            return Err(anyhow::anyhow!(
                "AI capability error: {} (code: {})",
                error.message,
                error.code
            ));
        }

        response
            .result
            .context("AI capability response missing 'result' field")
    }

    /// Get next request ID (monotonically increasing)
    fn next_request_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

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
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: JsonValue,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[expect(dead_code, reason = "deserialized from JSON-RPC at runtime")]
    jsonrpc: String,
    #[expect(dead_code, reason = "deserialized from JSON-RPC at runtime")]
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[expect(dead_code, reason = "deserialized from JSON-RPC at runtime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<JsonValue>,
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::items_after_statements,
        reason = "Local helpers after setup in macro-like blocks"
    )]

    use super::*;

    #[test]
    fn test_ai_client_creation() {
        let config = AiClientConfig::default();
        let client = AiClient::new(config);

        assert!(client.is_ok());
    }

    #[test]
    fn test_ai_client_from_env() {
        temp_env::with_var("AI_CAPABILITY_SOCKET", Some("/tmp/test-ai.sock"), || {
            let client = AiClient::from_env();
            assert!(client.is_ok());
        });
    }

    #[test]
    fn test_request_id_increments() {
        let config = AiClientConfig::default();
        let client = AiClient::new(config).expect("should succeed");

        let id1 = client.next_request_id();
        let id2 = client.next_request_id();
        let id3 = client.next_request_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_chat_message_serialization() {
        let message = ChatMessage {
            role: "user".to_string(),
            content: "Hello!".to_string(),
        };

        let json = serde_json::to_string(&message).expect("should succeed");
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello!\""));
    }

    #[test]
    fn test_chat_message_constructors_and_roundtrip() {
        let sys = ChatMessage::system("sys");
        assert_eq!(sys.role, "system");
        assert_eq!(sys.content, "sys");
        let user = ChatMessage::user("u");
        assert_eq!(user.role, "user");
        let asst = ChatMessage::assistant("a");
        assert_eq!(asst.role, "assistant");

        let msg = ChatMessage::user("payload");
        let json = serde_json::to_string(&msg).expect("should succeed");
        let back: ChatMessage = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(back.content, "payload");
    }

    #[test]
    fn test_chat_response_and_usage_serde_roundtrip() {
        let resp = ChatResponse {
            content: "out".to_string(),
            model: "m".to_string(),
            finish_reason: Some("stop".to_string()),
            usage: Some(Usage {
                prompt_tokens: 1,
                completion_tokens: 2,
                total_tokens: 3,
            }),
        };
        let json = serde_json::to_string(&resp).expect("should succeed");
        let back: ChatResponse = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(back.content, "out");
        assert_eq!(back.usage.as_ref().expect("should succeed").total_tokens, 3);
    }

    #[test]
    fn test_chat_options_default() {
        let o = ChatOptions::default();
        assert!(o.temperature.is_none());
        assert!(o.max_tokens.is_none());
        assert!(o.stream.is_none());
        assert!(o.top_p.is_none());
    }

    #[test]
    fn test_ai_client_config_default_socket() {
        temp_env::with_vars_unset(["AI_CAPABILITY_SOCKET"], || {
            let c = AiClientConfig::default();
            assert_eq!(c.timeout_secs, 30);
            assert_eq!(c.max_retries, 3);
            assert!(c.socket_path.to_string_lossy().contains("provider"));
        });
    }

    /// JSON-RPC envelope the client expects: inner `result` wraps a `result` object for chat.
    #[tokio::test]
    async fn chat_completion_happy_path_over_unix_socket() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock = dir.path().join("ai.sock");
        let sock_clone = sock.clone();

        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&sock_clone).expect("should succeed");
            let (stream, _) = listener.accept().await.expect("should succeed");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let (read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            let mut reader = BufReader::new(read_half);
            reader.read_line(&mut line).await.expect("should succeed");
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "result": {
                        "content": "hello-cap",
                        "model": "gpt-test",
                        "finish_reason": "stop",
                        "usage": { "prompt_tokens": 1, "completion_tokens": 2, "total_tokens": 3 }
                    }
                }
            });
            write_half
                .write_all(body.to_string().as_bytes())
                .await
                .expect("should succeed");
            write_half.write_all(b"\n").await.expect("should succeed");
        });

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let config = AiClientConfig {
            socket_path: sock,
            timeout_secs: 5,
            max_retries: 1,
            retry_delay_ms: 1,
        };
        let client = AiClient::new(config).expect("should succeed");
        let out = client
            .chat_completion(
                "gpt-test",
                vec![ChatMessage::user("hi")],
                Some(ChatOptions {
                    temperature: Some(0.5),
                    max_tokens: Some(10),
                    stream: Some(false),
                    top_p: None,
                }),
            )
            .await
            .expect("should succeed");
        assert_eq!(out.content, "hello-cap");
        assert_eq!(out.model, "gpt-test");
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn create_embedding_over_unix_socket() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock = dir.path().join("emb.sock");
        let sock_clone = sock.clone();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&sock_clone).expect("should succeed");
            let (stream, _) = listener.accept().await.expect("should succeed");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let (read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            let mut reader = BufReader::new(read_half);
            reader.read_line(&mut line).await.expect("should succeed");
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "result": { "embedding": [0.25, -1.0, 0.0] }
                }
            });
            write_half
                .write_all(body.to_string().as_bytes())
                .await
                .expect("should succeed");
            write_half.write_all(b"\n").await.expect("should succeed");
        });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let client = AiClient::new(AiClientConfig {
            socket_path: sock,
            timeout_secs: 5,
            max_retries: 1,
            retry_delay_ms: 1,
        })
        .expect("should succeed");
        let emb = client
            .create_embedding("emb", "text")
            .await
            .expect("should succeed");
        assert_eq!(emb.len(), 3);
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn text_generation_over_unix_socket() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock = dir.path().join("tg.sock");
        let sock_clone = sock.clone();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&sock_clone).expect("should succeed");
            let (stream, _) = listener.accept().await.expect("should succeed");
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let (read_half, mut write_half) = stream.into_split();
            let mut line = String::new();
            let mut reader = BufReader::new(read_half);
            reader.read_line(&mut line).await.expect("should succeed");
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": { "result": { "text": "generated" } }
            });
            write_half
                .write_all(body.to_string().as_bytes())
                .await
                .expect("should succeed");
            write_half.write_all(b"\n").await.expect("should succeed");
        });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let client = AiClient::new(AiClientConfig {
            socket_path: sock,
            timeout_secs: 5,
            max_retries: 1,
            retry_delay_ms: 1,
        })
        .expect("should succeed");
        let text = client
            .text_generation("m", "p", Some(16))
            .await
            .expect("should succeed");
        assert_eq!(text, "generated");
        server.await.expect("should succeed");
    }

    #[tokio::test]
    async fn json_rpc_error_from_provider() {
        let dir = tempfile::tempdir().expect("should succeed");
        let sock = dir.path().join("err.sock");
        let sock_clone = sock.clone();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&sock_clone).expect("should succeed");
            for _ in 0..3 {
                let (stream, _) = listener.accept().await.expect("should succeed");
                use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
                let (read_half, mut write_half) = stream.into_split();
                let mut line = String::new();
                let mut reader = BufReader::new(read_half);
                reader.read_line(&mut line).await.expect("should succeed");
                let body = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "error": { "code": -1, "message": "capability down" }
                });
                write_half
                    .write_all(body.to_string().as_bytes())
                    .await
                    .expect("should succeed");
                write_half.write_all(b"\n").await.expect("should succeed");
            }
        });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let client = AiClient::new(AiClientConfig {
            socket_path: sock,
            timeout_secs: 5,
            max_retries: 3,
            retry_delay_ms: 1,
        })
        .expect("should succeed");
        let err = client.text_generation("m", "p", None).await.unwrap_err();
        assert!(err.to_string().contains("capability down"));
        server.await.expect("should succeed");
    }

    // Integration tests (require AI capability provider running) are in integration tests
}
