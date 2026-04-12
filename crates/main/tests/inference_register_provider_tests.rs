// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Inference wire test mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Wire-style integration tests for `inference.register_provider` JSON-RPC.
//!
//! Uses a real `squirrel::rpc::JsonRpcServer` and the same newline-framed line
//! protocol as production Unix JSON-RPC (`handle_jsonrpc_loop`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde_json::{Value, json};
use squirrel::api::AiRouter;
use squirrel::rpc::JsonRpcServer;
use squirrel::rpc::jsonrpc_types::error_codes;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

/// Send one JSON-RPC request over UDS and read one JSON response line (production framing).
async fn jsonrpc_line_roundtrip(socket_path: &std::path::Path, request: &Value) -> Value {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .expect("connect to test JSON-RPC socket");
    let mut line = serde_json::to_string(request).expect("serialize request");
    line.push('\n');
    stream
        .write_all(line.as_bytes())
        .await
        .expect("write request");
    stream.flush().await.expect("flush");

    let mut buf = String::new();
    let mut reader = BufReader::new(stream);
    reader
        .read_line(&mut buf)
        .await
        .expect("read response line");
    serde_json::from_str(buf.trim()).expect("parse JSON-RPC response")
}

/// Bind `path`, spawn the same read-line / write-line loop the server uses for JSON-RPC over UDS.
fn spawn_line_framed_jsonrpc_server(
    path: &Path,
    server: Arc<JsonRpcServer>,
) -> tokio::task::JoinHandle<()> {
    let _ = std::fs::remove_file(path);
    let listener = UnixListener::bind(path).expect("bind unix socket for JSON-RPC test");
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let srv = Arc::clone(&server);
            tokio::spawn(async move {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                loop {
                    line.clear();
                    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                        break;
                    }
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Some(resp) = srv.test_handle_jsonrpc_line(trimmed).await {
                        let mut out = resp;
                        out.push('\n');
                        let w = reader.get_mut();
                        if w.write_all(out.as_bytes()).await.is_err() {
                            break;
                        }
                        if w.flush().await.is_err() {
                            break;
                        }
                    }
                }
            });
        }
    })
}

/// Mock neuralSpring: accepts `inference.complete` and returns a deterministic result.
fn spawn_mock_neural_spring(
    socket_path: &Path,
    saw_inference_complete: Arc<AtomicBool>,
) -> tokio::task::JoinHandle<()> {
    let _ = std::fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path).expect("bind mock spring socket");
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            let flag = Arc::clone(&saw_inference_complete);
            tokio::spawn(async move {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                    return;
                }
                let request: Value = match serde_json::from_str(line.trim()) {
                    Ok(v) => v,
                    Err(_) => return,
                };
                let method = request.get("method").and_then(Value::as_str).unwrap_or("");
                if method == "inference.complete" {
                    flag.store(true, Ordering::SeqCst);
                }
                let id = request.get("id").cloned().unwrap_or(json!(null));
                let response = json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "text": "from-mock-neuralspring",
                        "model": "mock-model",
                        "provider": "neuralspring-wire-test"
                    },
                    "id": id
                });
                let mut out = serde_json::to_string(&response).expect("response json");
                out.push('\n');
                let stream = reader.get_mut();
                let _ = stream.write_all(out.as_bytes()).await;
                let _ = stream.flush().await;
            });
        }
    })
}

fn make_server_with_empty_router(
    sock_placeholder: &str,
) -> (Arc<JsonRpcServer>, tempfile::TempDir, PathBuf) {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let path = tmp.path().join("squirrel-jsonrpc.sock");
    // `JsonRpcServer::socket_path` is unused for our test listener; keep a unique placeholder.
    let router = Arc::new(AiRouter::default());
    let server = Arc::new(JsonRpcServer::with_ai_router(
        sock_placeholder.to_string(),
        router,
    ));
    (server, tmp, path)
}

#[tokio::test]
async fn wire_register_provider_success_and_discoverable_via_inference_models() {
    let (server, _tmpdir, sock_path) =
        make_server_with_empty_router("/tmp/inference-reg-ok-placeholder.sock");
    let bg = spawn_line_framed_jsonrpc_server(&sock_path, Arc::clone(&server));
    tokio::task::yield_now().await;

    let req = json!({
        "jsonrpc": "2.0",
        "method": "inference.register_provider",
        "params": {
            "provider_id": "neuralspring-test",
            "socket": "/tmp/test-neuralspring.sock",
            "capabilities": ["inference.complete", "inference.embed"]
        },
        "id": 1
    });

    let resp = jsonrpc_line_roundtrip(&sock_path, &req).await;
    assert!(
        resp.get("result").is_some(),
        "expected result, got {resp:?}"
    );
    assert!(resp.get("error").is_none());
    let result = resp.get("result").expect("result");
    assert_eq!(
        result.get("registered").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        result.get("provider_id").and_then(Value::as_str),
        Some("neuralspring-test")
    );

    let models_req = json!({
        "jsonrpc": "2.0",
        "method": "inference.models",
        "id": 2
    });
    let models_resp = jsonrpc_line_roundtrip(&sock_path, &models_req).await;
    let models = models_resp
        .pointer("/result/models")
        .and_then(Value::as_array)
        .expect("models array");
    assert!(
        models
            .iter()
            .filter_map(|m| m.get("id").and_then(Value::as_str))
            .any(|id| id == "neuralspring-test"),
        "registered provider should appear in inference.models: {models_resp:?}"
    );

    bg.abort();
}

#[tokio::test]
async fn wire_register_provider_missing_params_invalid_params() {
    let (server, _tmpdir, sock_path) =
        make_server_with_empty_router("/tmp/inference-reg-noparams-placeholder.sock");
    let bg = spawn_line_framed_jsonrpc_server(&sock_path, Arc::clone(&server));
    tokio::task::yield_now().await;

    let req = json!({
        "jsonrpc": "2.0",
        "method": "inference.register_provider",
        "id": 1
    });

    let resp = jsonrpc_line_roundtrip(&sock_path, &req).await;
    assert!(resp.get("result").is_none());
    let err = resp.get("error").expect("error object");
    assert_eq!(
        err.get("code").and_then(Value::as_i64),
        Some(i64::from(error_codes::INVALID_PARAMS))
    );

    bg.abort();
}

#[tokio::test]
async fn wire_register_provider_missing_provider_id_invalid_params() {
    let (server, _tmpdir, sock_path) =
        make_server_with_empty_router("/tmp/inference-reg-noid-placeholder.sock");
    let bg = spawn_line_framed_jsonrpc_server(&sock_path, Arc::clone(&server));
    tokio::task::yield_now().await;

    let req = json!({
        "jsonrpc": "2.0",
        "method": "inference.register_provider",
        "params": { "socket": "/tmp/x.sock" },
        "id": 1
    });

    let resp = jsonrpc_line_roundtrip(&sock_path, &req).await;
    assert!(resp.get("result").is_none());
    let err = resp.get("error").expect("error object");
    assert_eq!(
        err.get("code").and_then(Value::as_i64),
        Some(i64::from(error_codes::INVALID_PARAMS))
    );
    let msg = err.get("message").and_then(Value::as_str).unwrap_or("");
    assert!(msg.contains("provider_id"), "unexpected message: {msg}");

    bg.abort();
}

#[tokio::test]
async fn wire_register_provider_duplicate_listing_is_deduped() {
    let (server, _tmpdir, sock_path) =
        make_server_with_empty_router("/tmp/inference-reg-dup-placeholder.sock");
    let bg = spawn_line_framed_jsonrpc_server(&sock_path, Arc::clone(&server));
    tokio::task::yield_now().await;

    let params = json!({
        "provider_id": "neuralspring-dup",
        "socket": "/tmp/dup-neuralspring.sock",
        "capabilities": []
    });

    for id in [1_i64, 2] {
        let req = json!({
            "jsonrpc": "2.0",
            "method": "inference.register_provider",
            "params": params,
            "id": id
        });
        let resp = jsonrpc_line_roundtrip(&sock_path, &req).await;
        assert!(
            resp.get("result").is_some(),
            "registration {id} should succeed: {resp:?}"
        );
    }

    let models_req = json!({
        "jsonrpc": "2.0",
        "method": "inference.models",
        "id": 3
    });
    let models_resp = jsonrpc_line_roundtrip(&sock_path, &models_req).await;
    let models = models_resp
        .pointer("/result/models")
        .and_then(Value::as_array)
        .expect("models array");
    let count = models
        .iter()
        .filter(|m| m.get("id").and_then(Value::as_str) == Some("neuralspring-dup"))
        .count();
    assert_eq!(
        count, 1,
        "list_providers dedupes by provider_id even if register_remote_provider pushed twice: {models_resp:?}"
    );

    bg.abort();
}

#[tokio::test]
async fn wire_inference_complete_routes_to_registered_socket() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let spring_sock = tmp.path().join("neuralspring-mock.sock");
    let saw = Arc::new(AtomicBool::new(false));
    let mock_bg = spawn_mock_neural_spring(&spring_sock, Arc::clone(&saw));

    let (server, _tmpdir, squirrel_sock) =
        make_server_with_empty_router("/tmp/inference-route-placeholder.sock");
    let bg = spawn_line_framed_jsonrpc_server(&squirrel_sock, Arc::clone(&server));
    tokio::task::yield_now().await;

    let reg = json!({
        "jsonrpc": "2.0",
        "method": "inference.register_provider",
        "params": {
            "provider_id": "neuralspring-wire-test",
            "socket": spring_sock.to_str().expect("utf8 path"),
            "capabilities": ["inference.complete"]
        },
        "id": 1
    });
    let reg_resp = jsonrpc_line_roundtrip(&squirrel_sock, &reg).await;
    assert!(
        reg_resp.get("result").is_some(),
        "register_provider: {reg_resp:?}"
    );

    let complete = json!({
        "jsonrpc": "2.0",
        "method": "inference.complete",
        "params": { "prompt": "hello route" },
        "id": 2
    });
    let complete_resp = jsonrpc_line_roundtrip(&squirrel_sock, &complete).await;
    assert!(
        complete_resp.get("error").is_none(),
        "inference.complete failed: {complete_resp:?}"
    );
    let text = complete_resp
        .pointer("/result/text")
        .and_then(Value::as_str)
        .expect("result.text");
    assert_eq!(text, "from-mock-neuralspring");
    assert!(
        saw.load(Ordering::SeqCst),
        "mock neuralSpring socket should have received inference.complete"
    );

    bg.abort();
    mock_bg.abort();
}
