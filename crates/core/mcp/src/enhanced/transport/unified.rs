// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! [`UnifiedTransport`] facade: wires connection management, routing, and transport services.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, instrument};

use crate::error::{Result, types::MCPError};

use super::connection::ConnectionManager;
use super::routing::{LoadBalancer, MessageRouter};
use super::services::{TarpcService, TcpService, WebSocketService};
use super::types::{
    ConnectionInfo, TransportConfig, TransportMessage, TransportMetrics, TransportService,
    TransportType,
};

use universal_constants::deployment::hosts;
use universal_constants::network::get_service_port;

/// Unified Transport - Manages all communication channels
#[derive(Debug)]
pub struct UnifiedTransport {
    /// Configuration
    config: Arc<TransportConfig>,

    /// Transport services
    services: Arc<RwLock<HashMap<TransportType, Arc<dyn TransportService>>>>,

    /// Connection manager
    connection_manager: Arc<ConnectionManager>,

    /// Load balancer
    load_balancer: Arc<LoadBalancer>,

    /// Message router
    message_router: Arc<MessageRouter>,

    /// Event broadcaster
    event_broadcaster: Option<Arc<super::super::events::EventBroadcaster>>,

    /// Metrics
    metrics: Arc<Mutex<TransportMetrics>>,
}

impl UnifiedTransport {
    /// Create a new Unified Transport
    #[instrument]
    pub async fn new(config: TransportConfig) -> Result<Self> {
        info!("Initializing Unified Transport");

        let config = Arc::new(config);

        // Initialize connection manager
        let connection_manager = Arc::new(ConnectionManager::new(config.clone()).await?);

        // Initialize load balancer
        let load_balancer = Arc::new(LoadBalancer::new(config.clone()).await?);

        // Initialize message router
        let message_router = Arc::new(MessageRouter::new(config.clone()).await?);

        let transport = Self {
            config: config.clone(),
            services: Arc::new(RwLock::new(HashMap::new())),
            connection_manager,
            load_balancer,
            message_router,
            event_broadcaster: None,
            metrics: Arc::new(Mutex::new(TransportMetrics::default())),
        };

        // Initialize transport services
        transport.initialize_services().await?;

        info!("Unified Transport initialized successfully");
        Ok(transport)
    }

    /// Set event broadcaster
    pub fn set_event_broadcaster(
        &mut self,
        broadcaster: Arc<super::super::events::EventBroadcaster>,
    ) {
        self.event_broadcaster = Some(broadcaster);
    }

    /// Start all transport services
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting Unified Transport");

        // Start connection manager
        self.connection_manager.start().await?;

        // Start load balancer
        self.load_balancer.start().await?;

        // Start message router
        self.message_router.start().await?;

        // Start all transport services
        let services = self.services.read().await;
        for (transport_type, service) in services.iter() {
            let addr = self.get_service_address(transport_type).await?;
            service.start(addr).await?;
            info!("Started transport service: {:?} on {}", transport_type, addr);
        }

        info!("Unified Transport started successfully");
        Ok(())
    }

    /// Stop all transport services
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Unified Transport");

        // Stop all transport services
        let services = self.services.read().await;
        for (transport_type, service) in services.iter() {
            if let Err(e) = service.stop().await {
                error!("Failed to stop transport service {:?}: {}", transport_type, e);
            }
        }

        // Stop components
        self.message_router.stop().await?;
        self.load_balancer.stop().await?;
        self.connection_manager.stop().await?;

        info!("Unified Transport stopped successfully");
        Ok(())
    }

    /// Send message
    #[instrument(skip(self, message))]
    pub async fn send_message(
        &self,
        connection_id: &str,
        message: TransportMessage,
    ) -> Result<()> {
        debug!("Sending message to connection: {}", connection_id);

        // Get connection info
        let connection = self.connection_manager.get_connection(connection_id).await?;

        // Get appropriate transport service
        let services = self.services.read().await;
        let service = services.get(&connection.transport_type).ok_or_else(|| {
            MCPError::NotFound(format!("Transport service not found: {:?}", connection.transport_type))
        })?;

        // Send message
        service.send_message(connection_id, message).await?;

        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.messages_sent += 1;
        }

        Ok(())
    }

    /// Broadcast message
    #[instrument(skip(self, message))]
    pub async fn broadcast_message(&self, message: TransportMessage) -> Result<()> {
        debug!("Broadcasting message to all connections");

        let connections = self.connection_manager.list_connections().await?;

        for connection in connections {
            if let Err(e) = self.send_message(&connection.id, message.clone()).await {
                tracing::warn!("Failed to send message to connection {}: {}", connection.id, e);
            }
        }

        Ok(())
    }

    /// Get connection info
    #[instrument(skip(self))]
    pub async fn get_connection(&self, connection_id: &str) -> Result<ConnectionInfo> {
        self.connection_manager.get_connection(connection_id).await
    }

    /// List all connections
    #[instrument(skip(self))]
    pub async fn list_connections(&self) -> Result<Vec<ConnectionInfo>> {
        self.connection_manager.list_connections().await
    }

    /// Get transport metrics
    pub async fn get_metrics(&self) -> TransportMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }

    async fn initialize_services(&self) -> Result<()> {
        let mut services = self.services.write().await;

        // Initialize WebSocket service
        if self.config.supported_transports.contains(&TransportType::WebSocket) {
            let websocket_service = Arc::new(
                WebSocketService::new(
                    self.config.clone(),
                    self.connection_manager.clone(),
                    self.message_router.clone(),
                )
                .await?,
            );
            services.insert(TransportType::WebSocket, websocket_service);
        }

        // Initialize tarpc service
        if self.config.supported_transports.contains(&TransportType::Tarpc) {
            let tarpc_service = Arc::new(
                TarpcService::new(
                    self.config.clone(),
                    self.connection_manager.clone(),
                    self.message_router.clone(),
                )
                .await?,
            );
            services.insert(TransportType::Tarpc, tarpc_service);
        }

        // Initialize TCP service
        if self.config.supported_transports.contains(&TransportType::TCP) {
            let tcp_service = Arc::new(
                TcpService::new(
                    self.config.clone(),
                    self.connection_manager.clone(),
                    self.message_router.clone(),
                )
                .await?,
            );
            services.insert(TransportType::TCP, tcp_service);
        }

        info!("Initialized {} transport services", services.len());
        Ok(())
    }

    async fn get_service_address(&self, transport_type: &TransportType) -> Result<SocketAddr> {
        let bind = hosts::all_interfaces();
        match transport_type {
            TransportType::WebSocket => {
                let port = get_service_port("http");
                Ok(format!("{bind}:{port}").parse()?)
            }
            TransportType::Tarpc => {
                let port = get_service_port("admin");
                Ok(format!("{bind}:{port}").parse()?)
            }
            TransportType::TCP => {
                let port = get_service_port("security");
                Ok(format!("{bind}:{port}").parse()?)
            }
            _ => Err(MCPError::NotSupported(format!(
                "Transport type not supported: {:?}",
                transport_type
            ))),
        }
    }
}
