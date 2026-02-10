// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Basic Lifecycle Hook Implementation
//!
//! This module provides a basic implementation of the ToolLifecycleHook trait
//! that tracks state history and provides fundamental lifecycle management.

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use crate::tool::{Tool, ToolError, ToolLifecycleHook, ToolState};
use super::types::StateHistoryMap;

/// Basic lifecycle hook implementation
#[derive(Debug)]
pub struct BasicLifecycleHook {
    /// History of state changes for each tool
    state_history: Arc<RwLock<StateHistoryMap>>,
    /// Maximum history entries to keep per tool
    max_history_entries: usize,
}

impl Default for BasicLifecycleHook {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicLifecycleHook {
    /// Create a new basic lifecycle hook
    #[must_use] 
    pub fn new() -> Self {
        Self {
            state_history: Arc::new(RwLock::new(StateHistoryMap::new())),
            max_history_entries: 100,
        }
    }

    /// Create a new basic lifecycle hook with custom max history entries
    #[must_use] 
    pub const fn with_max_history_entries(mut self, max_entries: usize) -> Self {
        self.max_history_entries = max_entries;
        self
    }

    /// Get the state history for a tool
    pub async fn get_state_history(
        &self,
        tool_id: &str,
    ) -> Vec<(ToolState, chrono::DateTime<Utc>)> {
        let history = self.state_history.read().await;
        history.get(tool_id).cloned().unwrap_or_default()
    }

    /// Record a state change for a tool
    async fn record_state_change(&self, tool_id: &str, state: ToolState) {
        let mut history = self.state_history.write().await;
        let tool_history = history.entry(tool_id.to_string()).or_insert_with(Vec::new);
        
        // Add the new state change
        tool_history.push((state, Utc::now()));
        
        // Trim history if it exceeds the maximum
        if tool_history.len() > self.max_history_entries {
            tool_history.drain(0..tool_history.len() - self.max_history_entries);
        }
        
        debug!(
            "Recorded state change for tool {}: {:?}",
            tool_id, state
        );
    }

    /// Attempt to recover the previous state of a tool
    async fn recover_state(&self, tool_id: &str) -> Result<(), ToolError> {
        let history = self.state_history.read().await;
        if let Some(tool_history) = history.get(tool_id) {
            if let Some((previous_state, _)) = tool_history.iter().rev().nth(1) {
                info!(
                    "Attempting to recover tool {} to previous state: {:?}",
                    tool_id, previous_state
                );
                // In a real implementation, this would actually restore the tool state
                return Ok(());
            }
        }
        
        warn!("No previous state found for tool {}", tool_id);
        Err(ToolError::StateTransition(format!(
            "No previous state available for tool {}",
            tool_id
        )))
    }

    /// Get this hook as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl ToolLifecycleHook for BasicLifecycleHook {
    #[instrument(skip(self, tool))]
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        info!("Registering tool: {}", tool.id);
        self.record_state_change(&tool.id, ToolState::Registered).await;
        
        // Validate tool configuration
        if tool.name.is_empty() {
            return Err(ToolError::InvalidConfiguration(
                "Tool name cannot be empty".to_string(),
            ));
        }
        
        debug!("Tool {} registered successfully", tool.id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Unregistering tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Unregistered).await;
        
        // Clean up history for unregistered tools
        let mut history = self.state_history.write().await;
        history.remove(tool_id);
        
        debug!("Tool {} unregistered successfully", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_activate(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Activating tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Active).await;
        debug!("Tool {} activated successfully", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Deactivating tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Inactive).await;
        debug!("Tool {} deactivated successfully", tool_id);
        Ok(())
    }

    #[instrument(skip(self, error))]
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        error!("Tool {} encountered error: {}", tool_id, error);
        self.record_state_change(tool_id, ToolState::Error).await;
        debug!("Error recorded for tool {}", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn pre_start(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Pre-start hook for tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Starting).await;
        debug!("Pre-start completed for tool {}", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_start(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Post-start hook for tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Running).await;
        debug!("Post-start completed for tool {}", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn pre_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Pre-stop hook for tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Stopping).await;
        debug!("Pre-stop completed for tool {}", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Post-stop hook for tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Stopped).await;
        debug!("Post-stop completed for tool {}", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_pause(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Pausing tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Paused).await;
        debug!("Tool {} paused successfully", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_resume(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Resuming tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Running).await;
        debug!("Tool {} resumed successfully", tool_id);
        Ok(())
    }

    #[instrument(skip(self, tool))]
    async fn on_update(&self, tool: &Tool) -> Result<(), ToolError> {
        info!("Updating tool: {}", tool.id);
        self.record_state_change(&tool.id, ToolState::Updating).await;
        debug!("Tool {} updated successfully", tool.id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_cleanup(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Cleaning up tool: {}", tool_id);
        debug!("Tool {} cleaned up successfully", tool_id);
        Ok(())
    }

    async fn register_tool(&self, tool: &Tool) -> Result<(), ToolError> {
        self.on_register(tool).await
    }

    async fn initialize_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Initializing tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Initializing).await;
        Ok(())
    }

    async fn pre_execute(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Pre-execute hook for tool: {}", tool_id);
        Ok(())
    }

    async fn post_execute(
        &self,
        tool_id: &str,
        result: Result<(), ToolError>,
    ) -> Result<(), ToolError> {
        match result {
            Ok(()) => {
                debug!("Tool {} executed successfully", tool_id);
            }
            Err(ref error) => {
                warn!("Tool {} execution failed: {}", tool_id, error);
                self.record_state_change(tool_id, ToolState::Error).await;
            }
        }
        Ok(())
    }

    async fn reset_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Resetting tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Resetting).await;
        Ok(())
    }

    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        self.on_cleanup(tool_id).await
    }

    /// Get this hook as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
} 