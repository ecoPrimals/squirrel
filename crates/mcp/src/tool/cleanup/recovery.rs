//! Tool recovery hook implementation for error handling and recovery strategies

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{error, info, warn, instrument};
use chrono::{DateTime, Utc};

use crate::tool::{Tool, ToolError, ToolState, ToolLifecycleHook};

/// Type alias for tool error history entry
pub type ErrorHistoryEntry = (DateTime<Utc>, ToolError);

/// Type alias for tool error history map
pub type ErrorHistoryMap = HashMap<String, Vec<ErrorHistoryEntry>>;

/// Recovery strategy type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry,
    /// Reset the tool to registered state
    Reset,
    /// Restart the tool by deactivating and reactivating
    Restart,
    /// Isolate the tool by deactivating and keeping it inactive
    Isolate,
    /// Unregister the tool completely
    Unregister,
}

impl fmt::Display for RecoveryStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retry => write!(f, "retry"),
            Self::Reset => write!(f, "reset"),
            Self::Restart => write!(f, "restart"),
            Self::Isolate => write!(f, "isolate"),
            Self::Unregister => write!(f, "unregister"),
        }
    }
}

/// Recovery attempt record
#[derive(Debug, Clone)]
struct RecoveryAttempt {
    /// Strategy used
    strategy: RecoveryStrategy,
    /// Timestamp of the attempt
    timestamp: DateTime<Utc>,
    /// Whether the attempt was successful
    successful: bool,
}

/// Hook for tool error recovery
#[derive(Debug)]
pub struct RecoveryHook {
    /// Tool error history
    error_history: RwLock<ErrorHistoryMap>,
    /// Recovery attempts history
    recovery_history: RwLock<HashMap<String, Vec<RecoveryAttempt>>>,
    /// Maximum number of recovery attempts before unregistering
    max_recovery_attempts: usize,
    /// Retry interval (milliseconds)
    retry_interval_ms: u64,
    /// Penalty timeout for failed recovery (milliseconds)
    penalty_timeout_ms: u64,
}

impl Default for RecoveryHook {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryHook {
    /// Creates a new recovery hook
    pub fn new() -> Self {
        Self {
            error_history: RwLock::new(HashMap::new()),
            recovery_history: RwLock::new(HashMap::new()),
            max_recovery_attempts: 3,
            retry_interval_ms: 1000,
            penalty_timeout_ms: 5000,
        }
    }

    /// Sets the maximum number of recovery attempts
    pub fn with_max_recovery_attempts(mut self, max_attempts: usize) -> Self {
        self.max_recovery_attempts = max_attempts;
        self
    }
    
    /// Sets the retry interval in milliseconds
    pub fn with_retry_interval(mut self, interval_ms: u64) -> Self {
        self.retry_interval_ms = interval_ms;
        self
    }
    
    /// Sets the penalty timeout in milliseconds
    pub fn with_penalty_timeout(mut self, timeout_ms: u64) -> Self {
        self.penalty_timeout_ms = timeout_ms;
        self
    }
    
    /// Records an error for a tool
    async fn record_error(&self, tool_id: &str, error: &ToolError) {
        let mut history = self.error_history.write().await;
        let tool_history = history.entry(tool_id.to_string()).or_insert_with(Vec::new);
        tool_history.push((Utc::now(), error.clone()));
        
        // Keep only the last 10 errors
        if tool_history.len() > 10 {
            tool_history.remove(0);
        }
    }
    
    /// Records a recovery attempt
    async fn record_recovery_attempt(&self, tool_id: &str, strategy: RecoveryStrategy, successful: bool) {
        let mut history = self.recovery_history.write().await;
        let tool_history = history.entry(tool_id.to_string()).or_insert_with(Vec::new);
        tool_history.push(RecoveryAttempt {
            strategy,
            timestamp: Utc::now(),
            successful,
        });
        
        // Keep only the last 10 attempts
        if tool_history.len() > 10 {
            tool_history.remove(0);
        }
    }
    
    /// Gets the recovery strategy based on error history
    async fn get_recovery_strategy(&self, tool_id: &str) -> RecoveryStrategy {
        let history = self.recovery_history.read().await;
        
        // Get the tool's recovery history
        let attempts = match history.get(tool_id) {
            Some(attempts) => attempts,
            None => return RecoveryStrategy::Retry, // No history, try a simple retry
        };
        
        // Count recent failures (last hour)
        let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
        let recent_failures = attempts.iter()
            .filter(|a| !a.successful && a.timestamp > one_hour_ago)
            .count();
        
        // Count consecutive failures
        let mut consecutive_failures = 0;
        for attempt in attempts.iter().rev() {
            if !attempt.successful {
                consecutive_failures += 1;
            } else {
                break;
            }
        }
        
        // Determine the strategy based on failure patterns
        if consecutive_failures >= self.max_recovery_attempts {
            // Too many consecutive failures, unregister the tool
            RecoveryStrategy::Unregister
        } else if consecutive_failures >= 2 {
            // Multiple consecutive failures, try restart
            RecoveryStrategy::Restart
        } else if recent_failures >= 5 {
            // Many recent failures, isolate the tool
            RecoveryStrategy::Isolate
        } else if consecutive_failures == 1 {
            // Single failure, try reset
            RecoveryStrategy::Reset
        } else {
            // Default strategy
            RecoveryStrategy::Retry
        }
    }
    
    /// Applies a recovery strategy
    #[instrument(skip(self))]
    pub async fn apply_recovery_strategy(
        &self,
        tool_id: &str,
        strategy: RecoveryStrategy,
        tool_manager: &crate::tool::ToolManager,
    ) -> Result<bool, ToolError> {
        match strategy {
            RecoveryStrategy::Retry => {
                // Simply wait and let the system retry naturally
                tokio::time::sleep(Duration::from_millis(self.retry_interval_ms)).await;
                self.record_recovery_attempt(tool_id, strategy, true).await;
                Ok(true)
            },
            RecoveryStrategy::Reset => {
                // Reset the tool to registered state
                tool_manager.update_tool_state(tool_id, ToolState::Registered).await?;
                self.record_recovery_attempt(tool_id, strategy, true).await;
                Ok(true)
            },
            RecoveryStrategy::Restart => {
                // Deactivate and reactivate the tool
                let result = tool_manager.deactivate_tool(tool_id).await;
                if result.is_ok() {
                    // Wait before reactivating
                    tokio::time::sleep(Duration::from_millis(self.retry_interval_ms)).await;
                    let activate_result = tool_manager.activate_tool(tool_id).await;
                    let success = activate_result.is_ok();
                    self.record_recovery_attempt(tool_id, strategy, success).await;
                    if success {
                        Ok(true)
                    } else {
                        warn!("Failed to restart tool {}: {:?}", tool_id, activate_result);
                        Ok(false)
                    }
                } else {
                    self.record_recovery_attempt(tool_id, strategy, false).await;
                    warn!("Failed to deactivate tool {} during restart: {:?}", tool_id, result);
                    Ok(false)
                }
            },
            RecoveryStrategy::Isolate => {
                // Deactivate the tool and keep it inactive
                let result = tool_manager.deactivate_tool(tool_id).await;
                let success = result.is_ok();
                self.record_recovery_attempt(tool_id, strategy, success).await;
                
                if success {
                    info!("Tool {} has been isolated due to errors", tool_id);
                    Ok(true)
                } else {
                    warn!("Failed to isolate tool {}: {:?}", tool_id, result);
                    Ok(false)
                }
            },
            RecoveryStrategy::Unregister => {
                // Unregister the tool completely
                let result = tool_manager.unregister_tool(tool_id).await;
                let success = result.is_ok();
                self.record_recovery_attempt(tool_id, strategy, success).await;
                
                if success {
                    error!("Tool {} has been unregistered due to persistent errors", tool_id);
                    Ok(true)
                } else {
                    error!("Failed to unregister tool {}: {:?}", tool_id, result);
                    Ok(false)
                }
            },
        }
    }
    
    /// Gets recovery success rate for a tool
    pub async fn get_success_rate(&self, tool_id: &str) -> f64 {
        let history = self.recovery_history.read().await;
        
        // Get the tool's recovery history
        let attempts = match history.get(tool_id) {
            Some(attempts) => attempts,
            None => return 1.0, // No history, assume 100% success
        };
        
        if attempts.is_empty() {
            return 1.0;
        }
        
        // Calculate success rate
        let successful_attempts = attempts.iter().filter(|a| a.successful).count();
        successful_attempts as f64 / attempts.len() as f64
    }
}

#[async_trait]
impl ToolLifecycleHook for RecoveryHook {
    /// Called when a tool is registered
    async fn on_register(&self, _tool: &Tool) -> Result<(), ToolError> {
        // Nothing to do on registration
        Ok(())
    }
    
    /// Called when a tool is unregistered
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up error and recovery history
        let mut error_history = self.error_history.write().await;
        error_history.remove(tool_id);
        
        let mut recovery_history = self.recovery_history.write().await;
        recovery_history.remove(tool_id);
        
        Ok(())
    }
    
    /// Called when a tool is activated
    async fn on_activate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Nothing to do on activation
        Ok(())
    }
    
    /// Called when a tool is deactivated
    async fn on_deactivate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // Nothing to do on deactivation
        Ok(())
    }
    
    /// Called when a tool encounters an error
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        // Record the error
        self.record_error(tool_id, error).await;
        
        // The actual recovery happens outside this hook in the ToolManager
        // We just record the error here
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_recovery_strategy_selection() {
        let hook = RecoveryHook::new();
        let tool_id = "test-tool";
        
        // Initial strategy should be Retry (no history)
        let strategy = hook.get_recovery_strategy(tool_id).await;
        assert_eq!(strategy, RecoveryStrategy::Retry);
        
        // Record a failed attempt with Retry strategy
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Retry, false).await;
        
        // Next strategy should be Reset (1 consecutive failure)
        let strategy = hook.get_recovery_strategy(tool_id).await;
        assert_eq!(strategy, RecoveryStrategy::Reset);
        
        // Record another failed attempt
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Reset, false).await;
        
        // Next strategy should be Restart (2 consecutive failures)
        let strategy = hook.get_recovery_strategy(tool_id).await;
        assert_eq!(strategy, RecoveryStrategy::Restart);
        
        // Record a successful attempt
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Restart, true).await;
        
        // Next strategy should be Retry (reset consecutive failures)
        let strategy = hook.get_recovery_strategy(tool_id).await;
        assert_eq!(strategy, RecoveryStrategy::Retry);
    }
    
    #[tokio::test]
    async fn test_success_rate_calculation() {
        let hook = RecoveryHook::new();
        let tool_id = "test-tool";
        
        // Initial success rate should be 100% (no history)
        let rate = hook.get_success_rate(tool_id).await;
        assert_eq!(rate, 1.0);
        
        // Record 3 attempts: 2 successful, 1 failed
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Retry, true).await;
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Reset, false).await;
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Restart, true).await;
        
        // Success rate should be 2/3 = 0.6666...
        let rate = hook.get_success_rate(tool_id).await;
        assert!((rate - 0.6666).abs() < 0.001);
    }
} 