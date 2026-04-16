// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use serde_json::json;
use std::path::Path;
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use crate::api::ai::universal::{
    ChatMessage, CostTier, MessageRole, ProviderType, UniversalAiRequest,
};

fn bind_unix_listener(path: &Path) -> UnixListener {
    let _ = std::fs::remove_file(path);
    UnixListener::bind(path).expect("bind unix listener")
}

#[tokio::test]
async fn complete_parses_jsonrpc_success_with_text_and_usage() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("ai.sock");
    let listener = bind_unix_listener(&sock);
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("read req");
        let mut stream = reader.into_inner();
        let body = json!({
            "jsonrpc": "2.0",
            "result": {
                "text": "completion-out",
                "model": "mm",
                "usage": {"prompt_tokens": 1, "completion_tokens": 2, "total_tokens": 3},
                "stop_reason": "length",
                "cost_usd": 0.02
            }
        });
        stream
            .write_all(format!("{}\n", serde_json::to_string(&body).unwrap()).as_bytes())
            .await
            .expect("write");
    });

    let provider = CapabilityProvider {
        id: "uds".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: sock.clone(),
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };
    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
        .await
        .expect("adapter");

    let mut req = UniversalAiRequest::default();
    req.prompt = Some("hello".to_string());
    req.stream = true;
    req.stop = Some(vec!["</s>".to_string()]);

    let out = adapter.complete(req).await.expect("complete");
    assert_eq!(out.text, "completion-out");
    assert_eq!(out.model.as_ref(), "mm");
    assert!(out.usage.is_some());
    assert_eq!(out.stop_reason.as_deref(), Some("length"));
    assert!((out.cost_usd.unwrap() - 0.02).abs() < f64::EPSILON);
    server.await.ok();
}

#[tokio::test]
async fn complete_accepts_content_instead_of_text() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("ai2.sock");
    let listener = bind_unix_listener(&sock);
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.ok();
        let mut stream = reader.into_inner();
        let body = json!({
            "jsonrpc": "2.0",
            "result": {
                "content": "from-content",
                "model": "m2"
            }
        });
        stream
            .write_all(format!("{}\n", serde_json::to_string(&body).unwrap()).as_bytes())
            .await
            .expect("write");
    });

    let provider = CapabilityProvider {
        id: "uds2".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: sock,
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };
    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
        .await
        .expect("adapter");

    let mut req = UniversalAiRequest::default();
    req.prompt = Some("p".to_string());

    let out = adapter.complete(req).await.expect("complete");
    assert_eq!(out.text, "from-content");
    server.await.ok();
}

#[tokio::test]
async fn complete_maps_finish_reason_to_stop_reason() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("ai3.sock");
    let listener = bind_unix_listener(&sock);
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.ok();
        let mut stream = reader.into_inner();
        let body = json!({
            "jsonrpc": "2.0",
            "result": {
                "text": "t",
                "finish_reason": "stop"
            }
        });
        stream
            .write_all(format!("{}\n", serde_json::to_string(&body).unwrap()).as_bytes())
            .await
            .expect("write");
    });

    let provider = CapabilityProvider {
        id: "uds3".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: sock,
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };
    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
        .await
        .expect("adapter");

    let mut req = UniversalAiRequest::default();
    req.prompt = Some("p".to_string());

    let out = adapter.complete(req).await.expect("complete");
    assert_eq!(out.stop_reason.as_deref(), Some("stop"));
    server.await.ok();
}

#[tokio::test]
async fn complete_errors_when_result_has_no_text_or_content() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("ai4.sock");
    let listener = bind_unix_listener(&sock);
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.ok();
        let mut stream = reader.into_inner();
        let body = json!({
            "jsonrpc": "2.0",
            "result": {"model": "only-model"}
        });
        stream
            .write_all(format!("{}\n", serde_json::to_string(&body).unwrap()).as_bytes())
            .await
            .expect("write");
    });

    let provider = CapabilityProvider {
        id: "uds4".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: sock,
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };
    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
        .await
        .expect("adapter");

    let mut req = UniversalAiRequest::default();
    req.prompt = Some("p".to_string());

    let err = adapter.complete(req).await.expect_err("no text");
    assert!(matches!(err, PrimalError::ParsingError(_)));
    server.await.ok();
}

#[tokio::test]
async fn complete_surfaces_jsonrpc_error_response() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("ai5.sock");
    let listener = bind_unix_listener(&sock);
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.ok();
        let mut stream = reader.into_inner();
        let body = json!({
            "jsonrpc": "2.0",
            "error": {"code": -32000, "message": "provider boom"}
        });
        stream
            .write_all(format!("{}\n", serde_json::to_string(&body).unwrap()).as_bytes())
            .await
            .expect("write");
    });

    let provider = CapabilityProvider {
        id: "uds5".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: sock,
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };
    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
        .await
        .expect("adapter");

    let mut req = UniversalAiRequest::default();
    req.prompt = Some("p".to_string());

    let err = adapter.complete(req).await.expect_err("rpc err");
    assert!(matches!(err, PrimalError::NetworkError(_)));
    server.await.ok();
}

#[tokio::test]
async fn complete_includes_messages_in_params() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("ai6.sock");
    let listener = bind_unix_listener(&sock);
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("read");
        let req: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
        let params = req.get("params").expect("params");
        assert!(params.get("messages").is_some());
        let mut stream = reader.into_inner();
        let body = json!({
            "jsonrpc": "2.0",
            "result": {"text": "ok", "model": "m"}
        });
        stream
            .write_all(format!("{}\n", serde_json::to_string(&body).unwrap()).as_bytes())
            .await
            .expect("write");
    });

    let provider = CapabilityProvider {
        id: "uds6".to_string(),
        capabilities: vec!["ai.chat".to_string()],
        socket: sock,
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };
    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.chat".to_string())
        .await
        .expect("adapter");

    let mut req = UniversalAiRequest::default();
    req.messages = Some(vec![ChatMessage {
        role: MessageRole::User,
        content: "u".to_string(),
        name: None,
    }]);

    let out = adapter.complete(req).await.expect("complete");
    assert_eq!(out.text, "ok");
    server.await.ok();
}

#[test]
fn test_extract_metadata() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("provider_type".to_string(), "local".to_string());
    metadata_map.insert(
        "models".to_string(),
        r#"["model-1", "model-2"]"#.to_string(),
    );
    metadata_map.insert("cost_tier".to_string(), "free".to_string());

    let provider = CapabilityProvider {
        id: "test-provider".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: PathBuf::from("/tmp/test.sock"),
        metadata: metadata_map,
        discovered_via: "test".to_string(),
    };

    let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");

    assert_eq!(metadata.name, "test-provider");
    assert_eq!(metadata.provider_type, ProviderType::Local);
    assert_eq!(metadata.models.len(), 2);
    assert_eq!(metadata.cost_tier, Some(CostTier::Free));
}

#[tokio::test]
async fn test_adapter_is_available_no_socket() {
    let provider = CapabilityProvider {
        id: "test-provider".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: PathBuf::from("/tmp/nonexistent-test.sock"),
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };

    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
        .await
        .expect("should succeed");

    // Should return false, not panic
    let available = adapter.is_available().await;
    assert!(!available);
}

#[test]
fn test_extract_metadata_cloud_provider() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("provider_type".to_string(), "cloud".to_string());
    metadata_map.insert("cost_tier".to_string(), "high".to_string());

    let provider = CapabilityProvider {
        id: "cloud-provider".to_string(),
        capabilities: vec!["ai.complete".to_string(), "ai.chat".to_string()],
        socket: PathBuf::from("/tmp/cloud.sock"),
        metadata: metadata_map,
        discovered_via: "discovery.find_primals".to_string(),
    };

    let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");

    assert_eq!(metadata.name, "cloud-provider");
    assert_eq!(metadata.provider_type, ProviderType::Cloud);
    assert_eq!(metadata.cost_tier, Some(CostTier::High));
    assert_eq!(metadata.capabilities.len(), 2);
}

#[test]
fn test_extract_metadata_custom_provider() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("provider_type".to_string(), "custom".to_string());
    metadata_map.insert("cost_tier".to_string(), "low".to_string());

    let provider = CapabilityProvider {
        id: "custom-provider".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: PathBuf::from("/tmp/custom.sock"),
        metadata: metadata_map,
        discovered_via: "manual".to_string(),
    };

    let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");

    assert_eq!(metadata.provider_type, ProviderType::Custom);
    assert_eq!(metadata.cost_tier, Some(CostTier::Low));
}

#[test]
fn test_extract_metadata_medium_cost() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("cost_tier".to_string(), "medium".to_string());

    let provider = CapabilityProvider {
        id: "med-provider".to_string(),
        capabilities: vec![],
        socket: PathBuf::from("/tmp/med.sock"),
        metadata: metadata_map,
        discovered_via: "test".to_string(),
    };

    let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");
    assert_eq!(metadata.cost_tier, Some(CostTier::Medium));
}

#[test]
fn test_extract_metadata_unknown_provider_type() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("provider_type".to_string(), "unknown_type".to_string());

    let provider = CapabilityProvider {
        id: "unknown-provider".to_string(),
        capabilities: vec![],
        socket: PathBuf::from("/tmp/unknown.sock"),
        metadata: metadata_map,
        discovered_via: "test".to_string(),
    };

    let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");
    // Unknown provider type should fall back to Custom
    assert_eq!(metadata.provider_type, ProviderType::Custom);
}

#[test]
fn test_extract_metadata_no_provider_type() {
    let provider = CapabilityProvider {
        id: "bare-provider".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: PathBuf::from("/tmp/bare.sock"),
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };

    let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");

    // No provider type → default Custom
    assert_eq!(metadata.provider_type, ProviderType::Custom);
    assert!(metadata.models.is_empty());
    assert!(metadata.cost_tier.is_none());
    assert!(metadata.avg_latency_ms.is_none());
}

#[test]
fn test_extract_metadata_invalid_models_json() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("models".to_string(), "not valid json".to_string());

    let provider = CapabilityProvider {
        id: "bad-models".to_string(),
        capabilities: vec![],
        socket: PathBuf::from("/tmp/bad.sock"),
        metadata: metadata_map,
        discovered_via: "test".to_string(),
    };

    let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");
    // Invalid JSON models → empty vec
    assert!(metadata.models.is_empty());
}

#[test]
fn test_extract_metadata_unknown_cost_tier() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("cost_tier".to_string(), "premium".to_string());

    let provider = CapabilityProvider {
        id: "premium-provider".to_string(),
        capabilities: vec![],
        socket: PathBuf::from("/tmp/premium.sock"),
        metadata: metadata_map,
        discovered_via: "test".to_string(),
    };

    let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");
    // Unknown cost tier → None
    assert!(metadata.cost_tier.is_none());
}

#[test]
fn test_extract_metadata_extra_fields() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("custom_key".to_string(), "custom_value".to_string());
    metadata_map.insert("region".to_string(), "us-west-2".to_string());

    let provider = CapabilityProvider {
        id: "extra-provider".to_string(),
        capabilities: vec![],
        socket: PathBuf::from("/tmp/extra.sock"),
        metadata: metadata_map,
        discovered_via: "test".to_string(),
    };

    let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");
    assert!(metadata.extra.contains_key("custom_key"));
    assert!(metadata.extra.contains_key("region"));
}

#[tokio::test]
async fn test_adapter_capabilities() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("provider_type".to_string(), "local".to_string());

    let provider = CapabilityProvider {
        id: "cap-test".to_string(),
        capabilities: vec![
            "ai.complete".to_string(),
            "ai.chat".to_string(),
            "ai.embeddings".to_string(),
        ],
        socket: PathBuf::from("/tmp/cap-test.sock"),
        metadata: metadata_map,
        discovered_via: "test".to_string(),
    };

    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
        .await
        .expect("should succeed");

    let caps = adapter.capabilities();
    assert_eq!(caps.len(), 3);
    assert!(caps.contains(&"ai.complete".to_string()));
    assert!(caps.contains(&"ai.chat".to_string()));
    assert!(caps.contains(&"ai.embeddings".to_string()));
}

#[tokio::test]
async fn test_adapter_metadata() {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("provider_type".to_string(), "local".to_string());
    metadata_map.insert(
        "models".to_string(),
        r#"["llama-3", "mistral-7b"]"#.to_string(),
    );

    let provider = CapabilityProvider {
        id: "meta-test".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: PathBuf::from("/tmp/meta-test.sock"),
        metadata: metadata_map,
        discovered_via: "test".to_string(),
    };

    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
        .await
        .expect("should succeed");

    let meta = adapter.metadata();
    assert_eq!(meta.name, "meta-test");
    assert_eq!(meta.provider_type, ProviderType::Local);
    assert_eq!(meta.models.len(), 2);
}

#[tokio::test]
async fn test_adapter_provider_id() {
    let provider = CapabilityProvider {
        id: "id-test-provider".to_string(),
        capabilities: vec!["ai.complete".to_string()],
        socket: PathBuf::from("/tmp/id-test.sock"),
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };

    let adapter = UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
        .await
        .expect("should succeed");

    assert_eq!(adapter.provider_id(), "id-test-provider");
}
