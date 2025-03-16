//! Context management for the Squirrel project
//!
//! This module provides the context management functionality, which is central
//! to the operation of the Squirrel system. The context maintains the state
//! and provides access to various system components.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::app::events::Event;
use serde_json::Value;
use uuid;
use std::time::Duration;
use crate::error::SquirrelError;
use crate::app::events::DefaultEventEmitter;
use crate::app::AppConfig;
use crate::app::Metrics;

/// Configuration for the context
#[derive(Debug, Clone)]
pub struct ContextConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub environment: String,
    pub version: String,
    pub metadata: HashMap<String, String>,
    pub persistence: bool,
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

/// The main context type that holds system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    pub id: String,
    pub metadata: HashMap<String, Value>,
    pub state: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub initialized: bool,
    pub shutting_down: bool,
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

/// The lifecycle stages of the context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LifecycleStage {
    Uninitialized,
    Created,
    Initializing,
    Running,
    Paused,
    Stopped,
    ShuttingDown,
    Shutdown,
    Error(String),
}

impl Default for LifecycleStage {
    fn default() -> Self {
        Self::Created
    }
}

/// The main context type that holds system state
#[derive(Debug, Clone)]
pub struct Context {
    #[allow(dead_code)]
    config: Arc<RwLock<ContextConfig>>,
    state_store: Arc<RwLock<ContextState>>,
    metrics: Arc<Metrics>,
    events: Arc<RwLock<Vec<Event>>>,
}

impl Context {
    /// Create a new context with default configuration
    pub fn new(config: ContextConfig) -> Result<Self> {
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            state_store: Arc::new(RwLock::new(ContextState::default())),
            metrics: Arc::new(Metrics::new()),
            events: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Initialize the context
    pub async fn initialize(&self) -> Result<()> {
        let mut state = self.state_store.write().await;
        if state.lifecycle_stage != LifecycleStage::Uninitialized {
            return Err(ContextError::AlreadyInitialized);
        }
        state.lifecycle_stage = LifecycleStage::Initializing;
        state.initialized = true;
        state.updated_at = Utc::now();
        Ok(())
    }

    /// Start the context
    pub async fn start(&self) -> Result<()> {
        let mut state = self.state_store.write().await;
        if !state.initialized {
            return Err(ContextError::NotInitialized);
        }
        state.lifecycle_stage = LifecycleStage::Running;
        Ok(())
    }

    /// Shutdown the context
    pub async fn shutdown(&self) -> Result<()> {
        let mut state = self.state_store.write().await;
        if state.lifecycle_stage == LifecycleStage::ShuttingDown {
            return Err(ContextError::AlreadyShuttingDown);
        }
        state.lifecycle_stage = LifecycleStage::ShuttingDown;
        state.shutting_down = true;
        state.updated_at = Utc::now();
        Ok(())
    }

    /// Stop the context
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state_store.write().await;
        if !state.initialized {
            return Err(ContextError::NotInitialized);
        }
        if state.lifecycle_stage != LifecycleStage::ShuttingDown {
            return Err(ContextError::Lifecycle("Context not shutting down".to_string()));
        }
        state.lifecycle_stage = LifecycleStage::Stopped;
        Ok(())
    }

    /// Get the current lifecycle stage
    pub async fn get_lifecycle_stage(&self) -> LifecycleStage {
        self.state_store.read().await.lifecycle_stage.clone()
    }

    /// Get the metrics collector
    pub fn metrics(&self) -> Arc<Metrics> {
        self.metrics.clone()
    }

    /// Get the events store
    pub fn events(&self) -> Arc<RwLock<Vec<Event>>> {
        self.events.clone()
    }

    /// Get the state store
    pub fn state(&self) -> Arc<RwLock<ContextState>> {
        self.state_store.clone()
    }

    pub async fn update_data(&self, key: &str, value: Value) -> Result<()> {
        let mut state = self.state_store.write().await;
        state.state.insert(key.to_string(), value);
        state.updated_at = Utc::now();
        Ok(())
    }

    pub async fn update_metadata(&self, key: String, value: Value) -> Result<()> {
        let mut state = self.state_store.write().await;
        state.metadata.insert(key, value);
        state.updated_at = Utc::now();
        Ok(())
    }

    pub async fn get_data(&self, key: &str) -> Result<Option<Value>> {
        let state = self.state_store.read().await;
        Ok(state.state.get(key).cloned())
    }

    pub async fn get_metadata(&self) -> Result<HashMap<String, Value>> {
        let state = self.state_store.read().await;
        Ok(state.metadata.clone())
    }

    pub async fn get_state(&self) -> Result<ContextState> {
        if !self.state_store.read().await.initialized {
            return Err(ContextError::NotInitialized);
        }
        Ok(self.state_store.read().await.clone())
    }

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

    pub async fn set_state(&self, new_state: HashMap<String, Value>) -> Result<()> {
        let mut state = self.state_store.write().await;
        if state.lifecycle_stage == LifecycleStage::ShuttingDown {
            return Err(ContextError::AlreadyShuttingDown);
        }
        state.state = new_state;
        state.updated_at = Utc::now();
        Ok(())
    }

    pub async fn is_initialized(&self) -> bool {
        self.state_store.read().await.initialized
    }

    pub async fn is_shutting_down(&self) -> bool {
        self.state_store.read().await.shutting_down
    }

    pub async fn set_lifecycle_stage(&self, stage: LifecycleStage) -> Result<()> {
        let mut ctx_state = self.state_store.write().await;
        ctx_state.lifecycle_stage = stage;
        ctx_state.updated_at = Utc::now();
        Ok(())
    }
}

/// Builder for creating a new context
pub struct ContextBuilder {
    config: ContextConfig,
}

impl ContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        Self {
            config: ContextConfig::default(),
        }
    }

    /// Set whether to enable metrics
    #[must_use]
    pub fn enable_metrics(self, enable: bool) -> Self {
        let mut builder = self;
        builder.config.metadata.insert("enable_metrics".to_string(), enable.to_string());
        builder
    }

    /// Set whether to enable events
    #[must_use]
    pub fn enable_events(self, enable: bool) -> Self {
        let mut builder = self;
        builder.config.metadata.insert("enable_events".to_string(), enable.to_string());
        builder
    }

    /// Set the maximum number of events to store
    #[must_use]
    pub fn max_events(self, max: usize) -> Self {
        let mut builder = self;
        builder.config.metadata.insert("max_events".to_string(), max.to_string());
        builder
    }

    /// Build the context
    pub fn build(self) -> Result<Context> {
        Context::new(self.config)
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub timestamp: DateTime<Utc>,
    pub data: HashMap<String, Value>,
    pub metrics: HashMap<String, Value>,
    pub state: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub lifecycle_stage: LifecycleStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub path: String,
    pub auto_save: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub peers: Vec<String>,
    pub sync_interval: Duration,
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

#[derive(Debug)]
pub enum ContextError {
    Initialization(String),
    Lifecycle(String),
    Data(String),
    Metadata(String),
    Other(Box<dyn std::error::Error + Send + Sync>),
    AlreadyInitialized,
    NotInitialized,
    AlreadyShuttingDown,
    ContextExists(String),
    ContextNotFound(String),
    InvalidState(String),
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextError::Initialization(msg) => write!(f, "Context initialization error: {msg}"),
            ContextError::Lifecycle(msg) => write!(f, "Context lifecycle error: {msg}"),
            ContextError::Data(msg) => write!(f, "Context data error: {msg}"),
            ContextError::Metadata(msg) => write!(f, "Context metadata error: {msg}"),
            ContextError::Other(e) => write!(f, "Other context error: {e}"),
            ContextError::AlreadyInitialized => write!(f, "Context already initialized"),
            ContextError::NotInitialized => write!(f, "Context not initialized"),
            ContextError::AlreadyShuttingDown => write!(f, "Context already shutting down"),
            ContextError::ContextExists(id) => write!(f, "Context with id {id} already exists"),
            ContextError::ContextNotFound(id) => write!(f, "Context with id {id} not found"),
            ContextError::InvalidState(msg) => write!(f, "Invalid context state: {msg}"),
        }
    }
}

impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ContextError::Other(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl From<ContextError> for SquirrelError {
    fn from(err: ContextError) -> Self {
        match err {
            ContextError::Initialization(msg) => SquirrelError::Context(format!("Initialization error: {msg}")),
            ContextError::Lifecycle(msg) => SquirrelError::Context(format!("Lifecycle error: {msg}")),
            ContextError::Data(msg) => SquirrelError::Context(format!("Data error: {msg}")),
            ContextError::Metadata(msg) => SquirrelError::Context(format!("Metadata error: {msg}")),
            ContextError::Other(e) => SquirrelError::Context(e.to_string()),
            ContextError::AlreadyInitialized => SquirrelError::Context("Context already initialized".to_string()),
            ContextError::NotInitialized => SquirrelError::Context("Context not initialized".to_string()),
            ContextError::AlreadyShuttingDown => SquirrelError::Context("Context already shutting down".to_string()),
            ContextError::ContextExists(id) => SquirrelError::Context(format!("Context with id {id} already exists")),
            ContextError::ContextNotFound(id) => SquirrelError::Context(format!("Context with id {id} not found")),
            ContextError::InvalidState(msg) => SquirrelError::Context(format!("Invalid context state: {msg}")),
        }
    }
}

type Result<T> = std::result::Result<T, ContextError>;

#[derive(Debug)]
pub struct ContextManager {
    pub metrics: Arc<Metrics>,
    pub contexts: Arc<RwLock<HashMap<String, Arc<Context>>>>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Metrics::new()),
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_context(&self, id: String) -> Result<Arc<Context>> {
        let mut contexts = self.contexts.write().await;
        if contexts.contains_key(&id) {
            return Err(ContextError::ContextExists(id));
        }
        let config = ContextConfig {
            id: id.clone(),
            ..ContextConfig::default()
        };
        let context = Arc::new(Context::new(config)?);
        contexts.insert(id, context.clone());
        Ok(context)
    }

    pub async fn get_context(&self, id: &str) -> Result<Arc<Context>> {
        let contexts = self.contexts.read().await;
        contexts.get(id).cloned().ok_or_else(|| ContextError::ContextNotFound(id.to_string()))
    }

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

#[derive(Debug, Clone)]
pub struct ContextTracker {
    manager: Arc<ContextManager>,
    active_context: Arc<RwLock<Option<String>>>,
}

impl ContextTracker {
    pub fn new(manager: Arc<ContextManager>) -> Self {
        Self {
            manager,
            active_context: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn activate_context(&self, id: &str) -> Result<()> {
        self.manager.get_context(id).await?;
        let mut active = self.active_context.write().await;
        *active = Some(id.to_string());
        Ok(())
    }

    pub async fn deactivate_context(&self) -> Result<()> {
        let mut active = self.active_context.write().await;
        *active = None;
        Ok(())
    }

    pub async fn get_active_context(&self) -> Result<Option<Arc<Context>>> {
        if let Some(id) = self.active_context.read().await.as_ref() {
            Ok(Some(self.manager.get_context(id).await?))
        } else {
            Ok(None)
        }
    }
}

/// Application context that stores the application configuration and components
#[derive(Debug)]
pub struct AppContext {
    config: AppConfig,
    event_emitter: Arc<DefaultEventEmitter>,
}

impl AppContext {
    pub fn new(config: AppConfig, event_emitter: Arc<DefaultEventEmitter>) -> Self {
        Self {
            config,
            event_emitter,
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn event_emitter(&self) -> Arc<DefaultEventEmitter> {
        self.event_emitter.clone()
    }
} 