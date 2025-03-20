use std::error::Error;
use std::sync::RwLock;
use crate::Command;

/// Represents the different stages in a command's lifecycle
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
#[allow(dead_code)]
pub enum LifecycleStage {
    /// Before command registration.
    Registration,
    
    /// The initialization stage, executed before any validation
    Initialization,
    
    /// The validation stage, where command parameters are validated
    Validation,
    
    /// The pre-execution stage, executed after validation but before execution
    PreExecution,
    
    /// Pre-validation stage, before command validation is performed
    PreValidation,
    
    /// The execution stage, where the command is actually executed
    Execution,
    
    /// The post-execution stage, executed after a successful execution
    PostExecution,
    
    /// Post-validation stage, after command validation is complete
    PostValidation,
    
    /// During command completion.
    Completion,
    
    /// During cleanup.
    Cleanup,
    
    /// The error handling stage, executed if an error occurs during any stage
    ErrorHandling,
}

/// Trait that defines a command lifecycle hook.
/// 
/// Lifecycle hooks allow hooking into different stages of command processing.
pub trait LifecycleHook: Send + Sync + std::fmt::Debug {
    /// Returns the name of the lifecycle hook.
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    
    /// Returns the lifecycle stages this hook is interested in.
    #[allow(dead_code)]
    fn stages(&self) -> Vec<LifecycleStage>;
    
    /// Called when a command is processed at different lifecycle stages.
    /// 
    /// # Arguments
    /// * `stage` - The current lifecycle stage
    /// * `command` - The command being processed
    /// * `context` - The context for the command
    /// 
    /// # Errors
    /// Returns an error if the hook fails
    fn on_stage(&self, stage: &LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>>;
    
    /// Clone the hook into a new Box.
    #[allow(dead_code)]
    fn clone_box(&self) -> Box<dyn LifecycleHook>;
}

/// Manager for command lifecycle stages.
#[derive(Debug)]
pub struct CommandLifecycle {
    /// List of lifecycle hooks to execute.
    hooks: RwLock<Vec<Box<dyn LifecycleHook>>>,
}

impl CommandLifecycle {
    /// Creates a new command lifecycle manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(Vec::new()),
        }
    }
    
    /// Executes a lifecycle stage for a command
    /// 
    /// # Arguments
    /// 
    /// * `stage` - The lifecycle stage to execute
    /// * `command` - The command to execute the stage for
    /// 
    /// # Errors
    /// 
    /// Returns an error if any lifecycle hook fails during execution
    pub fn execute_stage(&self, stage: LifecycleStage, command: &dyn Command) -> Result<(), Box<dyn Error>> {
        let hooks = self.hooks.read().map_err(|e| Box::new(ValidationError {
            rule_name: "LifecycleHook".to_string(),
            message: format!("Failed to acquire read lock: {e}"),
        }))?;
        
        // Execute pre-stage hooks
        for hook in hooks.iter() {
            hook.on_stage(&stage, command)?;
        }
        
        // Execute post-stage hooks
        for hook in hooks.iter() {
            hook.on_stage(&stage, command)?;
        }
        
        Ok(())
    }
    
    /// Adds a lifecycle hook to the manager
    /// 
    /// # Arguments
    /// 
    /// * `hook` - The hook to add
    /// 
    /// # Errors
    /// 
    /// Returns an error if the hook cannot be added
    pub fn add_hook(&self, hook: Box<dyn LifecycleHook>) -> Result<(), Box<dyn Error>> {
        let mut hooks = self.hooks.write().map_err(|e| Box::new(ValidationError {
            rule_name: "LifecycleHook".to_string(),
            message: format!("Failed to acquire write lock: {e}"),
        }))?;
        hooks.push(hook);
        Ok(())
    }
    
    /// Returns the number of hooks registered
    #[must_use]
    #[allow(dead_code)]
    pub fn hooks(&self) -> usize {
        self.hooks.read().map(|h| h.len()).unwrap_or(0)
    }
}

impl Default for CommandLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an error that occurred during command validation
#[derive(Debug, thiserror::Error)]
pub struct ValidationError {
    /// Name of the validation rule that failed
    pub rule_name: String,
    /// Error message describing what validation failed
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation error in rule {}: {}", self.rule_name, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_stages() {
        let stage = LifecycleStage::PreExecution;
        assert_eq!(stage, LifecycleStage::PreExecution);
    }
}