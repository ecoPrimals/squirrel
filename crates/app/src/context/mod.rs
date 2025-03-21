//! Context management for the Squirrel project
//!
//! This module provides the context management functionality, which is central
//! to the operation of the Squirrel system. The context maintains the state
//! and provides access to various system components.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::events::Event;
use serde_json::Value;
use uuid;
use std::time::Duration;
use crate::events::DefaultEventEmitter;
use crate::core::AppConfig;
use crate::metrics::Metrics;
use thiserror::Error;
use crate::error::Result;

/// Configuration for a context instance.
/// 
/// This struct holds the configuration parameters that define how a context
/// should be initialized and behave during its lifecycle.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Unique identifier for the context.
    pub id: String,
    /// Human-readable name for the context.
    pub name: String,
    /// Detailed description of the context's purpose.
    pub description: String,
    /// Environment in which the context is running (e.g., "development", "production").
    pub environment: String,
    /// Version string for the context.
    pub version: String,
    /// Additional metadata key-value pairs associated with the context.
    pub metadata: HashMap<String, String>,
    /// Whether the context should persist its state.
    pub persistence: bool,
    /// Maximum number of entries to store in the context's history.
    pub max_entries: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "default".to_string(),
            description: "Default context".to_string(),
            environment: "development".to_string(),
            version: "0.1.0".to_string(),
            metadata: HashMap::new(),
            persistence: false,
            max_entries: 1000,
        }
    }
}

/// Represents the current state of a context.
/// 
/// This struct maintains the runtime state of a context, including its metadata,
/// state data, timestamps, and lifecycle information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    /// Unique identifier for the context state.
    pub id: String,
    /// Additional metadata associated with the context state.
    pub metadata: HashMap<String, Value>,
    /// Current state data stored in the context.
    pub state: HashMap<String, Value>,
    /// Timestamp when the context state was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the context state was last updated.
    pub updated_at: DateTime<Utc>,
    /// Whether the context has been initialized.
    pub initialized: bool,
    /// Whether the context is in the process of shutting down.
    pub shutting_down: bool,
    /// Current stage in the context's lifecycle.
    pub lifecycle_stage: LifecycleStage,
}

impl Default for ContextState {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            metadata: HashMap::new(),
            state: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            initialized: false,
            shutting_down: false,
            lifecycle_stage: LifecycleStage::Created,
        }
    }
}

/// Represents the different stages in a context's lifecycle.
/// 
/// This enum defines all possible states that a context can be in during its
/// lifecycle, from creation to shutdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LifecycleStage {
    /// Initial state before any initialization has occurred.
    Uninitialized,
    /// Context has been created but not yet initialized.
    Created,
    /// Context is in the process of initializing.
    Initializing,
    /// Context is fully initialized and running normally.
    Running,
    /// Context execution has been temporarily paused.
    Paused,
    /// Context has been stopped but not shut down.
    Stopped,
    /// Context is in the process of shutting down.
    ShuttingDown,
    /// Context has been fully shut down.
    Shutdown,
    /// Context has encountered an error (with error message).
    Error(String),
}

impl Default for LifecycleStage {
    fn default() -> Self {
        Self::Created
    }
}

/// A context represents a managed system state with lifecycle control.
/// 
/// The Context struct is the central component that manages system state,
/// configuration, metrics, and events. It provides methods to control the
/// lifecycle of the context and access its various components.
#[derive(Debug, Clone)]
pub struct Context {
    /// The configuration for this context.
    #[allow(dead_code)]
    config: Arc<RwLock<ContextConfig>>,
    /// The current state of the context.
    state_store: Arc<RwLock<ContextState>>,
    /// Metrics collection for this context.
    metrics: Arc<Metrics>,
    /// Event history for this context.
    events: Arc<RwLock<Vec<Event>>>,
}

impl Context {
    /// Creates a new context with the specified configuration.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The configuration to use for this context.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing the new `Context` instance if successful.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the context creation fails due to invalid configuration.
    pub fn new(config: ContextConfig) -> Result<Self> {
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            state_store: Arc::new(RwLock::new(ContextState::default())),
            metrics: Arc::new(Metrics::new()),
            events: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Initializes the context, preparing it for use.
    /// 
    /// This method transitions the context from the `Uninitialized` state to
    /// `Initializing` and marks it as initialized.
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::AlreadyInitialized` if the context is not in the
    /// `Uninitialized` state.
    pub async fn initialize(&self) -> Result<()> {
        let mut state = self.state_store.write().await;
        if state.initialized {
            return Err(ContextError::AlreadyInitialized.into());
        }
        state.lifecycle_stage = LifecycleStage::Initializing;
        state.initialized = true;
        state.updated_at = Utc::now();
        Ok(())
    }

    /// Starts the context, transitioning it to the `Running` state.
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::NotInitialized` if the context has not been initialized.
    pub async fn start(&self) -> Result<()> {
        let mut state = self.state_store.write().await;
        if !state.initialized {
            return Err(ContextError::NotInitialized.into());
        }
        state.lifecycle_stage = LifecycleStage::Running;
        Ok(())
    }

    /// Begins the shutdown process for the context.
    /// 
    /// This method transitions the context to the `ShuttingDown` state and marks
    /// it as shutting down.
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::AlreadyShuttingDown` if the context is already
    /// in the process of shutting down.
    pub async fn shutdown(&self) -> Result<()> {
        let mut state = self.state_store.write().await;
        if state.shutting_down {
            return Err(ContextError::AlreadyShuttingDown.into());
        }
        state.lifecycle_stage = LifecycleStage::ShuttingDown;
        state.shutting_down = true;
        state.updated_at = Utc::now();
        Ok(())
    }

    /// Stops the context, transitioning it to the `Stopped` state.
    /// 
    /// This method should only be called after `shutdown()` has been called.
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::NotInitialized` if the context has not been initialized,
    /// or `ContextError::Lifecycle` if the context is not in the `ShuttingDown` state.
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state_store.write().await;
        if !state.initialized {
            return Err(ContextError::NotInitialized.into());
        }
        if !state.shutting_down {
            return Err(ContextError::Lifecycle("Context not shutting down".to_string()).into());
        }
        state.lifecycle_stage = LifecycleStage::Stopped;
        Ok(())
    }

    /// Returns the current lifecycle stage of the context.
    /// 
    /// This method is marked with `#[must_use]` because the returned stage
    /// may be needed for state-dependent operations.
    #[must_use = "This returns the current lifecycle stage which may be needed for state-dependent operations"]
    pub async fn get_lifecycle_stage(&self) -> LifecycleStage {
        self.state_store.read().await.lifecycle_stage.clone()
    }

    /// Returns the metrics collector for this context.
    /// 
    /// The returned metrics instance should be used for recording metrics
    /// related to this context.
    #[must_use = "This returns a metrics instance that should be used for recording metrics"]
    pub fn metrics(&self) -> Arc<Metrics> {
        self.metrics.clone()
    }

    /// Returns the events store for this context.
    /// 
    /// The returned events storage should be used for all event-related operations.
    #[must_use = "This returns the events storage that should be used for event operations"]
    pub fn events(&self) -> Arc<RwLock<Vec<Event>>> {
        self.events.clone()
    }

    /// Returns the state store for this context.
    /// 
    /// The returned state storage should be used for all state-related operations.
    #[must_use = "This returns the state storage that should be used for state operations"]
    pub fn state(&self) -> Arc<RwLock<ContextState>> {
        self.state_store.clone()
    }

    /// Updates a data value in the context's state.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to update
    /// * `value` - The new value to store
    /// 
    /// # Errors
    /// 
    /// Returns a `Result` indicating whether the update was successful.
    pub async fn update_data(&self, key: &str, value: Value) -> Result<()> {
        let mut state = self.state_store.write().await;
        state.state.insert(key.to_string(), value);
        state.updated_at = Utc::now();
        Ok(())
    }

    /// Updates a metadata value in the context's state.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to update
    /// * `value` - The new value to store
    /// 
    /// # Errors
    /// 
    /// Returns a `Result` indicating whether the update was successful.
    pub async fn update_metadata(&self, key: String, value: Value) -> Result<()> {
        let mut state = self.state_store.write().await;
        state.metadata.insert(key, value);
        state.updated_at = Utc::now();
        Ok(())
    }

    /// Retrieves a data value from the context's state.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to retrieve
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing an `Option` with the value if it exists.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the state store cannot be accessed.
    pub async fn get_data(&self, key: &str) -> Result<Option<Value>> {
        let state = self.state_store.read().await;
        Ok(state.state.get(key).cloned())
    }

    /// Retrieves all metadata from the context's state.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing a `HashMap` of all metadata key-value pairs.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the state store cannot be accessed.
    pub async fn get_metadata(&self) -> Result<HashMap<String, Value>> {
        let state = self.state_store.read().await;
        Ok(state.metadata.clone())
    }

    /// Retrieves the current state of the context.
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::NotInitialized` if the context has not been initialized.
    pub async fn get_state(&self) -> Result<ContextState> {
        let state = self.state_store.read().await;
        if !state.initialized {
            return Err(ContextError::NotInitialized.into());
        }
        Ok(state.clone())
    }

    /// Creates a snapshot of the current context state.
    /// 
    /// A snapshot includes the current state, metrics, and other relevant
    /// information at the time it is taken.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing the `ContextSnapshot` if successful.
    ///
    /// # Errors
    ///
    /// Will return an error if the internal state cannot be accessed or if the snapshot creation fails.
    pub async fn get_snapshot(&self) -> Result<ContextSnapshot> {
        let state = self.state_store.read().await;
        Ok(ContextSnapshot {
            timestamp: Utc::now(),
            data: state.state.clone(),
            metrics: state.metadata.clone(),
            state: state.state.clone(),
            created_at: state.created_at,
            updated_at: state.updated_at,
            lifecycle_stage: state.lifecycle_stage.clone(),
        })
    }

    /// Sets the application state
    ///
    /// # Arguments
    /// * `new_state` - New state to set
    ///
    /// # Returns
    /// * `Result<()>` - Success or error status
    /// 
    /// # Errors
    /// 
    /// Returns an error if the context is shutting down or if the state store cannot be accessed.
    pub async fn set_state(&self, new_state: HashMap<String, Value>) -> Result<()> {
        let mut state = self.state_store.write().await;
        if state.shutting_down {
            return Err(ContextError::AlreadyShuttingDown.into());
        }
        state.state = new_state;
        state.updated_at = Utc::now();
        Ok(())
    }

    /// Checks if the application is initialized
    ///
    /// # Returns
    /// * `bool` - True if initialized
    pub async fn is_initialized(&self) -> bool {
        self.state_store.read().await.initialized
    }

    /// Checks if the application is shutting down
    ///
    /// # Returns
    /// * `bool` - True if shutting down
    pub async fn is_shutting_down(&self) -> bool {
        self.state_store.read().await.shutting_down
    }

    /// Sets the application lifecycle stage
    ///
    /// # Arguments
    /// * `stage` - New lifecycle stage
    ///
    /// # Returns
    /// * `Result<()>` - Success or error status
    /// 
    /// # Errors
    /// 
    /// Returns an error if the state store cannot be accessed.
    pub async fn set_lifecycle_stage(&self, stage: LifecycleStage) -> Result<()> {
        let mut ctx_state = self.state_store.write().await;
        ctx_state.lifecycle_stage = stage;
        ctx_state.updated_at = Utc::now();
        Ok(())
    }
}

/// Builds context instances with a fluent API.
pub struct ContextBuilder {
    /// The configuration for the context being built
    config: ContextConfig,
}

impl ContextBuilder {
    /// Create a new context builder
    #[must_use = "This returns a builder that should be used to configure and create a Context"]
    pub fn new() -> Self {
        Self {
            config: ContextConfig::default(),
        }
    }

    /// Set whether to enable metrics
    #[must_use = "This method returns self for method chaining and the return value should be used"]
    pub fn enable_metrics(self, enable: bool) -> Self {
        let mut builder = self;
        builder.config.metadata.insert("enable_metrics".to_string(), enable.to_string());
        builder
    }

    /// Set whether to enable events
    #[must_use = "This method returns self for method chaining and the return value should be used"]
    pub fn enable_events(self, enable: bool) -> Self {
        let mut builder = self;
        builder.config.metadata.insert("enable_events".to_string(), enable.to_string());
        builder
    }

    /// Set the maximum number of events to store
    #[must_use = "This method returns self for method chaining and the return value should be used"]
    pub fn max_events(self, max: usize) -> Self {
        let mut builder = self;
        builder.config.metadata.insert("max_events".to_string(), max.to_string());
        builder
    }

    /// Build the context
    /// 
    /// # Returns
    /// 
    /// Returns a new `Context` instance configured with the builder's settings.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the context creation fails due to invalid configuration.
    pub fn build(self) -> Result<Context> {
        Context::new(self.config)
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A snapshot of application context at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Timestamp when the snapshot was taken
    pub timestamp: DateTime<Utc>,
    /// Application data
    pub data: HashMap<String, Value>,
    /// Application metrics
    pub metrics: HashMap<String, Value>,
    /// Application state
    pub state: HashMap<String, Value>,
    /// Time when the snapshot was created
    pub created_at: DateTime<Utc>,
    /// Time when the snapshot was last updated
    pub updated_at: DateTime<Utc>,
    /// Current lifecycle stage
    pub lifecycle_stage: LifecycleStage,
}

/// Configuration for context persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Whether persistence is enabled
    pub enabled: bool,
    /// Path to store persistent data
    pub path: String,
    /// Whether to automatically save state
    pub auto_save: bool,
    /// Interval between auto-saves
    pub save_interval: Duration,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: String::new(),
            auto_save: false,
            save_interval: Duration::from_secs(300),
        }
    }
}

/// Configuration for context synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Whether synchronization is enabled
    pub enabled: bool,
    /// List of peer nodes to sync with
    pub peers: Vec<String>,
    /// Interval between sync attempts
    pub sync_interval: Duration,
    /// Maximum number of sync retries
    pub max_retries: u32,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            peers: Vec::new(),
            sync_interval: Duration::from_secs(60),
            max_retries: 3,
        }
    }
}

/// Errors that can occur during context operations
#[derive(Debug, Error)]
pub enum ContextError {
    /// Error during initialization
    #[error("Initialization error: {0}")]
    Initialization(String),
    /// Error during lifecycle transition
    #[error("Lifecycle error: {0}")]
    Lifecycle(String),
    /// Error with context data
    #[error("Data error: {0}")]
    Data(String),
    /// Error with context metadata
    #[error("Metadata error: {0}")]
    Metadata(String),
    /// Other error type
    #[error("Error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
    /// Context is already initialized
    #[error("Context is already initialized")]
    AlreadyInitialized,
    /// Context is not initialized
    #[error("Context is not initialized")]
    NotInitialized,
    /// Context is already shutting down
    #[error("Context is already shutting down")]
    AlreadyShuttingDown,
    /// Context already exists
    #[error("Context already exists: {0}")]
    ContextExists(String),
    /// Context not found
    #[error("Context not found: {0}")]
    ContextNotFound(String),
    /// Invalid context state
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl From<ContextError> for squirrel_core::error::SquirrelError {
    fn from(err: ContextError) -> Self {
        Self::other(err.to_string())
    }
}

/// Manager for multiple contexts within an application.
/// 
/// The `ContextManager` is responsible for creating, tracking, and managing
/// multiple contexts, ensuring proper lifecycle management and resource
/// allocation.
#[derive(Debug)]
pub struct ContextManager {
    /// Metrics collection for all managed contexts.
    pub metrics: Arc<Metrics>,
    /// Map of context IDs to their corresponding Context instances.
    pub contexts: Arc<RwLock<HashMap<String, Arc<Context>>>>,
}

impl ContextManager {
    /// Creates a new `ContextManager` instance.
    /// 
    /// The manager is initialized with an empty context map and default metrics.
    #[must_use] pub fn new() -> Self {
        Self {
            metrics: Arc::new(Metrics::new()),
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new context with the specified ID.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The unique identifier for the new context
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::ContextExists` if a context with the given ID
    /// already exists.
    pub async fn create_context(&self, id: String) -> Result<Arc<Context>> {
        let contexts = self.contexts.read().await;
        if contexts.contains_key(&id) {
            return Err(ContextError::ContextExists(id).into());
        }
        drop(contexts);

        let config = ContextConfig {
            id: id.clone(),
            ..ContextConfig::default()
        };
        let context = Arc::new(Context::new(config)?);
        let mut contexts = self.contexts.write().await;
        contexts.insert(id, context.clone());
        Ok(context)
    }

    /// Retrieves a context by its ID.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the context to retrieve
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::ContextNotFound` if no context exists with the
    /// given ID.
    pub async fn get_context(&self, id: &str) -> Result<Arc<Context>> {
        let contexts = self.contexts.read().await;
        contexts.get(id).cloned().ok_or_else(|| ContextError::ContextNotFound(id.to_string()).into())
    }

    /// Deletes a context by its ID.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the context to delete
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::ContextNotFound` if no context exists with the
    /// given ID.
    pub async fn delete_context(&self, id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.remove(id) {
            context.stop().await?;
        }
        Ok(())
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating `ContextTracker` instances
#[derive(Debug, Clone, Default)]
pub struct ContextTrackerFactory {
    /// The default context manager to use
    manager: Option<Arc<ContextManager>>,
}

impl ContextTrackerFactory {
    /// Create a new `ContextTrackerFactory`
    ///
    /// # Arguments
    ///
    /// * `manager` - Optional default context manager to use
    #[must_use] pub fn new(manager: Option<Arc<ContextManager>>) -> Self {
        Self { 
            manager,
        }
    }

    /// Create a factory with an existing manager and configuration
    ///
    /// # Arguments
    ///
    /// * `manager` - Optional default context manager to use
    /// * `config` - Configuration for the context tracker
    #[must_use] pub fn with_config(manager: Option<Arc<ContextManager>>, _config: ContextConfig) -> Self {
        Self {
            manager,
        }
    }

    /// Create a new `ContextTracker` with the default manager
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a new `ContextTracker` instance using the default manager
    ///
    /// # Errors
    ///
    /// Returns `ContextError::NotInitialized` if no default manager was provided during factory creation
    pub fn create(&self) -> Result<ContextTracker> {
        match &self.manager {
            Some(manager) => Ok(ContextTracker::new(manager.clone())),
            None => Err(ContextError::NotInitialized.into())
        }
    }

    /// Create a context tracker with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for the context tracker
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a new `ContextTracker` instance with the specified configuration
    ///
    /// # Errors
    ///
    /// Returns `ContextError::NotInitialized` if no default manager was provided during factory creation
    pub fn create_with_config(&self, _config: ContextConfig) -> Result<ContextTracker> {
        match &self.manager {
            Some(manager) => {
                let tracker = ContextTracker::new(manager.clone());
                // Apply configuration if needed
                // Currently ContextTracker doesn't have a configure method, so we'd need to add that
                // For now, we just return the tracker
                Ok(tracker)
            },
            None => Err(ContextError::NotInitialized.into())
        }
    }

    /// Create a new `ContextTracker` with a specific manager
    ///
    /// # Arguments
    ///
    /// * `manager` - The `ContextManager` to use
    #[must_use] pub fn create_with_manager(&self, manager: Arc<ContextManager>) -> ContextTracker {
        ContextTracker::new(manager)
    }
}

/// Tracks the currently active context in a multi-context environment.
/// 
/// The `ContextTracker` maintains a reference to the currently active context
/// and provides methods to switch between contexts.
#[derive(Debug, Clone)]
pub struct ContextTracker {
    /// The context manager that owns all contexts.
    manager: Arc<ContextManager>,
    /// The ID of the currently active context, if any.
    active_context: Arc<RwLock<Option<String>>>,
}

impl ContextTracker {
    /// Creates a new `ContextTracker` with the specified manager.
    /// 
    /// # Arguments
    /// 
    /// * `manager` - The `ContextManager` to track contexts from
    #[must_use] pub fn new(manager: Arc<ContextManager>) -> Self {
        Self {
            manager,
            active_context: Arc::new(RwLock::new(None)),
        }
    }

    /// Sets the specified context as the active context.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the context to activate
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::ContextNotFound` if no context exists with the
    /// given ID.
    pub async fn activate_context(&self, id: &str) -> Result<()> {
        self.manager.get_context(id).await?;
        let mut active = self.active_context.write().await;
        *active = Some(id.to_string());
        Ok(())
    }

    /// Deactivates the currently active context, if any.
    /// 
    /// # Errors
    /// 
    /// Returns a `Result` indicating whether the deactivation was successful.
    pub async fn deactivate_context(&self) -> Result<()> {
        let mut active = self.active_context.write().await;
        *active = None;
        Ok(())
    }

    /// Retrieves the currently active context, if one exists.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing an `Option` with the active context.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the active context ID is invalid or if the context manager fails to retrieve the context.
    pub async fn get_active_context(&self) -> Result<Option<Arc<Context>>> {
        if let Some(id) = self.active_context.read().await.as_ref() {
            Ok(Some(self.manager.get_context(id).await?))
        } else {
            Ok(None)
        }
    }
}

/// Create a context tracker with default configuration
///
/// # Returns
///
/// Returns a `Result` containing a new `ContextTracker` instance
///
/// # Errors
///
/// Returns an error if the context tracker creation fails
pub fn create_context_tracker() -> Result<ContextTracker> {
    let manager = Arc::new(ContextManager::new());
    Ok(ContextTracker::new(manager))
}

/// Create a context tracker with custom configuration
///
/// # Arguments
///
/// * `config` - The configuration to use
///
/// # Returns
///
/// Returns a `Result` containing a new `ContextTracker` instance
///
/// # Errors
///
/// Returns an error if the context tracker creation fails
pub fn create_context_tracker_with_config(_config: ContextConfig) -> Result<ContextTracker> {
    let manager = Arc::new(ContextManager::new());
    let tracker = ContextTracker::new(manager);
    // Apply configuration if needed
    // Currently ContextTracker doesn't have a configure method, so we'd need to add that
    Ok(tracker)
}

/// Application-wide context that stores configuration and shared components.
/// 
/// The `AppContext` provides access to application-level configuration and
/// components that are shared across the entire application.
#[derive(Debug)]
pub struct AppContext {
    /// The application's configuration.
    config: AppConfig,
    /// The application's event emitter for broadcasting events.
    event_emitter: Arc<DefaultEventEmitter>,
}

impl AppContext {
    /// Creates a new `AppContext` with the specified configuration and event emitter.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The application configuration to use
    /// * `event_emitter` - The event emitter to use for broadcasting events
    #[must_use = "This returns a new application context that should be used for application operations"]
    pub const fn new(config: AppConfig, event_emitter: Arc<DefaultEventEmitter>) -> Self {
        Self {
            config,
            event_emitter,
        }
    }

    /// Returns a reference to the application's configuration.
    /// 
    /// The configuration contains important settings that control the
    /// application's behavior.
    #[must_use = "This returns the application configuration that contains important settings"]
    pub const fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Returns a reference to the application's event emitter.
    /// 
    /// The event emitter is used to broadcast events throughout the
    /// application.
    #[must_use = "This returns the event emitter that should be used for broadcasting events"]
    pub fn event_emitter(&self) -> Arc<DefaultEventEmitter> {
        self.event_emitter.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_tracker_factory_default() {
        let factory = ContextTrackerFactory::default();
        assert!(factory.manager.is_none());
    }
    
    #[test]
    fn test_context_tracker_factory_new() {
        let manager = Arc::new(ContextManager::new());
        let factory = ContextTrackerFactory::new(Some(manager.clone()));
        assert!(factory.manager.is_some());
        assert!(Arc::ptr_eq(&factory.manager.unwrap(), &manager));
    }
    
    #[test]
    fn test_context_tracker_factory_create_with_manager() {
        let manager1 = Arc::new(ContextManager::new());
        let manager2 = Arc::new(ContextManager::new());
        
        let factory = ContextTrackerFactory::new(Some(manager1));
        let tracker = factory.create_with_manager(manager2.clone());
        
        // Tracker should use the explicitly provided manager, not the factory's default
        assert!(Arc::ptr_eq(&tracker.manager, &manager2));
    }
    
    #[test]
    fn test_context_tracker_factory_create() {
        let manager = Arc::new(ContextManager::new());
        let factory = ContextTrackerFactory::new(Some(manager.clone()));
        
        let tracker = factory.create().unwrap();
        assert!(Arc::ptr_eq(&tracker.manager, &manager));
    }
    
    #[test]
    fn test_context_tracker_factory_create_no_default() {
        let factory = ContextTrackerFactory::default();
        // Should return an error due to no default manager
        let result = factory.create();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_context_tracker_factory_with_config() {
        let manager = Arc::new(ContextManager::new());
        let config = ContextConfig::default();
        let factory = ContextTrackerFactory::with_config(Some(manager.clone()), config);
        
        assert!(factory.manager.is_some());
        
        let tracker = factory.create().unwrap();
        assert!(Arc::ptr_eq(&tracker.manager, &manager));
    }
    
    #[test]
    fn test_context_tracker_factory_create_with_config() {
        let manager = Arc::new(ContextManager::new());
        let factory = ContextTrackerFactory::new(Some(manager.clone()));
        
        let config = ContextConfig::default();
        let tracker = factory.create_with_config(config).unwrap();
        
        assert!(Arc::ptr_eq(&tracker.manager, &manager));
    }
    
    #[tokio::test]
    async fn test_helper_create_context_tracker() {
        let tracker = create_context_tracker().unwrap();
        assert!(tracker.active_context.read().await.is_none());
    }
    
    #[tokio::test]
    async fn test_helper_create_context_tracker_with_config() {
        let config = ContextConfig::default();
        let tracker = create_context_tracker_with_config(config).unwrap();
        assert!(tracker.active_context.read().await.is_none());
    }
}