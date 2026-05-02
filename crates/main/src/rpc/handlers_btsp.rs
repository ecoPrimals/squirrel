// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! BTSP Phase 3 JSON-RPC handler — `btsp.negotiate`.
//!
//! After a successful Phase 2 handshake (authenticated NULL cipher), the client
//! sends `btsp.negotiate` to upgrade to an encrypted channel. This handler
//! validates the session, derives `SessionKeys` via HKDF-SHA256 when the client
//! requests `chacha20-poly1305` and a `handshake_key` is available, and returns
//! the negotiated cipher with a server nonce.
//!
//! ## Wire format
//!
//! ```text
//! Request:
//!   { "session_id": "<from Phase 2>",
//!     "preferred_cipher": "chacha20-poly1305",
//!     "client_nonce": "<base64 32 bytes>",
//!     "bond_type": "Covalent" }
//!
//! Response (keyed):
//!   { "cipher": "chacha20-poly1305",
//!     "server_nonce": "<base64 32 bytes>",
//!     "allowed": true }
//!
//! Response (fallback — no handshake_key):
//!   { "cipher": "null",
//!     "server_nonce": "<base64 32 bytes>",
//!     "allowed": true }
//! ```
//!
//! After a keyed response the caller must switch the connection to the encrypted
//! frame loop (see [`super::btsp_encrypted_framing`]).

use super::btsp_encrypted_framing::{self, SessionKeys, decode_nonce, generate_server_nonce};
use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use serde_json::Value;
use tracing::{debug, info, warn};

impl JsonRpcServer {
    /// Handle `btsp.negotiate` — Phase 3 cipher negotiation with key derivation.
    pub(crate) async fn handle_btsp_negotiate(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "btsp.negotiate requires params".into(),
            data: None,
        })?;

        let session_id = params
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "btsp.negotiate requires 'session_id' (string)".into(),
                data: None,
            })?;

        // Accept both "preferred_cipher" (string) and "ciphers" (array) wire formats.
        let preferred_cipher = params
            .get("preferred_cipher")
            .and_then(Value::as_str)
            .or_else(|| {
                params
                    .get("ciphers")
                    .and_then(Value::as_array)
                    .and_then(|arr| arr.first())
                    .and_then(Value::as_str)
            })
            .unwrap_or("null");

        let bond_type = params
            .get("bond_type")
            .and_then(Value::as_str)
            .unwrap_or("Covalent");

        let client_nonce_b64 = params.get("client_nonce").and_then(Value::as_str);

        let session = self.btsp_sessions.get(session_id).map(|r| r.clone());
        let session = session.ok_or_else(|| {
            warn!(
                session_id = session_id,
                "btsp.negotiate: unknown session — no Phase 2 handshake on record"
            );
            JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: format!("unknown session_id: {session_id}"),
                data: None,
            }
        })?;

        debug!(
            session_id = session_id,
            preferred_cipher = preferred_cipher,
            bond_type = bond_type,
            has_handshake_key = session.handshake_key.is_some(),
            has_client_nonce = client_nonce_b64.is_some(),
            "btsp.negotiate request"
        );

        let server_nonce_b64 = generate_server_nonce();

        // Determine if we can do encrypted framing:
        // - client requests chacha20-poly1305
        // - we have a handshake_key from Phase 2
        // - client provides a nonce
        let can_encrypt = preferred_cipher == "chacha20-poly1305"
            && session.handshake_key.is_some()
            && client_nonce_b64.is_some();

        let (cipher, session_keys) = if can_encrypt {
            let handshake_key_b64 = session.handshake_key.as_deref().unwrap_or_default();
            let client_nonce_raw =
                decode_nonce(client_nonce_b64.unwrap_or_default()).map_err(|e| JsonRpcError {
                    code: error_codes::INVALID_PARAMS,
                    message: format!("invalid client_nonce: {e}"),
                    data: None,
                })?;
            let server_nonce_raw = decode_nonce(&server_nonce_b64).map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("server nonce decode: {e}"),
                data: None,
            })?;
            let handshake_key_raw = btsp_encrypted_framing::decode_nonce(handshake_key_b64)
                .map_err(|e| JsonRpcError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("handshake_key decode: {e}"),
                    data: None,
                })?;

            let keys =
                SessionKeys::derive(&handshake_key_raw, &client_nonce_raw, &server_nonce_raw)
                    .map_err(|e| JsonRpcError {
                        code: error_codes::INTERNAL_ERROR,
                        message: format!("key derivation failed: {e}"),
                        data: None,
                    })?;

            ("chacha20-poly1305", Some(keys))
        } else {
            ("null", None)
        };

        // Store derived keys in the session for the transport layer to pick up.
        if let Some(keys) = session_keys {
            self.btsp_session_keys
                .insert(session_id.to_string(), std::sync::Arc::new(keys));
        }

        info!(
            session_id = session_id,
            preferred_cipher = preferred_cipher,
            negotiated_cipher = cipher,
            "BTSP Phase 3: cipher negotiated"
        );

        Ok(serde_json::json!({
            "cipher": cipher,
            "server_nonce": server_nonce_b64,
            "allowed": true,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::JsonRpcServer;
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use serde_json::json;

    fn make_session(
        session_id: &str,
        handshake_key: Option<&str>,
    ) -> crate::rpc::btsp_handshake::BtspSession {
        crate::rpc::btsp_handshake::BtspSession {
            session_id: session_id.to_string(),
            cipher: "null".to_string(),
            handshake_key: handshake_key.map(String::from),
            client_ephemeral_pub: None,
        }
    }

    #[tokio::test]
    async fn negotiate_requires_session_id() {
        let server = JsonRpcServer::new("/tmp/btsp-negotiate-test.sock".to_string());
        let result = server
            .handle_btsp_negotiate(Some(json!({"preferred_cipher": "chacha20-poly1305"})))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn negotiate_rejects_unknown_session() {
        let server = JsonRpcServer::new("/tmp/btsp-negotiate-test2.sock".to_string());
        let result = server
            .handle_btsp_negotiate(Some(json!({
                "session_id": "nonexistent-session",
                "preferred_cipher": "chacha20-poly1305"
            })))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("unknown session_id"),
            "error should mention unknown session"
        );
    }

    #[tokio::test]
    async fn negotiate_returns_null_without_handshake_key() {
        let server = JsonRpcServer::new("/tmp/btsp-negotiate-test3.sock".to_string());
        server
            .btsp_sessions
            .insert("sess-1".to_string(), make_session("sess-1", None));

        let result = server
            .handle_btsp_negotiate(Some(json!({
                "session_id": "sess-1",
                "preferred_cipher": "chacha20-poly1305",
                "client_nonce": BASE64.encode([0xABu8; 32]),
                "bond_type": "Covalent"
            })))
            .await
            .expect("should succeed");

        assert_eq!(result["cipher"], "null");
        assert!(result["server_nonce"].is_string());
        assert_eq!(result["allowed"], true);
    }

    #[tokio::test]
    async fn negotiate_returns_chacha_with_handshake_key() {
        let server = JsonRpcServer::new("/tmp/btsp-negotiate-test4.sock".to_string());
        let hk = BASE64.encode([0x42u8; 32]);
        server
            .btsp_sessions
            .insert("sess-2".to_string(), make_session("sess-2", Some(&hk)));

        let client_nonce = BASE64.encode([0xCDu8; 32]);
        let result = server
            .handle_btsp_negotiate(Some(json!({
                "session_id": "sess-2",
                "preferred_cipher": "chacha20-poly1305",
                "client_nonce": client_nonce,
                "bond_type": "Covalent"
            })))
            .await
            .expect("should succeed");

        assert_eq!(result["cipher"], "chacha20-poly1305");
        assert!(result["server_nonce"].is_string());
        assert_eq!(result["allowed"], true);

        // Verify session keys were stored
        assert!(server.btsp_session_keys.contains_key("sess-2"));
    }

    #[tokio::test]
    async fn negotiate_falls_back_without_client_nonce() {
        let server = JsonRpcServer::new("/tmp/btsp-negotiate-test5.sock".to_string());
        let hk = BASE64.encode([0x42u8; 32]);
        server
            .btsp_sessions
            .insert("sess-3".to_string(), make_session("sess-3", Some(&hk)));

        let result = server
            .handle_btsp_negotiate(Some(json!({
                "session_id": "sess-3",
                "preferred_cipher": "chacha20-poly1305",
                "bond_type": "Covalent"
            })))
            .await
            .expect("should succeed");

        assert_eq!(result["cipher"], "null", "no client_nonce → null fallback");
    }

    #[tokio::test]
    async fn negotiate_accepts_ciphers_array() {
        let server = JsonRpcServer::new("/tmp/btsp-negotiate-test6.sock".to_string());
        let hk = BASE64.encode([0x42u8; 32]);
        server
            .btsp_sessions
            .insert("sess-4".to_string(), make_session("sess-4", Some(&hk)));

        let client_nonce = BASE64.encode([0xEEu8; 32]);
        let result = server
            .handle_btsp_negotiate(Some(json!({
                "session_id": "sess-4",
                "ciphers": ["chacha20-poly1305", "null"],
                "client_nonce": client_nonce
            })))
            .await
            .expect("should succeed");

        assert_eq!(
            result["cipher"], "chacha20-poly1305",
            "ciphers[] array wire format"
        );
    }

    #[tokio::test]
    async fn negotiate_server_nonce_is_base64_32_bytes() {
        let server = JsonRpcServer::new("/tmp/btsp-negotiate-test7.sock".to_string());
        server
            .btsp_sessions
            .insert("sess-5".to_string(), make_session("sess-5", None));

        let result = server
            .handle_btsp_negotiate(Some(json!({
                "session_id": "sess-5",
                "preferred_cipher": "null"
            })))
            .await
            .expect("should succeed");

        let nonce_b64 = result["server_nonce"].as_str().expect("server_nonce");
        let nonce_raw = BASE64.decode(nonce_b64).expect("valid base64");
        assert_eq!(nonce_raw.len(), 32, "server nonce must be 32 bytes");
    }
}
