// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Rule data models
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Rule metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleMetadata {
    /// Custom metadata for the rule
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

impl RuleMetadata {
    /// Create a new empty rule metadata
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Set a metadata value
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.data.insert(key.into(), value.into());
    }

    /// Get a metadata value
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Check if metadata contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Remove a metadata key
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.data.remove(key)
    }

    /// Get all metadata
    pub fn all(&self) -> &HashMap<String, Value> {
        &self.data
    }
}

/// Condition for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum RuleCondition {
    /// Match a value against a pattern
    #[serde(rename = "match")]
    Match {
        /// Path to the value in the context
        path: String,
        /// Pattern to match
        pattern: String,
    },

    /// Check if a value exists
    #[serde(rename = "exists")]
    Exists {
        /// Path to the value in the context
        path: String,
    },

    /// Compare two values
    #[serde(rename = "compare")]
    Compare {
        /// Path to the first value in the context
        path1: String,
        /// Path to the second value in the context
        path2: String,
        /// Comparison operator
        operator: String,
    },

    /// JavaScript expression condition
    #[serde(rename = "js")]
    JavaScript {
        /// JavaScript expression
        expression: String,
    },

    /// Custom condition with JSON configuration
    #[serde(rename = "custom")]
    Custom {
        /// Custom condition ID
        id: String,
        /// Custom condition configuration
        config: Value,
    },
}

/// Action for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum RuleAction {
    /// Modify the context
    #[serde(rename = "modify")]
    ModifyContext {
        /// Path to the value in the context
        path: String,
        /// New value
        value: Value,
    },

    /// Create a recovery point
    #[serde(rename = "recovery")]
    CreateRecoveryPoint {
        /// Name of the recovery point
        name: String,
        /// Description of the recovery point
        description: Option<String>,
    },

    /// Execute a transformation
    #[serde(rename = "transform")]
    ExecuteTransformation {
        /// Transformation ID
        id: String,
        /// Transformation input path
        input_path: String,
        /// Transformation output path
        output_path: String,
        /// Transformation configuration
        config: Option<Value>,
    },

    /// Execute a command
    #[serde(rename = "command")]
    ExecuteCommand {
        /// Command to execute
        command: String,
        /// Command arguments
        args: Option<Vec<String>>,
        /// Working directory
        working_dir: Option<String>,
    },

    /// Call an API endpoint
    #[serde(rename = "api")]
    CallApi {
        /// API endpoint URL
        url: String,
        /// HTTP method
        method: String,
        /// Request headers
        headers: Option<HashMap<String, String>>,
        /// Request body
        body: Option<Value>,
        /// Response path for storing the result
        response_path: Option<String>,
    },

    /// Log a message
    #[serde(rename = "log")]
    LogMessage {
        /// Log level
        level: String,
        /// Message to log
        message: String,
    },

    /// Notify the user
    #[serde(rename = "notify")]
    NotifyUser {
        /// Notification title
        title: String,
        /// Notification message
        message: String,
        /// Notification level (info, warning, error)
        level: String,
    },

    /// Custom action with JSON configuration
    #[serde(rename = "custom")]
    Custom {
        /// Custom action ID
        id: String,
        /// Custom action configuration
        config: Value,
    },
}

/// A rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for the rule
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule version
    pub version: String,
    /// Rule category
    pub category: String,
    /// Rule priority (lower is higher priority)
    pub priority: i32,
    /// Rule pattern(s) for matching
    pub patterns: Vec<String>,
    /// Rule conditions
    pub conditions: Vec<RuleCondition>,
    /// Rule actions
    pub actions: Vec<RuleAction>,
    /// Rule metadata
    #[serde(default)]
    pub metadata: RuleMetadata,
}

impl Rule {
    /// Create a new rule using the builder pattern
    pub fn builder() -> super::RuleBuilder {
        super::RuleBuilder::new()
    }

    /// Get the rule ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the rule name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the rule description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the rule version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the rule category
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Get the rule priority
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// Get the rule patterns
    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }

    /// Get the rule conditions
    pub fn conditions(&self) -> &[RuleCondition] {
        &self.conditions
    }

    /// Get the rule actions
    pub fn actions(&self) -> &[RuleAction] {
        &self.actions
    }

    /// Get the rule metadata
    pub fn metadata(&self) -> &RuleMetadata {
        &self.metadata
    }

    /// Get mutable metadata
    pub fn metadata_mut(&mut self) -> &mut RuleMetadata {
        &mut self.metadata
    }
}
