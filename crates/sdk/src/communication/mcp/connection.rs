// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! WebSocket connection management for MCP client
//!
//! This module handles the low-level WebSocket connection operations including
//! connection establishment, reconnection logic, and connection cleanup.

use crate::config::McpClientConfig;
use std::time::Duration;
use tracing::{debug, info};

#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
#[cfg(target_arch = "wasm32")]
use web_sys::WebSocket;

/// Connection manager for WebSocket connections
///
/// Handles the platform-specific WebSocket connection management,
/// including connection establishment, message sending, and cleanup.
#[derive(Debug)]
pub struct ConnectionManager {
    /// Client configuration
    config: McpClientConfig,
    /// WebSocket connection (platform-specific)
    #[cfg(not(target_arch = "wasm32"))]
    websocket: Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>,
    #[cfg(target_arch = "wasm32")]
    websocket: Option<WebSocket>,
}

impl ConnectionManager {
    /// Create a new connection manager
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::connection::ConnectionManager;
    /// use squirrel_sdk::config::McpClientConfig;
    ///
    /// let config = McpClientConfig::default();
    /// let manager = ConnectionManager::new(config);
    /// ```
    pub fn new(config: McpClientConfig) -> Self {
        Self {
            config,
            websocket: None,
        }
    }

    /// Establish a WebSocket connection
    ///
    /// Creates a new WebSocket connection to the server specified in the configuration.
    /// This method handles platform-specific connection establishment.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration containing server URL and connection settings
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if connection is established successfully, or an error if it fails.
    ///
    /// # Errors
    ///
    /// This method may fail if:
    /// - The server URL is invalid or unreachable
    /// - The WebSocket handshake fails
    /// - Network connectivity issues occur
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::connection::ConnectionManager;
    /// use squirrel_sdk::config::McpClientConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = McpClientConfig::default();
    /// let mut manager = ConnectionManager::new(config.clone());
    /// manager.establish_connection(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn establish_connection(
        &mut self,
        config: &McpClientConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Establishing WebSocket connection to: {}",
            config.server_url
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            let (ws_stream, _) = connect_async(&config.server_url).await?;
            self.websocket = Some(ws_stream);
            info!("WebSocket connection established (native)");
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::prelude::*;

            let ws = WebSocket::new(&config.server_url)?;

            // Set up event handlers
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
            info!("WebSocket connection established (WASM)");
        }

        Ok(())
    }

    /// Send a message through the WebSocket connection
    ///
    /// Sends a message through the established WebSocket connection.
    /// This method handles platform-specific message sending.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send as a string
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the message is sent successfully, or an error if it fails.
    ///
    /// # Errors
    ///
    /// This method may fail if:
    /// - No WebSocket connection is established
    /// - The message exceeds the maximum size limit
    /// - Network issues prevent message sending
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::connection::ConnectionManager;
    /// use squirrel_sdk::config::McpClientConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = McpClientConfig::default();
    /// let mut manager = ConnectionManager::new(config.clone());
    /// manager.establish_connection(&config).await?;
    /// manager.send_message("{\"type\": \"ping\"}").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message(
        &mut self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if message.len() > self.config.max_message_size {
            return Err(format!(
                "Message size {} exceeds maximum {}",
                message.len(),
                self.config.max_message_size
            )
            .into());
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ws) = &mut self.websocket {
            use futures_util::sink::SinkExt;
            ws.send(Message::Text(message.to_string())).await?;
            debug!("Message sent (native): {}", message);
        } else {
            return Err("No WebSocket connection available".into());
        }

        #[cfg(target_arch = "wasm32")]
        if let Some(ws) = &self.websocket {
            ws.send_with_str(message)?;
            debug!("Message sent (WASM): {}", message);
        } else {
            return Err("No WebSocket connection available".into());
        }

        Ok(())
    }

    /// Close the WebSocket connection
    ///
    /// Gracefully closes the WebSocket connection and cleans up resources.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the connection is closed successfully, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::connection::ConnectionManager;
    /// use squirrel_sdk::config::McpClientConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = McpClientConfig::default();
    /// let mut manager = ConnectionManager::new(config.clone());
    /// manager.establish_connection(&config).await?;
    /// manager.close().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(mut ws) = self.websocket.take() {
            ws.close(None).await?;
            debug!("WebSocket connection closed (native)");
        }

        #[cfg(target_arch = "wasm32")]
        if let Some(ws) = self.websocket.take() {
            ws.close()?;
            debug!("WebSocket connection closed (WASM)");
        }

        Ok(())
    }

    /// Check if the connection is established
    ///
    /// # Returns
    ///
    /// `true` if a WebSocket connection is established, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::connection::ConnectionManager;
    /// use squirrel_sdk::config::McpClientConfig;
    ///
    /// let config = McpClientConfig::default();
    /// let manager = ConnectionManager::new(config);
    /// assert!(!manager.is_connected());
    /// ```
    pub fn is_connected(&self) -> bool {
        self.websocket.is_some()
    }

    /// Attempt to reconnect to the server
    ///
    /// Attempts to re-establish a WebSocket connection after a connection failure.
    /// This method includes exponential backoff and retry logic.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration
    /// * `attempt` - Current reconnection attempt number
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if reconnection is successful, or an error if it fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use squirrel_sdk::communication::mcp::connection::ConnectionManager;
    /// use squirrel_sdk::config::McpClientConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = McpClientConfig::default();
    /// let mut manager = ConnectionManager::new(config.clone());
    /// manager.reconnect(&config, 1).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reconnect(
        &mut self,
        config: &McpClientConfig,
        attempt: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if attempt >= config.max_reconnect_attempts {
            return Err("Max reconnection attempts reached".into());
        }

        info!(
            "Attempting to reconnect (attempt {}/{})",
            attempt + 1,
            config.max_reconnect_attempts
        );

        // Exponential backoff
        let delay = Duration::from_millis(config.reconnect_delay_ms * (2_u64.pow(attempt)));
        tokio::time::sleep(delay).await;

        // Close existing connection if any
        if self.websocket.is_some() {
            let _ = self.close().await;
        }

        // Attempt to establish new connection
        self.establish_connection(config).await?;

        info!("Successfully reconnected to MCP server");
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
        let mut config = McpClientConfig::default();
        config.max_reconnect_attempts = 2;
        let mut manager = ConnectionManager::new(config.clone());

        let result = manager.reconnect(&config, 2).await;
        assert!(result.is_err());
    }
}
