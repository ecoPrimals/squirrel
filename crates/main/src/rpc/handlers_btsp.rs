// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! BTSP Phase 3 JSON-RPC handler — `btsp.negotiate`.
//!
//! After a successful Phase 2 handshake (authenticated NULL cipher), the client
//! sends `btsp.negotiate` to upgrade to an encrypted channel. This handler
//! validates the session and returns the negotiated cipher.
//!
//! Currently returns `{"cipher":"null"}` (encrypted framing not yet active).
//! When ChaCha20-Poly1305 framing is wired at the transport layer, the handler
//! will return `{"cipher":"chacha20-poly1305","server_nonce":"<hex>"}` and
//! derive session keys via HKDF-SHA256.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use serde_json::Value;
use tracing::{debug, info, warn};

impl JsonRpcServer {
    /// Handle `btsp.negotiate` — Phase 3 cipher negotiation.
    ///
    /// # Wire format (request)
    ///
    /// ```json
    /// {
    ///   "session_id": "<from Phase 2 handshake>",
    ///   "preferred_cipher": "chacha20-poly1305",
    ///   "bond_type": "Covalent"
    /// }
    /// ```
    ///
    /// # Wire format (response — null cipher fallback)
    ///
    /// ```json
    /// { "cipher": "null" }
    /// ```
    ///
    /// # Wire format (response — encrypted, future)
    ///
    /// ```json
    /// {
    ///   "cipher": "chacha20-poly1305",
    ///   "server_nonce": "<hex-encoded 12 bytes>"
    /// }
    /// ```
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

        let preferred_cipher = params
            .get("preferred_cipher")
            .and_then(Value::as_str)
            .unwrap_or("null");

        let bond_type = params
            .get("bond_type")
            .and_then(Value::as_str)
            .unwrap_or("Covalent");

        if !self.btsp_sessions.contains_key(session_id) {
            warn!(
                session_id = session_id,
                "btsp.negotiate: unknown session — no Phase 2 handshake on record"
            );
            return Err(JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: format!("unknown session_id: {session_id}"),
                data: None,
            });
        }

        debug!(
            session_id = session_id,
            preferred_cipher = preferred_cipher,
            bond_type = bond_type,
            "btsp.negotiate request"
        );

        // Phase 3 encrypted framing (ChaCha20-Poly1305) is not yet active at
        // the transport layer. Return NULL cipher — primalSpring handles this
        // gracefully and the connection continues on authenticated plaintext.
        //
        // When encrypted framing is wired:
        // 1. Generate 12-byte server_nonce
        // 2. Derive keys via HKDF-SHA256(handshake_key, client_nonce || server_nonce)
        // 3. Return {"cipher":"chacha20-poly1305","server_nonce":"<hex>"}
        // 4. Switch transport to encrypted framing
        let cipher = "null";

        info!(
            session_id = session_id,
            preferred_cipher = preferred_cipher,
            negotiated_cipher = cipher,
            "BTSP Phase 3: cipher negotiated"
        );

        Ok(serde_json::json!({
            "cipher": cipher,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::JsonRpcServer;
    use serde_json::json;

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
    async fn negotiate_returns_null_cipher_for_valid_session() {
        let server = JsonRpcServer::new("/tmp/btsp-negotiate-test3.sock".to_string());

        server.btsp_sessions.insert(
            "test-session-42".to_string(),
            crate::rpc::btsp_handshake::BtspSession {
                session_id: "test-session-42".to_string(),
                cipher: "null".to_string(),
            },
        );

        let result = server
            .handle_btsp_negotiate(Some(json!({
                "session_id": "test-session-42",
                "preferred_cipher": "chacha20-poly1305",
                "bond_type": "Covalent"
            })))
            .await
            .expect("negotiate should succeed for valid session");

        assert_eq!(
            result.get("cipher").and_then(|v| v.as_str()),
            Some("null"),
            "should return null cipher (encrypted framing not yet active)"
        );
    }
}
