// Enhanced recovery strategies for tool errors
//
// This module provides improved error recovery capabilities for tools in the MCP system.
// It implements sophisticated error recovery mechanisms including:
// - Advanced backoff strategies
// - Multi-stage recovery attempts
// - Adaptive recovery based on error patterns
// - Recovery history tracking

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time;
use tracing::{debug, error, info, instrument, warn};
use std::any::Any;
use std::future::Future;
use std::pin::Pin;

use crate::tool::{Tool, ToolError, ToolLifecycleHook, ToolManager, ToolState};

/// Advanced backoff strategy for recovery attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvancedBackoffStrategy {
    /// Fixed delay between attempts (milliseconds)
    Fixed(u64),
    
    /// Exponential backoff with jitter (base milliseconds, max milliseconds, jitter factor 0.0-1.0)
    Exponential {
        base_ms: u64,
        max_ms: u64,
        jitter: f64,
    },
    
    /// Fibonacci backoff (base milliseconds, max milliseconds)
    Fibonacci {
        base_ms: u64,
        max_ms: u64,
    },
    
    /// Decorrelated jitter backoff (base milliseconds, max milliseconds)
    DecorrelatedJitter {
        base_ms: u64,
        max_ms: u64,
    },
}

impl AdvancedBackoffStrategy {
    /// Calculate the delay in milliseconds for a given attempt
    #[must_use] pub fn calculate_delay(&self, attempt: u32) -> u64 {
        use rand::Rng;
        
        match self {
            Self::Fixed(ms) => *ms,
            
            Self::Exponential { base_ms, max_ms, jitter } => {
                let mut delay = base_ms * 2u64.saturating_pow(attempt);
                delay = delay.min(*max_ms);
                
                if *jitter > 0.0 {
                    let jitter_amount = (delay as f64 * jitter) as u64;
                    if jitter_amount > 0 {
                        let mut rng = rand::thread_rng();
                        delay = delay.saturating_sub(rng.gen_range(0..jitter_amount));
                    }
                }
                
                delay
            },
            
            Self::Fibonacci { base_ms, max_ms } => {
                let mut a = 1;
                let mut b = 1;
                
                for _ in 0..attempt {
                    let next = a + b;
                    a = b;
                    b = next;
                }
                
                (base_ms * a as u64).min(*max_ms)
            },
            
            Self::DecorrelatedJitter { base_ms, max_ms } => {
                let mut rng = rand::thread_rng();
                if attempt == 0 {
                    *base_ms
                } else {
                    let prev = self.calculate_delay(attempt - 1);
                    let prev_times_three = prev.saturating_mul(3);
                    std::cmp::min(*max_ms, std::cmp::max(*base_ms, rng.gen_range(*base_ms..=prev_times_three)))
                }
            },
        }
    }
}

/// Advanced recovery action for tool errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvancedRecoveryAction {
    /// Reset the tool to initial state
    Reset,
    
    /// Restart the tool (stop and start)
    Restart,
    
    /// Recreate the tool (unregister and register)
    Recreate,
    
    /// Trigger recovery mode with specific parameters
    Recover {
        /// Recovery mode parameters
        params: HashMap<String, String>,
    },
    
    /// Wait and retry the operation
    RetryAfterDelay {
        /// Delay in milliseconds
        delay_ms: u64,
    },
    
    /// Custom action with name and parameters
    Custom {
        /// Action name
        name: String,
        /// Action parameters
        params: HashMap<String, String>,
    },
}

/// Enhanced recovery strategy for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRecoveryStrategy {
    /// Maximum number of recovery attempts
    pub max_attempts: u32,
    
    /// Backoff strategy
    pub backoff_strategy: AdvancedBackoffStrategy,
    
    /// Recovery actions to attempt in sequence
    pub recovery_actions: Vec<AdvancedRecoveryAction>,
    
    /// Whether to continue after the first successful recovery
    pub stop_on_success: bool,
    
    /// Maximum total recovery time in seconds
    pub max_recovery_time_seconds: Option<u64>,
}

impl Default for EnhancedRecoveryStrategy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_strategy: AdvancedBackoffStrategy::Exponential {
                base_ms: 1000,
                max_ms: 30000,
                jitter: 0.2,
            },
            recovery_actions: vec![
                AdvancedRecoveryAction::Reset,
                AdvancedRecoveryAction::Restart,
                AdvancedRecoveryAction::Recreate,
            ],
            stop_on_success: true,
            max_recovery_time_seconds: Some(300), // 5 minutes
        }
    }
}

/// Record of a recovery attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRecoveryAttempt {
    /// Tool ID
    pub tool_id: String,
    
    /// Recovery action attempted
    pub action: AdvancedRecoveryAction,
    
    /// Timestamp of the attempt
    pub timestamp: DateTime<Utc>,
    
    /// Whether the recovery was successful
    pub successful: bool,
    
    /// Error message if recovery failed
    pub error_message: Option<String>,
    
    /// Attempt number
    pub attempt_number: u32,
    
    /// Duration of the recovery attempt in milliseconds
    pub duration_ms: Option<u64>,
    
    /// Tool state before recovery
    pub previous_state: Option<ToolState>,
    
    /// Tool state after recovery
    pub new_state: Option<ToolState>,
}

/// Handler for custom recovery actions
pub trait EnhancedRecoveryHandler: fmt::Debug + Send + Sync {
    /// Handle a custom recovery action
    fn handle_action<'a>(
        &'a self,
        tool_id: &'a str,
        action: &'a AdvancedRecoveryAction,
        error: &'a ToolError,
        tool_manager: &'a ToolManager,
    ) -> Pin<Box<dyn Future<Output = Result<bool, ToolError>> + Send + 'a>>;
}

/// Enhanced recovery hook for handling tool errors
#[derive(Debug)]
pub struct EnhancedRecoveryHook {
    /// Default recovery strategy
    pub default_strategy: EnhancedRecoveryStrategy,
    
    /// Tool-specific recovery strategies
    strategies: RwLock<HashMap<String, EnhancedRecoveryStrategy>>,
    
    /// Custom recovery handlers
    handlers: Vec<Arc<dyn EnhancedRecoveryHandler>>,
    
    /// Recovery history
    history: RwLock<Vec<EnhancedRecoveryAttempt>>,
    
    /// Active recovery attempts (`tool_id` -> attempt count)
    active_recoveries: Mutex<HashMap<String, u32>>,
    
    /// Error patterns for each tool
    error_patterns: RwLock<HashMap<String, Vec<(DateTime<Utc>, String)>>>,
}

impl Default for EnhancedRecoveryHook {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedRecoveryHook {
    /// Creates a new enhanced recovery hook
    #[must_use] pub fn new() -> Self {
        Self {
            default_strategy: EnhancedRecoveryStrategy::default(),
            strategies: RwLock::new(HashMap::new()),
            handlers: Vec::new(),
            history: RwLock::new(Vec::new()),
            active_recoveries: Mutex::new(HashMap::new()),
            error_patterns: RwLock::new(HashMap::new()),
        }
    }
    
    /// Set the default recovery strategy
    pub fn with_default_strategy(mut self, strategy: EnhancedRecoveryStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }
    
    /// Add a custom recovery handler
    pub fn add_handler(mut self, handler: impl EnhancedRecoveryHandler + 'static) -> Self {
        self.handlers.push(Arc::new(handler));
        self
    }
    
    /// Set a tool-specific recovery strategy
    pub async fn set_strategy(&self, tool_id: &str, strategy: EnhancedRecoveryStrategy) {
        let mut strategies = self.strategies.write().await;
        strategies.insert(tool_id.to_string(), strategy);
    }
    
    /// Get the recovery strategy for a tool
    pub async fn get_strategy(&self, tool_id: &str) -> EnhancedRecoveryStrategy {
        let strategies = self.strategies.read().await;
        strategies
            .get(tool_id)
            .cloned()
            .unwrap_or_else(|| self.default_strategy.clone())
    }
    
    /// Record an error pattern for a tool
    async fn record_error_pattern(&self, tool_id: &str, error: &ToolError) {
        let mut patterns = self.error_patterns.write().await;
        let tool_patterns = patterns.entry(tool_id.to_string()).or_insert_with(Vec::new);
        
        // Keep only the last 10 errors
        if tool_patterns.len() >= 10 {
            tool_patterns.remove(0);
        }
        
        tool_patterns.push((Utc::now(), format!("{error:?}")));
    }
    
    /// Record a recovery attempt
    async fn record_attempt(&self, attempt: EnhancedRecoveryAttempt) {
        let mut history = self.history.write().await;
        
        // Keep history limited to a reasonable size
        if history.len() >= 100 {
            history.remove(0);
        }
        
        history.push(attempt);
    }
    
    /// Get recovery history for a tool
    pub async fn get_history(&self, tool_id: &str) -> Vec<EnhancedRecoveryAttempt> {
        let history = self.history.read().await;
        history
            .iter()
            .filter(|attempt| attempt.tool_id == tool_id)
            .cloned()
            .collect()
    }
    
    /// Apply a recovery action
    #[instrument(level = "debug", skip(self, tool_manager))]
    pub async fn apply_recovery_action(
        &self,
        tool_id: &str,
        action: &AdvancedRecoveryAction,
        error: &ToolError,
        tool_manager: &ToolManager,
    ) -> Result<bool, ToolError> {
        debug!("Applying recovery action {:?} for tool {}", action, tool_id);
        
        let start = std::time::Instant::now();
        let prev_state = tool_manager.get_tool_state(tool_id).await;
        
        let result = match action {
            AdvancedRecoveryAction::Reset => {
                tool_manager.reset_tool(tool_id).await.map(|()| true)
            },
            
            AdvancedRecoveryAction::Restart => {
                tool_manager.stop_tool(tool_id).await?;
                tool_manager.start_tool(tool_id).await.map(|()| true)
            },
            
            AdvancedRecoveryAction::Recreate => {
                if let Some(_tool) = tool_manager.get_tool(tool_id).await {
                    tool_manager.unregister_tool(tool_id).await?;
                    // We can't re-register the tool here since we don't have the executor
                    // This would require a callback mechanism to get a new executor
                    info!("Tool {} unregistered, needs manual re-registration", tool_id);
                    Ok(false)
                } else {
                    Err(ToolError::ToolNotFound(tool_id.to_string()))
                }
            },
            
            AdvancedRecoveryAction::Recover { params } => {
                info!("Triggering recovery mode for tool {} with params: {:?}", tool_id, params);
                tool_manager.recover_tool(tool_id).await.map(|()| true)
            },
            
            AdvancedRecoveryAction::RetryAfterDelay { delay_ms } => {
                time::sleep(std::time::Duration::from_millis(*delay_ms)).await;
                Ok(true)
            },
            
            AdvancedRecoveryAction::Custom { name, params: _ } => {
                // Try each handler until one succeeds
                for handler in &self.handlers {
                    match handler.handle_action(tool_id, action, error, tool_manager).await {
                        Ok(success) => return Ok(success),
                        Err(e) => {
                            warn!("Handler failed for custom action {}: {}", name, e);
                            // Continue to next handler
                        }
                    }
                }
                
                warn!("No handler succeeded for custom action {}", name);
                Ok(false)
            },
        };
        
        let duration = start.elapsed().as_millis() as u64;
        let new_state = tool_manager.get_tool_state(tool_id).await;
        
        // Record the attempt
        self.record_attempt(EnhancedRecoveryAttempt {
            tool_id: tool_id.to_string(),
            action: action.clone(),
            timestamp: Utc::now(),
            successful: result.is_ok(),
            error_message: result.as_ref().err().map(|e| format!("{e}")),
            attempt_number: 0, // Will be set by the caller
            duration_ms: Some(duration),
            previous_state: prev_state,
            new_state,
        })
        .await;
        
        result
    }
}

#[async_trait]
impl ToolLifecycleHook for EnhancedRecoveryHook {
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        // Initialize empty error pattern list for the tool
        let mut patterns = self.error_patterns.write().await;
        if !patterns.contains_key(&tool.id) {
            patterns.insert(tool.id.clone(), Vec::new());
        }
        Ok(())
    }
    
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        // Clean up error patterns and active recoveries
        let mut patterns = self.error_patterns.write().await;
        patterns.remove(tool_id);
        
        let mut active = self.active_recoveries.lock().await;
        active.remove(tool_id);
        
        Ok(())
    }
    
    async fn on_activate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // No specific action needed for activation
        Ok(())
    }
    
    async fn on_deactivate(&self, _tool_id: &str) -> Result<(), ToolError> {
        // No specific action needed for deactivation
        Ok(())
    }
    
    async fn on_error(&self, tool_id: &str, _error: &ToolError) -> Result<(), ToolError> {
        // Record the error pattern
        self.record_error_pattern(tool_id, _error).await;
        
        // We'll handle recovery in a separate process
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Extension to the `ToolManager` for enhanced recovery
pub trait ToolManagerRecoveryExt {
    /// Perform enhanced recovery for a tool
    fn perform_enhanced_recovery<'a>(
        &'a self,
        tool_id: &'a str,
        error: &'a ToolError,
        recovery_hook: &'a EnhancedRecoveryHook,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, ToolError>> + Send + 'a>>;
}

impl ToolManagerRecoveryExt for ToolManager {
    /// Perform enhanced recovery for a tool
    fn perform_enhanced_recovery<'a>(
        &'a self,
        tool_id: &'a str,
        error: &'a ToolError,
        recovery_hook: &'a EnhancedRecoveryHook,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            // Get the strategy for this tool
            let strategy = recovery_hook.get_strategy(tool_id).await;
            
            // Check if we're already at the maximum attempts
            let mut active = recovery_hook.active_recoveries.lock().await;
            let attempt = active.entry(tool_id.to_string()).or_insert(0);
            
            if *attempt >= strategy.max_attempts {
                error!(
                    "Tool {} has reached maximum recovery attempts ({})",
                    tool_id, strategy.max_attempts
                );
                
                // Reset the counter for next time
                *attempt = 0;
                return Err(ToolError::TooManyErrors(format!(
                    "Tool {} exceeded maximum recovery attempts ({})",
                    tool_id, strategy.max_attempts
                )));
            }
            
            // Increment the attempt counter
            *attempt += 1;
            let current_attempt = *attempt;
            drop(active);
            
            // Calculate delay based on backoff strategy
            let delay = strategy.backoff_strategy.calculate_delay(current_attempt - 1);
            
            // Wait for the calculated delay
            if delay > 0 {
                info!("Waiting {}ms before recovery attempt {}", delay, current_attempt);
                time::sleep(std::time::Duration::from_millis(delay)).await;
            }
            
            // Try each recovery action in sequence
            let mut overall_success = false;
            
            for (i, action) in strategy.recovery_actions.iter().enumerate() {
                info!(
                    "Attempting recovery action {} ({:?}) for tool {}",
                    i + 1, action, tool_id
                );
                
                match recovery_hook.apply_recovery_action(tool_id, action, error, self).await {
                    Ok(success) => {
                        if success {
                            info!(
                                "Recovery action {} ({:?}) succeeded for tool {}",
                                i + 1, action, tool_id
                            );
                            overall_success = true;
                            
                            if strategy.stop_on_success {
                                break;
                            }
                        } else {
                            warn!(
                                "Recovery action {} ({:?}) did not resolve the issue for tool {}",
                                i + 1, action, tool_id
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            "Recovery action {} ({:?}) failed for tool {}: {}",
                            i + 1, action, tool_id, e
                        );
                    }
                }
            }
            
            // Reset the attempt counter if successful
            if overall_success {
                let mut active = recovery_hook.active_recoveries.lock().await;
                active.insert(tool_id.to_string(), 0);
            }
            
            Ok(overall_success)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // We only need Tool for reference, not actually using it in the test
    
    #[tokio::test]
    async fn test_backoff_strategy_calculation() {
        // Test fixed strategy
        let fixed = AdvancedBackoffStrategy::Fixed(1000);
        assert_eq!(fixed.calculate_delay(0), 1000);
        assert_eq!(fixed.calculate_delay(1), 1000);
        assert_eq!(fixed.calculate_delay(5), 1000);
        
        // Test exponential strategy with no jitter
        let exp = AdvancedBackoffStrategy::Exponential {
            base_ms: 1000,
            max_ms: 10000,
            jitter: 0.0,
        };
        assert_eq!(exp.calculate_delay(0), 1000);
        assert_eq!(exp.calculate_delay(1), 2000);
        assert_eq!(exp.calculate_delay(2), 4000);
        assert_eq!(exp.calculate_delay(3), 8000);
        assert_eq!(exp.calculate_delay(4), 10000); // Capped at max
        
        // Test fibonacci strategy
        let fib = AdvancedBackoffStrategy::Fibonacci {
            base_ms: 1000,
            max_ms: 10000,
        };
        assert_eq!(fib.calculate_delay(0), 1000);
        assert_eq!(fib.calculate_delay(1), 1000);
        assert_eq!(fib.calculate_delay(2), 2000);
        assert_eq!(fib.calculate_delay(3), 3000);
        assert_eq!(fib.calculate_delay(4), 5000);
        assert_eq!(fib.calculate_delay(5), 8000);
        assert_eq!(fib.calculate_delay(6), 10000); // Capped at max
    }
} 