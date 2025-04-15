use squirrel_ai_tools::{
    common::{AIClient, ChatMessage, ChatRequest, ModelParameters},
    openai::OpenAIClient,
    config::Config,
};
use futures_util::StreamExt;
use secrecy::ExposeSecret;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_api_key() -> String {
        match Config::load() {
            Ok(config) => {
                let secret = config.openai_api_key.expose_secret();
                if !secret.0.is_empty() {
                    secret.0.clone()
                } else {
                    String::new()
                }
            }
            Err(_) => String::new(),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_live() {
        let api_key = get_api_key();
        let client = OpenAIClient::new(api_key);

        let request = ChatRequest::new()
            .add_user("Hello")
            .with_model("gpt-3.5-turbo")
            .with_parameters(ModelParameters {
                temperature: Some(0.7),
                top_p: Some(1.0),
                max_tokens: Some(100),
                frequency_penalty: Some(0.0),
                presence_penalty: Some(0.0),
                response_format: None,
                stop: None,
                stream: None,
            });

        let response = client.chat(request).await.unwrap();
        assert!(!response.choices.is_empty());
        assert!(!response.choices[0].content.as_ref().unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_live_stream() {
        let api_key = get_api_key();
        let client = OpenAIClient::new(api_key);

        let request = ChatRequest::new()
            .add_user("Hello")
            .with_model("gpt-3.5-turbo")
            .with_parameters(ModelParameters {
                temperature: Some(0.7),
                top_p: Some(1.0),
                max_tokens: Some(100),
                frequency_penalty: Some(0.0),
                presence_penalty: Some(0.0),
                response_format: None,
                stop: None,
                stream: Some(true),
            });

        let mut stream = client.chat_stream(request).await.unwrap();
        let mut chunks = Vec::new();

        while let Some(chunk) = stream.inner.next().await {
            chunks.push(chunk.unwrap());
        }

        assert!(!chunks.is_empty());
        assert!(chunks.iter().any(|chunk| chunk.content.is_some()));
    }
} 