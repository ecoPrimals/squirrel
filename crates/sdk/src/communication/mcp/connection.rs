// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP client transport: JSON-RPC 2.0 over a Unix domain socket (Tower Atomic / Songbird IPC).
//!
//! Native targets connect via [`tokio::net::UnixStream`]. Browser (WASM) builds may still use the
//! host `WebSocket` API when `MCP_SERVER_URL` is a `ws:` / `wss:` URL.

use crate::config::McpClientConfig;
use crate::infrastructure::error::{PluginError, PluginResult};
use serde_json::json;
use std::time::Duration;
use tracing::{debug, info};

#[cfg(all(not(target_arch = "wasm32"), unix))]
use tokio::io::AsyncWriteExt;
#[cfg(all(not(target_arch = "wasm32"), unix))]
use tokio::net::UnixStream;

#[cfg(target_arch = "wasm32")]
use web_sys::WebSocket;

/// Resolve the socket path from [`McpClientConfig::server_url`].
///
/// Accepts `unix:///absolute/path.sock`, or an absolute filesystem path. Legacy `ws://` / `wss://`
/// URLs are rejected on native targets (use Songbird IPC instead).
pub(crate) fn parse_unix_socket_path(server_url: &str) -> Result<std::path::PathBuf, PluginError> {
    let s = server_url.trim();
    if s.starts_with("ws://") || s.starts_with("wss://") {
        return Err(PluginError::InvalidConfiguration {
            message: "MCP_SERVER_URL must use Unix IPC (unix://…) on native targets; embedded WebSocket was removed. Route MCP via Songbird service mesh (Tower Atomic).".to_string(),
        });
    }
    if let Some(rest) = s.strip_prefix("unix://") {
        return Ok(std::path::PathBuf::from(rest));
    }
    if s.starts_with('/') {
        return Ok(std::path::PathBuf::from(s));
    }
    Err(PluginError::InvalidConfiguration {
        message: format!("MCP_SERVER_URL must be unix://… or an absolute path; got: {s}"),
    })
}

fn jsonrpc_envelope_bytes(message: &str) -> Result<Vec<u8>, PluginError> {
    let params: serde_json::Value =
        serde_json::from_str(message).map_err(|e| PluginError::JsonError {
            message: format!("invalid MCP JSON payload: {e}"),
        })?;
    let id = params.get("id").cloned().unwrap_or(json!(null));
    let envelope = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "mcp.message",
        "params": params,
    });
    let mut buf = serde_json::to_vec(&envelope).map_err(|e| PluginError::SerializationError {
        message: e.to_string(),
    })?;
    buf.push(b'\n');
    Ok(buf)
}

/// Connection manager for MCP IPC (native: Unix socket + JSON-RPC; WASM: browser WebSocket).
#[derive(Debug)]
pub struct ConnectionManager {
    /// Client configuration
    config: McpClientConfig,
    #[cfg(all(not(target_arch = "wasm32"), unix))]
    ipc_stream: Option<UnixStream>,
    #[cfg(all(not(target_arch = "wasm32"), not(unix)))]
    _native_no_unix: (),
    #[cfg(target_arch = "wasm32")]
    websocket: Option<WebSocket>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(config: McpClientConfig) -> Self {
        Self {
            config,
            #[cfg(all(not(target_arch = "wasm32"), unix))]
            ipc_stream: None,
            #[cfg(all(not(target_arch = "wasm32"), not(unix)))]
            _native_no_unix: (),
            #[cfg(target_arch = "wasm32")]
            websocket: None,
        }
    }

    /// Establish a connection to the configured MCP endpoint.
    pub async fn establish_connection(&mut self, config: &McpClientConfig) -> PluginResult<()> {
        debug!("Establishing MCP transport to: {}", config.server_url);

        #[cfg(all(not(target_arch = "wasm32"), unix))]
        {
            let path = parse_unix_socket_path(&config.server_url)?;
            let stream =
                UnixStream::connect(&path)
                    .await
                    .map_err(|e| PluginError::ConnectionError {
                        endpoint: config.server_url.clone(),
                        message: e.to_string(),
                    })?;
            self.ipc_stream = Some(stream);
            info!("MCP IPC connected (unix socket)");
        }

        #[cfg(all(not(target_arch = "wasm32"), not(unix)))]
        {
            let _ = config;
            return Err(PluginError::NotSupported {
                feature: "MCP IPC requires Unix domain sockets on this platform".to_string(),
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::prelude::*;

            let ws =
                WebSocket::new(&config.server_url).map_err(|e| PluginError::ConnectionError {
                    endpoint: config.server_url.clone(),
                    message: format!("{e:?}"),
                })?;

            let onopen_callback = Closure::wrap(Box::new(move || {
                info!("WebSocket connection opened");
            }) as Box<dyn FnMut()>);
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();

            let onerror_callback = Closure::wrap(Box::new(move |e: web_sys::ErrorEvent| {
                debug!("WebSocket error: {:?}", e);
            })
                as Box<dyn FnMut(web_sys::ErrorEvent)>);
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            let onclose_callback = Closure::wrap(Box::new(move |e: web_sys::CloseEvent| {
                debug!("WebSocket closed: code={}, reason={}", e.code(), e.reason());
            })
                as Box<dyn FnMut(web_sys::CloseEvent)>);
            ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();

            self.websocket = Some(ws);
            info!("MCP browser WebSocket handle created (WASM)");
        }

        Ok(())
    }

    /// Send an MCP JSON payload over the transport.
    ///
    /// On native Unix, the payload is framed as a JSON-RPC 2.0 object (one line, newline-terminated)
    /// for compatibility with the Songbird / primal JSON-RPC listeners.
    pub async fn send_message(&mut self, message: &str) -> PluginResult<()> {
        if message.len() > self.config.max_message_size {
            return Err(PluginError::ResourceLimitExceeded {
                resource: "mcp_transport_message".to_string(),
                limit: format!(
                    "max {} bytes (got {})",
                    self.config.max_message_size,
                    message.len()
                ),
            });
        }

        #[cfg(all(not(target_arch = "wasm32"), unix))]
        if let Some(stream) = &mut self.ipc_stream {
            let line = jsonrpc_envelope_bytes(message)?;
            stream
                .write_all(&line)
                .await
                .map_err(|e| PluginError::ConnectionError {
                    endpoint: self.config.server_url.clone(),
                    message: e.to_string(),
                })?;
            debug!("MCP IPC send (native)");
        } else {
            return Err(PluginError::ConnectionError {
                endpoint: self.config.server_url.clone(),
                message: "No MCP IPC connection available".to_string(),
            });
        }

        #[cfg(all(not(target_arch = "wasm32"), not(unix)))]
        {
            let _ = message;
            return Err(PluginError::NotSupported {
                feature: "MCP IPC requires Unix domain sockets on this platform".to_string(),
            });
        }

        #[cfg(target_arch = "wasm32")]
        if let Some(ws) = &self.websocket {
            ws.send_with_str(message)?;
            debug!("MCP message sent (WASM WebSocket)");
        } else {
            return Err(PluginError::ConnectionError {
                endpoint: self.config.server_url.clone(),
                message: "No MCP WebSocket connection available".to_string(),
            });
        }

        Ok(())
    }

    /// Close the connection.
    pub async fn close(&mut self) -> PluginResult<()> {
        #[cfg(all(not(target_arch = "wasm32"), unix))]
        if let Some(mut stream) = self.ipc_stream.take() {
            let _ = stream.shutdown().await;
            debug!("MCP IPC connection closed (native)");
        }

        #[cfg(all(not(target_arch = "wasm32"), not(unix)))]
        {}

        #[cfg(target_arch = "wasm32")]
        if let Some(ws) = self.websocket.take() {
            ws.close()?;
            debug!("MCP WebSocket closed (WASM)");
        }

        Ok(())
    }

    /// Returns `true` if a transport handle is held.
    pub fn is_connected(&self) -> bool {
        self.is_connected_impl()
    }

    #[cfg(all(not(target_arch = "wasm32"), unix))]
    fn is_connected_impl(&self) -> bool {
        self.ipc_stream.is_some()
    }

    #[cfg(all(not(target_arch = "wasm32"), not(unix)))]
    fn is_connected_impl(&self) -> bool {
        false
    }

    #[cfg(target_arch = "wasm32")]
    fn is_connected_impl(&self) -> bool {
        self.websocket.is_some()
    }

    /// Reconnect with exponential backoff (same policy as the previous WebSocket client).
    pub async fn reconnect(&mut self, config: &McpClientConfig, attempt: u32) -> PluginResult<()> {
        if attempt >= config.max_reconnect_attempts {
            return Err(PluginError::TemporaryFailure {
                operation: "mcp_reconnect".to_string(),
                message: "Max reconnection attempts reached".to_string(),
            });
        }

        info!(
            "Attempting MCP reconnect (attempt {}/{})",
            attempt + 1,
            config.max_reconnect_attempts
        );

        let delay = Duration::from_millis(config.reconnect_delay_ms * (2_u64.pow(attempt)));
        tokio::time::sleep(delay).await;

        if self.is_connected() {
            let _ = self.close().await;
        }

        self.establish_connection(config).await?;
        info!("Successfully reconnected to MCP endpoint");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_manager_creation() {
        let config = McpClientConfig::default();
        let manager = ConnectionManager::new(config);
        assert!(!manager.is_connected());
    }

    #[test]
    fn test_connection_manager_is_connected() {
        let config = McpClientConfig::default();
        let manager = ConnectionManager::new(config);
        assert!(!manager.is_connected());
    }

    #[tokio::test]
    async fn test_send_message_without_connection() {
        let config = McpClientConfig::default();
        let mut manager = ConnectionManager::new(config);

        let result = manager.send_message("test message").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_oversized_message() {
        let config = McpClientConfig::default();
        let mut manager = ConnectionManager::new(config.clone());

        let large_message = "x".repeat(config.max_message_size + 1);
        let result = manager.send_message(&large_message).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close_without_connection() {
        let config = McpClientConfig::default();
        let mut manager = ConnectionManager::new(config);

        let result = manager.close().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reconnect_max_attempts() {
        let config = McpClientConfig {
            max_reconnect_attempts: 2,
            ..McpClientConfig::default()
        };
        let mut manager = ConnectionManager::new(config.clone());

        let result = manager.reconnect(&config, 2).await;
        assert!(result.is_err());
    }

    #[cfg(all(not(target_arch = "wasm32"), unix))]
    #[test]
    fn test_parse_unix_socket_path() {
        assert_eq!(
            parse_unix_socket_path("unix:///run/mcp.sock").unwrap(),
            std::path::PathBuf::from("/run/mcp.sock")
        );
        assert_eq!(
            parse_unix_socket_path("/tmp/x.sock").unwrap(),
            std::path::PathBuf::from("/tmp/x.sock")
        );
        assert!(parse_unix_socket_path("ws://x").is_err());
    }
}
