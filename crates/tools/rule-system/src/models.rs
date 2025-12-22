//! Models for the rule system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

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
    /// Rule dependencies
    pub dependencies: Vec<String>,
    /// Rule metadata
    pub metadata: HashMap<String, Value>,
    /// Path to the rule file (not serialized)
    #[serde(skip)]
    pub file_path: Option<PathBuf>,
    /// When the rule was created
    pub created_at: Option<DateTime<Utc>>,
    /// When the rule was last updated
    pub updated_at: Option<DateTime<Utc>>,
}

impl Rule {
    /// Creates a new rule with the given ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            description: String::new(),
            version: "1.0.0".to_string(),
            category: "default".to_string(),
            priority: 100,
            patterns: Vec::new(),
            conditions: Vec::new(),
            actions: Vec::new(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
            file_path: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }

    /// Sets the rule name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the rule description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the rule version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets the rule category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    /// Sets the rule priority
    #[must_use]
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Adds a pattern to the rule
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    /// Adds a condition to the rule
    #[must_use]
    pub fn with_condition(mut self, condition: RuleCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Adds an action to the rule
    #[must_use]
    pub fn with_action(mut self, action: RuleAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Adds a dependency to the rule
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Adds metadata to the rule
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Sets the file path for the rule
    pub fn with_file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }
}

/// Condition type for rule conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum RuleCondition {
    /// Condition that checks if a value equals another value
    Equals {
        /// Path to the value in the context
        path: String,
        /// Value to compare against
        value: Value,
    },

    /// Condition that checks if a value matches a pattern
    Matches {
        /// Path to the value in the context
        path: String,
        /// Pattern to match against
        pattern: String,
    },

    /// Condition that checks if a value is greater than another value
    GreaterThan {
        /// Path to the value in the context
        path: String,
        /// Value to compare against
        value: Value,
    },

    /// Condition that checks if a value is less than another value
    LessThan {
        /// Path to the value in the context
        path: String,
        /// Value to compare against
        value: Value,
    },

    /// Condition that checks if a value exists
    Exists {
        /// Path to the value in the context
        path: String,
    },

    /// Condition that checks if a value does not exist
    NotExists {
        /// Path to the value in the context
        path: String,
    },

    /// Condition that checks if all sub-conditions are true
    All {
        /// Sub-conditions to check
        conditions: Vec<RuleCondition>,
    },

    /// Condition that checks if any sub-condition is true
    Any {
        /// Sub-conditions to check
        conditions: Vec<RuleCondition>,
    },

    /// Condition that checks if a sub-condition is not true
    Not {
        /// Sub-condition to check
        condition: Box<RuleCondition>,
    },

    /// Condition that uses a plugin for evaluation
    Plugin {
        /// Plugin ID
        plugin_id: String,
        /// Configuration for the plugin
        config: Value,
    },

    /// Condition that uses a custom script for evaluation
    Script {
        /// Script content
        script: String,
        /// Script language (e.g., "js", "py")
        language: String,
    },
}

/// Action type for rule actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum RuleAction {
    /// Action that modifies the context
    ModifyContext {
        /// Path to the value in the context
        path: String,
        /// New value to set
        value: Value,
    },

    /// Action that creates a recovery point
    CreateRecoveryPoint {
        /// Description of the recovery point
        description: String,
    },

    /// Action that executes a transformation on the context
    ExecuteTransformation {
        /// Transformation ID
        transformation_id: String,
        /// Configuration for the transformation
        config: Option<Value>,
    },

    /// Action that notifies an external system
    Notify {
        /// Notification channel (e.g., "email", "slack")
        channel: String,
        /// Message to send
        message: String,
        /// Additional data for the notification
        data: Option<Value>,
    },

    /// Action that executes a plugin
    ExecutePlugin {
        /// Plugin ID
        plugin_id: String,
        /// Configuration for the plugin
        config: Value,
    },

    /// Action that executes a custom script
    ExecuteScript {
        /// Script content
        script: String,
        /// Script language (e.g., "js", "py")
        language: String,
    },

    /// Action that validates the context
    ValidateContext {
        /// Schema to validate against
        schema: Value,
    },
}

/// Result of a rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// ID of the rule that was evaluated
    pub rule_id: String,
    /// ID of the context that was evaluated
    pub context_id: String,
    /// Whether the rule matched
    pub matches: bool,
    /// When the evaluation was performed
    pub timestamp: DateTime<Utc>,
}

/// Result of a rule action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// ID of the action that was executed
    pub action_id: String,
    /// ID of the rule that triggered the action
    pub rule_id: String,
    /// ID of the context that was processed
    pub context_id: String,
    /// Whether the action was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
    /// Optional data associated with the result
    pub data: Option<Value>,
    /// When the action was executed
    pub timestamp: DateTime<Utc>,
}

/// Metadata for a rule source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSourceMetadata {
    /// Path to the rule directory
    pub directory: PathBuf,
    /// Pattern for matching rule files
    pub pattern: String,
    /// Whether to watch for changes
    pub watch: bool,
}
