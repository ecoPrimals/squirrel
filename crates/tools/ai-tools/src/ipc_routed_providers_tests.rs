// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;

use super::ipc_routed_parsers::json_u64_as_u32_saturating;
use super::ipc_routed_providers_mocks::MockNeuralHttp;
use super::*;
use crate::common::capability::ModelType;
use crate::common::client::AIClient;
use crate::common::types::{ChatMessage, ChatRequest, MessageRole};

fn msg(role: MessageRole, content: &str) -> ChatMessage {
    ChatMessage {
        role,
        content: Some(content.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    }
}

#[test]
fn vendor_kind_default_models_match_provider_names() {
    let cases = [
        (VendorKind::OpenAI, "gpt-4o-mini", "openai-ipc"),
        (
            VendorKind::Anthropic,
            "claude-3-5-sonnet-20241022",
            "anthropic-ipc",
        ),
        (VendorKind::Gemini, "gemini-1.5-flash", "gemini-ipc"),
    ];
    for (vendor, expected_model, expected_name) in cases {
        let http = Arc::new(MockNeuralHttp::new());
        let client = IpcRoutedVendorClient::new_for_test(http, "k", vendor);
        assert_eq!(client.default_model(), expected_model);
        assert_eq!(client.provider_name(), expected_name);
    }
}

#[test]
fn json_u64_saturates_to_u32_max() {
    let n = u64::from(u32::MAX) + 1;
    assert_eq!(json_u64_as_u32_saturating(n), u32::MAX);
}

#[test]
fn parse_openai_response_maps_usage_and_content() {
    let body = r#"{
            "id": "chatcmpl-1",
            "model": "gpt-4o-mini",
            "choices": [{"message": {"content": "hi"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5}
        }"#;
    let r = parse_openai_chat_response(body).expect("should succeed");
    assert_eq!(r.id, "chatcmpl-1");
    assert_eq!(r.model, "gpt-4o-mini");
    assert_eq!(r.choices[0].content.as_deref(), Some("hi"));
    assert_eq!(r.choices[0].finish_reason.as_deref(), Some("stop"));
    let u = r.usage.expect("usage");
    assert_eq!(u.prompt_tokens, 3);
    assert_eq!(u.completion_tokens, 2);
    assert_eq!(u.total_tokens, 5);
}

#[test]
fn parse_openai_invalid_json_errors() {
    assert!(parse_openai_chat_response("not json").is_err());
}

#[test]
fn parse_anthropic_response_maps_text_block_and_usage() {
    let body = r#"{
            "id": "msg_1",
            "model": "claude-3-5-sonnet-20241022",
            "content": [{"type": "text", "text": "hello"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 20}
        }"#;
    let r = parse_anthropic_chat_response(body).expect("should succeed");
    assert_eq!(r.choices[0].content.as_deref(), Some("hello"));
    assert_eq!(r.choices[0].finish_reason.as_deref(), Some("end_turn"));
    let u = r.usage.expect("usage");
    assert_eq!(u.prompt_tokens, 10);
    assert_eq!(u.completion_tokens, 20);
    assert_eq!(u.total_tokens, 30);
}

#[test]
fn parse_gemini_response_concatenates_parts() {
    let body = r#"{"candidates":[{"content":{"parts":[{"text":"a"},{"text":"b"}]}}]}"#;
    let r = parse_gemini_chat_response(body, "gemini-1.5-flash").expect("should succeed");
    assert_eq!(r.choices[0].content.as_deref(), Some("ab"));
    assert_eq!(r.model, "gemini-1.5-flash");
}

#[tokio::test]
async fn chat_openai_builds_request_and_parses_mock_response() {
    let mock = Arc::new(MockNeuralHttp::new());
    mock
        .push_post_json(r#"{"id":"x","model":"m","choices":[{"message":{"content":"ok"},"finish_reason":null}],"usage":null}"#)
        .await;
    let client = IpcRoutedVendorClient::new_for_test(mock, "secret", VendorKind::OpenAI);
    let req = ChatRequest {
        model: Some("custom-model".to_string()),
        messages: vec![msg(MessageRole::System, "sys"), msg(MessageRole::User, "u")],
        parameters: None,
        tools: None,
    };
    let out = client.chat_openai(req).await.expect("should succeed");
    assert_eq!(out.choices[0].content.as_deref(), Some("ok"));
}

#[tokio::test]
async fn chat_anthropic_includes_system_and_skips_unknown_roles() {
    let mock = Arc::new(MockNeuralHttp::new());
    mock.push_post_json(
        r#"{"id":"a","model":"m","content":[{"type":"text","text":"y"}],"usage":null}"#,
    )
    .await;
    let client = IpcRoutedVendorClient::new_for_test(mock, "key", VendorKind::Anthropic);
    let req = ChatRequest {
        model: None,
        messages: vec![
            msg(MessageRole::System, "be nice"),
            msg(MessageRole::User, "hi"),
            msg(MessageRole::Tool, "ignored"),
        ],
        parameters: None,
        tools: None,
    };
    let out = client.chat_anthropic(req).await.expect("should succeed");
    assert_eq!(out.choices[0].content.as_deref(), Some("y"));
}

#[tokio::test]
async fn chat_gemini_posts_parts_from_user_and_system() {
    let mock = Arc::new(MockNeuralHttp::new());
    mock.push_post_json(r#"{"candidates":[{"content":{"parts":[{"text":"g"}]}}]}"#)
        .await;
    let client = IpcRoutedVendorClient::new_for_test(mock, "apikey", VendorKind::Gemini);
    let req = ChatRequest {
        model: Some("gemini-pro".to_string()),
        messages: vec![msg(MessageRole::System, "s"), msg(MessageRole::User, "u")],
        parameters: None,
        tools: None,
    };
    let out = client.chat_gemini(req).await.expect("should succeed");
    assert_eq!(out.choices[0].content.as_deref(), Some("g"));
}

#[tokio::test]
async fn list_models_openai_parses_data_array() {
    let mock = Arc::new(MockNeuralHttp::new());
    mock.push_get(r#"{"data":[{"id":"gpt-4"},{"id":"ada"}]}"#)
        .await;
    let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::OpenAI);
    let models = client.list_models().await.expect("should succeed");
    assert_eq!(models, vec!["gpt-4".to_string(), "ada".to_string()]);
}

#[tokio::test]
async fn list_models_anthropic_returns_default_only() {
    let mock = Arc::new(MockNeuralHttp::new());
    let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::Anthropic);
    let models = client.list_models().await.expect("should succeed");
    assert_eq!(models.len(), 1);
    assert!(models[0].contains("claude"));
}

#[tokio::test]
async fn chat_stream_is_unsupported() {
    let mock = Arc::new(MockNeuralHttp::new());
    let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::OpenAI);
    let res = client
        .chat_stream(ChatRequest {
            model: None,
            messages: vec![],
            parameters: None,
            tools: None,
        })
        .await;
    assert!(matches!(res, Err(Error::UnsupportedProvider(_))));
}

#[tokio::test]
async fn get_capabilities_sets_chat_and_context_window() {
    let mock = Arc::new(MockNeuralHttp::new());
    let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::OpenAI);
    let caps = client.get_capabilities("any").await.expect("capabilities");
    assert!(caps.max_context_size >= 128_000);
    assert!(caps.supported_model_types.contains(&ModelType::ChatModel));
}

#[tokio::test]
async fn is_available_is_true() {
    let mock = Arc::new(MockNeuralHttp::new());
    let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::Gemini);
    assert!(client.is_available().await);
}

#[test]
fn as_any_returns_ipc_client() {
    let mock = Arc::new(MockNeuralHttp::new());
    let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::OpenAI);
    assert!(
        client
            .as_any()
            .downcast_ref::<IpcRoutedVendorClient<MockNeuralHttp>>()
            .is_some()
    );
}

#[tokio::test]
async fn list_models_openai_invalid_json_errors() {
    let mock = Arc::new(MockNeuralHttp::new());
    mock.push_get("not-json").await;
    let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::OpenAI);
    let err = client.list_models().await.expect_err("parse error");
    assert!(matches!(err, Error::Parse(_)));
}

#[tokio::test]
async fn list_models_openai_empty_data_array() {
    let mock = Arc::new(MockNeuralHttp::new());
    mock.push_get(r#"{"data":[]}"#).await;
    let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::OpenAI);
    let models = client.list_models().await.expect("ok");
    assert!(models.is_empty());
}

#[test]
fn parse_openai_defaults_when_fields_missing() {
    let body = r#"{"choices":[{"message":{}}]}"#;
    let r = parse_openai_chat_response(body).expect("ok");
    assert_eq!(r.id, "openai-ipc");
    assert_eq!(r.model, "unknown");
    assert!(r.usage.is_none());
}

#[test]
fn parse_anthropic_without_text_block_leaves_content_none() {
    let body = r#"{"id":"x","model":"m","content":[{"type":"image"}],"usage":null}"#;
    let r = parse_anthropic_chat_response(body).expect("ok");
    assert!(r.choices[0].content.is_none());
}

#[test]
fn parse_gemini_empty_candidates() {
    let r = parse_gemini_chat_response("{}", "m").expect("ok");
    assert_eq!(r.choices[0].content.as_deref(), Some(""));
}

#[tokio::test]
async fn mock_neural_http_errors_when_queue_empty() {
    let mock = Arc::new(MockNeuralHttp::new());
    let err = mock
        .post_json("u", vec![], "")
        .await
        .expect_err("empty queue");
    assert!(err.to_string().contains("no post_json"));
}

#[tokio::test]
async fn chat_openai_http_error_maps_to_provider_error() {
    let mock = Arc::new(MockNeuralHttp::new());
    let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::OpenAI);
    let err = client
        .chat(ChatRequest {
            model: None,
            messages: vec![msg(MessageRole::User, "hi")],
            parameters: None,
            tools: None,
        })
        .await
        .expect_err("http");
    assert!(matches!(err, Error::Provider(_)));
}
