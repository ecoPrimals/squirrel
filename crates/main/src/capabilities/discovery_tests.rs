// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

use std::collections::HashMap;
use std::path::PathBuf;

use super::*;

#[test]
fn test_capability_provider_serialization() {
    let provider = CapabilityProvider {
        id: "test-provider".to_string(),
        capabilities: vec!["crypto.signing".to_string()],
        socket: PathBuf::from("/tmp/test.sock"),
        metadata: HashMap::new(),
        discovered_via: "test".to_string(),
    };

    let json = serde_json::to_string(&provider).expect("should succeed");
    let deserialized: CapabilityProvider = serde_json::from_str(&json).expect("should succeed");

    assert_eq!(provider.id, deserialized.id);
    assert_eq!(provider.capabilities, deserialized.capabilities);
}

#[test]
fn test_env_var_formatting() {
    let capability = "crypto.signing";
    let env_var = format!(
        "{}_PROVIDER_SOCKET",
        capability.to_uppercase().replace('.', "_")
    );
    assert_eq!(env_var, "CRYPTO_SIGNING_PROVIDER_SOCKET");
}

#[tokio::test]
async fn test_socket_directories() {
    // Isolate from concurrent tests that may set SOCKET_SCAN_DIR
    temp_env::with_var("SOCKET_SCAN_DIR", None::<&str>, || {
        let dirs = get_socket_directories();
        assert!(!dirs.is_empty(), "fallback directories must always exist");
        assert!(
            dirs.contains(&PathBuf::from("/tmp")),
            "fallback list must include /tmp; got {dirs:?}"
        );
    });
}

#[test]
fn discovery_error_display() {
    let e = DiscoveryError::CapabilityNotFound("cap.x".to_string());
    assert!(format!("{e}").contains("cap.x"));
    let e2 = DiscoveryError::ProbeFailed("read".to_string());
    assert!(format!("{e2}").contains("read"));
}

#[test]
fn discover_capability_returns_env_provider_without_probe() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("fake.sock");
    std::fs::File::create(&path).expect("touch");
    temp_env::with_var(
        "CRYPTO_SIGNING_PROVIDER_SOCKET",
        Some(path.to_str().expect("utf8")),
        || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("rt")
                .block_on(async {
                    let p = discover_capability("crypto.signing")
                        .await
                        .expect("discovered");
                    assert_eq!(p.socket, path);
                    assert!(p.discovered_via.starts_with("env:"));
                });
        },
    );
}

#[test]
fn get_socket_directories_respects_socket_scan_dir_override() {
    let dir = tempfile::tempdir().expect("tempdir");
    temp_env::with_var(
        "SOCKET_SCAN_DIR",
        Some(dir.path().to_str().expect("utf8")),
        || {
            let dirs = get_socket_directories();
            assert_eq!(
                dirs.first().map(std::path::PathBuf::as_path),
                Some(dir.path())
            );
        },
    );
}

#[tokio::test]
async fn probe_socket_success_parses_capabilities() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("cap.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

    let cap_name = "probe.test.cap";
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let mut stream = stream;
        let mut line = String::new();
        let mut reader = BufReader::new(&mut stream);
        reader.read_line(&mut line).await.expect("read");
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "1",
            "result": {
                "capabilities": [cap_name],
                "metadata": { "k": "v" }
            }
        });
        let mut out = serde_json::to_string(&resp).expect("should succeed");
        out.push('\n');
        stream.write_all(out.as_bytes()).await.expect("write");
        stream.flush().await.expect("flush");
    });

    let p = probe_socket(&sock_path).await.expect("probe");
    assert!(p.capabilities.contains(&cap_name.to_string()));
    assert_eq!(p.discovered_via, "probe");
    assert_eq!(p.metadata.get("k").map(String::as_str), Some("v"));
}

#[tokio::test]
async fn probe_socket_jsonrpc_error_returns_probe_failed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("err.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let mut stream = stream;
        let mut line = String::new();
        let mut reader = BufReader::new(&mut stream);
        reader.read_line(&mut line).await.expect("read");
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "code": -32601, "message": "nope" }
        });
        let mut out = serde_json::to_string(&resp).expect("should succeed");
        out.push('\n');
        stream.write_all(out.as_bytes()).await.expect("write");
        stream.flush().await.expect("flush");
    });

    let err = probe_socket(&sock_path).await.unwrap_err();
    match err {
        DiscoveryError::ProbeFailed(m) => assert!(m.contains("Method not supported")),
        _ => unreachable!("expected ProbeFailed, got {err:?}"),
    }
}

#[test]
fn discover_capability_not_found_without_env_or_registry() {
    let dir = tempfile::tempdir().expect("tempdir");
    temp_env::with_vars(
        [
            ("SOCKET_SCAN_DIR", Some(dir.path().to_str().expect("utf8"))),
            ("NEURAL_API_SOCKET", None::<&str>),
            ("CAPABILITY_REGISTRY_SOCKET", None::<&str>),
        ],
        || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("rt")
                .block_on(async {
                    let err =
                        discover_capability("zzzz.nonexistent.capability.discovery.test.99999")
                            .await
                            .expect_err("expected not found");
                    match err {
                        DiscoveryError::CapabilityNotFound(c) => {
                            assert!(c.contains("zzzz.nonexistent"));
                        }
                        _ => unreachable!("unexpected {err:?}"),
                    }
                });
        },
    );
}

#[tokio::test]
async fn discover_all_capabilities_returns_ok_map() {
    let map = discover_all_capabilities().await.expect("ok");
    let _ = map.len();
}

#[tokio::test]
async fn probe_socket_missing_result_returns_probe_failed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("nor.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let mut stream = stream;
        let mut line = String::new();
        let mut reader = BufReader::new(&mut stream);
        reader.read_line(&mut line).await.expect("read");
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "not_result": true
        });
        let mut out = serde_json::to_string(&resp).expect("json");
        out.push('\n');
        stream.write_all(out.as_bytes()).await.expect("write");
        stream.flush().await.expect("flush");
    });

    let err = probe_socket(&sock_path).await.unwrap_err();
    match err {
        DiscoveryError::ProbeFailed(m) => assert!(m.contains("No result")),
        _ => panic!("unexpected {err:?}"),
    }
}

#[test]
fn discover_capability_neural_api_happy_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let neural_sock = dir.path().join("neural.sock");
    let provider_sock = dir.path().join("backend.sock");
    std::fs::File::create(&provider_sock).expect("touch provider");

    let rt = tokio::runtime::Runtime::new().expect("rt");
    let _enter = rt.enter();
    let listener = tokio::net::UnixListener::bind(&neural_sock).expect("bind neural");
    let prov = provider_sock.clone();
    let server = rt.spawn(async move {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let Ok(Ok((stream, _))) =
            tokio::time::timeout(std::time::Duration::from_secs(2), listener.accept()).await
        else {
            return;
        };
        let mut stream = stream;
        let mut line = String::new();
        let mut reader = BufReader::new(&mut stream);
        reader.read_line(&mut line).await.expect("read");
        let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse req");
        let id = req
            .get("id")
            .cloned()
            .unwrap_or_else(|| serde_json::json!(0));
        let unix = format!("unix://{}", prov.display());
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "capability": "neural.discovery.test.cap",
                "primary_socket": unix
            }
        });
        let mut out = serde_json::to_string(&resp).expect("json");
        out.push('\n');
        stream.write_all(out.as_bytes()).await.expect("write");
        stream.flush().await.expect("flush");
    });

    temp_env::with_vars(
        [
            (
                "NEURAL_API_SOCKET",
                Some(neural_sock.to_str().expect("utf8")),
            ),
            ("CAPABILITY_REGISTRY_SOCKET", None::<&str>),
            ("SOCKET_SCAN_DIR", Some(dir.path().to_str().expect("utf8"))),
        ],
        || {
            rt.block_on(async {
                let p = discover_capability("neural.discovery.test.cap")
                    .await
                    .expect("discovered via neural API");
                assert_eq!(p.socket, provider_sock);
                assert_eq!(p.discovered_via, "neural_api");
            });
        },
    );
    server.abort();
}

#[test]
fn discover_capability_legacy_registry_socket() {
    let dir = tempfile::tempdir().expect("tempdir");
    let reg_sock = dir.path().join("legacy_reg.sock");
    let provider_sock = dir.path().join("legacy_prov.sock");
    std::fs::File::create(&provider_sock).expect("touch");

    let rt = tokio::runtime::Runtime::new().expect("rt");
    let _enter = rt.enter();
    let listener = tokio::net::UnixListener::bind(&reg_sock).expect("bind");
    let prov = provider_sock.clone();
    let server = rt.spawn(async move {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let Ok(Ok((stream, _))) =
            tokio::time::timeout(std::time::Duration::from_secs(2), listener.accept()).await
        else {
            return;
        };
        let mut stream = stream;
        let mut line = String::new();
        let mut reader = BufReader::new(&mut stream);
        reader.read_line(&mut line).await.expect("read");
        let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse");
        let id = req
            .get("id")
            .cloned()
            .unwrap_or_else(|| serde_json::json!(0));
        let unix = format!("unix://{}", prov.display());
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "capability": "legacy.registry.cap",
                "primary_endpoint": unix
            }
        });
        let mut out = serde_json::to_string(&resp).expect("json");
        out.push('\n');
        stream.write_all(out.as_bytes()).await.expect("write");
        stream.flush().await.expect("flush");
    });

    temp_env::with_vars(
        [
            ("NEURAL_API_SOCKET", None::<&str>),
            (
                "CAPABILITY_REGISTRY_SOCKET",
                Some(reg_sock.to_str().expect("utf8")),
            ),
            ("SOCKET_SCAN_DIR", Some(dir.path().to_str().expect("utf8"))),
        ],
        || {
            rt.block_on(async {
                let p = discover_capability("legacy.registry.cap")
                    .await
                    .expect("legacy registry");
                assert_eq!(p.socket, provider_sock);
                assert_eq!(p.discovered_via, "registry");
            });
        },
    );
    server.abort();
}
