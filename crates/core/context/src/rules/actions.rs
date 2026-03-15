// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Rule action executor for executing rule actions
use serde_json::json;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};

use super::error::{Result, RuleError};
use super::models::{Rule, RuleAction};
use super::plugin::RulePluginManager;

/// Result of applying rules to a context
#[derive(Debug, Clone)]
pub struct RuleApplicationResult {
    /// Rules that were applied
    pub rules_applied: Vec<AppliedRule>,
}

/// Information about an applied rule
#[derive(Debug, Clone)]
pub struct AppliedRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule version
    pub version: String,
    /// Rule category
    pub category: String,
    /// Time when the rule was applied
    pub applied_at: chrono::DateTime<chrono::Utc>,
}

/// Action executor for executing rule actions
#[derive(Debug)]
pub struct ActionExecutor {
    /// Plugin manager
    plugin_manager: Arc<RulePluginManager>,
    /// Default context
    default_context: Option<Value>,
}

impl ActionExecutor {
    /// Create a new action executor
    pub fn new(plugin_manager: Arc<RulePluginManager>) -> Self {
        Self {
            plugin_manager,
            default_context: None,
        }
    }

    /// Create a new action executor with a default context
    pub fn with_default_context(
        plugin_manager: Arc<RulePluginManager>,
        default_context: Value,
    ) -> Self {
        Self {
            plugin_manager,
            default_context: Some(default_context),
        }
    }

    /// Get the plugin manager
    pub fn plugin_manager(&self) -> &Arc<RulePluginManager> {
        &self.plugin_manager
    }

    /// Get the default context
    pub fn default_context(&self) -> Option<&Value> {
        self.default_context.as_ref()
    }

    /// Set the default context
    pub fn set_default_context(&mut self, context: Value) {
        self.default_context = Some(context);
    }

    /// Execute a rule's actions
    pub async fn execute_rule_actions(
        &self,
        rule: &Rule,
        context: Option<&mut Value>,
    ) -> Result<Value> {
        let mut ctx_copy: Value;

        // Either use the provided context or the default
        let context_ref = match context {
            Some(ctx) => {
                // Use the provided context directly (by reference)
                ctx
            }
            None => {
                // Create a copy of the default context
                ctx_copy = self.default_context.clone().unwrap_or_else(|| json!({}));
                &mut ctx_copy
            }
        };

        // Execute each action on the context
        for action in &rule.actions {
            self.execute_action(action, context_ref).await?;
        }

        // Return a copy of the context after modifications
        Ok(context_ref.clone())
    }

    /// Execute an action
    pub async fn execute_action(&self, action: &RuleAction, context: &mut Value) -> Result<()> {
        match action {
            RuleAction::ModifyContext { path, value } => {
                self.set_value_at_path(context, path, value.clone())?;
                Ok(())
            }

            RuleAction::CreateRecoveryPoint { name, description } => {
                // Simply log the recovery point creation for now
                debug!("Created recovery point: {name} - {description:?}");
                Ok(())
            }

            RuleAction::ExecuteTransformation {
                id,
                input_path,
                output_path,
                config: _config,
            } => {
                // Get the transformation
                let transformation = self.plugin_manager.get_transformation(id).await?;

                // Get the input data
                let input = self
                    .get_value_at_path(context, input_path)
                    .ok_or_else(|| RuleError::InvalidPath(input_path.to_string()))?
                    .clone();

                // Run the transformation
                let result = transformation.transform(input).await.map_err(|e| {
                    RuleError::ActionExecutionError(format!("Transformation error: {e}"))
                })?;

                // Set the output
                self.set_value_at_path(context, output_path, result)?;

                Ok(())
            }

            RuleAction::ExecuteCommand {
                command,
                args,
                working_dir: _working_dir,
            } => {
                // Log the command execution
                info!("Executing command: {command} {args:?}");

                // Not really executing commands for security reasons in this version
                Ok(())
            }

            RuleAction::CallApi {
                url,
                method,
                headers: _headers,
                body: _body,
                response_path: _response_path,
            } => {
                // Log the API call
                info!("Calling API: {method} {url}");

                // Not really calling APIs in this version
                Ok(())
            }

            RuleAction::LogMessage { level, message } => {
                // Log the message at the appropriate level
                match level.as_str() {
                    "trace" => trace!("{message}"),
                    "debug" => debug!("{message}"),
                    "info" => info!("{message}"),
                    "warn" => warn!("{message}"),
                    "error" => error!("{message}"),
                    _ => info!("{message}"),
                }

                Ok(())
            }

            RuleAction::NotifyUser {
                title,
                message,
                level,
            } => {
                // Log the notification
                info!("User notification ({level}): {title} - {message}");

                Ok(())
            }

            RuleAction::Custom {
                id,
                config: _config,
            } => Err(RuleError::ActionExecutionError(format!(
                "Custom action '{id}' not supported"
            ))),
        }
    }

    /// Set a value at a path in a context
    fn set_value_at_path(&self, context: &mut Value, path: &str, value: Value) -> Result<()> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = context;

        // Navigate to the parent of the final path component
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // This is the final part, set the value
                if let Value::Object(obj) = current {
                    obj.insert(part.to_string(), value);
                    return Ok(());
                }
                return Err(RuleError::InvalidPath(format!(
                    "Cannot set value at path '{path}', parent is not an object"
                )));
            }

            // Handle array indices
            if let Ok(index) = part.parse::<usize>() {
                if let Value::Array(arr) = current {
                    if index >= arr.len() {
                        return Err(RuleError::InvalidPath(format!(
                            "Array index out of bounds: {index}"
                        )));
                    }
                    current = &mut arr[index];
                } else {
                    return Err(RuleError::InvalidPath(format!(
                        "Expected array at '{part}', got {current}"
                    )));
                }
            } else {
                // Handle object properties
                if let Value::Object(obj) = current {
                    if !obj.contains_key(*part) {
                        obj.insert(part.to_string(), json!({}));
                    }
                    current = obj.get_mut(*part).ok_or_else(|| {
                        RuleError::InvalidPath(format!("Failed to access object property: {part}"))
                    })?;
                } else {
                    return Err(RuleError::InvalidPath(format!(
                        "Expected object at '{part}', got {current}"
                    )));
                }
            }
        }

        // Should never reach here if path is not empty
        Err(RuleError::InvalidPath(format!("Invalid path: {path}")))
    }

    /// Get a value at a path in a context
    fn get_value_at_path<'a>(&self, context: &'a Value, path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = context;

        for part in parts {
            if let Some(obj) = current.as_object() {
                if let Some(value) = obj.get(part) {
                    current = value;
                } else {
                    return None;
                }
            } else if let Some(array) = current.as_array() {
                if let Ok(index) = part.parse::<usize>() {
                    if index < array.len() {
                        current = &array[index];
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        Some(current)
    }
}

impl Default for ActionExecutor {
    fn default() -> Self {
        use super::DummyPluginManager;
        Self::new(Arc::new(RulePluginManager::new(Arc::new(
            DummyPluginManager::default(),
        ))))
    }
}
