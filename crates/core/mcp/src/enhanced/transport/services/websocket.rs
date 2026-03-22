// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! WebSocket-oriented TCP accept loop (upgrade path not yet implemented).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{error, info};
use uuid::Uuid;

use crate::error::Result;

use super::super::connection::ConnectionManager;
use super::super::routing::MessageRouter;
use super::super::types::{
    ConnectionInfo, ConnectionState, TransportConfig, TransportMessage, TransportService,
    TransportServiceMetrics, TransportType,
};

/// WebSocket Service Implementation
#[derive(Debug)]
pub struct WebSocketService {
    pub(super) config: Arc<TransportConfig>,
    pub(super) connection_manager: Arc<ConnectionManager>,
    pub(super) message_router: Arc<MessageRouter>,
    pub(super) server_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl WebSocketService {
    pub async fn new(
        config: Arc<TransportConfig>,
        connection_manager: Arc<ConnectionManager>,
        message_router: Arc<MessageRouter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            connection_manager,
            message_router,
            server_handle: Arc::new(Mutex::new(None)),
        })
    }
}

#[async_trait::async_trait]
impl TransportService for WebSocketService {
    async fn start(&self, addr: SocketAddr) -> Result<()> {
        info!("Starting WebSocket service on: {}", addr);

        let listener = TcpListener::bind(addr).await?;
        let connection_manager = self.connection_manager.clone();
        let message_router = self.message_router.clone();

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, remote_addr)) => {
                        let connection_id = Uuid::new_v4().to_string();
                        let connection = ConnectionInfo {
                            id: connection_id.clone(),
                            transport_type: TransportType::WebSocket,
                            remote_addr,
                            state: ConnectionState::Connecting,
                            created_at: chrono::Utc::now(),
                            last_activity: chrono::Utc::now(),
                            metadata: HashMap::new(),
                        };

                        if let Err(e) = connection_manager.add_connection(connection).await {
                            error!("Failed to add WebSocket connection: {}", e);
                        }

                        // Handle WebSocket connection
                        let connection_manager = connection_manager.clone();
                        let message_router = message_router.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_websocket_connection(
                                stream,
                                connection_id,
                                connection_manager,
                                message_router,
                            )
                            .await
                            {
                                error!("WebSocket connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept WebSocket connection: {}", e);
                    }
                }
            }
        });

        {
            let mut server_handle = self.server_handle.lock().await;
            *server_handle = Some(handle);
        }

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping WebSocket service");

        let mut server_handle = self.server_handle.lock().await;
        if let Some(handle) = server_handle.take() {
            handle.abort();
        }

        Ok(())
    }

    async fn send_message(&self, connection_id: &str, message: TransportMessage) -> Result<()> {
        tracing::debug!("Sending WebSocket message to connection: {}", connection_id);
        // Implementation would send message via WebSocket
        let _ = message;
        Ok(())
    }

    fn service_type(&self) -> TransportType {
        TransportType::WebSocket
    }

    async fn get_metrics(&self) -> TransportServiceMetrics {
        TransportServiceMetrics::default()
    }
}

impl WebSocketService {
    async fn handle_websocket_connection(
        _stream: TcpStream,
        connection_id: String,
        connection_manager: Arc<ConnectionManager>,
        _message_router: Arc<MessageRouter>,
    ) -> Result<()> {
        // WebSocket connection handling would be implemented here
        info!("Handling WebSocket connection: {}", connection_id);

        // Update connection state
        connection_manager
            .update_connection_state(&connection_id, ConnectionState::Connected)
            .await?;

        // Handle messages...

        Ok(())
    }
}
