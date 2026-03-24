// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Recovery Strategies and Hooks for Tool Lifecycle Management
//!
//! This module provides recovery mechanisms for handling tool errors and failures.

use async_trait::async_trait;
use std::fmt;
use std::sync::Arc;

use crate::tool::ToolError;
use super::types::{BackoffStrategy, RecoveryAction, RecoveryAttempt, RecoveryStrategy};

/// Custom recovery handler trait
#[async_trait]
pub trait CustomRecoveryHandler: fmt::Debug + Send + Sync {
    /// Handle a custom recovery action
    async fn handle_custom_action(
        &self,
        tool_id: &str,
        action_name: &str,
        error: &ToolError,
    ) -> Result<bool, ToolError>;
}

/// Recovery hook for handling tool errors
#[derive(Debug, Clone)]
pub struct RecoveryHook {
    /// Default recovery strategy
    pub default_strategy: RecoveryStrategy,
    /// Custom recovery handlers
    pub custom_handlers: Vec<Arc<dyn CustomRecoveryHandler>>,
    /// Recovery history
    pub history: Vec<RecoveryAttempt>,
}

impl RecoveryHook {
    /// Create a new recovery hook with default settings
    #[must_use] 
    pub fn new() -> Self {
        Self {
            default_strategy: RecoveryStrategy {
                max_attempts: 3,
                backoff_strategy: BackoffStrategy::Exponential(1000),
                recovery_actions: vec![
                    RecoveryAction::Reset,
                    RecoveryAction::Restart,
                    RecoveryAction::Recreate,
                ],
            },
            custom_handlers: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Add a custom recovery handler
    pub fn add_handler(&mut self, handler: impl CustomRecoveryHandler + 'static) {
        self.custom_handlers.push(Arc::new(handler));
    }

    /// Get the recovery strategy for a specific tool
    #[must_use] 
    pub const fn get_strategy_for_tool(&self, _tool_id: &str) -> &RecoveryStrategy {
        // For now, return the default strategy for all tools
        // In the future, this could be customized per tool
        &self.default_strategy
    }

    /// Record a recovery attempt
    pub fn record_attempt(&mut self, attempt: RecoveryAttempt) {
        self.history.push(attempt);
    }

    /// Get recovery history for a specific tool
    #[must_use] 
    pub fn get_history_for_tool(&self, tool_id: &str) -> Vec<RecoveryAttempt> {
        self.history
            .iter()
            .filter(|attempt| attempt.tool_id == tool_id)
            .cloned()
            .collect()
    }

    /// Calculate backoff delay based on strategy and attempt number
    #[must_use] 
    pub fn calculate_backoff_delay(&self, strategy: &BackoffStrategy, attempt: u32) -> u64 {
        match strategy {
            BackoffStrategy::Fixed(delay) => *delay,
            BackoffStrategy::Exponential(base) => base * 2_u64.pow(attempt.saturating_sub(1)),
            BackoffStrategy::Linear(increment) => increment * u64::from(attempt),
        }
    }

    /// Check if recovery should be attempted for a given error
    #[must_use] 
    pub const fn should_attempt_recovery(&self, _tool_id: &str, error: &ToolError) -> bool {
        // For now, attempt recovery for all errors except critical ones
        // In the future, this could be more sophisticated
        match error {
            ToolError::Critical(_) => false,
            _ => true,
        }
    }

    /// Get the next recovery action to attempt for a tool
    #[must_use] 
    pub fn get_next_action(&self, tool_id: &str) -> Option<RecoveryAction> {
        let strategy = self.get_strategy_for_tool(tool_id);
        let history = self.get_history_for_tool(tool_id);
        
        // Count failed attempts
        let failed_attempts = history.iter().filter(|a| !a.successful).count();
        
        if failed_attempts >= strategy.max_attempts as usize {
            return None;
        }
        
        // Get the next action from the strategy
        strategy.recovery_actions.get(failed_attempts).cloned()
    }
}

impl Default for RecoveryHook {
    fn default() -> Self {
        Self::new()
    }
} 