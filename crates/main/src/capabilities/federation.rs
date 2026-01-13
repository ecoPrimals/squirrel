//! Federation capability (inter-primal communication)

use crate::error::PrimalError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Message for primal-to-primal federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    /// Source primal name
    pub from: String,

    /// Destination primal name (or "broadcast")
    pub to: String,

    /// Message type
    pub message_type: String,

    /// Message payload
    pub payload: serde_json::Value,

    /// Optional correlation ID for request/response
    pub correlation_id: Option<String>,
}

/// Capability for primal-to-primal federation
///
/// Allows primals to communicate with each other through a service mesh or federation layer.
/// Typically provided by Songbird or a dedicated federation coordinator.
#[async_trait]
pub trait FederationCapability: Send + Sync {
    /// Send a message to another primal
    async fn send(&self, message: FederationMessage) -> Result<(), PrimalError>;

    /// Subscribe to messages for this primal
    async fn subscribe(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<FederationMessage>, PrimalError>;

    /// Get list of connected primals
    async fn list_primals(&self) -> Result<Vec<String>, PrimalError>;
}
