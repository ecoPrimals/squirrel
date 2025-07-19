//! Rule action executor for executing actions based on rule evaluations

use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{RuleActionError, RuleSystemError, RuleSystemResult};
use crate::models::{ActionResult, RuleAction};

/// Action executor for executing rule actions
#[derive(Debug)]
pub struct ActionExecutor {
    /// Plugin registry for custom actions
    plugin_registry: Arc<RwLock<HashMap<String, Box<dyn ActionPlugin>>>>,
    /// Action execution history
    execution_history: Arc<RwLock<Vec<ActionExecution>>>,
    /// Action statistics
    stats: Arc<RwLock<ActionStatistics>>,
}

/// Action execution record
#[derive(Debug, Clone)]
struct ActionExecution {
    /// Action ID
    action_id: String,
    /// Rule ID that triggered the action
    rule_id: String,
    /// Context ID
    context_id: String,
    /// When the action was executed
    timestamp: DateTime<Utc>,
    /// Execution result
    result: ActionResult,
}

/// Action execution statistics
#[derive(Debug, Clone)]
struct ActionStatistics {
    /// Total number of actions executed
    total_actions: u64,
    /// Number of successful actions
    successful_actions: u64,
    /// Number of failed actions
    failed_actions: u64,
    /// Average execution time
    average_duration: chrono::Duration,
    /// Actions by type
    actions_by_type: HashMap<String, u64>,
}

impl Default for ActionStatistics {
    fn default() -> Self {
        Self {
            total_actions: 0,
            successful_actions: 0,
            failed_actions: 0,
            average_duration: chrono::Duration::zero(),
            actions_by_type: HashMap::new(),
        }
    }
}

/// Trait for custom action plugins
#[async_trait::async_trait]
pub trait ActionPlugin: Send + Sync + std::fmt::Debug {
    /// Execute an action
    async fn execute(
        &self,
        action: &RuleAction,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult>;

    /// Get the name of the action plugin
    fn name(&self) -> &str;

    /// Get the description of the action plugin
    fn description(&self) -> &str;
}

impl ActionExecutor {
    /// Create a new action executor
    pub fn new() -> Self {
        Self {
            plugin_registry: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(ActionStatistics::default())),
        }
    }

    /// Initialize the action executor
    pub async fn initialize(&self) -> RuleSystemResult<()> {
        // Register built-in action plugins
        self.register_builtin_actions().await?;

        Ok(())
    }

    /// Register built-in action plugins
    async fn register_builtin_actions(&self) -> RuleSystemResult<()> {
        // Register notification action
        self.register_plugin(Box::new(NotificationAction)).await?;

        // Register context modification action
        self.register_plugin(Box::new(ContextModificationAction))
            .await?;

        // Register validation action
        self.register_plugin(Box::new(ValidationAction)).await?;

        Ok(())
    }

    /// Execute an action
    pub async fn execute_action(
        &self,
        action: &RuleAction,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        let start_time = Utc::now();

        // Execute the action based on its type
        let result = match action {
            RuleAction::ModifyContext { path, value } => {
                self.execute_modify_context(path, value, context_id, rule_id)
                    .await
            }

            RuleAction::CreateRecoveryPoint { description } => {
                self.execute_create_recovery_point(description, context_id, rule_id)
                    .await
            }

            RuleAction::ExecuteTransformation {
                transformation_id,
                config,
            } => {
                self.execute_transformation(transformation_id, config.as_ref(), context_id, rule_id)
                    .await
            }

            RuleAction::Notify {
                channel,
                message,
                data,
            } => {
                self.execute_notify(channel, message, data.as_ref(), context_id, rule_id)
                    .await
            }

            RuleAction::ExecutePlugin { plugin_id, config } => {
                self.execute_plugin(plugin_id, config, context_id, rule_id)
                    .await
            }

            RuleAction::ExecuteScript { script, language } => {
                self.execute_script(script, language, context_id, rule_id)
                    .await
            }

            RuleAction::ValidateContext { schema } => {
                self.execute_validate_context(schema, context_id, rule_id)
                    .await
            }
        };

        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(start_time);

        // Update statistics
        match &result {
            Ok(action_result) => {
                self.update_stats(action_result, duration, action).await;
                self.record_execution(action, context_id, rule_id, start_time, action_result)
                    .await;
            }
            Err(_) => {
                // Create a failure result for stats
                let failure_result = ActionResult {
                    action_id: format!("error_{}", uuid::Uuid::new_v4()),
                    rule_id: rule_id.to_string(),
                    context_id: context_id.to_string(),
                    success: false,
                    message: "Action execution failed".to_string(),
                    data: None,
                    timestamp: start_time,
                };
                self.update_stats(&failure_result, duration, action).await;
                self.record_execution(action, context_id, rule_id, start_time, &failure_result)
                    .await;
            }
        }

        result
    }

    /// Execute modify context action
    async fn execute_modify_context(
        &self,
        path: &str,
        value: &Value,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        // In a real implementation, this would modify the actual context
        // For now, we'll just return a success result
        Ok(ActionResult {
            action_id: format!("modify_context_{}", uuid::Uuid::new_v4()),
            rule_id: rule_id.to_string(),
            context_id: context_id.to_string(),
            success: true,
            message: format!("Modified context at path '{}' with value '{}'", path, value),
            data: Some(serde_json::json!({
                "path": path,
                "value": value
            })),
            timestamp: Utc::now(),
        })
    }

    /// Execute create recovery point action
    async fn execute_create_recovery_point(
        &self,
        description: &str,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        // In a real implementation, this would create a recovery point
        let recovery_id = uuid::Uuid::new_v4().to_string();

        Ok(ActionResult {
            action_id: format!("recovery_point_{}", recovery_id),
            rule_id: rule_id.to_string(),
            context_id: context_id.to_string(),
            success: true,
            message: format!("Created recovery point: {}", description),
            data: Some(serde_json::json!({
                "recovery_id": recovery_id,
                "description": description
            })),
            timestamp: Utc::now(),
        })
    }

    /// Execute transformation action
    async fn execute_transformation(
        &self,
        transformation_id: &str,
        config: Option<&Value>,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        // In a real implementation, this would execute a transformation
        Ok(ActionResult {
            action_id: format!("transformation_{}", uuid::Uuid::new_v4()),
            rule_id: rule_id.to_string(),
            context_id: context_id.to_string(),
            success: true,
            message: format!("Executed transformation: {}", transformation_id),
            data: Some(serde_json::json!({
                "transformation_id": transformation_id,
                "config": config
            })),
            timestamp: Utc::now(),
        })
    }

    /// Execute notify action
    async fn execute_notify(
        &self,
        channel: &str,
        message: &str,
        data: Option<&Value>,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        // In a real implementation, this would send a notification
        Ok(ActionResult {
            action_id: format!("notify_{}", uuid::Uuid::new_v4()),
            rule_id: rule_id.to_string(),
            context_id: context_id.to_string(),
            success: true,
            message: format!("Sent notification to {} channel: {}", channel, message),
            data: Some(serde_json::json!({
                "channel": channel,
                "message": message,
                "data": data
            })),
            timestamp: Utc::now(),
        })
    }

    /// Execute plugin action
    async fn execute_plugin(
        &self,
        plugin_id: &str,
        config: &Value,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        // Check if plugin is registered
        let plugin_registry = self.plugin_registry.read().await;
        if let Some(plugin) = plugin_registry.get(plugin_id) {
            let action = RuleAction::ExecutePlugin {
                plugin_id: plugin_id.to_string(),
                config: config.clone(),
            };
            plugin.execute(&action, context_id, rule_id).await
        } else {
            Ok(ActionResult {
                action_id: format!("plugin_error_{}", uuid::Uuid::new_v4()),
                rule_id: rule_id.to_string(),
                context_id: context_id.to_string(),
                success: false,
                message: format!("Plugin not found: {}", plugin_id),
                data: None,
                timestamp: Utc::now(),
            })
        }
    }

    /// Execute script action
    async fn execute_script(
        &self,
        script: &str,
        language: &str,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        // In a real implementation, this would execute a script
        // For now, we'll return an error indicating it's not implemented
        Ok(ActionResult {
            action_id: format!("script_{}", uuid::Uuid::new_v4()),
            rule_id: rule_id.to_string(),
            context_id: context_id.to_string(),
            success: false,
            message: format!(
                "Script execution not implemented: {} ({})",
                script, language
            ),
            data: Some(serde_json::json!({
                "script": script,
                "language": language
            })),
            timestamp: Utc::now(),
        })
    }

    /// Execute validate context action
    async fn execute_validate_context(
        &self,
        schema: &Value,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        // In a real implementation, this would validate the context against a schema
        Ok(ActionResult {
            action_id: format!("validate_{}", uuid::Uuid::new_v4()),
            rule_id: rule_id.to_string(),
            context_id: context_id.to_string(),
            success: true,
            message: "Context validation completed".to_string(),
            data: Some(serde_json::json!({
                "schema": schema
            })),
            timestamp: Utc::now(),
        })
    }

    /// Register a custom action plugin
    pub async fn register_plugin(&self, plugin: Box<dyn ActionPlugin>) -> RuleSystemResult<()> {
        let name = plugin.name().to_string();
        self.plugin_registry.write().await.insert(name, plugin);
        Ok(())
    }

    /// Unregister a custom action plugin
    pub async fn unregister_plugin(&self, name: &str) -> RuleSystemResult<()> {
        self.plugin_registry.write().await.remove(name);
        Ok(())
    }

    /// Get registered plugins
    pub async fn get_registered_plugins(&self) -> Vec<String> {
        self.plugin_registry.read().await.keys().cloned().collect()
    }

    /// Update action statistics
    async fn update_stats(
        &self,
        result: &ActionResult,
        duration: chrono::Duration,
        action: &RuleAction,
    ) {
        let mut stats = self.stats.write().await;

        stats.total_actions += 1;

        if result.success {
            stats.successful_actions += 1;
        } else {
            stats.failed_actions += 1;
        }

        // Update average duration
        let total_duration = stats.average_duration * (stats.total_actions as i32 - 1) + duration;
        stats.average_duration = total_duration / (stats.total_actions as i32);

        // Update actions by type
        let action_type = match action {
            RuleAction::ModifyContext { .. } => "modify_context",
            RuleAction::CreateRecoveryPoint { .. } => "create_recovery_point",
            RuleAction::ExecuteTransformation { .. } => "execute_transformation",
            RuleAction::Notify { .. } => "notify",
            RuleAction::ExecutePlugin { .. } => "execute_plugin",
            RuleAction::ExecuteScript { .. } => "execute_script",
            RuleAction::ValidateContext { .. } => "validate_context",
        };

        *stats
            .actions_by_type
            .entry(action_type.to_string())
            .or_insert(0) += 1;
    }

    /// Record action execution
    async fn record_execution(
        &self,
        action: &RuleAction,
        context_id: &str,
        rule_id: &str,
        timestamp: DateTime<Utc>,
        result: &ActionResult,
    ) {
        let execution = ActionExecution {
            action_id: result.action_id.clone(),
            rule_id: rule_id.to_string(),
            context_id: context_id.to_string(),
            timestamp,
            result: result.clone(),
        };

        self.execution_history.write().await.push(execution);
    }

    /// Get action execution history
    pub async fn get_execution_history(&self) -> Vec<ActionExecution> {
        self.execution_history.read().await.clone()
    }

    /// Get action statistics
    pub async fn get_statistics(&self) -> ActionStatistics {
        self.stats.read().await.clone()
    }

    /// Clear execution history
    pub async fn clear_history(&self) -> RuleSystemResult<()> {
        self.execution_history.write().await.clear();
        Ok(())
    }

    /// Reset statistics
    pub async fn reset_statistics(&self) -> RuleSystemResult<()> {
        *self.stats.write().await = ActionStatistics::default();
        Ok(())
    }
}

impl Default for ActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in notification action plugin
#[derive(Debug)]
pub struct NotificationAction;

#[async_trait::async_trait]
impl ActionPlugin for NotificationAction {
    async fn execute(
        &self,
        action: &RuleAction,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        if let RuleAction::Notify {
            channel,
            message,
            data,
        } = action
        {
            // In a real implementation, this would send notifications to external systems
            Ok(ActionResult {
                action_id: format!("notification_{}", uuid::Uuid::new_v4()),
                rule_id: rule_id.to_string(),
                context_id: context_id.to_string(),
                success: true,
                message: format!("Notification sent to {}: {}", channel, message),
                data: data.clone(),
                timestamp: Utc::now(),
            })
        } else {
            Err(RuleSystemError::ActionError(RuleActionError::Other(
                "Invalid action type for notification plugin".to_string(),
            )))
        }
    }

    fn name(&self) -> &str {
        "notification"
    }

    fn description(&self) -> &str {
        "Sends notifications to external systems"
    }
}

/// Built-in context modification action plugin
#[derive(Debug)]
pub struct ContextModificationAction;

#[async_trait::async_trait]
impl ActionPlugin for ContextModificationAction {
    async fn execute(
        &self,
        action: &RuleAction,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        if let RuleAction::ModifyContext { path, value } = action {
            // In a real implementation, this would modify the actual context data
            Ok(ActionResult {
                action_id: format!("context_mod_{}", uuid::Uuid::new_v4()),
                rule_id: rule_id.to_string(),
                context_id: context_id.to_string(),
                success: true,
                message: format!("Context modified at path: {}", path),
                data: Some(serde_json::json!({
                    "path": path,
                    "value": value
                })),
                timestamp: Utc::now(),
            })
        } else {
            Err(RuleSystemError::ActionError(RuleActionError::Other(
                "Invalid action type for context modification plugin".to_string(),
            )))
        }
    }

    fn name(&self) -> &str {
        "context_modification"
    }

    fn description(&self) -> &str {
        "Modifies context data"
    }
}

/// Built-in validation action plugin
#[derive(Debug)]
pub struct ValidationAction;

#[async_trait::async_trait]
impl ActionPlugin for ValidationAction {
    async fn execute(
        &self,
        action: &RuleAction,
        context_id: &str,
        rule_id: &str,
    ) -> RuleSystemResult<ActionResult> {
        if let RuleAction::ValidateContext { schema } = action {
            // In a real implementation, this would validate context against a schema
            Ok(ActionResult {
                action_id: format!("validation_{}", uuid::Uuid::new_v4()),
                rule_id: rule_id.to_string(),
                context_id: context_id.to_string(),
                success: true,
                message: "Context validation completed".to_string(),
                data: Some(serde_json::json!({
                    "schema": schema
                })),
                timestamp: Utc::now(),
            })
        } else {
            Err(RuleSystemError::ActionError(RuleActionError::Other(
                "Invalid action type for validation plugin".to_string(),
            )))
        }
    }

    fn name(&self) -> &str {
        "validation"
    }

    fn description(&self) -> &str {
        "Validates context data against schemas"
    }
}

/// Create a new action executor with default configuration
pub fn create_action_executor() -> RuleSystemResult<ActionExecutor> {
    Ok(ActionExecutor::new())
}

/// Create an action executor with custom configuration
pub fn create_action_executor_with_config() -> RuleSystemResult<ActionExecutor> {
    let executor = ActionExecutor::new();
    // Add any custom configuration here
    Ok(executor)
}
