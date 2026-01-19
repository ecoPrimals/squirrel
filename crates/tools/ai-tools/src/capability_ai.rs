//! Capability-Based AI Client (TRUE PRIMAL!)
//!
//! **Evolution**: Direct HTTP → Capability Discovery
//! - OLD: Direct `reqwest` to OpenAI/Anthropic (brings `ring`!)
//! - NEW: Discover "ai.chat" capability (Pure Rust via Songbird!)
//!
//! **Philosophy**: Deploy like an infant - knows nothing, discovers everything!
//! - Squirrel doesn't know "Songbird" exists
//! - Squirrel asks: "Who provides ai.chat.completion?"
//! - Runtime answers: "Service at /var/run/ai/provider.sock"
//! - Could be Songbird, could be any AI proxy primal!

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};

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
            // Default path from environment or discovery
            socket_path: std::env::var("AI_CAPABILITY_SOCKET")
                .unwrap_or_else(|_| "/var/run/ai/provider.sock".to_string())
                .into(),
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
///         socket_path: "/var/run/ai/provider.sock".into(),  // From discovery!
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
    /// * `options` - Optional parameters (temperature, max_tokens, etc.)
    ///
    /// # Returns
    /// Chat completion response
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
    pub role: String,
    pub content: String,
}

/// Optional parameters for chat completion
#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
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
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<JsonValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_client_creation() {
        let config = AiClientConfig::default();
        let client = AiClient::new(config);

        assert!(client.is_ok());
    }

    #[test]
    fn test_ai_client_from_env() {
        std::env::set_var("AI_CAPABILITY_SOCKET", "/tmp/test-ai.sock");

        let client = AiClient::from_env();
        assert!(client.is_ok());

        std::env::remove_var("AI_CAPABILITY_SOCKET");
    }

    #[test]
    fn test_request_id_increments() {
        let config = AiClientConfig::default();
        let client = AiClient::new(config).unwrap();

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

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello!\""));
    }

    // Integration tests (require AI capability provider running) are in integration tests
}
