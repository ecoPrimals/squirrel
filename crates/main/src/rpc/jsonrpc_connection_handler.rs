// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Per-connection handling for the JSON-RPC server.
//!
//! Extracted from [`super::jsonrpc_server`] for module clarity.
//! Contains riboCipher/BTSP routing, protocol negotiation entry,
//! the JSON-RPC request/response loop, and the BTSP Phase 3 encrypted
//! frame loop.

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, info, warn};
use universal_patterns::transport::UniversalTransport;

use super::jsonrpc_server::JsonRpcServer;

impl JsonRpcServer {
    /// Handle a single UDS connection with riboCipher + BTSP auto-detect.
    ///
    /// **Eukaryotic genetics model (Wave 114):** Reads the first byte and
    /// classifies by stream:
    ///
    /// - `0xEC` / `0xED` — **MitoBeacon** (shared access). Reads the second
    ///   byte (protocol type) and routes: `0x01` → NDJSON JSON-RPC, `0x02`/`0x03`
    ///   → BTSP handshake.
    /// - `0xEE` — **Nuclear Lineage** (per-user). Requires BearDog key material;
    ///   not yet implemented — closes gracefully.
    /// - Anything else — passed to `maybe_handshake` for BTSP/JSON auto-detect.
    pub(super) async fn handle_uds_connection(
        server: Arc<Self>,
        mut transport: UniversalTransport,
    ) -> Result<()> {
        use super::ribocipher_prefix::{
            BTSP_BINARY, BTSP_JSON_LINE, CLEAR_SIGNAL, MITO_SIGNAL, NDJSON_JSONRPC, NUCLEAR_SIGNAL,
        };
        use tokio::io::AsyncReadExt;

        let mut first = [0u8; 1];
        let n = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            transport.read(&mut first),
        )
        .await
        .unwrap_or(Ok(0))
        .unwrap_or(0);

        if n == 0 {
            debug!("Client disconnected before sending data (UDS)");
            return Ok(());
        }

        match first[0] {
            signal @ (CLEAR_SIGNAL | MITO_SIGNAL) => {
                let tier = if signal == CLEAR_SIGNAL {
                    "clear"
                } else {
                    "mito"
                };
                let mut proto = [0u8; 1];
                transport
                    .read_exact(&mut proto)
                    .await
                    .context("failed to read riboCipher protocol type byte")?;
                let protocol_type = proto[0];

                match protocol_type {
                    NDJSON_JSONRPC => {
                        debug!(tier, "riboCipher MitoBeacon: NDJSON JSON-RPC (0x01)");
                        server.handle_universal_connection(transport).await
                    }
                    BTSP_BINARY | BTSP_JSON_LINE => {
                        debug!(
                            tier,
                            protocol_type, "riboCipher MitoBeacon: BTSP — proceeding to handshake"
                        );
                        Self::run_btsp_then_jsonrpc(server, transport).await
                    }
                    other => {
                        warn!(
                            tier,
                            protocol_type = other,
                            "riboCipher MitoBeacon: unsupported protocol type — closing"
                        );
                        Ok(())
                    }
                }
            }
            NUCLEAR_SIGNAL => {
                warn!("riboCipher Nuclear Lineage (0xEE) not yet implemented — closing");
                Ok(())
            }
            first_byte => Self::run_btsp_with_first_byte(server, transport, first_byte).await,
        }
    }

    /// Run BTSP handshake then fall through to JSON-RPC.
    async fn run_btsp_then_jsonrpc(
        server: Arc<Self>,
        mut transport: UniversalTransport,
    ) -> Result<()> {
        let first_line = match super::btsp_handshake::maybe_handshake(&mut transport).await {
            Ok(session) => {
                if let Some(ref s) = session {
                    debug!(session_id = %s.session_id, "BTSP authenticated");
                }
                None
            }
            Err(super::btsp_handshake::BtspError::PlainJsonRpc { first_line }) => {
                debug!("PG-14: plain JSON-RPC on BTSP socket — proceeding unauthenticated");
                Some(first_line)
            }
            Err(e) => {
                warn!("BTSP handshake failed, refusing connection: {e}");
                return Ok(());
            }
        };

        if let Some(line) = first_line {
            let reader = tokio::io::BufReader::new(transport);
            server.handle_jsonrpc_with_first_line(reader, line).await
        } else {
            server.handle_universal_connection(transport).await
        }
    }

    /// Handle a connection where we already consumed the first byte (no riboCipher
    /// signal detected). Re-inject the byte into the BTSP/JSON-RPC classification.
    async fn run_btsp_with_first_byte(
        server: Arc<Self>,
        transport: UniversalTransport,
        first_byte: u8,
    ) -> Result<()> {
        use super::btsp_handshake;

        if btsp_handshake::is_btsp_required() {
            match first_byte {
                b'{' => {
                    let mut reader = BufReader::new(transport);
                    let mut rest = String::new();
                    reader.read_line(&mut rest).await.ok();
                    let first_line = format!("{}{rest}", '{');
                    let trimmed = first_line.trim();

                    if trimmed.contains("\"protocol\"") && trimmed.contains("\"btsp\"") {
                        warn!("raw BTSP JSON-line after no riboCipher signal — PG-14 fallback");
                    } else {
                        debug!("PG-14: plain JSON-RPC on BTSP socket (first byte re-injected)");
                    }
                    server
                        .handle_jsonrpc_with_first_line(reader, first_line)
                        .await
                }
                0x00 => {
                    let mut transport = transport;
                    super::btsp_handshake::btsp_handshake_server(&mut transport, Some(first_byte))
                        .await
                        .map(|_session| ())?;
                    server.handle_universal_connection(transport).await
                }
                other => {
                    debug!(
                        first_byte = format_args!("0x{other:02x}"),
                        "non-BTSP binary preamble on BTSP-guarded socket — closing"
                    );
                    Ok(())
                }
            }
        } else {
            let first_char = first_byte as char;
            if first_char == '{' || first_char == '[' || first_char.is_ascii_whitespace() {
                let mut reader = BufReader::new(transport);
                let mut line = String::new();
                line.push(first_char);
                reader
                    .read_line(&mut line)
                    .await
                    .context("failed to read rest of first line after re-injected byte")?;
                let trimmed = line.trim();
                if trimmed.starts_with("PROTOCOLS:") {
                    #[cfg(feature = "tarpc-rpc")]
                    {
                        return server.handle_protocol_negotiation(reader, &line).await;
                    }
                    #[cfg(not(feature = "tarpc-rpc"))]
                    {
                        return server.handle_jsonrpc_loop(reader).await;
                    }
                }
                server.handle_jsonrpc_with_first_line(reader, line).await
            } else {
                debug!(
                    first_byte = format_args!("0x{first_byte:02x}"),
                    "non-JSON first byte in dev mode — closing"
                );
                Ok(())
            }
        }
    }

    /// Handle a client connection via Universal Transport with protocol negotiation.
    pub(crate) async fn handle_universal_connection(
        self: std::sync::Arc<Self>,
        transport: UniversalTransport,
    ) -> Result<()> {
        let mut reader = BufReader::new(transport);
        let mut line = String::new();

        match reader.read_line(&mut line).await {
            Ok(0) => {
                debug!("Client disconnected before sending data");
                Ok(())
            }
            Ok(_) => {
                let trimmed = line.trim();

                if trimmed.starts_with("PROTOCOLS:") {
                    #[cfg(feature = "tarpc-rpc")]
                    {
                        self.handle_protocol_negotiation(reader, &line).await
                    }
                    #[cfg(not(feature = "tarpc-rpc"))]
                    {
                        info!(
                            "Protocol negotiation requested, tarpc not enabled, selecting JSON-RPC"
                        );
                        let response = "PROTOCOL: jsonrpc\n";
                        reader
                            .get_mut()
                            .write_all(response.as_bytes())
                            .await
                            .context("Failed to write protocol response")?;
                        reader
                            .get_mut()
                            .flush()
                            .await
                            .context("Failed to flush protocol response")?;
                        self.handle_jsonrpc_loop(reader).await
                    }
                } else {
                    self.handle_jsonrpc_with_first_line(reader, line).await
                }
            }
            Err(e) => {
                warn!("Error reading from connection: {}", e);
                Err(e).context("Failed to read first line from connection")
            }
        }
    }

    /// Handle JSON-RPC loop after processing first line.
    pub(super) async fn handle_jsonrpc_with_first_line(
        &self,
        mut reader: BufReader<UniversalTransport>,
        first_line: String,
    ) -> Result<()> {
        let switch_session_id = self.detect_btsp_switch(&first_line).await;

        if let Some(response_json) = self.handle_request_or_batch(&first_line).await {
            let mut out = response_json;
            out.push('\n');
            reader
                .get_mut()
                .write_all(out.as_bytes())
                .await
                .context("Failed to write JSON-RPC response")?;
            reader
                .get_mut()
                .flush()
                .await
                .context("Failed to flush JSON-RPC response")?;

            if let Some(keys) = switch_session_id
                .as_ref()
                .and_then(|sid| self.btsp_session_keys.get(sid))
            {
                let session_id = switch_session_id.as_deref().unwrap_or("?");
                info!(
                    session_id,
                    "BTSP Phase 3: switching to encrypted frame loop (first line)"
                );
                let keys = Arc::clone(keys.value());
                let transport = reader.into_inner();
                return self.encrypted_frame_loop(transport, keys).await;
            }
        }

        self.handle_jsonrpc_loop(reader).await
    }

    /// Handle protocol negotiation for multi-protocol support.
    #[cfg(feature = "tarpc-rpc")]
    pub(super) async fn handle_protocol_negotiation(
        self: std::sync::Arc<Self>,
        mut reader: BufReader<UniversalTransport>,
        first_line: &str,
    ) -> Result<()> {
        use super::protocol::IpcProtocol;
        use super::protocol_negotiation::{ProtocolRequest, ProtocolResponse, select_protocol};
        use super::tarpc_server::TarpcRpcServer;

        info!("🔄 Protocol negotiation requested");

        let request = match ProtocolRequest::from_wire(first_line) {
            Ok(req) => req,
            Err(e) => {
                warn!("Invalid protocol request: {}", e);
                let response = "PROTOCOL: jsonrpc\n";
                reader
                    .get_mut()
                    .write_all(response.as_bytes())
                    .await
                    .context("Failed to write protocol fallback response")?;
                reader
                    .get_mut()
                    .flush()
                    .await
                    .context("Failed to flush protocol fallback response")?;
                return self.handle_jsonrpc_loop(reader).await;
            }
        };

        let server_supported = vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        let selected = select_protocol(&request.supported, &server_supported);

        let response = ProtocolResponse::new(selected);
        let response_line = response.to_wire();
        reader
            .get_mut()
            .write_all(response_line.as_bytes())
            .await
            .context("Failed to write protocol negotiation response")?;
        reader
            .get_mut()
            .flush()
            .await
            .context("Failed to flush protocol negotiation response")?;

        info!("✅ Protocol negotiated: {}", selected);

        match selected {
            IpcProtocol::Tarpc => {
                let transport = reader.into_inner();
                let tarpc_server = TarpcRpcServer::from_jsonrpc(self);
                tarpc_server.handle_connection(transport).await
            }
            IpcProtocol::JsonRpc => self.handle_jsonrpc_loop(reader).await,
        }
    }

    /// Standard JSON-RPC request/response loop (supports batch per Section 6).
    ///
    /// After a successful `btsp.negotiate` with `chacha20-poly1305`, transitions
    /// the connection to the encrypted frame loop automatically.
    pub(super) async fn handle_jsonrpc_loop(
        &self,
        mut reader: BufReader<UniversalTransport>,
    ) -> Result<()> {
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("Client disconnected");
                    break;
                }
                Ok(_) => {
                    let switch_session_id = self.detect_btsp_switch(&line).await;

                    if let Some(response_json) = self.handle_request_or_batch(&line).await {
                        let mut out = response_json;
                        out.push('\n');
                        reader
                            .get_mut()
                            .write_all(out.as_bytes())
                            .await
                            .context("Failed to write JSON-RPC response in loop")?;
                        reader
                            .get_mut()
                            .flush()
                            .await
                            .context("Failed to flush JSON-RPC response in loop")?;

                        if let Some(keys) = switch_session_id
                            .as_ref()
                            .and_then(|sid| self.btsp_session_keys.get(sid))
                        {
                            let session_id = switch_session_id.as_deref().unwrap_or("?");
                            info!(
                                session_id,
                                "BTSP Phase 3: switching to encrypted frame loop"
                            );
                            let keys = Arc::clone(keys.value());
                            let transport = reader.into_inner();
                            return self.encrypted_frame_loop(transport, keys).await;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error reading from connection: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Detect whether a line contains a `btsp.negotiate` request that will
    /// trigger a transport switch. Returns the session_id if so.
    async fn detect_btsp_switch(&self, line: &str) -> Option<String> {
        let parsed: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
        let method = parsed.get("method")?.as_str()?;
        if method != "btsp.negotiate" {
            return None;
        }
        let params = parsed.get("params")?;
        let cipher = params.get("preferred_cipher")?.as_str()?;
        if cipher != "chacha20-poly1305" {
            return None;
        }
        let session_id = params.get("session_id")?.as_str()?;
        if self
            .btsp_sessions
            .get(session_id)
            .is_some_and(|s| s.handshake_key.is_some())
        {
            Some(session_id.to_string())
        } else {
            None
        }
    }

    /// Process encrypted frames using BTSP Phase 3 session keys.
    async fn encrypted_frame_loop(
        &self,
        mut transport: UniversalTransport,
        keys: Arc<super::btsp_encrypted_framing::SessionKeys>,
    ) -> Result<()> {
        use super::btsp_encrypted_framing::{encrypt_frame, read_encrypted_frame};

        loop {
            match read_encrypted_frame(&mut transport, &keys.c2s_key).await {
                Ok(plaintext) => {
                    let request_str =
                        String::from_utf8(plaintext).context("invalid UTF-8 in encrypted frame")?;

                    if let Some(response_json) = self.handle_request_or_batch(&request_str).await {
                        let response_bytes = response_json.as_bytes();
                        let frame = encrypt_frame(&keys.s2c_key, response_bytes)
                            .context("failed to encrypt response frame")?;
                        transport
                            .write_all(&frame)
                            .await
                            .context("failed to write encrypted response frame")?;
                        transport
                            .flush()
                            .await
                            .context("failed to flush encrypted response frame")?;
                    }
                }
                Err(e) => {
                    debug!("Encrypted frame loop ended: {e}");
                    break;
                }
            }
        }

        Ok(())
    }
}
