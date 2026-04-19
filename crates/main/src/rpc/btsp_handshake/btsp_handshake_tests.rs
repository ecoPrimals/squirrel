// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::btsp_handshake_wire::{
    MAX_FRAME_SIZE, read_frame, read_frame_with_first_byte, write_frame,
};
use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};

// ── Frame I/O ────────────────────────────────────────────────────

#[tokio::test]
async fn frame_roundtrip() {
    let (mut client, mut server) = duplex(4096);
    let payload = b"hello btsp";

    write_frame(&mut client, payload).await.expect("write");
    let got = read_frame(&mut server).await.expect("read");
    assert_eq!(got, payload);
}

#[tokio::test]
async fn frame_rejects_oversized() {
    let (mut client, mut server) = duplex(64);
    let fake_len = (MAX_FRAME_SIZE as u32 + 1).to_be_bytes();
    client.write_all(&fake_len).await.expect("write len");

    let err = read_frame(&mut server).await.unwrap_err();
    assert!(matches!(err, BtspError::FrameTooLarge { .. }));
}

#[tokio::test]
async fn frame_empty_payload() {
    let (mut client, mut server) = duplex(64);
    write_frame(&mut client, b"").await.expect("write");
    let got = read_frame(&mut server).await.expect("read");
    assert!(got.is_empty());
}

// ── Message serde ────────────────────────────────────────────────

#[tokio::test]
async fn message_roundtrip_client_hello() {
    let (mut client, mut server) = duplex(4096);
    let hello = ClientHello {
        version: 1,
        client_ephemeral_pub: "dGVzdA==".into(),
    };

    write_message(&mut client, &hello).await.expect("write");
    let got: ClientHello = read_message(&mut server).await.expect("read");
    assert_eq!(got.version, 1);
    assert_eq!(got.client_ephemeral_pub, "dGVzdA==");
}

#[tokio::test]
async fn message_roundtrip_server_hello() {
    let (mut client, mut server) = duplex(4096);
    let hello = ServerHello {
        version: 1,
        server_ephemeral_pub: "c2VydmVy".into(),
        challenge: "Y2hhbGxlbmdl".into(),
    };

    write_message(&mut client, &hello).await.expect("write");
    let got: ServerHello = read_message(&mut server).await.expect("read");
    assert_eq!(got.version, 1);
    assert_eq!(got.server_ephemeral_pub, "c2VydmVy");
    assert_eq!(got.challenge, "Y2hhbGxlbmdl");
}

#[tokio::test]
async fn message_roundtrip_challenge_response() {
    let (mut client, mut server) = duplex(4096);
    let resp = ChallengeResponse {
        response: "aG1hYw==".into(),
        preferred_cipher: "null".into(),
    };

    write_message(&mut client, &resp).await.expect("write");
    let got: ChallengeResponse = read_message(&mut server).await.expect("read");
    assert_eq!(got.response, "aG1hYw==");
    assert_eq!(got.preferred_cipher, "null");
}

#[tokio::test]
async fn message_roundtrip_handshake_complete() {
    let (mut client, mut server) = duplex(4096);
    let complete = HandshakeComplete {
        cipher: "null".into(),
        session_id: "abc123".into(),
    };

    write_message(&mut client, &complete).await.expect("write");
    let got: HandshakeComplete = read_message(&mut server).await.expect("read");
    assert_eq!(got.cipher, "null");
    assert_eq!(got.session_id, "abc123");
}

#[tokio::test]
async fn message_roundtrip_error() {
    let (mut client, mut server) = duplex(4096);
    let err = HandshakeErrorMsg {
        error: "handshake_failed".into(),
        reason: "family_verification".into(),
    };

    write_message(&mut client, &err).await.expect("write");
    let got: HandshakeErrorMsg = read_message(&mut server).await.expect("read");
    assert_eq!(got.error, "handshake_failed");
    assert_eq!(got.reason, "family_verification");
}

#[tokio::test]
async fn malformed_message_returns_protocol_error() {
    let (mut client, mut server) = duplex(4096);
    write_frame(&mut client, b"not json").await.expect("write");

    let err = read_message::<_, ClientHello>(&mut server)
        .await
        .unwrap_err();
    assert!(matches!(err, BtspError::Protocol(_)));
}

// ── is_btsp_required ─────────────────────────────────────────────

#[test]
fn btsp_not_required_without_family_id() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            assert!(!is_btsp_required());
        },
    );
}

#[test]
fn btsp_not_required_with_default_family() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", Some("default")),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            assert!(!is_btsp_required());
        },
    );
}

#[test]
fn btsp_required_with_production_family() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", Some("prod-family")),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            assert!(is_btsp_required());
        },
    );
}

#[test]
fn btsp_not_required_when_insecure() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", Some("prod-family")),
            ("BIOMEOS_INSECURE", Some("1")),
        ],
        || {
            assert!(!is_btsp_required());
        },
    );
}

#[test]
fn btsp_required_with_primal_specific_family() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", Some("squirrel-fam")),
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            assert!(is_btsp_required());
        },
    );
}

// ── is_btsp_required with empty family ─────────────────────────

#[test]
fn btsp_not_required_with_empty_family() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", Some("")),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            assert!(!is_btsp_required());
        },
    );
}

// ── Multi-frame sequence ─────────────────────────────────────────

#[tokio::test]
async fn multiple_frames_in_sequence() {
    let (mut client, mut server) = duplex(8192);

    let hello = ClientHello {
        version: 1,
        client_ephemeral_pub: "a2V5MQ==".into(),
    };
    let resp = ChallengeResponse {
        response: "cmVzcA==".into(),
        preferred_cipher: "null".into(),
    };

    write_message(&mut client, &hello)
        .await
        .expect("write hello");
    write_message(&mut client, &resp).await.expect("write resp");

    let got_hello: ClientHello = read_message(&mut server).await.expect("read hello");
    let got_resp: ChallengeResponse = read_message(&mut server).await.expect("read resp");

    assert_eq!(got_hello.client_ephemeral_pub, "a2V5MQ==");
    assert_eq!(got_resp.preferred_cipher, "null");
}

// ── Verify transport is clean after handshake frames ─────────────

#[tokio::test]
async fn transport_clean_after_frames() {
    let (mut client, mut server) = duplex(8192);

    let hello = ClientHello {
        version: 1,
        client_ephemeral_pub: "dGVzdA==".into(),
    };
    write_message(&mut client, &hello).await.expect("write");

    // Write raw JSON-RPC line after the frame (simulating post-handshake)
    let jsonrpc = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"id\":1}\n";
    client.write_all(jsonrpc).await.expect("write jsonrpc");

    // Read the BTSP frame
    let _got: ClientHello = read_message(&mut server).await.expect("read hello");

    // Read the subsequent JSON-RPC line (proving the transport is clean)
    let mut line_buf = vec![0u8; jsonrpc.len()];
    server
        .read_exact(&mut line_buf)
        .await
        .expect("read jsonrpc");
    assert_eq!(&line_buf, jsonrpc);
}

// ── discover_btsp_provider (env precedence + filesystem; no network) ─

#[test]
fn discover_btsp_prefers_btsp_provider_socket() {
    let primary = tempfile::NamedTempFile::new().expect("temp file");
    let secondary = tempfile::NamedTempFile::new().expect("temp file");
    temp_env::with_vars(
        [
            (
                "BTSP_PROVIDER_SOCKET",
                Some(primary.path().to_str().expect("utf8 path")),
            ),
            (
                "BTSP_CAPABILITY_SOCKET",
                Some(secondary.path().to_str().expect("utf8 path")),
            ),
            ("SECURITY_SOCKET", None::<&str>),
            ("BEARDOG_SOCKET", None::<&str>),
        ],
        || {
            let got = discover_btsp_provider().expect("discover");
            assert_eq!(got, primary.path());
        },
    );
}

#[test]
fn discover_btsp_falls_back_when_provider_path_missing() {
    let cap = tempfile::NamedTempFile::new().expect("temp file");
    temp_env::with_vars(
        [
            (
                "BTSP_PROVIDER_SOCKET",
                Some("/nonexistent/btsp_provider_missing.sock"),
            ),
            (
                "BTSP_CAPABILITY_SOCKET",
                Some(cap.path().to_str().expect("utf8 path")),
            ),
            ("SECURITY_SOCKET", None::<&str>),
            ("BEARDOG_SOCKET", None::<&str>),
        ],
        || {
            let got = discover_btsp_provider().expect("discover");
            assert_eq!(got, cap.path());
        },
    );
}

#[test]
fn discover_btsp_security_socket_after_env_misses() {
    let sec = tempfile::NamedTempFile::new().expect("temp file");
    temp_env::with_vars(
        [
            (
                "BTSP_PROVIDER_SOCKET",
                Some("/nonexistent/btsp_provider_missing.sock"),
            ),
            (
                "BTSP_CAPABILITY_SOCKET",
                Some("/nonexistent/btsp_capability_missing.sock"),
            ),
            (
                "SECURITY_SOCKET",
                Some(sec.path().to_str().expect("utf8 path")),
            ),
            ("BEARDOG_SOCKET", None::<&str>),
        ],
        || {
            let got = discover_btsp_provider().expect("discover");
            assert_eq!(got, sec.path());
        },
    );
}

#[test]
fn discover_btsp_beardog_socket_after_higher_tiers_miss() {
    let bd = tempfile::NamedTempFile::new().expect("temp file");
    temp_env::with_vars(
        [
            (
                "BTSP_PROVIDER_SOCKET",
                Some("/nonexistent/btsp_provider_missing.sock"),
            ),
            (
                "BTSP_CAPABILITY_SOCKET",
                Some("/nonexistent/btsp_capability_missing.sock"),
            ),
            (
                "SECURITY_SOCKET",
                Some("/nonexistent/security_missing.sock"),
            ),
            (
                "BEARDOG_SOCKET",
                Some(bd.path().to_str().expect("utf8 path")),
            ),
        ],
        || {
            let got = discover_btsp_provider().expect("discover");
            assert_eq!(got, bd.path());
        },
    );
}

#[test]
fn maybe_handshake_returns_none_when_btsp_not_required() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            let (_client, mut server) = duplex(4096);
            let out = rt.block_on(maybe_handshake(&mut server));
            assert!(out.expect("handshake").is_none());
        },
    );
}

#[test]
fn btsp_error_display_variants() {
    use std::io;
    let io_err = BtspError::Io(io::Error::other("eof"));
    assert!(io_err.to_string().contains("BTSP I/O"));

    let large = BtspError::FrameTooLarge {
        size: MAX_FRAME_SIZE + 1,
    };
    assert!(large.to_string().to_lowercase().contains("large"));

    assert!(BtspError::Timeout.to_string().contains("timed out"));
    assert!(
        BtspError::HandshakeFailed("bad".into())
            .to_string()
            .contains("handshake")
    );
    assert!(
        BtspError::ProviderUnavailable("x".into())
            .to_string()
            .contains("provider")
    );
    assert!(
        BtspError::Protocol("p".into())
            .to_string()
            .contains("protocol")
    );
}

#[tokio::test]
async fn read_frame_maps_first_read_io_error() {
    let mut mock = tokio_test::io::Builder::new()
        .read_error(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "short",
        ))
        .build();
    let err = read_frame(&mut mock).await.expect_err("expected io err");
    assert!(matches!(err, BtspError::Io(_)));
}

#[tokio::test]
async fn write_frame_maps_second_write_io_error() {
    let payload = br#"{"a":1}"#;
    let mut len = [0u8; 4];
    len.copy_from_slice(&(payload.len() as u32).to_be_bytes());
    let mut mock = tokio_test::io::Builder::new()
        .write(&len)
        .write_error(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "payload write fail",
        ))
        .build();
    let err = write_frame(&mut mock, payload).await.expect_err("write");
    assert!(matches!(err, BtspError::Io(_)));
}

// ── Auto-detect (PG-14 resolution) ────────────────────────────

#[tokio::test]
async fn read_frame_with_first_byte_roundtrip() {
    let (mut client, mut server) = duplex(4096);
    let payload = b"hello btsp";
    write_frame(&mut client, payload).await.expect("write");

    let mut first = [0u8; 1];
    server.read_exact(&mut first).await.expect("peek");

    let got = read_frame_with_first_byte(&mut server, first[0])
        .await
        .expect("read");
    assert_eq!(got, payload);
}

#[tokio::test]
async fn read_frame_with_first_byte_rejects_oversized() {
    let (mut client, mut server) = duplex(64);
    let fake_len = (MAX_FRAME_SIZE as u32 + 1).to_be_bytes();
    client.write_all(&fake_len).await.expect("write len");

    let mut first = [0u8; 1];
    server.read_exact(&mut first).await.expect("peek");

    let err = read_frame_with_first_byte(&mut server, first[0])
        .await
        .unwrap_err();
    assert!(matches!(err, BtspError::FrameTooLarge { .. }));
}

#[test]
fn maybe_handshake_detects_plain_jsonrpc() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", Some("prod-family")),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            rt.block_on(async {
                let (mut client, mut server) = duplex(4096);
                let jsonrpc = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"id\":1}\n";
                client.write_all(jsonrpc).await.expect("write");
                drop(client);

                let result = maybe_handshake(&mut server).await;
                assert!(
                    matches!(result, Err(BtspError::PlainJsonRpc)),
                    "expected PlainJsonRpc, got {result:?}"
                );
            });
        },
    );
}

#[test]
fn maybe_handshake_passes_btsp_framing_through() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", Some("prod-family")),
            ("BIOMEOS_INSECURE", None::<&str>),
            ("BTSP_PROVIDER_SOCKET", None::<&str>),
            ("BTSP_CAPABILITY_SOCKET", None::<&str>),
            ("SECURITY_SOCKET", None::<&str>),
            ("BEARDOG_SOCKET", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            rt.block_on(async {
                let (mut client, mut server) = duplex(4096);
                let hello = ClientHello {
                    version: 1,
                    client_ephemeral_pub: "dGVzdA==".into(),
                };
                write_message(&mut client, &hello).await.expect("write");
                drop(client);

                // BTSP framing detected (first byte != `{`), but no provider
                // socket available → ProviderUnavailable error (not PlainJsonRpc).
                let result = maybe_handshake(&mut server).await;
                assert!(
                    matches!(result, Err(BtspError::ProviderUnavailable(_))),
                    "expected ProviderUnavailable (no provider socket), got {result:?}"
                );
            });
        },
    );
}

#[test]
fn btsp_error_display_plain_jsonrpc() {
    let err = BtspError::PlainJsonRpc;
    assert!(err.to_string().contains("plain JSON-RPC"));
}
