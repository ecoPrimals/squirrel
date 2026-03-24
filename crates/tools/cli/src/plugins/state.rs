// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin state management
//!
//! This module defines the different states a plugin can be in
//! and provides functionality for managing state transitions.

use std::cmp::PartialEq;
use std::fmt;
use std::sync::Arc;

use std::sync::LazyLock;
use tokio::sync::Mutex;

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
    #[allow(
        clippy::unnested_or_patterns,
        reason = "Tuple transitions read clearly as listed pairs"
    )]
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
static PLUGIN_MANAGER: LazyLock<Arc<Mutex<PluginManager>>> =
    LazyLock::new(|| Arc::new(Mutex::new(PluginManager::new())));

/// Get the global plugin manager instance
pub fn get_plugin_manager() -> Arc<Mutex<PluginManager>> {
    Arc::clone(&PLUGIN_MANAGER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_state_display_and_transitions() {
        assert_eq!(PluginState::Created.to_string(), "Created");
        assert_eq!(PluginState::Initialized.to_string(), "Initialized");
        assert_eq!(PluginState::Started.to_string(), "Started");
        assert_eq!(PluginState::Stopped.to_string(), "Stopped");
        assert_eq!(PluginState::Cleaned.to_string(), "Cleaned");
        assert_eq!(PluginState::Disposed.to_string(), "Disposed");

        assert!(PluginState::is_valid_transition(
            PluginState::Created,
            PluginState::Initialized
        ));
        assert!(PluginState::is_valid_transition(
            PluginState::Initialized,
            PluginState::Started
        ));
        assert!(PluginState::is_valid_transition(
            PluginState::Started,
            PluginState::Stopped
        ));
        assert!(!PluginState::is_valid_transition(
            PluginState::Created,
            PluginState::Started
        ));
        assert!(!PluginState::is_valid_transition(
            PluginState::Disposed,
            PluginState::Created
        ));
    }

    #[test]
    fn plugin_state_debug_clone_eq() {
        let s = PluginState::Initialized;
        let c = s;
        assert_eq!(s, c);
        let _ = format!("{s:?}");
    }

    #[test]
    fn global_manager_singleton() {
        let a = get_plugin_manager();
        let b = get_plugin_manager();
        assert!(Arc::ptr_eq(&a, &b));
    }
}
