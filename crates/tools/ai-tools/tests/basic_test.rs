// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(warnings)]
use squirrel_ai_tools::{
    Result,
    common::{AIClient, ChatMessage, ChatRequest, MessageRole},
    dispatch::DispatcherBuilder,
};

// Fix MockAIClient import
use squirrel_ai_tools::common::clients::mock::MockAIClient;

#[tokio::test]
async fn test_basic_dispatcher_creation() -> Result<()> {
    let dispatcher = DispatcherBuilder::new().build().expect("build dispatcher");

    assert_eq!(dispatcher.router().get_provider_count(), 0);
    Ok(())
}

#[tokio::test]
async fn test_chat_request_creation() -> Result<()> {
    let request = ChatRequest {
        model: None,
        messages: vec![
            ChatMessage {
                role: MessageRole::User,
                content: Some("Hello, world!".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::System,
                content: Some("You are a helpful assistant.".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        parameters: None,
        tools: None,
    };

    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].role, MessageRole::User);
    assert_eq!(request.messages[1].role, MessageRole::System);

    Ok(())
}

#[tokio::test]
async fn test_mock_client_creation() -> Result<()> {
    let client = MockAIClient::new();
    assert_eq!(client.provider_name(), "mock");

    let models = client.list_models().await?;
    assert!(!models.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_mock_client_chat() -> Result<()> {
    let client = MockAIClient::new();

    let request = ChatRequest {
        model: Some("mock-model".to_string()),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some("Hello!".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
        parameters: None,
        tools: None,
    };

    let response = client.chat(request).await?;
    assert_eq!(response.choices.len(), 1);
    assert!(response.choices[0].content.is_some());

    Ok(())
}
