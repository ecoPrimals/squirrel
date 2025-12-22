//! Server state management
//!
//! This module defines the internal state tracking for the API server.

use chrono::{DateTime, Utc};

/// Server state tracking
#[derive(Debug, Clone)]
pub struct ServerState {
    /// When the server was started
    pub started_at: DateTime<Utc>,
    /// Request count
    pub request_count: u64,
    /// Active connections
    pub active_connections: u32,
    /// Service mesh registration status
    pub service_mesh_registered: bool,
    /// Last Songbird heartbeat
    pub last_songbird_heartbeat: Option<DateTime<Utc>>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_songbird_heartbeat: None,
        }
    }
}

/// Middleware to increment request count for all requests
pub async fn increment_request_count(state: std::sync::Arc<tokio::sync::RwLock<ServerState>>) {
    let mut state_guard = state.write().await;
    state_guard.request_count += 1;
}

