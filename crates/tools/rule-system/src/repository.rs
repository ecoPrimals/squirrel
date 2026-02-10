// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Rule repository for storing and indexing rules

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::directory::RuleDirectoryManager;
use crate::error::{RuleRepositoryError, RuleSystemError, RuleSystemResult};
use crate::models::Rule;
use crate::parser::RuleParser;

/// Repository for storing and managing rules
#[derive(Debug)]
pub struct RuleRepository {
    /// Rules indexed by ID
    rules: Arc<RwLock<HashMap<String, Rule>>>,
    /// Rules indexed by category
    categories: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Rule patterns for quick lookup
    patterns: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Directory manager for file operations
    directory_manager: RuleDirectoryManager,
    /// Parser for rule files
    parser: RuleParser,
    /// Last update timestamp
    last_update: Arc<RwLock<DateTime<Utc>>>,
}

impl RuleRepository {
    /// Create a new rule repository
    #[must_use]
    pub fn new(directory_manager: RuleDirectoryManager, parser: RuleParser) -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
            directory_manager,
            parser,
            last_update: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Initialize the repository by loading all rules from directories
    pub async fn initialize(&self) -> RuleSystemResult<()> {
        // Initialize directory manager
        self.directory_manager.initialize().await?;

        // Load all rules
        self.load_all_rules().await?;

        Ok(())
    }

    /// Load all rules from all sources
    pub async fn load_all_rules(&self) -> RuleSystemResult<()> {
        // Get all rule files
        let rule_files = self.directory_manager.get_all_rule_files().await?;

        // Clear existing rules
        self.rules.write().await.clear();
        self.categories.write().await.clear();
        self.patterns.write().await.clear();

        // Load each rule file
        for file_path in rule_files {
            match self.load_rule_from_file(&file_path).await {
                Ok(rule) => {
                    self.add_rule_to_index(rule).await?;
                }
                Err(e) => {
                    tracing::warn!("Failed to load rule from {}: {}", file_path.display(), e);
                }
            }
        }

        // Update last update timestamp
        *self.last_update.write().await = Utc::now();

        Ok(())
    }

    /// Load a single rule from a file
    async fn load_rule_from_file(&self, file_path: &PathBuf) -> RuleSystemResult<Rule> {
        self.parser.parse_rule_file(file_path).await
    }

    /// Add a rule to the repository
    pub async fn add_rule(&self, rule: Rule) -> RuleSystemResult<()> {
        // Check if rule already exists
        if self.rules.read().await.contains_key(&rule.id) {
            return Err(RuleSystemError::RepositoryError(
                RuleRepositoryError::RuleAlreadyExists(rule.id.clone()),
            ));
        }

        // Add to index
        self.add_rule_to_index(rule).await?;

        // Update last update timestamp
        *self.last_update.write().await = Utc::now();

        Ok(())
    }

    /// Add a rule to internal indexes
    async fn add_rule_to_index(&self, rule: Rule) -> RuleSystemResult<()> {
        let rule_id = rule.id.clone();
        let rule_category = rule.category.clone();
        let rule_patterns = rule.patterns.clone();

        // Add to rules index
        self.rules.write().await.insert(rule_id.clone(), rule);

        // Add to category index
        self.categories
            .write()
            .await
            .entry(rule_category)
            .or_insert_with(Vec::new)
            .push(rule_id.clone());

        // Add to patterns index
        for pattern in rule_patterns {
            self.patterns
                .write()
                .await
                .entry(pattern)
                .or_insert_with(Vec::new)
                .push(rule_id.clone());
        }

        Ok(())
    }

    /// Get a rule by ID
    pub async fn get_rule(&self, id: &str) -> RuleSystemResult<Option<Rule>> {
        Ok(self.rules.read().await.get(id).cloned())
    }

    /// Get all rules
    pub async fn get_all_rules(&self) -> RuleSystemResult<Vec<Rule>> {
        Ok(self.rules.read().await.values().cloned().collect())
    }

    /// Get rules by category
    pub async fn get_rules_by_category(&self, category: &str) -> RuleSystemResult<Vec<Rule>> {
        let rules = self.rules.read().await;
        let categories = self.categories.read().await;

        if let Some(rule_ids) = categories.get(category) {
            let mut result = Vec::new();
            for rule_id in rule_ids {
                if let Some(rule) = rules.get(rule_id) {
                    result.push(rule.clone());
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get rules by pattern
    pub async fn get_rules_by_pattern(&self, pattern: &str) -> RuleSystemResult<Vec<Rule>> {
        let rules = self.rules.read().await;
        let patterns = self.patterns.read().await;

        if let Some(rule_ids) = patterns.get(pattern) {
            let mut result = Vec::new();
            for rule_id in rule_ids {
                if let Some(rule) = rules.get(rule_id) {
                    result.push(rule.clone());
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get rules that match a context ID
    pub async fn get_matching_rules(&self, context_id: &str) -> RuleSystemResult<Vec<Rule>> {
        let rules = self.rules.read().await;
        let mut matching_rules = Vec::new();

        for rule in rules.values() {
            if crate::utils::rule_matches_context(rule, context_id) {
                matching_rules.push(rule.clone());
            }
        }

        // Sort by priority (lower values first)
        matching_rules.sort_by(|a, b| a.priority.cmp(&b.priority));

        Ok(matching_rules)
    }

    /// Update a rule
    pub async fn update_rule(&self, rule: Rule) -> RuleSystemResult<()> {
        let rule_id = rule.id.clone();

        // Check if rule exists
        if !self.rules.read().await.contains_key(&rule_id) {
            return Err(RuleSystemError::RepositoryError(
                RuleRepositoryError::RuleNotFound(rule_id),
            ));
        }

        // Remove from indexes
        self.remove_rule_from_index(&rule_id).await?;

        // Add back to indexes
        self.add_rule_to_index(rule).await?;

        // Update last update timestamp
        *self.last_update.write().await = Utc::now();

        Ok(())
    }

    /// Remove a rule
    pub async fn remove_rule(&self, id: &str) -> RuleSystemResult<()> {
        // Check if rule exists
        if !self.rules.read().await.contains_key(id) {
            return Err(RuleSystemError::RepositoryError(
                RuleRepositoryError::RuleNotFound(id.to_string()),
            ));
        }

        // Remove from indexes
        self.remove_rule_from_index(id).await?;

        // Update last update timestamp
        *self.last_update.write().await = Utc::now();

        Ok(())
    }

    /// Remove a rule from internal indexes
    async fn remove_rule_from_index(&self, rule_id: &str) -> RuleSystemResult<()> {
        // Get rule to find its category and patterns
        let rule = self.rules.read().await.get(rule_id).cloned();

        if let Some(rule) = rule {
            // Remove from rules index
            self.rules.write().await.remove(rule_id);

            // Remove from category index
            if let Some(category_rules) = self.categories.write().await.get_mut(&rule.category) {
                category_rules.retain(|id| id != rule_id);
            }

            // Remove from patterns index
            for pattern in &rule.patterns {
                if let Some(pattern_rules) = self.patterns.write().await.get_mut(pattern) {
                    pattern_rules.retain(|id| id != rule_id);
                }
            }
        }

        Ok(())
    }

    /// Get all categories
    pub async fn get_categories(&self) -> RuleSystemResult<Vec<String>> {
        Ok(self.categories.read().await.keys().cloned().collect())
    }

    /// Get all patterns
    pub async fn get_patterns(&self) -> RuleSystemResult<Vec<String>> {
        Ok(self.patterns.read().await.keys().cloned().collect())
    }

    /// Get repository statistics
    pub async fn get_statistics(&self) -> RuleSystemResult<RepositoryStatistics> {
        let rules = self.rules.read().await;
        let categories = self.categories.read().await;
        let patterns = self.patterns.read().await;
        let last_update = *self.last_update.read().await;

        Ok(RepositoryStatistics {
            total_rules: rules.len(),
            total_categories: categories.len(),
            total_patterns: patterns.len(),
            last_update,
        })
    }

    /// Reload rules from disk
    pub async fn reload(&self) -> RuleSystemResult<()> {
        self.load_all_rules().await
    }
}

/// Repository statistics
#[derive(Debug, Clone)]
pub struct RepositoryStatistics {
    /// Total number of rules
    pub total_rules: usize,
    /// Total number of categories
    pub total_categories: usize,
    /// Total number of patterns
    pub total_patterns: usize,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}

/// Create a new rule repository with default configuration
pub fn create_rule_repository() -> RuleSystemResult<RuleRepository> {
    let directory_manager = RuleDirectoryManager::default();
    let parser = RuleParser::default();

    Ok(RuleRepository::new(directory_manager, parser))
}

/// Create a rule repository with custom configuration
pub fn create_rule_repository_with_config(
    directory_manager: RuleDirectoryManager,
    parser: RuleParser,
) -> RuleSystemResult<RuleRepository> {
    Ok(RuleRepository::new(directory_manager, parser))
}
