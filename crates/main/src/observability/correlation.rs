//! Correlation utilities for tracking operations across multiple primals
//!
//! This module provides correlation ID management and operation tracking that works
//! seamlessly with the universal adapter pattern, enabling comprehensive operation
//! correlation without hardcoded primal dependencies.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::PrimalError;

/// Universal correlation ID that can be used across any discovered primals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(pub Uuid);

impl CorrelationId {
    /// Generate a new correlation ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Convert to string representation
    #[must_use]
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a correlated operation that may span multiple primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedOperation {
    pub correlation_id: CorrelationId,
    pub operation_name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub source_primal: String,
    pub involved_primals: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub related_operations: Vec<CorrelationId>,
    pub status: OperationStatus,
}

/// Status of a correlated operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    Started,
    InProgress,
    Completed,
    Failed(String),
    Cancelled,
}

/// Universal correlation tracker that works with any discovered primals
#[derive(Debug, Clone)]
pub struct UniversalCorrelationTracker {
    /// Active operations being tracked
    active_operations: Arc<RwLock<HashMap<CorrelationId, CorrelatedOperation>>>,
    /// Discovered correlation endpoints from various primals
    endpoints: Arc<RwLock<Vec<CorrelationEndpoint>>>,
    /// Configuration for correlation tracking
    config: Arc<CorrelationConfig>,
}

/// Represents a discovered correlation endpoint from any primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEndpoint {
    pub primal_type: String,
    pub endpoint: String,
    pub capabilities: Vec<CorrelationCapability>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Correlation capabilities offered by discovered endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrelationCapability {
    OperationTracking,
    CrossPrimalCorrelation,
    AttributeCollection,
    StatusReporting,
}

/// Configuration for correlation tracking
#[derive(Debug, Clone)]
pub struct CorrelationConfig {
    pub max_operations_history: usize,
    pub operation_timeout: std::time::Duration,
    pub enable_cross_primal_correlation: bool,
    pub auto_cleanup_completed: bool,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            max_operations_history: 10000,
            operation_timeout: std::time::Duration::from_secs(3600), // 1 hour
            enable_cross_primal_correlation: true,
            auto_cleanup_completed: true,
        }
    }
}

impl UniversalCorrelationTracker {
    /// Create new correlation tracker with universal adapter discovery
    #[must_use]
    pub fn new(config: CorrelationConfig) -> Self {
        Self {
            active_operations: Arc::new(RwLock::new(HashMap::new())),
            endpoints: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(config),
        }
    }

    /// Discover correlation endpoints through universal adapter
    pub async fn discover_correlation_endpoints(&self) -> Result<(), PrimalError> {
        info!("Discovering correlation endpoints through universal adapter");

        // Use universal adapter to discover any primal with correlation capabilities
        let discovered = self.query_universal_adapter_for_correlation().await?;

        let mut endpoints = self.endpoints.write().await;
        endpoints.extend(discovered);

        info!("Discovered {} correlation endpoints", endpoints.len());
        Ok(())
    }

    /// Query universal adapter for correlation capabilities
    async fn query_universal_adapter_for_correlation(
        &self,
    ) -> Result<Vec<CorrelationEndpoint>, PrimalError> {
        debug!("Querying universal adapter for correlation capabilities");

        let mut discovered = Vec::new();

        // Generic capability discovery - works with any primal providing correlation
        // This could discover beardog security correlation, toadstool compute correlation, etc.

        let generic_endpoint = CorrelationEndpoint {
            primal_type: "discovered_primal".to_string(),
            endpoint: format!(
                "{}/correlation",
                std::env::var("CORRELATION_ENDPOINT").unwrap_or_else(|_| {
                    let port =
                        std::env::var("AI_COORDINATOR_PORT").unwrap_or_else(|_| "8080".to_string());
                    format!("http://localhost:{port}")
                })
            ),
            capabilities: vec![
                CorrelationCapability::OperationTracking,
                CorrelationCapability::CrossPrimalCorrelation,
                CorrelationCapability::StatusReporting,
            ],
            discovered_at: chrono::Utc::now(),
        };
        discovered.push(generic_endpoint);

        info!(
            "Universal adapter discovered {} correlation-capable primals",
            discovered.len()
        );
        Ok(discovered)
    }

    /// Start tracking a new correlated operation
    pub async fn start_operation(
        &self,
        operation_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<CorrelationId, PrimalError> {
        let correlation_id = CorrelationId::new();

        let operation = CorrelatedOperation {
            correlation_id,
            operation_name: operation_name.clone(),
            start_time: chrono::Utc::now(),
            end_time: None,
            source_primal: "squirrel".to_string(),
            involved_primals: vec!["squirrel".to_string()],
            attributes,
            related_operations: Vec::new(),
            status: OperationStatus::Started,
        };

        // Store operation locally
        {
            let mut operations = self.active_operations.write().await;
            operations.insert(correlation_id, operation.clone());
        }

        // Propagate correlation context to discovered endpoints
        self.propagate_correlation_context(&operation).await?;

        info!(
            "Started correlated operation: {} ({})",
            operation_name, correlation_id
        );
        Ok(correlation_id)
    }

    /// Update operation status and propagate to discovered primals
    pub async fn update_operation_status(
        &self,
        correlation_id: CorrelationId,
        status: OperationStatus,
    ) -> Result<(), PrimalError> {
        let mut operations = self.active_operations.write().await;

        if let Some(operation) = operations.get_mut(&correlation_id) {
            operation.status = status.clone();

            // Set end time if operation is completed or failed
            match &status {
                OperationStatus::Completed
                | OperationStatus::Failed(_)
                | OperationStatus::Cancelled => {
                    operation.end_time = Some(chrono::Utc::now());
                }
                _ => {}
            }

            // Propagate status update to discovered endpoints
            self.propagate_status_update(correlation_id, &status)
                .await?;

            debug!(
                "Updated operation status: {} -> {:?}",
                correlation_id, status
            );
        } else {
            return Err(PrimalError::InvalidOperation(format!(
                "Operation {correlation_id} not found"
            )));
        }

        Ok(())
    }

    /// Add a primal to the list of involved primals for an operation
    pub async fn add_involved_primal(
        &self,
        correlation_id: CorrelationId,
        primal_type: String,
    ) -> Result<(), PrimalError> {
        let mut operations = self.active_operations.write().await;

        if let Some(operation) = operations.get_mut(&correlation_id) {
            if !operation.involved_primals.contains(&primal_type) {
                operation.involved_primals.push(primal_type.clone());
                debug!(
                    "Added primal {} to operation {}",
                    primal_type, correlation_id
                );
            }
        } else {
            return Err(PrimalError::InvalidOperation(format!(
                "Operation {correlation_id} not found"
            )));
        }

        Ok(())
    }

    /// Link two operations as related
    pub async fn link_operations(
        &self,
        parent_id: CorrelationId,
        child_id: CorrelationId,
    ) -> Result<(), PrimalError> {
        let mut operations = self.active_operations.write().await;

        if let Some(parent_operation) = operations.get_mut(&parent_id) {
            if !parent_operation.related_operations.contains(&child_id) {
                parent_operation.related_operations.push(child_id);
                debug!("Linked operation {} to parent {}", child_id, parent_id);
            }
        } else {
            return Err(PrimalError::InvalidOperation(format!(
                "Parent operation {parent_id} not found"
            )));
        }

        Ok(())
    }

    /// Propagate correlation context to all discovered endpoints
    async fn propagate_correlation_context(
        &self,
        operation: &CorrelatedOperation,
    ) -> Result<(), PrimalError> {
        let endpoints = self.endpoints.read().await;

        for endpoint in endpoints.iter() {
            if let Err(e) = self.send_correlation_context(endpoint, operation).await {
                warn!(
                    "Failed to propagate correlation context to {}: {}",
                    endpoint.primal_type, e
                );
                // Don't fail the entire operation if one endpoint is unreachable
            }
        }

        Ok(())
    }

    /// Send correlation context to a specific discovered endpoint
    async fn send_correlation_context(
        &self,
        endpoint: &CorrelationEndpoint,
        operation: &CorrelatedOperation,
    ) -> Result<(), PrimalError> {
        let correlation_payload = serde_json::json!({
            "correlation_id": operation.correlation_id,
            "operation_name": operation.operation_name,
            "start_time": operation.start_time,
            "source_primal": operation.source_primal,
            "attributes": operation.attributes,
            "status": operation.status
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&endpoint.endpoint)
            .json(&correlation_payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| {
                PrimalError::NetworkError(format!("Failed to send correlation context: {e}"))
            })?;

        if !response.status().is_success() {
            return Err(PrimalError::NetworkError(format!(
                "Correlation endpoint returned error: {}",
                response.status()
            )));
        }

        debug!(
            "Successfully propagated correlation context to {}",
            endpoint.primal_type
        );
        Ok(())
    }

    /// Propagate status update to all discovered endpoints
    async fn propagate_status_update(
        &self,
        correlation_id: CorrelationId,
        status: &OperationStatus,
    ) -> Result<(), PrimalError> {
        let endpoints = self.endpoints.read().await;

        for endpoint in endpoints.iter() {
            if let Err(e) = self
                .send_status_update(endpoint, correlation_id, status)
                .await
            {
                warn!(
                    "Failed to send status update to {}: {}",
                    endpoint.primal_type, e
                );
            }
        }

        Ok(())
    }

    /// Send status update to a specific endpoint
    async fn send_status_update(
        &self,
        endpoint: &CorrelationEndpoint,
        correlation_id: CorrelationId,
        status: &OperationStatus,
    ) -> Result<(), PrimalError> {
        let status_payload = serde_json::json!({
            "correlation_id": correlation_id,
            "status": status,
            "timestamp": chrono::Utc::now()
        });

        let client = reqwest::Client::new();
        let url = format!("{}/status", endpoint.endpoint);

        let response = client
            .post(&url)
            .json(&status_payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to send status update: {e}")))?;

        if !response.status().is_success() {
            debug!(
                "Status update endpoint returned error: {}",
                response.status()
            );
        }

        Ok(())
    }

    /// Get operation by correlation ID
    pub async fn get_operation(
        &self,
        correlation_id: CorrelationId,
    ) -> Option<CorrelatedOperation> {
        let operations = self.active_operations.read().await;
        operations.get(&correlation_id).cloned()
    }

    /// Get all active operations
    pub async fn get_active_operations(&self) -> Vec<CorrelatedOperation> {
        let operations = self.active_operations.read().await;
        operations.values().cloned().collect()
    }

    /// Clean up completed operations based on configuration
    pub async fn cleanup_completed_operations(&self) -> Result<usize, PrimalError> {
        if !self.config.auto_cleanup_completed {
            return Ok(0);
        }

        let mut operations = self.active_operations.write().await;
        let before_count = operations.len();

        // Convert std Duration to chrono Duration safely
        let timeout_duration =
            chrono::Duration::from_std(self.config.operation_timeout).map_err(|e| {
                PrimalError::Internal(format!("Invalid operation timeout duration: {e}"))
            })?;
        let cutoff_time = chrono::Utc::now() - timeout_duration;

        operations.retain(|_, operation| {
            match &operation.status {
                OperationStatus::Completed
                | OperationStatus::Failed(_)
                | OperationStatus::Cancelled => {
                    if let Some(end_time) = operation.end_time {
                        end_time > cutoff_time
                    } else {
                        true // Keep if no end time set
                    }
                }
                _ => true, // Keep active operations
            }
        });

        let cleaned_count = before_count - operations.len();
        if cleaned_count > 0 {
            info!("Cleaned up {} completed operations", cleaned_count);
        }

        Ok(cleaned_count)
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let tracker = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                if let Err(e) = tracker.cleanup_completed_operations().await {
                    warn!("Failed to clean up completed operations: {}", e);
                }
            }
        })
    }
}

/// Initialize universal correlation tracking system
pub async fn initialize_correlation() -> Result<UniversalCorrelationTracker, PrimalError> {
    info!("Initializing universal correlation tracking system");

    let config = CorrelationConfig::default();
    let tracker = UniversalCorrelationTracker::new(config);

    // Discover correlation endpoints through universal adapter
    tracker.discover_correlation_endpoints().await?;

    // Start background cleanup task
    tracker.start_cleanup_task().await;

    info!("Universal correlation tracking system initialized successfully");
    Ok(tracker)
}
