// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Client Connection Management
//!
//! Types for managing client connections.

use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::session::Session;
use crate::transport::Transport;

/// Client connection information
#[derive(Clone)]
pub struct ClientConnection {
    /// Client ID
    pub client_id: String,
    
    /// Client address
    pub address: SocketAddr,
    
    /// Client session
    pub session: Arc<Session>,
    
    /// Client transport
    pub transport: Arc<dyn Transport>,
    
    /// Connection time
    pub connected_at: chrono::DateTime<chrono::Utc>,
    
    /// Client metadata
    pub metadata: HashMap<String, Value>,
}

/// Manual Debug implementation (Transport doesn't implement Debug)
impl std::fmt::Debug for ClientConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientConnection")
            .field("client_id", &self.client_id)
            .field("address", &self.address)
            .field("session", &self.session)
            .field("connected_at", &self.connected_at)
            .field("metadata", &self.metadata)
            .field("transport", &"<Transport>")
            .finish()
    }
}

