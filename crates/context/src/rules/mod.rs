//! Rule System for Context Management
//!
//! This module provides the Rule System functionality for the Context Management System.
//! It enables defining, discovering, evaluating, and applying rules to context data.
//!
//! The Rule System consists of several key components:
//!
//! 1. **Rule Models** - Core data structures for rules, conditions, and actions
//! 2. **Rule Parser** - Parser for MDC/YAML rule format
//! 3. **Rule Repository** - Storage and indexing for rules
//! 4. **Rule Manager** - Management of rule dependencies and lifecycle
//! 5. **Rule Evaluator** - Evaluation of rules against context data
//! 6. **Rule Actions** - Execution of rule-based actions

mod models;
mod error;
mod directory;
mod repository;
mod evaluator;
mod parser;
mod actions;
mod plugin;
mod tests;

use async_trait::async_trait;
use std::sync::Arc;
use std::path::Path;
use serde_json::Value;

use squirrel_interfaces::context::ContextTransformation;
use anyhow::Result as AnyhowResult;

// Re-export public items
pub use self::error::{RuleError, Result};
pub use self::models::{Rule, RuleCondition, RuleAction, RuleMetadata};
pub use self::directory::RuleDirectoryManager;
pub use self::repository::RuleRepository;
pub use self::evaluator::RuleEvaluator;
pub use self::actions::ActionExecutor;
pub use self::plugin::RulePluginManager;

/// Dummy plugin manager for testing purposes
#[derive(Debug, Default)]
pub struct DummyPluginManager {
    // Add fields as needed for your tests
}

#[async_trait]
impl ContextPlugin for DummyPluginManager {
    async fn get_transformation(&self, id: &str) -> AnyhowResult<Arc<dyn ContextTransformation>> {
        Err(anyhow::anyhow!("Transformation not found: {}", id))
    }

    async fn get_adapter(&self, id: &str) -> AnyhowResult<Arc<dyn ContextAdapter>> {
        Err(anyhow::anyhow!("Adapter not found: {}", id))
    }

    async fn get_transformations(&self) -> AnyhowResult<Vec<Arc<dyn ContextTransformation>>> {
        Ok(Vec::new())
    }

    async fn get_adapters(&self) -> AnyhowResult<Vec<Arc<dyn ContextAdapter>>> {
        Ok(Vec::new())
    }
}

/// Minimal ContextPlugin trait for testing purposes
#[async_trait]
pub trait ContextPlugin: Send + Sync + std::fmt::Debug {
    async fn get_transformation(&self, id: &str) -> AnyhowResult<Arc<dyn ContextTransformation>>;
    
    async fn get_adapter(&self, id: &str) -> AnyhowResult<Arc<dyn ContextAdapter>>;
    
    async fn get_transformations(&self) -> AnyhowResult<Vec<Arc<dyn ContextTransformation>>>;
    
    async fn get_adapters(&self) -> AnyhowResult<Vec<Arc<dyn ContextAdapter>>>;
}

/// Minimal ContextAdapter trait for testing purposes
#[async_trait]
pub trait ContextAdapter: Send + Sync + std::fmt::Debug {
    fn get_id(&self) -> &str;
    
    fn get_name(&self) -> &str;
    
    fn get_description(&self) -> &str;
    
    async fn adapt(&self, data: serde_json::Value) -> AnyhowResult<serde_json::Value>;
}

/// Rule builder for creating rules
#[derive(Debug, Default)]
pub struct RuleBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    category: Option<String>,
    priority: Option<i32>,
    patterns: Vec<String>,
    conditions: Vec<RuleCondition>,
    actions: Vec<RuleAction>,
    metadata: RuleMetadata,
}

impl RuleBuilder {
    /// Create a new rule builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the rule ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    
    /// Set the rule name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Set the rule description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Set the rule version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
    
    /// Set the rule category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
    
    /// Set the rule priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = Some(priority);
        self
    }
    
    /// Add a pattern to the rule
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }
    
    /// Add multiple patterns to the rule
    pub fn with_patterns(mut self, patterns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for pattern in patterns {
            self.patterns.push(pattern.into());
        }
        self
    }
    
    /// Add a condition to the rule
    pub fn with_condition(mut self, condition: RuleCondition) -> Self {
        self.conditions.push(condition);
        self
    }
    
    /// Add an action to the rule
    pub fn with_action(mut self, action: RuleAction) -> Self {
        self.actions.push(action);
        self
    }
    
    /// Set the rule metadata
    pub fn with_metadata(mut self, metadata: RuleMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Build the rule
    pub fn build(self) -> Result<Rule> {
        // Validate required fields
        let id = self.id.ok_or_else(|| RuleError::ValidationError("Rule ID is required".to_string()))?;
        let name = self.name.unwrap_or_else(|| id.clone());
        let description = self.description.unwrap_or_default();
        let version = self.version.unwrap_or_else(|| "1.0.0".to_string());
        let category = self.category.unwrap_or_else(|| "default".to_string());
        let priority = self.priority.unwrap_or(100);
        
        // Create the rule
        Ok(Rule {
            id,
            name,
            description,
            version,
            category,
            priority,
            patterns: self.patterns,
            conditions: self.conditions,
            actions: self.actions,
            metadata: self.metadata,
        })
    }
}

/// Rule example for documentation and testing
#[derive(Debug)]
pub struct RuleExample {
    /// Example rule
    pub rule: Rule,
    /// Example context that matches the rule
    pub matching_context: serde_json::Value,
    /// Example context that doesn't match the rule
    pub non_matching_context: Option<serde_json::Value>,
    /// Expected output after applying the rule to the matching context
    pub expected_output: Option<serde_json::Value>,
}

/// Rule manager for high-level rule operations
#[derive(Debug)]
pub struct RuleManager {
    repository: Arc<RuleRepository>,
    evaluator: Arc<RuleEvaluator>,
    action_executor: ActionExecutor,
}

impl RuleManager {
    /// Create a new rule manager
    pub fn new(rules_dir: impl AsRef<Path>) -> Self {
        let rules_dir_str = rules_dir.as_ref().to_string_lossy().to_string();
        let repository = Arc::new(RuleRepository::new(rules_dir_str));
        let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(DummyPluginManager::default())));
        let evaluator = Arc::new(RuleEvaluator::new());
        let action_executor = ActionExecutor::new(Arc::clone(&plugin_manager));
        
        Self {
            repository,
            evaluator,
            action_executor,
        }
    }
    
    /// Initialize the rule manager
    pub async fn initialize(&self) -> Result<()> {
        self.repository.initialize().await
    }
    
    /// Get a rule by ID
    pub async fn get_rule(&self, id: &str) -> Result<Option<Arc<Rule>>> {
        self.repository.get_rule(id).await
    }
    
    /// Add or update a rule
    pub async fn add_or_update_rule(&self, rule: Rule) -> Result<()> {
        if self.repository.get_rule(&rule.id).await?.is_some() {
            self.repository.update_rule(rule).await
        } else {
            self.repository.add_rule(rule).await
        }
    }
    
    /// Remove a rule
    pub async fn remove_rule(&self, id: &str) -> Result<()> {
        self.repository.remove_rule(id).await
    }
    
    /// Apply rules to a context
    pub async fn apply_rules(&self, context: &mut Value) -> Result<actions::RuleApplicationResult> {
        // Get all rules
        let rules = self.repository.get_all_rules().await?;
        
        // Find matching rules
        let matching_rules = self.evaluator.find_matching_rules(&rules, context).await?;
        
        if matching_rules.is_empty() {
            return Ok(actions::RuleApplicationResult { rules_applied: Vec::new() });
        }
        
        // Apply rules
        let mut rules_applied = Vec::new();
        
        for rule in matching_rules {
            self.action_executor.execute_rule_actions(&rule, Some(context)).await?;
            
            rules_applied.push(actions::AppliedRule {
                id: rule.id().to_string(),
                name: rule.name().to_string(),
                version: rule.version().to_string(),
                category: rule.category().to_string(),
                applied_at: chrono::Utc::now(),
            });
        }
        
        Ok(actions::RuleApplicationResult { rules_applied })
    }
    
    /// Reload rules from disk
    pub async fn reload(&self) -> Result<()> {
        self.repository.reload().await
    }
} 