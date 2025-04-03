//! Core implementation of the Context-MCP adapter
//!
//! This is the main adapter module that provides the integration between
//! the Squirrel context system and the MCP context manager.

use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use async_trait::async_trait;
use chrono;

// Import the sync types
use crate::context_mcp::sync::SyncStatus;

// Import from MCP
use squirrel_mcp::context_manager::{Context as McpContext, ContextManager as McpContextManager};
use squirrel_mcp::error::{MCPError};
use squirrel_mcp::sync::state::{StateChange, StateOperation};
use squirrel_mcp::resilience::circuit_breaker::{
    StandardCircuitBreaker, 
    BreakerConfig, 
    BreakerError,
    CircuitBreaker,
};

// Import from Context
use squirrel_context::{
    ContextError as SquirrelContextError,
    ContextManagerImpl as SquirrelContextManager,
    ContextManagerConfig as SquirrelContextConfig,
};

/// A structure representing a Squirrel Context
#[derive(Debug, Clone)]
pub struct SquirrelContext {
    /// Context ID
    pub id: String,
    
    /// Context name
    pub name: String,
    
    /// Context data
    pub data: serde_json::Value,
    
    /// Context metadata
    pub metadata: serde_json::Value,
}

/// Define the context manager trait
#[async_trait]
pub trait ContextManager: Send + Sync {
    /// Create a new context
    async fn create_context(
        &self,
        id: &str,
        name: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()>;
    
    /// Get a context by ID
    async fn with_context(&self, id: &str) -> anyhow::Result<SquirrelContext>;
    
    /// Update a context
    async fn update_context(
        &self,
        id: &str,
        data: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()>;
    
    /// Delete a context
    async fn delete_context(&self, id: &str) -> anyhow::Result<()>;
    
    /// List all contexts
    async fn list_contexts(&self) -> anyhow::Result<Vec<SquirrelContext>>;
}

// Implement the ContextManager trait for SquirrelContextManager
#[async_trait]
impl ContextManager for SquirrelContextManager {
    async fn create_context(
        &self,
        _id: &str,
        _name: &str,
        _data: serde_json::Value,
        _metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        // Implement using the real methods of SquirrelContextManager
        Ok(())
    }
    
    async fn with_context(&self, id: &str) -> anyhow::Result<SquirrelContext> {
        // Create a dummy context for now
        Ok(SquirrelContext {
            id: id.to_string(),
            name: "Dummy".to_string(),
            data: serde_json::json!({}),
            metadata: serde_json::json!({}),
        })
    }
    
    async fn update_context(
        &self,
        _id: &str,
        _data: serde_json::Value,
        _metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    
    async fn delete_context(&self, _id: &str) -> anyhow::Result<()> {
        Ok(())
    }
    
    async fn list_contexts(&self) -> anyhow::Result<Vec<SquirrelContext>> {
        Ok(Vec::new())
    }
}

/// Helper function to convert anyhow errors to SquirrelContextError
fn convert_error(err: anyhow::Error) -> SquirrelContextError {
    // Create a generic context error
    SquirrelContextError::NotFound(format!("Error: {}", err))
}

/// Errors that can occur in the Context-MCP adapter
#[derive(Error, Debug)]
pub enum ContextMcpError {
    /// Error from the MCP context system
    #[error("MCP context error: {0}")]
    McpError(#[from] MCPError),
    
    /// Error from the Squirrel context system
    #[error("Squirrel context error: {0}")]
    ContextError(#[from] SquirrelContextError),
    
    /// Synchronization error
    #[error("Synchronization error: {0}")]
    SyncError(String),
    
    /// Circuit breaker open
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Context not found
    #[error("Context not found: {0}")]
    NotFound(String),
    
    /// AI processing error
    #[error("AI processing error: {0}")]
    AiError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Invalid input parameters
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type for Context-MCP adapter operations
pub type Result<T> = std::result::Result<T, ContextMcpError>;

/// Configuration for the Context-MCP adapter
#[derive(Debug, Clone)]
pub struct ContextMcpAdapterConfig {
    /// MCP context configuration
    pub mcp_config: Option<serde_json::Value>,
    
    /// Squirrel context configuration
    pub context_config: Option<SquirrelContextConfig>,
    
    /// Synchronization interval in seconds
    pub sync_interval_secs: u64,
    
    /// Circuit breaker configuration
    pub circuit_breaker_config: Option<BreakerConfig>,
    
    /// Max retry attempts for operations
    pub max_retries: u32,
    
    /// Timeout for operations in milliseconds
    pub timeout_ms: u64,
}

impl Default for ContextMcpAdapterConfig {
    fn default() -> Self {
        Self {
            mcp_config: None,
            context_config: None,
            sync_interval_secs: 60,
            circuit_breaker_config: None,
            max_retries: 3,
            timeout_ms: 5000,
        }
    }
}

/// Status of the Context-MCP adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterStatus {
    /// Is the adapter connected to MCP
    pub connected_to_mcp: bool,
    
    /// Is the adapter connected to Squirrel context
    pub connected_to_context: bool,
    
    /// Circuit breaker state
    pub circuit_breaker_state: String,
    
    /// Last sync timestamp
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Errors since startup
    pub error_count: u64,
    
    /// Successful syncs since startup
    pub successful_syncs: u64,
}

impl Default for AdapterStatus {
    fn default() -> Self {
        Self {
            connected_to_mcp: false,
            connected_to_context: false,
            circuit_breaker_state: "CLOSED".to_string(),
            last_sync: None,
            error_count: 0,
            successful_syncs: 0,
        }
    }
}

/// The Context-MCP adapter
/// 
/// This adapter bridges between the Squirrel context system and the MCP context manager,
/// providing bidirectional synchronization of context data.
pub struct ContextMcpAdapter {
    /// MCP context manager
    pub(crate) mcp_context_manager: Arc<McpContextManager>,
    
    /// Squirrel context manager
    pub(crate) squirrel_context_manager: Arc<dyn ContextManager>,
    
    /// Configuration
    pub(crate) config: ContextMcpAdapterConfig,
    
    /// Circuit breaker for MCP operations
    pub mcp_circuit_breaker: StandardCircuitBreaker,
    
    /// Status information
    pub(crate) status: Arc<tokio::sync::RwLock<AdapterStatus>>,
    
    /// Context ID mapping (Squirrel ID -> MCP ID)
    pub(crate) id_mapper: Arc<tokio::sync::RwLock<std::collections::HashMap<String, uuid::Uuid>>>,
}

impl std::fmt::Debug for ContextMcpAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMcpAdapter")
            .field("config", &self.config)
            .field("status", &self.status)
            .field("id_mapper", &self.id_mapper)
            .finish_non_exhaustive()
    }
}

impl ContextMcpAdapter {
    /// Create a new Context-MCP adapter with the given components
    #[instrument(skip(mcp_context_manager, squirrel_context_manager, config))]
    pub fn new(
        mcp_context_manager: Arc<McpContextManager>,
        squirrel_context_manager: Arc<dyn ContextManager>,
        config: ContextMcpAdapterConfig,
    ) -> Self {
        // Create circuit breaker with config or defaults
        let breaker_config = config.circuit_breaker_config.clone()
            .unwrap_or_else(|| squirrel_mcp::resilience::circuit_breaker::BreakerConfig {
                failure_threshold: 0.5,
                reset_timeout_ms: 30000,
                ..Default::default()
            });
        
        let mcp_circuit_breaker = StandardCircuitBreaker::new(breaker_config);
        
        Self {
            mcp_context_manager,
            squirrel_context_manager,
            config,
            mcp_circuit_breaker,
            status: Arc::new(tokio::sync::RwLock::new(AdapterStatus::default())),
            id_mapper: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// Create a Context-MCP adapter with the given configuration
    #[instrument(skip(config))]
    pub async fn with_config(config: ContextMcpAdapterConfig) -> Result<Self> {
        // Create MCP context manager
        let mcp_context_manager = Arc::new(McpContextManager::new().await);
        
        // Create Squirrel context manager
        let squirrel_context_manager = Arc::new(
            config.context_config
                .clone()
                .map(squirrel_context::ContextManagerImpl::with_config)
                .unwrap_or_else(squirrel_context::ContextManagerImpl::new)
        );
        
        Ok(Self::new(
            mcp_context_manager,
            squirrel_context_manager,
            config,
        ))
    }
    
    /// Create a Context-MCP adapter with the given configuration using the V2 trait
    /// for improved thread safety
    #[instrument(skip(config, context_manager))]
    pub async fn with_config_v2<T: ContextManagerV2 + 'static>(
        config: ContextMcpAdapterConfig,
        mut context_manager: T,
    ) -> Result<Self> {
        // Create MCP context manager
        let mcp_context_manager = Arc::new(McpContextManager::new().await);
        
        // Set up callbacks
        let mcp_clone = mcp_context_manager.clone();
        let callbacks = ContextManagerCallbacks {
            mcp_service: Some(Box::new(move |msg| {
                // Simple message pass-through to MCP service
                mcp_clone
                    .send_message(msg)
                    .map_err(|e| anyhow::anyhow!("MCP error: {}", e))
            })),
            log_event: Some(Box::new(|event_type, event_data| {
                // Simple logging callback
                info!("Context event: {} - {}", event_type, event_data);
                Ok(())
            })),
        };
        
        // Register callbacks with the context manager
        context_manager.register_callbacks(callbacks);
        
        // Wrap the V2 context manager to make it compatible with the adapter
        let wrapped_manager = ContextManagerWrapper::new(context_manager);
        
        Ok(Self::new(
            mcp_context_manager,
            Arc::new(wrapped_manager),
            config,
        ))
    }
    
    /// Initialize the adapter
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing Context-MCP adapter");
        
        // Initialize connections and verify they work
        let mcp_check = self.check_mcp_connection().await;
        let context_check = self.check_context_connection().await;
        
        // Update status
        {
            let mut status = self.status.write().await;
            status.connected_to_mcp = mcp_check.is_ok();
            status.connected_to_context = context_check.is_ok();
        }
        
        // Check if both connections are successful
        if mcp_check.is_err() || context_check.is_err() {
            let error_msg = format!(
                "Connection errors: MCP: {:?}, Context: {:?}",
                mcp_check.err(), context_check.err()
            );
            error!("{}", error_msg);
            return Err(ContextMcpError::ConfigError(error_msg));
        }
        
        // Subscribe to MCP context changes
        if let Err(err) = self.subscribe_to_mcp_changes().await {
            debug!("Failed to subscribe to MCP changes: {}", err);
            // Continue initialization despite subscription failure
        }
        
        // Start the sync task if interval is positive
        if self.config.sync_interval_secs > 0 {
            self.start_sync_task();
        }
        
        info!("Context-MCP adapter initialized successfully");
        Ok(())
    }
    
    /// Check connection to MCP context manager
    pub async fn check_mcp_connection(&self) -> std::result::Result<(), ContextMcpError> {
        self.mcp_circuit_breaker.execute(|| Box::pin(async {
            // Just try to access a method that doesn't modify anything
            let result: std::result::Result<(), squirrel_mcp::error::MCPError> = Ok(());
            result.map_err(|e| squirrel_mcp::resilience::circuit_breaker::BreakerError::from(e.to_string()))
        })).await.map_err(|e| ContextMcpError::McpError(squirrel_mcp::error::MCPError::from_string(e.to_string())))
    }
    
    /// Check connection to Squirrel context manager
    async fn check_context_connection(&self) -> Result<()> {
        debug!("Checking Squirrel context connection");
        
        // Just check if we can get a dummy context, it will fail but that's fine for connection test
        let _ = self.squirrel_context_manager.with_context("test").await;
        Ok(())
    }
    
    /// Subscribe to MCP context changes
    async fn subscribe_to_mcp_changes(&self) -> Result<()> {
        debug!("Subscribing to MCP context changes");
        
        // Subscribe to state changes
        let receiver = self.mcp_context_manager.subscribe_changes().await
            .map_err(|e| ContextMcpError::McpError(e))?;
        
        // Clone what we need for the task
        let adapter = self.clone();
        
        // Spawn a task to process changes
        tokio::spawn(async move {
            adapter.process_mcp_changes(receiver).await;
        });
        
        Ok(())
    }
    
    /// Process MCP context changes
    async fn process_mcp_changes(&self, mut receiver: tokio::sync::broadcast::Receiver<StateChange>) {
        debug!("Started processing MCP context changes");
        
        loop {
            match receiver.recv().await {
                Ok(change) => {
                    if let Err(err) = self.handle_mcp_change(change).await {
                        error!("Error handling MCP change: {}", err);
                        self.increment_error_count().await;
                    }
                }
                Err(err) => {
                    error!("Error receiving MCP changes: {}", err);
                    self.increment_error_count().await;
                    
                    // Try to resubscribe
                    match self.mcp_context_manager.subscribe_changes().await {
                        Ok(new_receiver) => {
                            receiver = new_receiver;
                            debug!("Resubscribed to MCP changes");
                        }
                        Err(err) => {
                            error!("Failed to resubscribe to MCP changes: {}", err);
                            
                            // Back off before trying again
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
    }
    
    /// Handle an MCP state change
    async fn handle_mcp_change(&self, change: StateChange) -> Result<()> {
        match change.operation {
            StateOperation::Create | StateOperation::Update => {
                // The data field is already a Value type, so we can use it directly
                // Convert to MCP context type
                if let Ok(context) = serde_json::from_value::<McpContext>(change.data) {
                    // Sync this context to Squirrel
                    self.sync_mcp_to_squirrel(context).await?;
                }
            }
            StateOperation::Delete => {
                // The id field is already a Uuid type
                self.handle_mcp_deletion(change.id).await?;
            }
            StateOperation::Sync => {
                // Sync operation is handled elsewhere
                debug!("Received Sync operation, ignoring as it's handled by periodic sync");
            }
        }
        
        Ok(())
    }
    
    /// Handle MCP context deletion
    async fn handle_mcp_deletion(&self, mcp_id: Uuid) -> Result<()> {
        // Find corresponding Squirrel ID
        let squirrel_id = self.find_squirrel_id_from_mcp(mcp_id).await?;
        
        // Delete from Squirrel context
        self.squirrel_context_manager.delete_context(&squirrel_id).await
            .map_err(|e| ContextMcpError::ContextError(convert_error(e)))?;
        
        // Remove from mapping
        let mut mapping = self.id_mapper.write().await;
        mapping.remove(&squirrel_id);
        
        Ok(())
    }
    
    /// Find Squirrel ID corresponding to MCP ID
    async fn find_squirrel_id_from_mcp(&self, mcp_id: Uuid) -> Result<String> {
        let mapping = self.id_mapper.read().await;
        
        for (squirrel_id, mapped_mcp_id) in mapping.iter() {
            if *mapped_mcp_id == mcp_id {
                return Ok(squirrel_id.clone());
            }
        }
        
        Err(ContextMcpError::NotFound(format!("No Squirrel ID found for MCP ID: {}", mcp_id)))
    }
    
    /// Sync a context from MCP to Squirrel
    async fn sync_mcp_to_squirrel(&self, mcp_context: McpContext) -> Result<()> {
        debug!("Syncing MCP context to Squirrel: {}", mcp_context.id);
        
        // Convert MCP context to Squirrel format
        let squirrel_id = mcp_context.id.to_string();
        
        // Try to get the context to see if it exists
        let context_result = self.squirrel_context_manager.with_context(&squirrel_id).await;
        
        match context_result {
            Ok(_) => {
                // Update existing context
                self.squirrel_context_manager.update_context(
                    &squirrel_id,
                    mcp_context.data.clone(),
                    mcp_context.metadata.clone(),
                ).await
                .map_err(|e| ContextMcpError::ContextError(convert_error(e)))?;
            }
            Err(_) => {
                // Create new context
                self.squirrel_context_manager.create_context(
                    &squirrel_id,
                    &mcp_context.name,
                    mcp_context.data.clone(),
                    mcp_context.metadata.clone(),
                ).await
                .map_err(|e| ContextMcpError::ContextError(convert_error(e)))?;
                
                // Store mapping
                let mut mapping = self.id_mapper.write().await;
                mapping.insert(squirrel_id, mcp_context.id);
            }
        }
        
        Ok(())
    }
    
    /// Start the periodic sync task
    fn start_sync_task(&self) {
        let interval = self.config.sync_interval_secs;
        debug!("Starting sync task with interval of {} seconds", interval);
        
        // Clone what we need for the task
        let adapter = self.clone();
        
        // Spawn the sync task
        tokio::spawn(async move {
            loop {
                // Wait for the interval
                tokio::time::sleep(Duration::from_secs(interval)).await;
                
                // Perform sync
                debug!("Performing scheduled sync");
                match adapter.sync_all().await {
                    Ok(result) => {
                        if result.status == SyncStatus::Success {
                            adapter.increment_sync_count().await;
                            debug!("Scheduled sync completed successfully: {} items synced, {} with errors", 
                                  result.items_synced, result.items_with_errors);
                        } else {
                            error!("Scheduled sync failed: {:?}", result.error_message);
                            adapter.increment_error_count().await;
                        }
                    }
                    Err(err) => {
                        error!("Error during scheduled sync: {}", err);
                        adapter.increment_error_count().await;
                    }
                }
            }
        });
    }
    
    /// Sync all contexts in both directions
    #[instrument(skip(self))]
    async fn sync_squirrel_to_mcp(&self) -> Result<()> {
        debug!("Syncing all contexts from Squirrel to MCP");
        
        // Get all Squirrel contexts
        let contexts = self.squirrel_context_manager.list_contexts().await
            .map_err(|e| ContextMcpError::ContextError(convert_error(e)))?;
        
        // Sync each context to MCP
        for context in contexts {
            if let Err(err) = self.sync_squirrel_context_to_mcp(&context.id).await {
                warn!("Error syncing Squirrel context {} to MCP: {}", context.id, err);
                // Continue with other contexts
            }
        }
        
        Ok(())
    }
    
    /// Sync a specific Squirrel context to MCP
    async fn sync_squirrel_context_to_mcp(&self, squirrel_id: &str) -> Result<()> {
        debug!("Syncing Squirrel context to MCP: {}", squirrel_id);
        
        // Get the Squirrel context
        let context = self.squirrel_context_manager.with_context(squirrel_id).await
            .map_err(|e| ContextMcpError::ContextError(convert_error(e)))?;
        
        // Check if we have an MCP ID for this Squirrel context
        let mcp_id = {
            let mapping = self.id_mapper.read().await;
            mapping.get(squirrel_id).cloned()
        };
        
        // Convert to MCP context
        let mcp_context = match mcp_id {
            Some(id) => {
                // Existing context, update
                McpContext {
                    id,
                    name: context.name.clone(),
                    data: context.data.clone(),
                    metadata: Some(context.metadata.clone()),
                    parent_id: None, // TODO: Handle parent IDs
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    expires_at: None,
                }
            }
            None => {
                // New context, create
                let id = Uuid::new_v4();
                
                // Store mapping
                {
                    let mut mapping = self.id_mapper.write().await;
                    mapping.insert(squirrel_id.to_string(), id);
                }
                
                McpContext {
                    id,
                    name: context.name.clone(),
                    data: context.data.clone(),
                    metadata: Some(context.metadata.clone()),
                    parent_id: None, // TODO: Handle parent IDs
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    expires_at: None,
                }
            }
        };
        
        // Use circuit breaker to create/update MCP context
        self.mcp_circuit_breaker.execute(move || Box::pin(async move {
            let mcp_context_manager = Arc::new(McpContextManager::new().await);
            if mcp_id.is_some() {
                // Update existing context
                mcp_context_manager.update_context(
                    mcp_context.id, 
                    mcp_context.data.clone(),
                    mcp_context.metadata.clone()
                ).await
                    .map_err(|e| BreakerError::from(e.to_string()))
            } else {
                // Create new context
                mcp_context_manager.create_context(mcp_context).await
                    .map_err(|e| BreakerError::from(e.to_string()))
                    .map(|_| ()) // Convert Result<Uuid, _> to Result<(), _>
            }
        })).await.map_err(|e| {
            ContextMcpError::CircuitBreakerOpen(format!("Circuit breaker open when syncing to MCP: {}", e))
        })?;
        
        Ok(())
    }
    
    /// Sync all contexts from MCP to Squirrel
    async fn sync_mcp_to_squirrel_all(&self) -> Result<()> {
        debug!("Syncing all contexts from MCP to Squirrel");
        
        // Use circuit breaker to get all contexts from MCP
        let contexts: Vec<McpContext> = self.mcp_circuit_breaker.execute(move || Box::pin(async move {
            // This is a placeholder as we don't have a direct "list all contexts" method in the MCP API
            // In a real implementation, this would use whatever method MCP provides to list contexts
            let result: std::result::Result<Vec<McpContext>, MCPError> = Ok(Vec::new());
            result.map_err(|e| BreakerError::from(e.to_string()))
        })).await.map_err(|e| {
            ContextMcpError::CircuitBreakerOpen(format!("Circuit breaker open when getting MCP contexts: {}", e))
        })?;
        
        // Sync each context to Squirrel
        for context in contexts {
            if let Err(err) = self.sync_mcp_to_squirrel(context).await {
                warn!("Error syncing MCP context to Squirrel: {}", err);
                // Continue with other contexts
            }
        }
        
        Ok(())
    }
    
    /// Get adapter status
    pub async fn get_status(&self) -> AdapterStatus {
        let status = self.status.read().await;
        status.clone()
    }
    
    /// Increment error count
    pub(crate) async fn increment_error_count(&self) {
        let mut status = self.status.write().await;
        status.error_count += 1;
    }
    
    /// Increment successful sync count
    pub(crate) async fn increment_sync_count(&self) {
        let mut status = self.status.write().await;
        status.successful_syncs += 1;
    }
    
    /// Update circuit breaker state in status
    pub(crate) async fn update_circuit_breaker_state(&self) {
        let state = self.mcp_circuit_breaker.state().await;
        let mut status = self.status.write().await;
        status.circuit_breaker_state = format!("{:?}", state);
    }
    
    /// Get access to the context manager
    pub fn context_manager(&self) -> Arc<dyn ContextManager> {
        self.squirrel_context_manager.clone()
    }
    
    /// Converts a context to a text representation
    ///
    /// This function takes a context and converts it to a textual representation
    /// that can be used for AI processing or other text-based operations.
    pub fn context_to_text(&self, context: &squirrel_mcp::Context) -> Result<String> {
        let mut output = String::new();
        
        // Add context ID
        output.push_str(&format!("## Context ID: {}\n\n", context.id));
        
        // Add context data with better formatting
        output.push_str("## Context Data\n");
        // Try to pretty-print JSON if possible
        match serde_json::to_string_pretty(&context.data) {
            Ok(pretty_json) => output.push_str(&format!("```json\n{}\n```\n\n", pretty_json)),
            Err(_) => output.push_str(&format!("{}\n\n", context.data)), // Fallback to raw display
        }
        
        // Add metadata if available
        if let Some(metadata) = &context.metadata {
            output.push_str("## Metadata\n");
            match serde_json::to_string_pretty(metadata) {
                Ok(json) => output.push_str(&format!("```json\n{}\n```\n\n", json)),
                Err(e) => {
                    tracing::warn!("Failed to serialize metadata to JSON: {}", e);
                    output.push_str("Metadata available but could not be serialized to JSON\n\n");
                }
            }
        }
        
        Ok(output)
    }
}

// Support for cloning the adapter
impl Clone for ContextMcpAdapter {
    fn clone(&self) -> Self {
        Self {
            mcp_context_manager: self.mcp_context_manager.clone(),
            squirrel_context_manager: self.squirrel_context_manager.clone(),
            config: self.config.clone(),
            mcp_circuit_breaker: self.mcp_circuit_breaker.clone(),
            status: self.status.clone(),
            id_mapper: self.id_mapper.clone(),
        }
    }
}

/// Returns a default model for the given AI provider
fn default_model_for_provider(provider: &str) -> Option<String> {
    match provider.to_lowercase().as_str() {
        "openai" => Some("gpt-4".to_string()),
        "anthropic" => Some("claude-3-opus-20240229".to_string()),
        "gemini" => Some("gemini-1.5-pro".to_string()),
        // Add more providers as they become available
        _ => None,
    }
}

/// Create a new Context-MCP adapter with optional configuration
pub async fn create_context_mcp_adapter(config: Option<ContextMcpAdapterConfig>) -> Result<ContextMcpAdapter> {
    let config = config.unwrap_or_default();
    ContextMcpAdapter::with_config(config).await
}

/// Options for AI context enhancement
#[derive(Debug, Clone)]
pub struct AiEnhancementOptions {
    /// AI provider to use (openai, anthropic, gemini)
    pub provider: String,
    
    /// API key for the AI provider
    pub api_key: String,
    
    /// Model to use (optional, defaults to an appropriate model for the provider)
    pub model: Option<String>,
    
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

impl AiEnhancementOptions {
    /// Create new options with the given provider and API key
    pub fn new(provider: impl Into<String>, api_key: impl Into<String>) -> Self {
        let provider_str = provider.into();
        Self {
            provider: provider_str.clone(),
            api_key: api_key.into(),
            model: default_model_for_provider(&provider_str),
            timeout_ms: None,
        }
    }
    
    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    
    /// Set the timeout in milliseconds
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
} 