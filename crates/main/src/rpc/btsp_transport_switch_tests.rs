// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Integration tests verifying the BTSP Phase 3 post-negotiate transport switch.
//!
//! These tests exercise the full live-connection flow:
//! 1. NDJSON JSON-RPC exchange (pre-negotiate)
//! 2. `btsp.negotiate` → `chacha20-poly1305` response
//! 3. Transport switches to encrypted frames
//! 4. Subsequent JSON-RPC over encrypted frames works end-to-end
//!
//! Uses real Unix socket pairs to verify the transport layer.

use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::JsonRpcServer;
use super::btsp_encrypted_framing::{SessionKeys, encrypt_frame, read_encrypted_frame};
use super::btsp_handshake::BtspSession;
use universal_patterns::transport::UniversalTransport;

/// Set up a server with a pre-populated BTSP session and handshake key.
fn setup_server(session_id: &str, handshake_key: &[u8; 32]) -> Arc<JsonRpcServer> {
    let server = JsonRpcServer::new("/tmp/btsp-switch-test.sock".to_string());
    let hk_b64 = BASE64.encode(handshake_key);

    server.btsp_sessions.insert(
        session_id.to_string(),
        BtspSession {
            session_id: session_id.to_string(),
            cipher: "null".to_string(),
            handshake_key: Some(hk_b64),
            client_ephemeral_pub: Some("test-ephemeral-pub".to_string()),
        },
    );

    Arc::new(server)
}

/// Send an NDJSON request and read the NDJSON response.
async fn ndjson_roundtrip(stream: &mut tokio::net::UnixStream, request: &Value) -> Value {
    let mut line = serde_json::to_string(request).unwrap();
    line.push('\n');
    stream.write_all(line.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    let mut reader = BufReader::new(&mut *stream);
    let mut resp_line = String::new();
    reader.read_line(&mut resp_line).await.unwrap();
    serde_json::from_str(resp_line.trim()).unwrap()
}

#[tokio::test]
async fn transport_switch_full_roundtrip() {
    let session_id = "live-switch-test-001";
    let handshake_key = [0x77u8; 32];
    let client_nonce = [0xAAu8; 32];

    let server = setup_server(session_id, &handshake_key);

    let (mut client_stream, server_stream) = tokio::net::UnixStream::pair().unwrap();

    let server_clone = Arc::clone(&server);
    let _server_handle = tokio::spawn(async move {
        let transport = UniversalTransport::UnixSocket(server_stream);
        let _ = server_clone.handle_universal_connection(transport).await;
    });

    // ── Phase 1: Normal NDJSON request before negotiate ─────────────────
    let ping_resp = ndjson_roundtrip(
        &mut client_stream,
        &json!({"jsonrpc":"2.0","method":"system.ping","id":0}),
    )
    .await;
    assert!(
        ping_resp.get("result").is_some(),
        "system.ping should succeed before negotiate"
    );

    // ── Phase 2: btsp.negotiate → chacha20-poly1305 ─────────────────────
    let client_nonce_b64 = BASE64.encode(client_nonce);
    let neg_resp = ndjson_roundtrip(
        &mut client_stream,
        &json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": session_id,
                "preferred_cipher": "chacha20-poly1305",
                "client_nonce": client_nonce_b64,
                "bond_type": "Covalent"
            },
            "id": 1
        }),
    )
    .await;

    let result = neg_resp.get("result").expect("negotiate must have result");
    assert_eq!(result["cipher"], "chacha20-poly1305");
    assert_eq!(result["allowed"], true);

    let server_nonce_b64 = result["server_nonce"].as_str().unwrap();
    let server_nonce = BASE64.decode(server_nonce_b64).unwrap();

    // ── Phase 3: Derive client-side keys ────────────────────────────────
    let keys =
        SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce).expect("key derivation");

    // Client writes with c2s_key, reads with s2c_key (opposite of server)
    let write_key = keys.c2s_key;
    let read_key = keys.s2c_key;

    // ── Phase 4: Send encrypted JSON-RPC frame ──────────────────────────
    let health_req = json!({"jsonrpc":"2.0","method":"health.liveness","id":2});
    let health_bytes = serde_json::to_vec(&health_req).unwrap();

    let frame = encrypt_frame(&write_key, &health_bytes).expect("encrypt");
    client_stream.write_all(&frame).await.unwrap();
    client_stream.flush().await.unwrap();

    // ── Phase 5: Read encrypted response ────────────────────────────────
    let decrypted = read_encrypted_frame(&mut client_stream, &read_key)
        .await
        .expect("read encrypted response");

    let health_resp: Value =
        serde_json::from_slice(&decrypted).expect("response must be valid JSON");
    assert!(
        health_resp.get("result").is_some(),
        "health.liveness should succeed over encrypted channel"
    );
    assert_eq!(health_resp["id"], 2);
}

#[tokio::test]
async fn transport_switch_null_cipher_stays_ndjson() {
    let session_id = "null-cipher-test-002";
    let server = JsonRpcServer::new("/tmp/btsp-null-switch-test.sock".to_string());

    // Session without handshake_key → null cipher fallback
    server.btsp_sessions.insert(
        session_id.to_string(),
        BtspSession {
            session_id: session_id.to_string(),
            cipher: "null".to_string(),
            handshake_key: None,
            client_ephemeral_pub: None,
        },
    );

    let server = Arc::new(server);
    let (mut client_stream, server_stream) = tokio::net::UnixStream::pair().unwrap();

    let server_clone = Arc::clone(&server);
    let _server_handle = tokio::spawn(async move {
        let transport = UniversalTransport::UnixSocket(server_stream);
        let _ = server_clone.handle_universal_connection(transport).await;
    });

    // Negotiate → should get null cipher (no switch)
    let neg_resp = ndjson_roundtrip(
        &mut client_stream,
        &json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": session_id,
                "preferred_cipher": "chacha20-poly1305",
                "client_nonce": BASE64.encode([0xBBu8; 32]),
                "bond_type": "Covalent"
            },
            "id": 1
        }),
    )
    .await;
    assert_eq!(
        neg_resp["result"]["cipher"], "null",
        "should fall back to null"
    );

    // Connection should still be in NDJSON mode → send another plain request
    let ping_resp = ndjson_roundtrip(
        &mut client_stream,
        &json!({"jsonrpc":"2.0","method":"system.ping","id":2}),
    )
    .await;
    assert!(
        ping_resp.get("result").is_some(),
        "system.ping should succeed — still in NDJSON mode after null negotiate"
    );
}

#[tokio::test]
async fn transport_switch_multiple_encrypted_frames() {
    let session_id = "multi-frame-test-003";
    let handshake_key = [0x33u8; 32];
    let client_nonce = [0x44u8; 32];

    let server = setup_server(session_id, &handshake_key);
    let (mut client_stream, server_stream) = tokio::net::UnixStream::pair().unwrap();

    let server_clone = Arc::clone(&server);
    let _server_handle = tokio::spawn(async move {
        let transport = UniversalTransport::UnixSocket(server_stream);
        let _ = server_clone.handle_universal_connection(transport).await;
    });

    // Negotiate
    let client_nonce_b64 = BASE64.encode(client_nonce);
    let neg_resp = ndjson_roundtrip(
        &mut client_stream,
        &json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": session_id,
                "preferred_cipher": "chacha20-poly1305",
                "client_nonce": client_nonce_b64
            },
            "id": 1
        }),
    )
    .await;

    let server_nonce = BASE64
        .decode(neg_resp["result"]["server_nonce"].as_str().unwrap())
        .unwrap();

    let keys =
        SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce).expect("derive keys");
    let write_key = keys.c2s_key;
    let read_key = keys.s2c_key;

    // Send multiple encrypted requests sequentially
    for i in 0..5 {
        let req = json!({"jsonrpc":"2.0","method":"system.ping","id":i + 10});
        let req_bytes = serde_json::to_vec(&req).unwrap();
        let frame = encrypt_frame(&write_key, &req_bytes).expect("encrypt");
        client_stream.write_all(&frame).await.unwrap();
        client_stream.flush().await.unwrap();

        let decrypted = read_encrypted_frame(&mut client_stream, &read_key)
            .await
            .expect("read encrypted frame");
        let resp: Value = serde_json::from_slice(&decrypted).expect("valid JSON");
        assert_eq!(resp["id"], i + 10, "response id must match request");
        assert!(
            resp.get("result").is_some(),
            "system.ping must succeed over encrypted channel"
        );
    }
}
