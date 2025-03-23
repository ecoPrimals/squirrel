//! Tool lifecycle hooks for MCP
//!
//! This module provides implementations of tool lifecycle hooks for the MCP protocol.

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use crate::tool::{Tool, ToolError, ToolLifecycleHook, ToolState};

/// Type alias for tool state history entries
pub type StateHistoryEntry = (ToolState, chrono::DateTime<Utc>);

/// Type alias for tool state history map
pub type StateHistoryMap = HashMap<String, Vec<StateHistoryEntry>>;

/// Lifecycle event types for tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleEvent {
    /// Tool is registered
    Register,
    /// Tool is unregistered
    Unregister,
    /// Tool is activated
    Activate,
    /// Tool is deactivated
    Deactivate,
    /// Tool encounters an error
    Error,
    /// Tool is about to start
    PreStart,
    /// Tool has started
    PostStart,
    /// Tool is about to stop
    PreStop,
    /// Tool has stopped
    PostStop,
    /// Tool is paused
    Pause,
    /// Tool is resumed
    Resume,
    /// Tool is updated
    Update,
    /// Tool is cleaned up
    Cleanup,
    /// Tool is initialized
    Initialize,
    /// Tool is about to execute
    PreExecute,
    /// Tool has executed
    PostExecute,
    /// Tool is reset
    Reset,
}

impl fmt::Display for LifecycleEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register => write!(f, "register"),
            Self::Unregister => write!(f, "unregister"),
            Self::Activate => write!(f, "activate"),
            Self::Deactivate => write!(f, "deactivate"),
            Self::Error => write!(f, "error"),
            Self::PreStart => write!(f, "pre_start"),
            Self::PostStart => write!(f, "post_start"),
            Self::PreStop => write!(f, "pre_stop"),
            Self::PostStop => write!(f, "post_stop"),
            Self::Pause => write!(f, "pause"),
            Self::Resume => write!(f, "resume"),
            Self::Update => write!(f, "update"),
            Self::Cleanup => write!(f, "cleanup"),
            Self::Initialize => write!(f, "initialize"),
            Self::PreExecute => write!(f, "pre_execute"),
            Self::PostExecute => write!(f, "post_execute"),
            Self::Reset => write!(f, "reset"),
        }
    }
}

/// Tool lifecycle event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRecord {
    /// Tool ID
    pub tool_id: String,
    /// Event type
    pub event: LifecycleEvent,
    /// Event timestamp
    pub timestamp: chrono::DateTime<Utc>,
    /// Tool state before the event
    pub previous_state: Option<ToolState>,
    /// Tool state after the event
    pub new_state: Option<ToolState>,
    /// Error message if applicable
    pub error_message: Option<String>,
    /// Duration of the operation in milliseconds (if applicable)
    pub duration_ms: Option<u64>,
}

/// Tool recovery strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// Maximum number of recovery attempts
    pub max_attempts: u32,
    /// Backoff strategy for retry attempts
    pub backoff_strategy: BackoffStrategy,
    /// Recovery actions to attempt
    pub recovery_actions: Vec<RecoveryAction>,
}

/// Backoff strategy for recovery attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between attempts (milliseconds)
    Fixed(u64),
    /// Exponential backoff (base milliseconds)
    Exponential(u64),
    /// Linear backoff (increase milliseconds)
    Linear(u64),
}

/// Recovery action for tool errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Reset the tool
    Reset,
    /// Restart the tool
    Restart,
    /// Recreate the tool
    Recreate,
    /// Custom action name (to be handled by the recovery hook)
    Custom(String),
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

/// Custom recovery handler
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

/// Record of a recovery attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    /// Tool ID
    pub tool_id: String,
    /// Recovery action attempted
    pub action: RecoveryAction,
    /// Timestamp of the attempt
    pub timestamp: chrono::DateTime<Utc>,
    /// Whether the recovery was successful
    pub successful: bool,
    /// Error message if recovery failed
    pub error_message: Option<String>,
    /// Attempt number
    pub attempt_number: u32,
}

impl RecoveryHook {
    /// Create a new recovery hook with default settings
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

    /// Get tool-specific recovery strategy
    pub fn get_strategy_for_tool(&self, _tool_id: &str) -> &RecoveryStrategy {
        // In a real implementation, this would look up custom strategies by tool ID
        // For now, we'll just return the default strategy
        &self.default_strategy
    }

    /// Record a recovery attempt
    pub fn record_attempt(&mut self, attempt: RecoveryAttempt) {
        self.history.push(attempt);
    }

    /// Get recovery history for a tool
    pub fn get_history_for_tool(&self, tool_id: &str) -> Vec<RecoveryAttempt> {
        self.history
            .iter()
            .filter(|attempt| attempt.tool_id == tool_id)
            .cloned()
            .collect()
    }

    /// Calculate backoff delay for an attempt
    pub fn calculate_backoff_delay(&self, strategy: &BackoffStrategy, attempt: u32) -> u64 {
        match strategy {
            BackoffStrategy::Fixed(delay) => *delay,
            BackoffStrategy::Exponential(base) => base * 2u64.pow(attempt - 1),
            BackoffStrategy::Linear(increment) => increment * attempt as u64,
        }
    }

    /// Check if recovery should be attempted
    pub fn should_attempt_recovery(&self, _tool_id: &str, error: &ToolError) -> bool {
        // Check if the error is recoverable
        match error {
            ToolError::NeedsReset(_) => true,
            ToolError::ExecutionFailed { .. } => true,
            ToolError::InitializationFailed { .. } => true,
            ToolError::ResourceError(_) => true,
            ToolError::ExecutionError(_) => true,
            // Not recoverable
            ToolError::SecurityViolation(_) => false,
            ToolError::TooManyErrors(_) => false,
            ToolError::PermissionDenied(_) => false,
            // Others may be recoverable
            _ => true,
        }
    }

    /// Get next recovery action for a tool
    pub fn get_next_action(&self, tool_id: &str) -> Option<RecoveryAction> {
        let strategy = self.get_strategy_for_tool(tool_id);
        let attempts = self.get_history_for_tool(tool_id);

        // Check if max attempts reached
        if attempts.len() >= strategy.max_attempts as usize {
            return None;
        }

        // Get next action based on attempt count
        let attempt_index = attempts.len();
        if attempt_index < strategy.recovery_actions.len() {
            Some(strategy.recovery_actions[attempt_index].clone())
        } else {
            // Default to the last action if we've tried all actions but still have attempts left
            strategy.recovery_actions.last().cloned()
        }
    }
}

impl Default for RecoveryHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic implementation of ToolLifecycleHook
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
    /// Creates a new basic lifecycle hook
    pub fn new() -> Self {
        Self {
            state_history: Arc::new(RwLock::new(HashMap::new())),
            max_history_entries: 100,
        }
    }

    /// Sets the maximum number of history entries to keep per tool
    pub fn with_max_history_entries(mut self, max_entries: usize) -> Self {
        self.max_history_entries = max_entries;
        self
    }

    /// Gets the state history for a tool
    pub async fn get_state_history(
        &self,
        tool_id: &str,
    ) -> Vec<(ToolState, chrono::DateTime<Utc>)> {
        let history = self.state_history.read().await;
        history.get(tool_id).cloned().unwrap_or_default()
    }

    /// Adds a state change to the history
    async fn record_state_change(&self, tool_id: &str, state: ToolState) {
        let mut history = self.state_history.write().await;
        let tool_history = history.entry(tool_id.to_string()).or_insert_with(Vec::new);

        // Add the new state change
        tool_history.push((state, Utc::now()));

        // Trim the history if it exceeds the maximum size
        if tool_history.len() > self.max_history_entries {
            let excess = tool_history.len() - self.max_history_entries;
            tool_history.drain(0..excess);
        }
    }

    async fn recover_state(&self, tool_id: &str) -> Result<(), ToolError> {
        // Check if the tool exists in our history
        let history = self.state_history.read().await;
        if let Some(tool_history) = history.get(tool_id) {
            if let Some((last_state, _)) = tool_history.last() {
                // Clone the last state instead of dropping the history while it's borrowed
                let last_state_clone = *last_state;
                // Release the read lock before making changes
                drop(history);
                // Record the state change with the cloned value
                self.record_state_change(tool_id, last_state_clone).await;
                return Ok(());
            }
        }
        // If we get here, either the tool doesn't exist or has no history
        Err(ToolError::NoStateHistory(tool_id.to_string()))
    }
}

#[async_trait]
impl crate::tool::ToolLifecycleHook for BasicLifecycleHook {
    #[instrument(skip(self, tool))]
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        info!("Tool registered: {} ({})", tool.name, tool.id);

        // Record the initial state
        self.record_state_change(&tool.id, ToolState::Registered)
            .await;

        // Log the tool capabilities
        for capability in &tool.capabilities {
            debug!("Capability registered: {} for tool {}", capability, tool.id);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool unregistered: {}", tool_id);

        // Record the final state
        self.record_state_change(tool_id, ToolState::Unregistered)
            .await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_activate(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool activated: {}", tool_id);

        // Record the state change
        self.record_state_change(tool_id, ToolState::Active).await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool deactivated: {}", tool_id);

        // Record the state change
        self.record_state_change(tool_id, ToolState::Stopped).await;

        Ok(())
    }

    #[instrument(skip(self, error))]
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        error!("Tool error: {} - {}", tool_id, error);

        // Record the state change
        self.record_state_change(tool_id, ToolState::Error).await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn pre_start(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool pre-start: {}", tool_id);

        // Record the state change
        self.record_state_change(tool_id, ToolState::Starting).await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_start(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool post-start: {}", tool_id);

        // Record the state change
        self.record_state_change(tool_id, ToolState::Started).await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn pre_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool pre-stop: {}", tool_id);

        // Record the state change
        self.record_state_change(tool_id, ToolState::Stopping).await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool post-stop: {}", tool_id);

        // Record the state change
        self.record_state_change(tool_id, ToolState::Stopped).await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_pause(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool paused: {}", tool_id);

        // Record the state change
        self.record_state_change(tool_id, ToolState::Paused).await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_resume(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool resumed: {}", tool_id);

        // Record the state change
        self.record_state_change(tool_id, ToolState::Active).await;

        Ok(())
    }

    #[instrument(skip(self, tool))]
    async fn on_update(&self, tool: &Tool) -> Result<(), ToolError> {
        info!("Tool updated: {} ({})", tool.name, tool.id);

        // Record the state change
        self.record_state_change(&tool.id, ToolState::Active).await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_cleanup(&self, tool_id: &str) -> Result<(), ToolError> {
        info!("Tool cleanup: {}", tool_id);

        // No state change for cleanup, just record the event with current state
        self.recover_state(tool_id).await?;

        Ok(())
    }

    async fn register_tool(&self, tool: &Tool) -> Result<(), ToolError> {
        // Delegate to existing on_register method
        self.on_register(tool).await
    }

    async fn initialize_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Initializing tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Registered).await;
        Ok(())
    }

    async fn pre_execute(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Pre-execute for tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Starting).await;
        Ok(())
    }

    async fn post_execute(
        &self,
        tool_id: &str,
        result: Result<(), ToolError>,
    ) -> Result<(), ToolError> {
        match result {
            Ok(_) => {
                debug!("Post-execute successful for tool: {}", tool_id);
                self.record_state_change(tool_id, ToolState::Stopped).await;
            }
            Err(e) => {
                warn!("Post-execute failed for tool: {}: {:?}", tool_id, e);
                self.record_state_change(tool_id, ToolState::Error).await;
            }
        }
        Ok(())
    }

    async fn reset_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Resetting tool: {}", tool_id);
        self.record_state_change(tool_id, ToolState::Registered).await;
        Ok(())
    }

    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Delegate to existing on_cleanup method
        self.on_cleanup(tool_id).await
    }
}

/// A lifecycle hook that performs additional validation and security checks
#[derive(Debug)]
pub struct SecurityLifecycleHook {
    /// The security level required for capabilities by default
    default_security_level: u8,
    /// Tool IDs that are allowed to register
    allowed_tool_ids: Vec<String>,
    /// Whether to enforce allowed tool IDs
    enforce_allowed_tools: bool,
}

impl Default for SecurityLifecycleHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityLifecycleHook {
    /// Creates a new security lifecycle hook
    pub fn new() -> Self {
        Self {
            default_security_level: 1,
            allowed_tool_ids: Vec::new(),
            enforce_allowed_tools: false,
        }
    }

    /// Sets the default security level for capabilities
    pub fn with_default_security_level(mut self, level: u8) -> Self {
        self.default_security_level = level;
        self
    }

    /// Adds an allowed tool ID
    pub fn allow_tool(mut self, tool_id: impl Into<String>) -> Self {
        self.allowed_tool_ids.push(tool_id.into());
        self
    }

    /// Sets whether to enforce allowed tool IDs
    pub fn enforce_allowed_tools(mut self, enforce: bool) -> Self {
        self.enforce_allowed_tools = enforce;
        self
    }

    /// Validates a tool's security metadata
    fn validate_tool_security(&self, tool: &Tool) -> Result<(), ToolError> {
        // Check if the tool is allowed to register
        if self.enforce_allowed_tools && !self.allowed_tool_ids.contains(&tool.id) {
            return Err(ToolError::ValidationFailed(format!(
                "Tool ID '{}' is not in the allowed list",
                tool.id
            )));
        }

        // Ensure the tool has a security level
        if tool.security_level < self.default_security_level {
            return Err(ToolError::ValidationFailed(format!(
                "Tool '{}' has insufficient security level: {} (minimum: {})",
                tool.id, tool.security_level, self.default_security_level
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl crate::tool::ToolLifecycleHook for SecurityLifecycleHook {
    #[instrument(skip(self, tool))]
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        // Validate tool security
        self.validate_tool_security(tool)?;

        // Check if tool ID is allowed
        if self.enforce_allowed_tools && !self.allowed_tool_ids.contains(&tool.id) {
            return Err(ToolError::SecurityViolation(format!(
                "Tool ID '{}' is not in the allowed list",
                tool.id
            )));
        }

        debug!("Tool '{}' security validation passed", tool.id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        // No security checks on unregister
        debug!("Tool '{}' unregistered", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_activate(&self, tool_id: &str) -> Result<(), ToolError> {
        // No additional security checks on activate
        debug!("Tool '{}' activated", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        // No additional security checks on deactivate
        debug!("Tool '{}' deactivated", tool_id);
        Ok(())
    }

    #[instrument(skip(self, error))]
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        // Log the error
        warn!("Tool '{}' encountered error: {}", tool_id, error);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn pre_start(&self, tool_id: &str) -> Result<(), ToolError> {
        // Verify security before starting
        if self.enforce_allowed_tools && !self.allowed_tool_ids.contains(&tool_id.to_string()) {
            return Err(ToolError::SecurityViolation(format!(
                "Tool ID '{}' is not in the allowed list",
                tool_id
            )));
        }

        debug!("Tool '{}' pre-start security check passed", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_start(&self, tool_id: &str) -> Result<(), ToolError> {
        // No additional security checks after start
        debug!("Tool '{}' started", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn pre_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        // No additional security checks before stop
        debug!("Tool '{}' stopping", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        // No additional security checks after stop
        debug!("Tool '{}' stopped", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_pause(&self, tool_id: &str) -> Result<(), ToolError> {
        // No additional security checks for pause
        debug!("Tool '{}' paused", tool_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_resume(&self, tool_id: &str) -> Result<(), ToolError> {
        // Verify security before resuming
        if self.enforce_allowed_tools && !self.allowed_tool_ids.contains(&tool_id.to_string()) {
            return Err(ToolError::SecurityViolation(format!(
                "Tool ID '{}' is not in the allowed list",
                tool_id
            )));
        }

        debug!("Tool '{}' resume security check passed", tool_id);
        Ok(())
    }

    #[instrument(skip(self, tool))]
    async fn on_update(&self, tool: &Tool) -> Result<(), ToolError> {
        // Validate tool security on update
        self.validate_tool_security(tool)?;

        // Check if tool ID is allowed
        if self.enforce_allowed_tools && !self.allowed_tool_ids.contains(&tool.id) {
            return Err(ToolError::SecurityViolation(format!(
                "Tool ID '{}' is not in the allowed list",
                tool.id
            )));
        }

        debug!("Tool '{}' update security validation passed", tool.id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_cleanup(&self, tool_id: &str) -> Result<(), ToolError> {
        // No additional security checks for cleanup
        debug!("Tool '{}' cleanup", tool_id);
        Ok(())
    }

    async fn register_tool(&self, tool: &Tool) -> Result<(), ToolError> {
        // Delegate to existing on_register method
        self.on_register(tool).await
    }

    async fn initialize_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Security lifecycle: Initializing tool: {}", tool_id);
        Ok(())
    }

    async fn pre_execute(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Security lifecycle: Pre-execute for tool: {}", tool_id);
        Ok(())
    }

    async fn post_execute(
        &self,
        tool_id: &str,
        result: Result<(), ToolError>,
    ) -> Result<(), ToolError> {
        match result {
            Ok(_) => debug!("Security lifecycle: Post-execute successful for tool: {}", tool_id),
            Err(e) => warn!("Security lifecycle: Post-execute failed for tool: {}: {:?}", tool_id, e),
        }
        Ok(())
    }

    async fn reset_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        debug!("Security lifecycle: Resetting tool: {}", tool_id);
        Ok(())
    }

    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Delegate to existing on_cleanup method
        self.on_cleanup(tool_id).await
    }
}

/// A composite lifecycle hook that combines multiple hooks
#[derive(Debug, Default)]
pub struct CompositeLifecycleHook {
    /// The hooks to execute
    hooks: Vec<Arc<dyn ToolLifecycleHook + Send + Sync>>,
}

impl CompositeLifecycleHook {
    /// Creates a new composite lifecycle hook
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Adds a hook to the composite
    pub fn add_hook<H>(&mut self, hook: H)
    where
        H: ToolLifecycleHook + Send + Sync + 'static,
    {
        self.hooks.push(Arc::new(hook));
    }

    /// Creates a new composite lifecycle hook with the given hooks
    pub fn with_hooks<I, H>(hooks: I) -> Self
    where
        I: IntoIterator<Item = H>,
        H: ToolLifecycleHook + Send + Sync + 'static,
    {
        Self {
            hooks: hooks
                .into_iter()
                .map(|h| Arc::new(h) as Arc<dyn ToolLifecycleHook + Send + Sync>)
                .collect(),
        }
    }
}

#[async_trait]
impl crate::tool::ToolLifecycleHook for CompositeLifecycleHook {
    #[instrument(skip(self, tool))]
    async fn on_register(&self, tool: &Tool) -> Result<(), ToolError> {
        for hook in &self.hooks {
            if let Err(err) = hook.on_register(tool).await {
                error!("Hook failed during on_register: {}", err);
                return Err(err);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        // For unregistration, we want to call all hooks even if some fail
        let mut last_error = None;

        for hook in &self.hooks {
            if let Err(err) = hook.on_unregister(tool_id).await {
                error!("Hook failed during on_unregister: {}", err);
                last_error = Some(err);
                // Continue to next hook
            }
        }

        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self))]
    async fn on_activate(&self, tool_id: &str) -> Result<(), ToolError> {
        for hook in &self.hooks {
            if let Err(err) = hook.on_activate(tool_id).await {
                error!("Hook failed during on_activate: {}", err);
                return Err(err);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_deactivate(&self, tool_id: &str) -> Result<(), ToolError> {
        // For deactivation, we want to call all hooks even if some fail
        let mut last_error = None;

        for hook in &self.hooks {
            if let Err(err) = hook.on_deactivate(tool_id).await {
                error!("Hook failed during on_deactivate: {}", err);
                last_error = Some(err);
                // Continue to next hook
            }
        }

        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self, error))]
    async fn on_error(&self, tool_id: &str, error: &ToolError) -> Result<(), ToolError> {
        // For error handling, we want to call all hooks even if some fail
        let mut last_error = None;

        for hook in &self.hooks {
            if let Err(err) = hook.on_error(tool_id, error).await {
                error!("Hook failed during on_error: {}", err);
                last_error = Some(err);
                // Continue to next hook
            }
        }

        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self))]
    async fn pre_start(&self, tool_id: &str) -> Result<(), ToolError> {
        for hook in &self.hooks {
            if let Err(err) = hook.pre_start(tool_id).await {
                error!("Hook failed during pre_start: {}", err);
                return Err(err);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_start(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut last_error = None;

        for hook in &self.hooks {
            if let Err(err) = hook.post_start(tool_id).await {
                error!("Hook failed during post_start: {}", err);
                last_error = Some(err);
                // Continue to ensure all hooks are called
            }
        }

        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self))]
    async fn pre_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        for hook in &self.hooks {
            if let Err(err) = hook.pre_stop(tool_id).await {
                error!("Hook failed during pre_stop: {}", err);
                return Err(err);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn post_stop(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut last_error = None;

        for hook in &self.hooks {
            if let Err(err) = hook.post_stop(tool_id).await {
                error!("Hook failed during post_stop: {}", err);
                last_error = Some(err);
                // Continue to ensure all hooks are called
            }
        }

        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self))]
    async fn on_pause(&self, tool_id: &str) -> Result<(), ToolError> {
        for hook in &self.hooks {
            if let Err(err) = hook.on_pause(tool_id).await {
                error!("Hook failed during on_pause: {}", err);
                return Err(err);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_resume(&self, tool_id: &str) -> Result<(), ToolError> {
        for hook in &self.hooks {
            if let Err(err) = hook.on_resume(tool_id).await {
                error!("Hook failed during on_resume: {}", err);
                return Err(err);
            }
        }

        Ok(())
    }

    #[instrument(skip(self, tool))]
    async fn on_update(&self, tool: &Tool) -> Result<(), ToolError> {
        for hook in &self.hooks {
            if let Err(err) = hook.on_update(tool).await {
                error!("Hook failed during on_update: {}", err);
                return Err(err);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_cleanup(&self, tool_id: &str) -> Result<(), ToolError> {
        let mut last_error = None;

        for hook in &self.hooks {
            if let Err(err) = hook.on_cleanup(tool_id).await {
                error!("Hook failed during on_cleanup: {}", err);
                last_error = Some(err);
                // Continue to ensure all hooks are called
            }
        }

        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    async fn register_tool(&self, tool: &Tool) -> Result<(), ToolError> {
        // Delegate to existing on_register method
        self.on_register(tool).await
    }

    async fn initialize_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        for hook in &self.hooks {
            hook.initialize_tool(tool_id).await?;
        }
        Ok(())
    }

    async fn pre_execute(&self, tool_id: &str) -> Result<(), ToolError> {
        for hook in &self.hooks {
            hook.pre_execute(tool_id).await?;
        }
        Ok(())
    }

    async fn post_execute(
        &self,
        tool_id: &str,
        result: Result<(), ToolError>,
    ) -> Result<(), ToolError> {
        for hook in &self.hooks {
            hook.post_execute(tool_id, result.clone()).await?;
        }
        Ok(())
    }

    async fn reset_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        for hook in &self.hooks {
            hook.reset_tool(tool_id).await?;
        }
        Ok(())
    }

    async fn cleanup_tool(&self, tool_id: &str) -> Result<(), ToolError> {
        // Delegate to existing on_cleanup method
        self.on_cleanup(tool_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::Capability;

    #[tokio::test]
    async fn test_basic_lifecycle_hook() {
        let hook = BasicLifecycleHook::new().with_max_history_entries(10);

        // Create a test tool
        let tool = Tool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool".to_string(),
            capabilities: vec![Capability {
                name: "test".to_string(),
                description: "A test capability".to_string(),
                parameters: Vec::new(),
                return_type: None,
            }],
            security_level: 1,
        };

        // Test the registration hook
        let result = hook.on_register(&tool).await;
        assert!(result.is_ok(), "Registration hook failed: {:?}", result);

        // Test the activation hook
        let result = hook.on_activate(&tool.id).await;
        assert!(result.is_ok(), "Activation hook failed: {:?}", result);

        // Test the deactivation hook
        let result = hook.on_deactivate(&tool.id).await;
        assert!(result.is_ok(), "Deactivation hook failed: {:?}", result);

        // Test the error hook
        let error = ToolError::ExecutionFailed {
            tool_id: tool.id.clone(),
            reason: "Test error".to_string(),
        };
        let result = hook.on_error(&tool.id, &error).await;
        assert!(result.is_ok(), "Error hook failed: {:?}", result);

        // Test the unregistration hook
        let result = hook.on_unregister(&tool.id).await;
        assert!(result.is_ok(), "Unregistration hook failed: {:?}", result);

        // Check the state history
        let history = hook.get_state_history(&tool.id).await;
        assert_eq!(
            history.len(),
            5,
            "Expected 5 state changes, got {}",
            history.len()
        );

        assert_eq!(history[0].0, ToolState::Registered);
        assert_eq!(history[1].0, ToolState::Active);
        assert_eq!(history[2].0, ToolState::Stopped);
        assert_eq!(history[3].0, ToolState::Error);
        assert_eq!(history[4].0, ToolState::Unregistered);
    }

    #[tokio::test]
    async fn test_security_lifecycle_hook() {
        let hook = SecurityLifecycleHook::new()
            .with_default_security_level(2)
            .allow_tool("allowed-tool")
            .enforce_allowed_tools(true);

        // Create an allowed tool with sufficient security level
        let allowed_tool = Tool {
            id: "allowed-tool".to_string(),
            name: "Allowed Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "An allowed tool".to_string(),
            capabilities: Vec::new(),
            security_level: 2,
        };

        // Test the registration hook with an allowed tool
        let result = hook.on_register(&allowed_tool).await;
        assert!(
            result.is_ok(),
            "Registration hook failed for allowed tool: {:?}",
            result
        );

        // Create a disallowed tool
        let disallowed_tool = Tool {
            id: "disallowed-tool".to_string(),
            name: "Disallowed Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A disallowed tool".to_string(),
            capabilities: Vec::new(),
            security_level: 2,
        };

        // Test the registration hook with a disallowed tool
        let result = hook.on_register(&disallowed_tool).await;
        assert!(
            result.is_err(),
            "Registration hook should fail for disallowed tool"
        );

        // Create a tool with insufficient security level
        let insecure_tool = Tool {
            id: "allowed-tool".to_string(), // Same ID as allowed tool
            name: "Insecure Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "An insecure tool".to_string(),
            capabilities: Vec::new(),
            security_level: 1, // Below required level
        };

        // Test the registration hook with an insecure tool
        let result = hook.on_register(&insecure_tool).await;
        assert!(
            result.is_err(),
            "Registration hook should fail for insecure tool"
        );
    }

    #[tokio::test]
    async fn test_composite_lifecycle_hook() {
        let basic_hook = BasicLifecycleHook::new();
        let security_hook = SecurityLifecycleHook::new()
            .with_default_security_level(1)
            .enforce_allowed_tools(false);

        let mut composite_hook = CompositeLifecycleHook::new();
        composite_hook.add_hook(basic_hook);
        composite_hook.add_hook(security_hook);

        // Create a test tool
        let tool = Tool {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool".to_string(),
            capabilities: Vec::new(),
            security_level: 1,
        };

        // Test the registration hook
        let result = composite_hook.on_register(&tool).await;
        assert!(
            result.is_ok(),
            "Composite registration hook failed: {:?}",
            result
        );

        // Test the activation hook
        let result = composite_hook.on_activate(&tool.id).await;
        assert!(
            result.is_ok(),
            "Composite activation hook failed: {:?}",
            result
        );

        // Test the error hook
        let error = ToolError::ExecutionFailed {
            tool_id: tool.id.clone(),
            reason: "Test error".to_string(),
        };
        let result = composite_hook.on_error(&tool.id, &error).await;
        assert!(result.is_ok(), "Composite error hook failed: {:?}", result);
    }
}
