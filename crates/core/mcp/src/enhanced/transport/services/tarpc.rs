// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! tarpc TCP accept loop. Full MCP tarpc service wiring is tracked separately.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::Result;

use super::super::connection::ConnectionManager;
use super::super::routing::MessageRouter;
use super::super::types::{
    ConnectionInfo, ConnectionState, TransportConfig, TransportMessage, TransportService,
    TransportServiceMetrics, TransportType,
};

/// tarpc Service Implementation
#[derive(Debug)]
pub struct TarpcService {
    pub(super) config: Arc<TransportConfig>,
    pub(super) connection_manager: Arc<ConnectionManager>,
    pub(super) _message_router: Arc<MessageRouter>,
    pub(super) server_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl TarpcService {
    pub async fn new(
        config: Arc<TransportConfig>,
        connection_manager: Arc<ConnectionManager>,
        message_router: Arc<MessageRouter>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            connection_manager,
            _message_router: message_router,
            server_handle: Arc::new(Mutex::new(None)),
        })
    }
}

#[async_trait::async_trait]
impl TransportService for TarpcService {
    async fn start(&self, addr: SocketAddr) -> Result<()> {
        info!("Starting tarpc service on: {}", addr);

        let listener = TcpListener::bind(addr).await?;
        let connection_manager = self.connection_manager.clone();

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, remote_addr)) => {
                        let connection_id = Uuid::new_v4().to_string();
                        let connection = ConnectionInfo {
                            id: connection_id.clone(),
                            transport_type: TransportType::Tarpc,
                            remote_addr,
                            state: ConnectionState::Connecting,
                            created_at: chrono::Utc::now(),
                            last_activity: chrono::Utc::now(),
                            metadata: HashMap::new(),
                        };

                        if let Err(e) = connection_manager.add_connection(connection).await {
                            error!("Failed to add tarpc connection: {}", e);
                        }

                        let connection_manager = connection_manager.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_tarpc_connection(
                                stream,
                                connection_id,
                                connection_manager,
                            )
                            .await
                            {
                                error!("tarpc connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept tarpc connection: {}", e);
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
        info!("Stopping tarpc service");

        let mut server_handle = self.server_handle.lock().await;
        if let Some(handle) = server_handle.take() {
            handle.abort();
        }

        Ok(())
    }

    async fn send_message(&self, connection_id: &str, message: TransportMessage) -> Result<()> {
        tracing::debug!("Sending tarpc message to connection: {}", connection_id);
        let _ = message;
        Ok(())
    }

    fn service_type(&self) -> TransportType {
        TransportType::Tarpc
    }

    async fn get_metrics(&self) -> TransportServiceMetrics {
        TransportServiceMetrics::default()
    }
}

impl TarpcService {
    /// Handle incoming tarpc TCP connection until the peer closes.
    async fn handle_tarpc_connection(
        mut stream: TcpStream,
        connection_id: String,
        connection_manager: Arc<ConnectionManager>,
    ) -> Result<()> {
        info!("Handling tarpc connection: {}", connection_id);

        if let Err(e) = connection_manager
            .update_connection_state(&connection_id, ConnectionState::Connected)
            .await
        {
            error!("Failed to update connection state: {}", e);
            return Err(e.into());
        }

        if let Err(e) = tokio::io::copy(&mut stream, &mut tokio::io::sink()).await {
            warn!("tarpc connection {} I/O ended: {}", connection_id, e);
        }

        let _ = stream.shutdown().await;

        let _ = connection_manager
            .update_connection_state(&connection_id, ConnectionState::Disconnected)
            .await;

        Ok(())
    }
}
