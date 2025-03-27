// Plugin Lifecycle Module
//
// This module provides functionality for managing the lifecycle of plugins.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::plugins::errors::PluginError;
use crate::plugins::Result;
use squirrel_mcp::plugins::interfaces::{Plugin, PluginStatus};
use squirrel_mcp::plugins::lifecycle::PluginLifecycleHook;

/// Plugin lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    /// Plugin has been created
    Created,
    
    /// Plugin is being loaded
    Loading,
    
    /// Plugin has been loaded
    Loaded,
    
    /// Plugin is being initialized
    Initializing,
    
    /// Plugin has been initialized
    Initialized,
    
    /// Plugin is being started
    Starting,
    
    /// Plugin has been started
    Started,
    
    /// Plugin is being stopped
    Stopping,
    
    /// Plugin has been stopped
    Stopped,
    
    /// Plugin is being unloaded
    Unloading,
    
    /// Plugin has been unloaded
    Unloaded,
    
    /// Plugin has failed
    Failed,
}

/// Plugin lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// Previous state
    pub previous_state: LifecycleState,
    
    /// Current state
    pub current_state: LifecycleState,
    
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Event message
    pub message: Option<String>,
}

impl LifecycleEvent {
    /// Create a new lifecycle event
    pub fn new(
        plugin_id: Uuid,
        previous_state: LifecycleState,
        current_state: LifecycleState,
        message: Option<String>,
    ) -> Self {
        Self {
            plugin_id,
            previous_state,
            current_state,
            timestamp: chrono::Utc::now(),
            message,
        }
    }
}

/// Plugin lifecycle manager
///
/// This trait defines the interface for managing the lifecycle of plugins.
#[async_trait]
pub trait PluginLifecycle: Send + Sync + Debug {
    /// Get the current state of a plugin
    async fn get_state(&self, plugin_id: Uuid) -> Result<LifecycleState>;
    
    /// Transition a plugin to a new state
    async fn transition(&self, plugin_id: Uuid, state: LifecycleState, message: Option<String>) -> Result<LifecycleEvent>;
    
    /// Get the lifecycle history of a plugin
    async fn get_history(&self, plugin_id: Uuid) -> Result<Vec<LifecycleEvent>>;
    
    /// Check if a transition is valid
    fn is_valid_transition(&self, from: LifecycleState, to: LifecycleState) -> bool;
    
    /// Initialize a plugin
    async fn initialize_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;
    
    /// Start a plugin
    async fn start_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;
    
    /// Stop a plugin
    async fn stop_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;
    
    /// Unload a plugin
    async fn unload_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;
}

/// Implementation of the PluginLifecycle trait
#[derive(Debug)]
pub struct PluginLifecycleManager {
    /// Plugin states
    states: RwLock<std::collections::HashMap<Uuid, LifecycleState>>,
    
    /// Plugin lifecycle events
    events: RwLock<std::collections::HashMap<Uuid, Vec<LifecycleEvent>>>,
    
    /// Plugin lifecycle hooks
    hooks: Vec<Arc<dyn PluginLifecycleHook>>,
}

impl PluginLifecycleManager {
    /// Create a new plugin lifecycle manager
    pub fn new() -> Self {
        Self {
            states: RwLock::new(std::collections::HashMap::new()),
            events: RwLock::new(std::collections::HashMap::new()),
            hooks: Vec::new(),
        }
    }
    
    /// Add a lifecycle hook
    pub fn add_hook(&mut self, hook: Arc<dyn PluginLifecycleHook>) {
        self.hooks.push(hook);
    }
    
    /// Execute hooks for a plugin
    async fn execute_hooks(&self, plugin: Arc<dyn Plugin>, state: LifecycleState) -> Result<()> {
        for hook in &self.hooks {
            match state {
                LifecycleState::Initializing => {
                    if let Err(e) = hook.pre_initialize(plugin.clone()).await {
                        error!("Hook pre_initialize failed: {}", e);
                        return Err(PluginError::InitializeError(e.to_string()));
                    }
                }
                LifecycleState::Initialized => {
                    if let Err(e) = hook.post_initialize(plugin.clone()).await {
                        error!("Hook post_initialize failed: {}", e);
                        return Err(PluginError::InitializeError(e.to_string()));
                    }
                }
                LifecycleState::Starting => {
                    if let Err(e) = hook.pre_start(plugin.clone()).await {
                        error!("Hook pre_start failed: {}", e);
                        return Err(PluginError::StartError(e.to_string()));
                    }
                }
                LifecycleState::Started => {
                    if let Err(e) = hook.post_start(plugin.clone()).await {
                        error!("Hook post_start failed: {}", e);
                        return Err(PluginError::StartError(e.to_string()));
                    }
                }
                LifecycleState::Stopping => {
                    if let Err(e) = hook.pre_stop(plugin.clone()).await {
                        error!("Hook pre_stop failed: {}", e);
                        return Err(PluginError::StopError(e.to_string()));
                    }
                }
                LifecycleState::Stopped => {
                    if let Err(e) = hook.post_stop(plugin.clone()).await {
                        error!("Hook post_stop failed: {}", e);
                        return Err(PluginError::StopError(e.to_string()));
                    }
                }
                LifecycleState::Unloading => {
                    if let Err(e) = hook.pre_unload(plugin.clone()).await {
                        error!("Hook pre_unload failed: {}", e);
                        return Err(PluginError::StopError(e.to_string()));
                    }
                }
                LifecycleState::Unloaded => {
                    if let Err(e) = hook.post_unload(plugin.clone()).await {
                        error!("Hook post_unload failed: {}", e);
                        return Err(PluginError::StopError(e.to_string()));
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl PluginLifecycle for PluginLifecycleManager {
    async fn get_state(&self, plugin_id: Uuid) -> Result<LifecycleState> {
        let states = self.states.read().await;
        match states.get(&plugin_id) {
            Some(state) => Ok(*state),
            None => Err(PluginError::NotFound(plugin_id)),
        }
    }
    
    async fn transition(&self, plugin_id: Uuid, state: LifecycleState, message: Option<String>) -> Result<LifecycleEvent> {
        let mut states = self.states.write().await;
        let mut events = self.events.write().await;
        
        let previous_state = states.get(&plugin_id).copied().unwrap_or(LifecycleState::Created);
        
        if !self.is_valid_transition(previous_state, state) {
            return Err(PluginError::StateError(format!(
                "Invalid transition from {:?} to {:?}",
                previous_state, state
            )));
        }
        
        states.insert(plugin_id, state);
        
        let event = LifecycleEvent::new(plugin_id, previous_state, state, message);
        
        events
            .entry(plugin_id)
            .or_insert_with(Vec::new)
            .push(event.clone());
        
        Ok(event)
    }
    
    async fn get_history(&self, plugin_id: Uuid) -> Result<Vec<LifecycleEvent>> {
        let events = self.events.read().await;
        match events.get(&plugin_id) {
            Some(history) => Ok(history.clone()),
            None => Err(PluginError::NotFound(plugin_id)),
        }
    }
    
    fn is_valid_transition(&self, from: LifecycleState, to: LifecycleState) -> bool {
        match (from, to) {
            (LifecycleState::Created, LifecycleState::Loading) => true,
            (LifecycleState::Loading, LifecycleState::Loaded) => true,
            (LifecycleState::Loading, LifecycleState::Failed) => true,
            (LifecycleState::Loaded, LifecycleState::Initializing) => true,
            (LifecycleState::Initializing, LifecycleState::Initialized) => true,
            (LifecycleState::Initializing, LifecycleState::Failed) => true,
            (LifecycleState::Initialized, LifecycleState::Starting) => true,
            (LifecycleState::Starting, LifecycleState::Started) => true,
            (LifecycleState::Starting, LifecycleState::Failed) => true,
            (LifecycleState::Started, LifecycleState::Stopping) => true,
            (LifecycleState::Stopping, LifecycleState::Stopped) => true,
            (LifecycleState::Stopping, LifecycleState::Failed) => true,
            (LifecycleState::Stopped, LifecycleState::Unloading) => true,
            (LifecycleState::Stopped, LifecycleState::Initializing) => true,
            (LifecycleState::Unloading, LifecycleState::Unloaded) => true,
            (LifecycleState::Unloading, LifecycleState::Failed) => true,
            (_, LifecycleState::Failed) => true, // Can always transition to Failed
            _ => false,
        }
    }
    
    async fn initialize_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let plugin_id = plugin.metadata().id;
        
        // Transition to initializing
        self.transition(
            plugin_id,
            LifecycleState::Initializing,
            Some(format!("Initializing plugin {}", plugin.metadata().name)),
        ).await?;
        
        // Execute pre-initialize hooks
        self.execute_hooks(plugin.clone(), LifecycleState::Initializing).await?;
        
        // Initialize the plugin
        if let Err(e) = plugin.initialize().await {
            error!("Plugin initialization failed: {}", e);
            self.transition(
                plugin_id,
                LifecycleState::Failed,
                Some(format!("Initialization failed: {}", e)),
            ).await?;
            return Err(PluginError::InitializeError(e.to_string()));
        }
        
        // Transition to initialized
        self.transition(
            plugin_id,
            LifecycleState::Initialized,
            Some(format!("Plugin {} initialized", plugin.metadata().name)),
        ).await?;
        
        // Execute post-initialize hooks
        self.execute_hooks(plugin.clone(), LifecycleState::Initialized).await?;
        
        Ok(())
    }
    
    async fn start_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let plugin_id = plugin.metadata().id;
        
        // Get current state
        let current_state = self.get_state(plugin_id).await?;
        
        // Check if we can start the plugin
        if current_state != LifecycleState::Initialized && current_state != LifecycleState::Stopped {
            return Err(PluginError::StateError(format!(
                "Cannot start plugin in state {:?}",
                current_state
            )));
        }
        
        // Transition to starting
        self.transition(
            plugin_id,
            LifecycleState::Starting,
            Some(format!("Starting plugin {}", plugin.metadata().name)),
        ).await?;
        
        // Execute pre-start hooks
        self.execute_hooks(plugin.clone(), LifecycleState::Starting).await?;
        
        // Start the plugin
        if let Err(e) = plugin.start().await {
            error!("Plugin start failed: {}", e);
            self.transition(
                plugin_id,
                LifecycleState::Failed,
                Some(format!("Start failed: {}", e)),
            ).await?;
            return Err(PluginError::StartError(e.to_string()));
        }
        
        // Transition to started
        self.transition(
            plugin_id,
            LifecycleState::Started,
            Some(format!("Plugin {} started", plugin.metadata().name)),
        ).await?;
        
        // Execute post-start hooks
        self.execute_hooks(plugin.clone(), LifecycleState::Started).await?;
        
        Ok(())
    }
    
    async fn stop_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let plugin_id = plugin.metadata().id;
        
        // Get current state
        let current_state = self.get_state(plugin_id).await?;
        
        // Check if we can stop the plugin
        if current_state != LifecycleState::Started {
            return Err(PluginError::StateError(format!(
                "Cannot stop plugin in state {:?}",
                current_state
            )));
        }
        
        // Transition to stopping
        self.transition(
            plugin_id,
            LifecycleState::Stopping,
            Some(format!("Stopping plugin {}", plugin.metadata().name)),
        ).await?;
        
        // Execute pre-stop hooks
        self.execute_hooks(plugin.clone(), LifecycleState::Stopping).await?;
        
        // Stop the plugin
        if let Err(e) = plugin.stop().await {
            error!("Plugin stop failed: {}", e);
            self.transition(
                plugin_id,
                LifecycleState::Failed,
                Some(format!("Stop failed: {}", e)),
            ).await?;
            return Err(PluginError::StopError(e.to_string()));
        }
        
        // Transition to stopped
        self.transition(
            plugin_id,
            LifecycleState::Stopped,
            Some(format!("Plugin {} stopped", plugin.metadata().name)),
        ).await?;
        
        // Execute post-stop hooks
        self.execute_hooks(plugin.clone(), LifecycleState::Stopped).await?;
        
        Ok(())
    }
    
    async fn unload_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let plugin_id = plugin.metadata().id;
        
        // Get current state
        let current_state = self.get_state(plugin_id).await?;
        
        // Check if we can unload the plugin
        if current_state != LifecycleState::Stopped {
            return Err(PluginError::StateError(format!(
                "Cannot unload plugin in state {:?}",
                current_state
            )));
        }
        
        // Transition to unloading
        self.transition(
            plugin_id,
            LifecycleState::Unloading,
            Some(format!("Unloading plugin {}", plugin.metadata().name)),
        ).await?;
        
        // Execute pre-unload hooks
        self.execute_hooks(plugin.clone(), LifecycleState::Unloading).await?;
        
        // Transition to unloaded
        self.transition(
            plugin_id,
            LifecycleState::Unloaded,
            Some(format!("Plugin {} unloaded", plugin.metadata().name)),
        ).await?;
        
        // Execute post-unload hooks
        self.execute_hooks(plugin.clone(), LifecycleState::Unloaded).await?;
        
        Ok(())
    }
} 