//! Tool recovery hook implementation for error handling and recovery strategies

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
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
    /// Reset the tool to its initial state and try again
    Reset,
    
    /// Terminate the tool and clean up all resources
    Terminate,
    
    /// Ignore the error and continue execution
    Continue,
}

impl fmt::Display for RecoveryStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryStrategy::Reset => write!(f, "Reset"),
            RecoveryStrategy::Terminate => write!(f, "Terminate"),
            RecoveryStrategy::Continue => write!(f, "Continue"),
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
    /// Maps tool IDs to their recovery strategies
    strategies: Mutex<HashMap<String, RecoveryStrategy>>,
    
    /// Default recovery strategy for tools without a specific strategy
    default_strategy: RecoveryStrategy,
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
            strategies: Mutex::new(HashMap::new()),
            default_strategy: RecoveryStrategy::Reset,
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
            None => return RecoveryStrategy::Reset, // No history, try a simple retry
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
            RecoveryStrategy::Terminate
        } else if consecutive_failures >= 2 {
            // Multiple consecutive failures, try restart
            RecoveryStrategy::Continue
        } else if recent_failures >= 5 {
            // Many recent failures, isolate the tool
            RecoveryStrategy::Terminate
        } else if consecutive_failures == 1 {
            // Single failure, try reset
            RecoveryStrategy::Reset
        } else {
            // Default strategy
            RecoveryStrategy::Continue
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
            RecoveryStrategy::Reset => {
                // Reset the tool to registered state
                tool_manager.update_tool_state(tool_id, ToolState::Registered).await?;
                self.record_recovery_attempt(tool_id, strategy, true).await;
                Ok(true)
            },
            RecoveryStrategy::Terminate => {
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
            RecoveryStrategy::Continue => {
                // Simply wait and let the system retry naturally
                tokio::time::sleep(Duration::from_millis(self.retry_interval_ms)).await;
                self.record_recovery_attempt(tool_id, strategy, true).await;
                Ok(true)
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

    /// Set the recovery strategy for a specific tool
    pub fn set_strategy(&self, tool_id: &str, strategy: RecoveryStrategy) {
        let mut strategies = self.strategies.lock().unwrap();
        strategies.insert(tool_id.to_string(), strategy);
        info!("Set recovery strategy for tool '{}' to {}", tool_id, strategy);
    }
    
    /// Get the recovery strategy for a specific tool
    pub fn get_strategy(&self, tool_id: &str) -> RecoveryStrategy {
        let strategies = self.strategies.lock().unwrap();
        strategies.get(tool_id)
            .copied()
            .unwrap_or(self.default_strategy)
    }
}

#[async_trait]
impl ToolLifecycleHook for RecoveryHook {
    #[instrument(skip(self, _tool))]
    async fn on_register(&self, _tool: &Tool) -> Result<(), ToolError> {
        // No recovery actions needed on register
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up history when a tool is unregistered
        {
            let mut error_history = self.error_history.write().await;
            error_history.remove(tool_id);
        }
        
        {
            let mut recovery_history = self.recovery_history.write().await;
            recovery_history.remove(tool_id);
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_activate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // No recovery actions needed on activate
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn on_deactivate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // No recovery actions needed on deactivate
        Ok(())
    }
    
    #[instrument(skip(self, error))]
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        // Record the error
        self.record_error(tool_id, error).await;
        
        // Determine the recovery strategy
        let strategy = self.get_recovery_strategy(tool_id).await;
        
        info!(
            "Selected recovery strategy for tool {}: {}",
            tool_id, strategy
        );
        
        // Recovery will be handled externally
        Ok(())
    }

    #[instrument(skip(self))]
    async fn pre_start(&self, tool_id: &str) -> Result<(), ToolError> {
        // Check error history before starting
        let error_count = {
            let history = self.error_history.read().await;
            history.get(tool_id).map_or(0, |errors| errors.len())
        };

        // If too many errors, don't allow starting
        if error_count > self.max_recovery_attempts {
            return Err(ToolError::TooManyErrors(format!(
                "Tool {} has too many errors ({}), refusing to start", 
                tool_id, error_count
            )));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_start(&self, tool_id: &str) -> Result<(), ToolError> {
        // Record successful start as a positive recovery sign
        if let Some(history) = self.recovery_history.write().await.get_mut(tool_id) {
            // If there were recovery attempts, record this as a successful outcome
            if !history.is_empty() {
                let last_strategy = history.last().map(|attempt| attempt.strategy).unwrap_or(RecoveryStrategy::Reset);
                self.record_recovery_attempt(tool_id, last_strategy, true).await;
                
                info!("Tool {} started successfully after recovery", tool_id);
            }
        }
        
        Ok(())
    }

    #[instrument(skip(self))]
    async fn pre_stop(&self, _tool_id: &str) -> Result<(), ToolError> {
        // No recovery actions needed before stop
        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        // Record clean stop in recovery history
        let has_errors = {
            let history = self.error_history.read().await;
            history.get(tool_id).is_some_and(|errors| !errors.is_empty())
        };

        // If there were errors but tool stopped cleanly, consider it recovered
        if has_errors {
            info!("Tool {} stopped cleanly after previous errors", tool_id);
            
            // Add a successful recovery entry
            self.record_recovery_attempt(tool_id, RecoveryStrategy::Reset, true).await;
        }
        
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_pause(&self, _tool_id: &str) -> Result<(), ToolError> {
        // No recovery actions needed on pause
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_resume(&self, tool_id: &str) -> Result<(), ToolError> {
        // Similar check as pre_start
        let error_count = {
            let history = self.error_history.read().await;
            history.get(tool_id).map_or(0, |errors| errors.len())
        };

        // If too many errors, don't allow resuming
        if error_count > self.max_recovery_attempts {
            return Err(ToolError::TooManyErrors(format!(
                "Tool {} has too many errors ({}), refusing to resume", 
                tool_id, error_count
            )));
        }

        Ok(())
    }

    #[instrument(skip(self, _tool))]
    async fn on_update(&self, _tool: &Tool) -> Result<(), ToolError> {
        // No recovery actions needed on update
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_cleanup(&self, tool_id: &str) -> Result<(), ToolError> {
        // When cleaning up, check if there were unresolved errors
        let error_count = {
            let history = self.error_history.read().await;
            history.get(tool_id).map_or(0, |errors| errors.len())
        };

        if error_count > 0 {
            warn!("Tool {} had {} unresolved errors during cleanup", tool_id, error_count);
            
            // Record this as a recovery attempt (neutral outcome)
            self.record_recovery_attempt(tool_id, RecoveryStrategy::Terminate, true).await;
            
            // Clean up history
            {
                let mut error_history = self.error_history.write().await;
                error_history.remove(tool_id);
            }
        }
        
        Ok(())
    }

    async fn register_tool(&self, tool: &Tool) -> Result<(), ToolError> {
        let tool_id = &tool.id;
        info!("Registering tool '{}' with RecoveryHook", tool_id);
        // By default, use the default strategy (no explicit entry needed)
        Ok(())
    }
    
    async fn initialize_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Initializing tool '{}' in RecoveryHook", tool_id);
        Ok(())
    }
    
    async fn pre_execute(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Pre-execute for tool '{}' in RecoveryHook", tool_id);
        Ok(())
    }
    
    async fn post_execute(&self, tool_id: &str, result: Result<(), ToolError>) -> Result<(), ToolError> {
        match result {
            Ok(_) => {
                info!("Tool '{}' executed successfully, no recovery needed", tool_id);
                Ok(())
            }
            Err(err) => {
                // Handle the error according to the strategy
                let strategy = self.get_strategy(tool_id);
                match strategy {
                    RecoveryStrategy::Continue => {
                        info!("Continuing execution for tool '{}' despite error", tool_id);
                        Ok(())
                    }
                    RecoveryStrategy::Reset => {
                        // The actual reset will be handled by the ToolManager
                        // Here we just signal that a reset is needed
                        Err(ToolError::NeedsReset(tool_id.to_string()))
                    }
                    RecoveryStrategy::Terminate => {
                        // Signal that the tool should be terminated
                        Err(ToolError::ExecutionFailed {
                            tool_id: tool_id.to_string(),
                            reason: format!("Tool terminated due to error: {}", err),
                        })
                    }
                }
            }
        }
    }
    
    async fn reset_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Resetting tool '{}' in RecoveryHook", tool_id);
        // No specific action needed for reset in this hook
        Ok(())
    }
    
    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Cleaning up tool '{}' in RecoveryHook", tool_id);
        let mut strategies = self.strategies.lock().unwrap();
        strategies.remove(tool_id);
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
        
        // Initial strategy should be Reset (no history)
        let strategy = hook.get_recovery_strategy(tool_id).await;
        assert_eq!(strategy, RecoveryStrategy::Reset);
        
        // Record a failed attempt with Reset strategy
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Reset, false).await;
        
        // Next strategy should be Reset again (strategy is manually set, not automatically changed)
        let strategy = hook.get_recovery_strategy(tool_id).await;
        assert_eq!(strategy, RecoveryStrategy::Reset);
        
        // Manually set the strategy to Terminate
        hook.set_strategy(tool_id, RecoveryStrategy::Terminate);
        
        // Verify the strategy was set correctly using get_strategy which directly accesses the strategies map
        let strategy = hook.get_strategy(tool_id);
        assert_eq!(strategy, RecoveryStrategy::Terminate);
        
        // Record a successful attempt with Terminate strategy
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Terminate, true).await;
        
        // Strategy should remain Terminate when accessed directly
        let strategy = hook.get_strategy(tool_id);
        assert_eq!(strategy, RecoveryStrategy::Terminate);
        
        // Reset the strategy manually
        hook.set_strategy(tool_id, RecoveryStrategy::Reset);
        
        // Verify the reset worked
        let strategy = hook.get_strategy(tool_id);
        assert_eq!(strategy, RecoveryStrategy::Reset);
    }
    
    #[tokio::test]
    async fn test_success_rate_calculation() {
        let hook = RecoveryHook::new();
        let tool_id = "test-tool";
        
        // Initial success rate should be 100% (no history)
        let rate = hook.get_success_rate(tool_id).await;
        assert_eq!(rate, 1.0);
        
        // Record 3 attempts: 2 successful, 1 failed
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Reset, true).await;
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Terminate, false).await;
        hook.record_recovery_attempt(tool_id, RecoveryStrategy::Continue, true).await;
        
        // Success rate should be 2/3 = 0.6666...
        let rate = hook.get_success_rate(tool_id).await;
        assert!((rate - 0.6666).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_recovery_hook_strategies() {
        let hook = RecoveryHook::new();
        
        // Test default strategy
        assert_eq!(hook.get_strategy("unknown-tool"), RecoveryStrategy::Reset);
        
        // Test setting and getting a strategy
        hook.set_strategy("test-tool", RecoveryStrategy::Terminate);
        assert_eq!(hook.get_strategy("test-tool"), RecoveryStrategy::Terminate);
        
        // Test cleaning up removes the strategy
        hook.cleanup_tool("test-tool").await.unwrap();
        assert_eq!(hook.get_strategy("test-tool"), RecoveryStrategy::Reset);
    }
    
    #[tokio::test]
    async fn test_recovery_hook_error_handling() {
        let hook = RecoveryHook::new();
        let tool = Tool::builder()
            .id("test-tool")
            .name("Test Tool")
            .build();
        
        // Register the tool
        hook.register_tool(&tool).await.unwrap();
        
        // Test Continue strategy
        hook.set_strategy("test-tool", RecoveryStrategy::Continue);
        let result = hook.post_execute(
            "test-tool",
            Err(ToolError::InvalidState(
                format!("Tool 'test-tool' is not in the expected state: expected Registered, found Active")
            ))
        ).await;
        assert!(result.is_ok());
        
        // Test Reset strategy
        hook.set_strategy("test-tool", RecoveryStrategy::Reset);
        let result = hook.post_execute(
            "test-tool",
            Err(ToolError::InvalidState(
                format!("Tool 'test-tool' is not in the expected state: expected Registered, found Active")
            ))
        ).await;
        match result {
            Err(ToolError::NeedsReset(id)) => {
                assert_eq!(id, "test-tool");
            }
            _ => panic!("Expected NeedsReset error"),
        }
        
        // Test Terminate strategy
        hook.set_strategy("test-tool", RecoveryStrategy::Terminate);
        let result = hook.post_execute(
            "test-tool",
            Err(ToolError::InvalidState(
                format!("Tool 'test-tool' is not in the expected state: expected Registered, found Active")
            ))
        ).await;
        match result {
            Err(ToolError::ExecutionFailed { tool_id, .. }) => {
                assert_eq!(tool_id, "test-tool");
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }
} 