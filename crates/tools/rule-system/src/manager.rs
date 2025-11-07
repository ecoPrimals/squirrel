//! Rule manager for high-level rule operations

use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::actions::ActionExecutor;
use crate::error::{RuleManagerError, RuleSystemError, RuleSystemResult};
use crate::evaluator::RuleEvaluator;
use crate::models::{ActionResult, EvaluationResult, Rule};
use crate::repository::RuleRepository;

/// Rule manager for high-level operations
#[derive(Debug)]
pub struct RuleManager {
    /// Rule repository
    repository: Arc<RuleRepository>,
    /// Rule evaluator
    evaluator: Arc<RuleEvaluator>,
    /// Action executor
    action_executor: Arc<ActionExecutor>,
    /// Dependency cache
    dependency_cache: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Active rules cache
    active_rules: Arc<RwLock<HashSet<String>>>,
}

impl RuleManager {
    /// Create a new rule manager
    #[must_use] 
    pub fn new(
        repository: Arc<RuleRepository>,
        evaluator: Arc<RuleEvaluator>,
        action_executor: Arc<ActionExecutor>,
    ) -> Self {
        Self {
            repository,
            evaluator,
            action_executor,
            dependency_cache: Arc::new(RwLock::new(HashMap::new())),
            active_rules: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Initialize the rule manager
    pub async fn initialize(&self) -> RuleSystemResult<()> {
        // Initialize repository
        self.repository.initialize().await?;

        // Initialize evaluator
        self.evaluator.initialize().await?;

        // Initialize action executor
        self.action_executor.initialize().await?;

        // Build dependency cache
        self.build_dependency_cache().await?;

        Ok(())
    }

    /// Build dependency cache for all rules
    async fn build_dependency_cache(&self) -> RuleSystemResult<()> {
        let rules = self.repository.get_all_rules().await?;
        let mut cache = HashMap::new();

        for rule in rules {
            let resolved_deps = self
                .resolve_dependencies(&rule.id, &rule.dependencies)
                .await?;
            cache.insert(rule.id.clone(), resolved_deps);
        }

        *self.dependency_cache.write().await = cache;

        Ok(())
    }

    /// Resolve dependencies for a rule
    async fn resolve_dependencies(
        &self,
        rule_id: &str,
        dependencies: &[String],
    ) -> RuleSystemResult<Vec<String>> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with direct dependencies
        for dep in dependencies {
            queue.push_back(dep.clone());
        }

        // Breadth-first search to resolve all dependencies
        while let Some(dep_id) = queue.pop_front() {
            if visited.contains(&dep_id) {
                continue;
            }

            visited.insert(dep_id.clone());

            // Check for circular dependencies
            if dep_id == rule_id {
                return Err(RuleSystemError::ManagerError(
                    RuleManagerError::DependencyError(format!(
                        "Circular dependency detected: {rule_id}"
                    )),
                ));
            }

            // Get the dependency rule
            if let Some(dep_rule) = self.repository.get_rule(&dep_id).await? {
                resolved.push(dep_id.clone());

                // Add transitive dependencies
                for transitive_dep in &dep_rule.dependencies {
                    if !visited.contains(transitive_dep) {
                        queue.push_back(transitive_dep.clone());
                    }
                }
            } else {
                return Err(RuleSystemError::ManagerError(
                    RuleManagerError::DependencyError(format!("Dependency not found: {dep_id}")),
                ));
            }
        }

        Ok(resolved)
    }

    /// Add a rule to the manager
    pub async fn add_rule(&self, rule: Rule) -> RuleSystemResult<()> {
        // Validate dependencies
        self.validate_dependencies(&rule.id, &rule.dependencies)
            .await?;

        // Add to repository
        self.repository.add_rule(rule.clone()).await?;

        // Update dependency cache
        let resolved_deps = self
            .resolve_dependencies(&rule.id, &rule.dependencies)
            .await?;
        self.dependency_cache
            .write()
            .await
            .insert(rule.id.clone(), resolved_deps);

        Ok(())
    }

    /// Update a rule
    pub async fn update_rule(&self, rule: Rule) -> RuleSystemResult<()> {
        // Validate dependencies
        self.validate_dependencies(&rule.id, &rule.dependencies)
            .await?;

        // Update in repository
        self.repository.update_rule(rule.clone()).await?;

        // Update dependency cache
        let resolved_deps = self
            .resolve_dependencies(&rule.id, &rule.dependencies)
            .await?;
        self.dependency_cache
            .write()
            .await
            .insert(rule.id.clone(), resolved_deps);

        // Rebuild cache for rules that depend on this one
        self.invalidate_dependent_cache(&rule.id).await?;

        Ok(())
    }

    /// Remove a rule
    pub async fn remove_rule(&self, id: &str) -> RuleSystemResult<()> {
        // Check if any rules depend on this one
        let dependent_rules = self.find_dependent_rules(id).await?;
        if !dependent_rules.is_empty() {
            return Err(RuleSystemError::ManagerError(
                RuleManagerError::DependencyError(format!(
                    "Cannot remove rule {} - it is depended upon by: {}",
                    id,
                    dependent_rules.join(", ")
                )),
            ));
        }

        // Remove from repository
        self.repository.remove_rule(id).await?;

        // Remove from dependency cache
        self.dependency_cache.write().await.remove(id);

        // Remove from active rules
        self.active_rules.write().await.remove(id);

        Ok(())
    }

    /// Find rules that depend on a given rule
    async fn find_dependent_rules(&self, rule_id: &str) -> RuleSystemResult<Vec<String>> {
        let rules = self.repository.get_all_rules().await?;
        let mut dependent_rules = Vec::new();

        for rule in rules {
            if rule.dependencies.contains(&rule_id.to_string()) {
                dependent_rules.push(rule.id);
            }
        }

        Ok(dependent_rules)
    }

    /// Validate dependencies for a rule
    async fn validate_dependencies(
        &self,
        rule_id: &str,
        dependencies: &[String],
    ) -> RuleSystemResult<()> {
        for dep_id in dependencies {
            // Check if dependency exists
            if self.repository.get_rule(dep_id).await?.is_none() {
                return Err(RuleSystemError::ManagerError(
                    RuleManagerError::DependencyError(format!("Dependency not found: {dep_id}")),
                ));
            }

            // Check for circular dependencies
            if dep_id == rule_id {
                return Err(RuleSystemError::ManagerError(
                    RuleManagerError::DependencyError(format!(
                        "Circular dependency detected: {rule_id}"
                    )),
                ));
            }

            // Check for indirect circular dependencies
            if let Some(dep_dependencies) = self.dependency_cache.read().await.get(dep_id) {
                if dep_dependencies.contains(&rule_id.to_string()) {
                    return Err(RuleSystemError::ManagerError(
                        RuleManagerError::DependencyError(format!(
                            "Circular dependency detected: {rule_id} -> {dep_id} -> {rule_id}"
                        )),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Invalidate dependency cache for rules that depend on a given rule
    async fn invalidate_dependent_cache(&self, rule_id: &str) -> RuleSystemResult<()> {
        let dependent_rules = self.find_dependent_rules(rule_id).await?;

        // Rebuild cache for dependent rules
        for dep_rule_id in dependent_rules {
            if let Some(dep_rule) = self.repository.get_rule(&dep_rule_id).await? {
                let resolved_deps = self
                    .resolve_dependencies(&dep_rule_id, &dep_rule.dependencies)
                    .await?;
                self.dependency_cache
                    .write()
                    .await
                    .insert(dep_rule_id, resolved_deps);
            }
        }

        Ok(())
    }

    /// Evaluate rules against a context
    pub async fn evaluate_rules(
        &self,
        context_id: &str,
        context_data: serde_json::Value,
    ) -> RuleSystemResult<Vec<EvaluationResult>> {
        // Get matching rules
        let rules = self.repository.get_matching_rules(context_id).await?;

        // Filter rules based on dependencies
        let executable_rules = self.filter_executable_rules(&rules).await?;

        // Evaluate rules
        let mut results = Vec::new();
        for rule in executable_rules {
            let result = self
                .evaluator
                .evaluate_rule(&rule, context_id, &context_data)
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Filter rules that can be executed (dependencies satisfied)
    async fn filter_executable_rules(&self, rules: &[Rule]) -> RuleSystemResult<Vec<Rule>> {
        let mut executable_rules = Vec::new();
        let active_rules = self.active_rules.read().await;

        for rule in rules {
            // Check if all dependencies are satisfied
            if let Some(dependencies) = self.dependency_cache.read().await.get(&rule.id) {
                let all_satisfied = dependencies.iter().all(|dep| active_rules.contains(dep));
                if all_satisfied {
                    executable_rules.push(rule.clone());
                }
            } else {
                // No dependencies, can execute
                executable_rules.push(rule.clone());
            }
        }

        Ok(executable_rules)
    }

    /// Execute actions for successful rule evaluations
    pub async fn execute_actions(
        &self,
        evaluation_results: &[EvaluationResult],
    ) -> RuleSystemResult<Vec<ActionResult>> {
        let mut action_results = Vec::new();

        for eval_result in evaluation_results {
            if eval_result.matches {
                // Get the rule to execute its actions
                if let Some(rule) = self.repository.get_rule(&eval_result.rule_id).await? {
                    // Mark rule as active
                    self.active_rules.write().await.insert(rule.id.clone());

                    // Execute actions
                    for action in &rule.actions {
                        let action_result = self
                            .action_executor
                            .execute_action(action, &eval_result.context_id, &rule.id)
                            .await?;
                        action_results.push(action_result);
                    }
                }
            }
        }

        Ok(action_results)
    }

    /// Process a context (evaluate rules and execute actions)
    pub async fn process_context(
        &self,
        context_id: &str,
        context_data: serde_json::Value,
    ) -> RuleSystemResult<ProcessingResult> {
        let start_time = Utc::now();

        // Evaluate rules
        let evaluation_results = self.evaluate_rules(context_id, context_data).await?;

        // Execute actions for successful evaluations
        let action_results = self.execute_actions(&evaluation_results).await?;

        let end_time = Utc::now();
        let processing_time = end_time - start_time;

        Ok(ProcessingResult {
            context_id: context_id.to_string(),
            evaluation_results,
            action_results,
            processing_time,
            timestamp: start_time,
        })
    }

    /// Get rule by ID
    pub async fn get_rule(&self, id: &str) -> RuleSystemResult<Option<Rule>> {
        self.repository.get_rule(id).await
    }

    /// Get all rules
    pub async fn get_all_rules(&self) -> RuleSystemResult<Vec<Rule>> {
        self.repository.get_all_rules().await
    }

    /// Get rules by category
    pub async fn get_rules_by_category(&self, category: &str) -> RuleSystemResult<Vec<Rule>> {
        self.repository.get_rules_by_category(category).await
    }

    /// Get manager statistics
    pub async fn get_statistics(&self) -> RuleSystemResult<ManagerStatistics> {
        let repository_stats = self.repository.get_statistics().await?;
        let active_rules_count = self.active_rules.read().await.len();
        let dependency_cache_size = self.dependency_cache.read().await.len();

        Ok(ManagerStatistics {
            total_rules: repository_stats.total_rules,
            total_categories: repository_stats.total_categories,
            total_patterns: repository_stats.total_patterns,
            active_rules_count,
            dependency_cache_size,
            last_update: repository_stats.last_update,
        })
    }

    /// Reload rules from disk
    pub async fn reload(&self) -> RuleSystemResult<()> {
        // Reload repository
        self.repository.reload().await?;

        // Rebuild dependency cache
        self.build_dependency_cache().await?;

        // Clear active rules
        self.active_rules.write().await.clear();

        Ok(())
    }

    /// Activate a rule
    pub async fn activate_rule(&self, rule_id: &str) -> RuleSystemResult<()> {
        // Check if rule exists
        if self.repository.get_rule(rule_id).await?.is_none() {
            return Err(RuleSystemError::ManagerError(
                RuleManagerError::RuleNotFound(rule_id.to_string()),
            ));
        }

        // Add to active rules
        self.active_rules.write().await.insert(rule_id.to_string());

        Ok(())
    }

    /// Deactivate a rule
    pub async fn deactivate_rule(&self, rule_id: &str) -> RuleSystemResult<()> {
        // Remove from active rules
        self.active_rules.write().await.remove(rule_id);

        Ok(())
    }

    /// Check if a rule is active
    pub async fn is_rule_active(&self, rule_id: &str) -> bool {
        self.active_rules.read().await.contains(rule_id)
    }

    /// Get active rules
    pub async fn get_active_rules(&self) -> Vec<String> {
        self.active_rules.read().await.iter().cloned().collect()
    }
}

/// Result of processing a context
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Context ID that was processed
    pub context_id: String,
    /// Results of rule evaluations
    pub evaluation_results: Vec<EvaluationResult>,
    /// Results of action executions
    pub action_results: Vec<ActionResult>,
    /// Time taken for processing
    pub processing_time: chrono::Duration,
    /// When processing started
    pub timestamp: DateTime<Utc>,
}

/// Manager statistics
#[derive(Debug, Clone)]
pub struct ManagerStatistics {
    /// Total number of rules
    pub total_rules: usize,
    /// Total number of categories
    pub total_categories: usize,
    /// Total number of patterns
    pub total_patterns: usize,
    /// Number of active rules
    pub active_rules_count: usize,
    /// Size of dependency cache
    pub dependency_cache_size: usize,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

/// Create a new rule manager with default configuration
pub async fn create_rule_manager() -> RuleSystemResult<RuleManager> {
    let repository = Arc::new(crate::repository::create_rule_repository()?);
    let evaluator = Arc::new(crate::evaluator::create_rule_evaluator()?);
    let action_executor = Arc::new(crate::actions::create_action_executor()?);

    Ok(RuleManager::new(repository, evaluator, action_executor))
}

/// Create a rule manager with custom components
#[must_use] 
pub fn create_rule_manager_with_components(
    repository: Arc<RuleRepository>,
    evaluator: Arc<RuleEvaluator>,
    action_executor: Arc<ActionExecutor>,
) -> RuleManager {
    RuleManager::new(repository, evaluator, action_executor)
}
