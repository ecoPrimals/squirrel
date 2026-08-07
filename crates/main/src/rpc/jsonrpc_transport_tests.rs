// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Transport-level tests: TCP loopback, UDS riboCipher signal acceptance,
//! protocol negotiation, and raw JSON fallback.

use super::*;
use anyhow::Context;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use universal_patterns::transport::UniversalTransport;

/// TCP loopback pair: server uses [`UniversalTransport::Tcp`], client uses raw [`tokio::net::TcpStream`].
async fn tcp_server_transport() -> (
    tokio::task::JoinHandle<anyhow::Result<()>>,
    tokio::net::TcpStream,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = Arc::new(JsonRpcServer::new("/tmp/jsonrpc-tcp.sock".to_string()));
    let jh = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.context("accept")?;
        server
            .handle_universal_connection(UniversalTransport::Tcp(stream))
            .await
    });
    let client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    (jh, client)
}

#[tokio::test]
async fn universal_connection_eof_before_first_line_ok() {
    let (jh, client) = tcp_server_transport().await;
    drop(client);
    let res = jh.await.expect("join");
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn universal_connection_jsonrpc_line_roundtrip() {
    let (jh, mut client) = tcp_server_transport().await;
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#)
        .await
        .expect("write");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    let mut reader = BufReader::new(&mut client);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("readline");
    let v: Value = serde_json::from_str(line.trim()).expect("json");
    assert_eq!(
        v.pointer("/result/pong").and_then(Value::as_bool),
        Some(true)
    );
    let client = reader.into_inner();
    client.shutdown().await.expect("shutdown");
    let _ = jh.await;
}

#[tokio::test]
async fn universal_connection_protocol_negotiation_jsonrpc_then_ping() {
    let (jh, mut client) = tcp_server_transport().await;
    client
        .write_all(b"PROTOCOLS: jsonrpc\n")
        .await
        .expect("protocols");
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    let mut reader = BufReader::new(&mut client);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("proto line");
    assert!(
        line.starts_with("PROTOCOL:"),
        "expected PROTOCOL response, got {line:?}"
    );
    line.clear();
    reader.read_line(&mut line).await.expect("json line");
    let v: Value = serde_json::from_str(line.trim()).expect("json");
    assert_eq!(
        v.pointer("/result/pong").and_then(Value::as_bool),
        Some(true)
    );
    let client = reader.into_inner();
    client.shutdown().await.expect("shutdown");
    let _ = jh.await;
}

#[tokio::test]
async fn universal_connection_invalid_protocol_request_falls_back_to_jsonrpc() {
    let (jh, mut client) = tcp_server_transport().await;
    client
        .write_all(b"PROTOCOLS: not-a-real-protocol-list\n")
        .await
        .expect("bad proto");
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    let mut reader = BufReader::new(&mut client);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("fallback proto");
    assert!(line.starts_with("PROTOCOL:"));
    line.clear();
    reader.read_line(&mut line).await.expect("json");
    let v: Value = serde_json::from_str(line.trim()).expect("json");
    assert_eq!(
        v.pointer("/result/pong").and_then(Value::as_bool),
        Some(true)
    );
    let client = reader.into_inner();
    client.shutdown().await.expect("shutdown");
    let _ = jh.await;
}

// -----------------------------------------------------------------------
// riboCipher signal acceptance (UDS) — Eukaryotic genetics model
// G66: UDS tests gated to unix — TCP tests above are platform-neutral.
// -----------------------------------------------------------------------
#[cfg(unix)]
mod uds_transport_tests {
    use super::*;

/// UDS loopback pair routed through `handle_uds_connection` (riboCipher + BTSP auto-detect).
async fn uds_server_transport() -> (
    tokio::task::JoinHandle<anyhow::Result<()>>,
    tokio::net::UnixStream,
) {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("ribo.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    let server = Arc::new(JsonRpcServer::new(sock.to_str().expect("utf8").to_string()));
    let jh = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.context("accept")?;
        let transport = UniversalTransport::UnixSocket(stream);
        JsonRpcServer::handle_uds_connection(server, transport).await
    });
    let client = tokio::net::UnixStream::connect(&sock)
        .await
        .expect("connect");
    std::mem::forget(dir);
    (jh, client)
}

#[tokio::test]
async fn uds_clear_signal_ndjson_health_roundtrip() {
    let (jh, mut client) = uds_server_transport().await;
    client.write_all(&[0xEC, 0x01]).await.expect("preamble");
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"health","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    client.shutdown().await.expect("shutdown");

    let mut buf = Vec::new();
    client.read_to_end(&mut buf).await.expect("read");
    let v: Value = serde_json::from_slice(&buf).expect("json");
    assert_eq!(
        v.pointer("/result/status").and_then(Value::as_str),
        Some("healthy")
    );
    let _ = jh.await;
}

#[tokio::test]
async fn uds_mito_signal_ndjson_health_roundtrip() {
    let (jh, mut client) = uds_server_transport().await;
    client.write_all(&[0xED, 0x01]).await.expect("preamble");
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"health","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    client.shutdown().await.expect("shutdown");

    let mut buf = Vec::new();
    client.read_to_end(&mut buf).await.expect("read");
    let v: Value = serde_json::from_slice(&buf).expect("json");
    assert_eq!(
        v.pointer("/result/status").and_then(Value::as_str),
        Some("healthy"),
        "mito-beacon (0xED) should be accepted identically to clear (0xEC): {v}"
    );
    let _ = jh.await;
}

#[tokio::test]
async fn uds_nuclear_signal_returns_json_error_for_ndjson() {
    let (jh, mut client) = uds_server_transport().await;
    client
        .write_all(&[0xEE, 0x01])
        .await
        .expect("nuclear+ndjson");
    client.flush().await.expect("flush");

    let mut buf = Vec::new();
    let _ = client.read_to_end(&mut buf).await;
    assert!(
        !buf.is_empty(),
        "nuclear+ndjson should return a JSON-RPC error"
    );
    let v: Value = serde_json::from_slice(&buf).expect("valid json");
    assert_eq!(
        v.pointer("/error/code").and_then(Value::as_i64),
        Some(-32050),
        "error code must be -32050"
    );
    assert_eq!(
        v.pointer("/error/data/resolution").and_then(Value::as_str),
        Some("awaiting_security_keys"),
        "resolution field must guide client"
    );
    assert_eq!(
        v.pointer("/error/data/tier").and_then(Value::as_str),
        Some("nuclear"),
        "tier must identify nuclear lineage"
    );
    let res = jh.await.expect("join");
    assert!(res.is_ok(), "server should not error: {res:?}");
}

#[tokio::test]
async fn uds_nuclear_signal_btsp_protocol_closes_silently() {
    let (jh, mut client) = uds_server_transport().await;
    client.write_all(&[0xEE, 0x02]).await.expect("nuclear+btsp");
    client.flush().await.expect("flush");

    let mut buf = Vec::new();
    let _ = client.read_to_end(&mut buf).await;
    assert!(
        buf.is_empty(),
        "nuclear+btsp should close silently (no NDJSON response)"
    );
    let res = jh.await.expect("join");
    assert!(res.is_ok(), "server should not error: {res:?}");
}

#[tokio::test]
async fn uds_raw_json_still_works_without_prefix() {
    let (jh, mut client) = uds_server_transport().await;
    client
        .write_all(br#"{"jsonrpc":"2.0","method":"system.ping","id":1}"#)
        .await
        .expect("rpc");
    client.write_all(b"\n").await.expect("newline");
    client.flush().await.expect("flush");
    client.shutdown().await.expect("shutdown");

    let mut buf = Vec::new();
    client.read_to_end(&mut buf).await.expect("read");
    let v: Value = serde_json::from_slice(&buf).expect("json");
    assert_eq!(
        v.pointer("/result/pong").and_then(Value::as_bool),
        Some(true),
        "raw JSON without riboCipher prefix should still work: {v}"
    );
    let _ = jh.await;
}
} // mod uds_transport_tests
