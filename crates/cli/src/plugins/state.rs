//! Plugin state management
//!
//! This module defines the different states a plugin can be in
//! and provides functionality for managing state transitions.

use std::fmt;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

use crate::plugins::manager::PluginManager;

/// Represents the current state of a plugin
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// The plugin has been created but not initialized
    Created,
    
    /// The plugin has been initialized but not started
    Initialized,
    
    /// The plugin is currently running
    Started,
    
    /// The plugin has been stopped but not cleaned up
    Stopped,
    
    /// The plugin has been cleaned up but not disposed
    Cleaned,
    
    /// The plugin has been disposed
    Disposed,
}

impl PluginState {
    /// Check if a state transition is valid
    ///
    /// # Arguments
    ///
    /// * `from` - The current state
    /// * `to` - The desired state
    ///
    /// # Returns
    ///
    /// * `true` if the transition is valid
    /// * `false` otherwise
    pub fn is_valid_transition(from: PluginState, to: PluginState) -> bool {
        matches!(
            (from, to),
            (PluginState::Created, PluginState::Initialized)
                | (PluginState::Initialized, PluginState::Started)
                | (PluginState::Started, PluginState::Stopped)
                | (PluginState::Stopped, PluginState::Initialized)
                | (PluginState::Stopped, PluginState::Cleaned)
                | (PluginState::Cleaned, PluginState::Disposed)
                | (PluginState::Initialized, PluginState::Disposed)
        )
    }
}

impl fmt::Display for PluginState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginState::Created => write!(f, "Created"),
            PluginState::Initialized => write!(f, "Initialized"),
            PluginState::Started => write!(f, "Started"),
            PluginState::Stopped => write!(f, "Stopped"),
            PluginState::Cleaned => write!(f, "Cleaned"),
            PluginState::Disposed => write!(f, "Disposed"),
        }
    }
}

// Global state for plugin management
lazy_static! {
    static ref PLUGIN_MANAGER: Arc<Mutex<PluginManager>> = Arc::new(Mutex::new(PluginManager::new()));
}

/// Get the global plugin manager instance
pub fn get_plugin_manager() -> Arc<Mutex<PluginManager>> {
    Arc::clone(&PLUGIN_MANAGER)
} 