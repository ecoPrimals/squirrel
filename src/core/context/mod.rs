//! Context management for the Squirrel project
//!
//! This module provides the context management functionality, which is central
//! to the operation of the Squirrel system. The context maintains the state
//! and provides access to various system components.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

use crate::core::error::{Error, ContextError};
use crate::core::events::Event;
use crate::core::metrics::Metrics;

/// Configuration for the context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Whether to enable metrics collection
    pub enable_metrics: bool,
    /// Whether to enable event logging
    pub enable_events: bool,
    /// Maximum number of events to store
    pub max_events: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_events: true,
            max_events: 1000,
        }
    }
}

/// The main context type that holds system state
pub struct Context {
    /// Configuration for the context
    config: ContextConfig,
    /// Metrics collector
    metrics: Arc<RwLock<Metrics>>,
    /// Event store
    events: Arc<RwLock<Vec<Event>>>,
    /// State store
    state: Arc<RwLock<ContextState>>,
}

/// The state managed by the context
#[derive(Debug, Default)]
pub struct ContextState {
    /// Whether the context is initialized
    pub initialized: bool,
    /// Whether the context is shutting down
    pub shutting_down: bool,
    /// Current lifecycle stage
    pub lifecycle_stage: LifecycleStage,
}

/// The lifecycle stages of the context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleStage {
    /// Initial stage
    Initial,
    /// Starting up
    Starting,
    /// Running
    Running,
    /// Shutting down
    ShuttingDown,
    /// Shut down
    ShutDown,
}

impl Context {
    /// Create a new context with default configuration
    pub async fn new() -> Result<Self, Error> {
        Self::with_config(ContextConfig::default()).await
    }

    /// Create a new context with the specified configuration
    pub async fn with_config(config: ContextConfig) -> Result<Self, Error> {
        let context = Self {
            config,
            metrics: Arc::new(RwLock::new(Metrics::default())),
            events: Arc::new(RwLock::new(Vec::new())),
            state: Arc::new(RwLock::new(ContextState::default())),
        };

        // Initialize the context
        context.initialize().await?;

        Ok(context)
    }

    /// Initialize the context
    async fn initialize(&self) -> Result<(), Error> {
        let mut state = self.state.write().await;
        
        if state.initialized {
            return Err(ContextError::Initialization("Context already initialized".to_string()).into());
        }

        // Set up initial state
        state.initialized = true;
        state.lifecycle_stage = LifecycleStage::Starting;

        // Initialize metrics if enabled
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.initialize()?;
        }

        // Update lifecycle stage
        state.lifecycle_stage = LifecycleStage::Running;

        Ok(())
    }

    /// Shutdown the context
    pub async fn shutdown(&self) -> Result<(), Error> {
        let mut state = self.state.write().await;
        
        if !state.initialized {
            return Err(ContextError::Shutdown("Context not initialized".to_string()).into());
        }

        if state.shutting_down {
            return Err(ContextError::Shutdown("Context already shutting down".to_string()).into());
        }

        // Update state
        state.shutting_down = true;
        state.lifecycle_stage = LifecycleStage::ShuttingDown;

        // Shutdown metrics if enabled
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.shutdown()?;
        }

        // Clear events
        let mut events = self.events.write().await;
        events.clear();

        // Update final state
        state.lifecycle_stage = LifecycleStage::ShutDown;

        Ok(())
    }

    /// Get the current lifecycle stage
    pub async fn lifecycle_stage(&self) -> LifecycleStage {
        let state = self.state.read().await;
        state.lifecycle_stage
    }

    /// Get the metrics collector
    pub fn metrics(&self) -> Arc<RwLock<Metrics>> {
        self.metrics.clone()
    }

    /// Get the events store
    pub fn events(&self) -> Arc<RwLock<Vec<Event>>> {
        self.events.clone()
    }

    /// Get the state store
    pub fn state(&self) -> Arc<RwLock<ContextState>> {
        self.state.clone()
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
    pub fn enable_metrics(mut self, enable: bool) -> Self {
        self.config.enable_metrics = enable;
        self
    }

    /// Set whether to enable events
    pub fn enable_events(mut self, enable: bool) -> Self {
        self.config.enable_events = enable;
        self
    }

    /// Set the maximum number of events to store
    pub fn max_events(mut self, max: usize) -> Self {
        self.config.max_events = max;
        self
    }

    /// Build the context
    pub async fn build(self) -> Result<Context, Error> {
        Context::with_config(self.config).await
    }
} 