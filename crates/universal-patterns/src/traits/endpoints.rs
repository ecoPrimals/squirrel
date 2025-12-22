//! Primal API endpoints.

use serde::{Deserialize, Serialize};

/// Primal API endpoints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Primary API endpoint
    pub primary: String,
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: Option<String>,
    /// Admin endpoint
    pub admin: Option<String>,
    /// WebSocket endpoint
    pub websocket: Option<String>,
    /// Additional custom endpoints
    pub custom: String, // Changed from HashMap to String to fix Hash issues
}

impl Default for PrimalEndpoints {
    fn default() -> Self {
        Self {
            primary: "http://localhost:8080".to_string(),
            health: "http://localhost:8080/health".to_string(),
            metrics: None,
            admin: None,
            websocket: None,
            custom: String::new(),
        }
    }
}
