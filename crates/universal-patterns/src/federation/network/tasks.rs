// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Background Tasks
//!
//! Background task management for heartbeat monitoring, message processing,
//! and peer discovery.

use super::core::{FederationNetwork, NetworkConnection};
use super::types::NetworkMessage;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

impl<C: NetworkConnection + 'static> FederationNetwork<C> {
    /// Start heartbeat task for peer monitoring
    pub(super) async fn start_heartbeat_task(&self) {
        let _peers = Arc::clone(&self.peers);
        let connections = Arc::clone(&self.connections);
        let running = Arc::clone(&self.running);
        let node_id = self.node_id;
        let interval = self.config.heartbeat_interval;

        tokio::spawn(async move {
            while *running.read().await {
                let health_check = NetworkMessage::HealthCheck {
                    node_id,
                    timestamp: Utc::now(),
                };

                let conn_map = connections.read().await;
                for (peer_id, connection) in conn_map.iter() {
                    let _ = connection
                        .send_message(*peer_id, health_check.clone())
                        .await;
                }

                sleep(Duration::from_secs(interval)).await;
            }
        });
    }

    /// Start message processing task
    pub(super) async fn start_message_processing_task(&self) {
        let message_queue = Arc::clone(&self.message_queue);
        let message_handlers = Arc::clone(&self.message_handlers);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            while *running.read().await {
                let mut queue = message_queue.write().await;
                let handlers = message_handlers.read().await;

                // Process messages
                while let Some(queued) = queue.pop() {
                    if let Some(handler) = handlers.get("default") {
                        let _ = handler(queued.message);
                    }
                }

                drop(queue);
                drop(handlers);

                // Small delay to prevent busy waiting
                sleep(Duration::from_millis(100)).await;
            }
        });
    }

    /// Start peer discovery task
    pub(super) async fn start_peer_discovery_task(&self) {
        let _peers = Arc::clone(&self.peers);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            while *running.read().await {
                // Periodic peer discovery logic
                sleep(Duration::from_secs(60)).await;
            }
        });
    }
}
