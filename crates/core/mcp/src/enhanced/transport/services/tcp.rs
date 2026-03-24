// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plain TCP accept loop for internal/security channels.

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

/// TCP Service Implementation
#[derive(Debug)]
pub struct TcpService {
    pub(super) config: Arc<TransportConfig>,
    pub(super) connection_manager: Arc<ConnectionManager>,
    pub(super) message_router: Arc<MessageRouter>,
    pub(super) server_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl TcpService {
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
impl TransportService for TcpService {
    async fn start(&self, addr: SocketAddr) -> Result<()> {
        info!("Starting TCP service on: {}", addr);

        let listener = TcpListener::bind(addr).await?;
        let connection_manager = self.connection_manager.clone();

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, remote_addr)) => {
                        let connection_id = Uuid::new_v4().to_string();
                        let connection = ConnectionInfo {
                            id: connection_id.clone(),
                            transport_type: TransportType::TCP,
                            remote_addr,
                            state: ConnectionState::Connecting,
                            created_at: chrono::Utc::now(),
                            last_activity: chrono::Utc::now(),
                            metadata: HashMap::new(),
                        };

                        if let Err(e) = connection_manager.add_connection(connection).await {
                            error!("Failed to add TCP connection: {}", e);
                        }

                        // Handle TCP connection
                        let connection_manager = connection_manager.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_tcp_connection(
                                stream,
                                connection_id,
                                connection_manager,
                            )
                            .await
                            {
                                error!("TCP connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept TCP connection: {}", e);
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
        info!("Stopping TCP service");

        let mut server_handle = self.server_handle.lock().await;
        if let Some(handle) = server_handle.take() {
            handle.abort();
        }

        Ok(())
    }

    async fn send_message(&self, connection_id: &str, message: TransportMessage) -> Result<()> {
        tracing::debug!("Sending TCP message to connection: {}", connection_id);
        let _ = message;
        Ok(())
    }

    fn service_type(&self) -> TransportType {
        TransportType::TCP
    }

    async fn get_metrics(&self) -> TransportServiceMetrics {
        TransportServiceMetrics::default()
    }
}

impl TcpService {
    async fn handle_tcp_connection(
        _stream: TcpStream,
        connection_id: String,
        connection_manager: Arc<ConnectionManager>,
    ) -> Result<()> {
        // TCP connection handling would be implemented here
        info!("Handling TCP connection: {}", connection_id);

        // Update connection state
        connection_manager
            .update_connection_state(&connection_id, ConnectionState::Connected)
            .await?;

        // Handle messages...

        Ok(())
    }
}
