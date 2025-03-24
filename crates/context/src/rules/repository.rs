//! Rule repository for managing rules
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::{Result, RuleError};
use super::models::Rule;
use super::directory::RuleDirectoryManager;
use super::parser::{RuleParser, rule_to_mdc};

/// Rule repository for managing rules
#[derive(Debug)]
pub struct RuleRepository {
    /// Path to the rules directory
    rules_dir: PathBuf,
    /// Directory manager
    directory_manager: Arc<RuleDirectoryManager>,
    /// Map of rule ID to rule
    rules: Arc<RwLock<HashMap<String, Arc<Rule>>>>,
    /// Index of rules by category
    category_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Index of rules by pattern
    pattern_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl RuleRepository {
    /// Create a new rule repository
    pub fn new(rules_dir: impl Into<PathBuf>) -> Self {
        let rules_dir = rules_dir.into();
        let directory_manager = Arc::new(RuleDirectoryManager::new(&rules_dir));
        
        Self {
            rules_dir,
            directory_manager,
            rules: Arc::new(RwLock::new(HashMap::new())),
            category_index: Arc::new(RwLock::new(HashMap::new())),
            pattern_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize the repository
    pub async fn initialize(&self) -> Result<()> {
        // Create the rules directory if it doesn't exist
        RuleDirectoryManager::ensure_directory(&self.rules_dir).await?;
        
        // Load all rules
        self.load_all_rules().await?;
        
        Ok(())
    }
    
    /// Get the rules directory
    pub fn rules_dir(&self) -> &Path {
        &self.rules_dir
    }
    
    /// Get the directory manager
    pub fn directory_manager(&self) -> &Arc<RuleDirectoryManager> {
        &self.directory_manager
    }
    
    /// Load all rules from the rules directory
    pub async fn load_all_rules(&self) -> Result<()> {
        // Get all rule files
        let rule_files = self.directory_manager.list_rule_files(None).await?;
        
        // Load each rule
        for file in rule_files {
            let rule = RuleParser::parse_file(&file).await?;
            self.add_rule_to_indexes(Arc::new(rule)).await?;
        }
        
        Ok(())
    }
    
    /// Add a rule to the in-memory indexes
    async fn add_rule_to_indexes(&self, rule: Arc<Rule>) -> Result<()> {
        let rule_id = rule.id().to_string();
        let category = rule.category().to_string();
        let patterns = rule.patterns().to_vec();
        
        // Add to rules map
        {
            let mut rules = self.rules.write().await;
            if rules.contains_key(&rule_id) {
                return Err(RuleError::AlreadyExists(rule_id));
            }
            rules.insert(rule_id.clone(), Arc::clone(&rule));
        }
        
        // Add to category index
        {
            let mut category_index = self.category_index.write().await;
            let category_rules = category_index.entry(category).or_insert_with(Vec::new);
            category_rules.push(rule_id.clone());
        }
        
        // Add to pattern index
        {
            let mut pattern_index = self.pattern_index.write().await;
            for pattern in patterns {
                let pattern_rules = pattern_index.entry(pattern).or_insert_with(Vec::new);
                pattern_rules.push(rule_id.clone());
            }
        }
        
        Ok(())
    }
    
    /// Remove a rule from the in-memory indexes
    async fn remove_rule_from_indexes(&self, id: &str) -> Result<()> {
        // Get the rule first
        let rule = match self.get_rule(id).await? {
            Some(rule) => rule,
            None => return Err(RuleError::NotFound(id.to_string())),
        };
        
        let category = rule.category().to_string();
        let patterns = rule.patterns().to_vec();
        
        // Remove from rules map
        {
            let mut rules = self.rules.write().await;
            rules.remove(id);
        }
        
        // Remove from category index
        {
            let mut category_index = self.category_index.write().await;
            if let Some(category_rules) = category_index.get_mut(&category) {
                category_rules.retain(|r| r != id);
            }
        }
        
        // Remove from pattern index
        {
            let mut pattern_index = self.pattern_index.write().await;
            for pattern in patterns {
                if let Some(pattern_rules) = pattern_index.get_mut(&pattern) {
                    pattern_rules.retain(|r| r != id);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a rule by ID
    pub async fn get_rule(&self, id: &str) -> Result<Option<Arc<Rule>>> {
        let rules = self.rules.read().await;
        Ok(rules.get(id).cloned())
    }
    
    /// Add a rule
    pub async fn add_rule(&self, rule: Rule) -> Result<()> {
        // Create the rule file
        let content = rule_to_mdc(&rule)?;
        self.directory_manager.create_rule_file(&rule.category, &rule.id, &content).await?;
        
        // Add to indexes
        self.add_rule_to_indexes(Arc::new(rule)).await?;
        
        Ok(())
    }
    
    /// Update a rule
    pub async fn update_rule(&self, rule: Rule) -> Result<()> {
        // Check if the rule exists
        if self.get_rule(&rule.id).await?.is_none() {
            return Err(RuleError::NotFound(rule.id.clone()));
        }
        
        // Remove from indexes
        self.remove_rule_from_indexes(&rule.id).await?;
        
        // Update the rule file
        let content = rule_to_mdc(&rule)?;
        self.directory_manager.update_rule_file(&rule.category, &rule.id, &content).await?;
        
        // Add back to indexes
        self.add_rule_to_indexes(Arc::new(rule)).await?;
        
        Ok(())
    }
    
    /// Remove a rule
    pub async fn remove_rule(&self, id: &str) -> Result<()> {
        // Get the rule first
        let rule = match self.get_rule(id).await? {
            Some(rule) => rule,
            None => return Err(RuleError::NotFound(id.to_string())),
        };
        
        // Remove the rule file
        self.directory_manager.delete_rule_file(rule.category(), id).await?;
        
        // Remove from indexes
        self.remove_rule_from_indexes(id).await?;
        
        Ok(())
    }
    
    /// Get rules by category
    pub async fn get_rules_by_category(&self, category: &str) -> Result<Vec<Arc<Rule>>> {
        let category_index = self.category_index.read().await;
        let rules = self.rules.read().await;
        
        let rule_ids = match category_index.get(category) {
            Some(ids) => ids,
            None => return Ok(Vec::new()),
        };
        
        let mut result = Vec::new();
        for id in rule_ids {
            if let Some(rule) = rules.get(id) {
                result.push(Arc::clone(rule));
            }
        }
        
        Ok(result)
    }
    
    /// Get rules by pattern
    pub async fn get_rules_by_pattern(&self, pattern: &str) -> Result<Vec<Arc<Rule>>> {
        let pattern_index = self.pattern_index.read().await;
        let rules = self.rules.read().await;
        
        let rule_ids = match pattern_index.get(pattern) {
            Some(ids) => ids,
            None => return Ok(Vec::new()),
        };
        
        let mut result = Vec::new();
        for id in rule_ids {
            if let Some(rule) = rules.get(id) {
                result.push(Arc::clone(rule));
            }
        }
        
        Ok(result)
    }
    
    /// Match a pattern against rule patterns
    pub async fn match_pattern(&self, pattern: &str) -> Result<Vec<Arc<Rule>>> {
        let rules = self.rules.read().await;
        
        let mut result = Vec::new();
        for rule in rules.values() {
            for rule_pattern in rule.patterns() {
                // Only match rules where our pattern is matched by the rule's pattern
                // If input="test.special" and rule.pattern="test.*", rule should match
                // (The rule with pattern "test.*" applies to input "test.special")
                if self.pattern_matches(pattern, rule_pattern) {
                    result.push(Arc::clone(rule));
                    break;
                }
            }
        }
        
        // Sort by priority (lower is higher priority)
        result.sort_by_key(|r| r.priority());
        
        Ok(result)
    }
    
    /// Check if a pattern matches a rule pattern
    pub fn pattern_matches(&self, input: &str, pattern: &str) -> bool {
        // Special case: wildcard pattern
        if pattern == "*" {
            return true;
        }
        
        // Handle *.* pattern - matches any string containing a dot
        if pattern == "*.*" {
            return input.contains('.');
        }
        
        // Suffix wildcard: prefix*
        if pattern.ends_with("*") && !pattern.ends_with(".*") {
            let prefix = &pattern[0..pattern.len() - 1];
            return input.starts_with(prefix);
        }
        
        // Format test.* - matches test.anything
        if pattern.ends_with(".*") {
            let prefix = &pattern[0..pattern.len() - 1]; // Remove the *
            return input.starts_with(prefix);
        }
        
        // Prefix wildcard: *suffix
        if pattern.starts_with("*") {
            if let Some(suffix) = pattern.strip_prefix("*") {
                return input.ends_with(suffix);
            }
        }
        
        // Default: exact match
        pattern == input
    }
    
    /// Get all rules
    pub async fn get_all_rules(&self) -> Result<Vec<Arc<Rule>>> {
        let rules = self.rules.read().await;
        
        let mut result: Vec<_> = rules.values().cloned().collect();
        
        // Sort by ID for consistent ordering
        result.sort_by(|a, b| a.id().cmp(b.id()));
        
        Ok(result)
    }
    
    /// Get all categories
    pub async fn get_all_categories(&self) -> Result<Vec<String>> {
        let category_index = self.category_index.read().await;
        
        let mut result: Vec<_> = category_index.keys().cloned().collect();
        
        // Sort alphabetically
        result.sort();
        
        Ok(result)
    }
    
    /// Reload rules
    pub async fn reload(&self) -> Result<()> {
        // Clear indexes
        {
            let mut rules = self.rules.write().await;
            let mut category_index = self.category_index.write().await;
            let mut pattern_index = self.pattern_index.write().await;
            
            rules.clear();
            category_index.clear();
            pattern_index.clear();
        }
        
        // Load all rules
        self.load_all_rules().await?;
        
        Ok(())
    }
} 