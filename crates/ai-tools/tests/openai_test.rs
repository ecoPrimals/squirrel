use std::sync::Arc;
use futures::stream;
use futures_util::StreamExt;
use squirrel_ai_tools::{
    common::{AIClient, ChatMessage, ChatRequest, ChatResponse, ChatResponseChunk, ChatResponseStream, MessageRole, UsageInfo},
    Result,
};
use async_trait::async_trait;

struct MockOpenAIClient;

impl MockOpenAIClient {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AIClient for MockOpenAIClient {
    fn provider_name(&self) -> &str {
        "openai"
    }

    fn default_model(&self) -> &str {
        "gpt-3.5-turbo"
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec!["gpt-3.5-turbo".to_string()])
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        Ok(ChatResponse {
            choices: vec![ChatMessage {
                role: MessageRole::Assistant,
                content: Some("Hello! How can I help you today?".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            usage: Some(UsageInfo {
                prompt_tokens: 10,
                completion_tokens: 10,
                total_tokens: 20,
                estimated_cost_usd: Some(0.0),
            }),
        })
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatResponseStream> {
        let chunks = vec![
            ChatResponseChunk {
                role: Some("assistant".to_string()),
                content: Some("Hello".to_string()),
                tool_calls: None,
            },
            ChatResponseChunk {
                role: None,
                content: Some("! How can I help you today?".to_string()),
                tool_calls: None,
            },
        ];

        Ok(ChatResponseStream {
            inner: Box::new(Box::pin(stream::iter(chunks.into_iter().map(Ok)))),
        })
    }
}

#[tokio::test]
async fn test_openai_chat() {
    let client = MockOpenAIClient::new();
    let request = ChatRequest::new()
        .add_user("Hello")
        .with_model("gpt-3.5-turbo");

    let response = client.chat(request).await.unwrap();
    assert!(!response.choices[0].content.as_ref().unwrap().is_empty());
}

#[tokio::test]
async fn test_openai_chat_stream() {
    let client = MockOpenAIClient::new();
    let request = ChatRequest::new()
        .add_user("Hello")
        .with_model("gpt-3.5-turbo");

    let mut stream = client.chat_stream(request).await.unwrap();
    let mut chunks = Vec::new();

    while let Some(chunk) = stream.inner.next().await {
        chunks.push(chunk.unwrap());
    }

    assert!(!chunks.is_empty());
    assert!(chunks.iter().any(|chunk| chunk.content.is_some()));
}

#[tokio::test]
async fn test_openai_models() {
    let client = Arc::new(MockOpenAIClient);
    let models = client.list_models().await;
    assert!(models.is_ok());
    let models = models.unwrap();
    assert!(!models.is_empty());
    assert!(models.contains(&"gpt-3.5-turbo".to_string()));
}

#[tokio::test]
async fn test_openai_error_handling() {
    let client = Arc::new(MockOpenAIClient);
    let request = ChatRequest::new(); // Empty request should still work with our mock
    let response = client.chat(request).await;
    assert!(response.is_ok()); // Our mock always returns Ok
} 