//! Tests for rule manager

use super::*;
use crate::actions::ActionExecutor;
use crate::directory::{RuleDirectoryConfig, RuleDirectoryManager};
use crate::evaluator::RuleEvaluator;
use crate::models::Rule;
use crate::parser::{ParserConfig, RuleParser};
use crate::repository::RuleRepository;
use std::sync::Arc;
use tempfile::TempDir;

/// Helper to create test manager with dependencies
async fn create_test_manager() -> (RuleManager, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let dir_config = RuleDirectoryConfig {
        root_directory: temp_dir.path().to_path_buf(),
        default_extension: "yaml".to_string(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        watch_for_changes: false,
        recursion_depth: 10,
    };
    let dir_manager = RuleDirectoryManager::new(dir_config);

    let parser_config = ParserConfig::default();
    let parser = RuleParser::new(parser_config);

    let repository = Arc::new(RuleRepository::new(dir_manager, parser));
    let evaluator = Arc::new(RuleEvaluator::new());
    let action_executor = Arc::new(ActionExecutor::new());

    let manager = RuleManager::new(repository, evaluator, action_executor);

    (manager, temp_dir)
}

/// Helper to create a test rule
fn create_test_rule(id: &str) -> Rule {
    Rule::new(id)
        .with_name(format!("Test Rule {id}"))
        .with_description(format!("Description for {id}"))
        .with_category("test")
        .with_priority(100)
}

#[tokio::test]
async fn test_manager_creation() {
    let (_manager, _temp_dir) = create_test_manager().await;

    // Manager should be created successfully
    // No panics = success
}

#[tokio::test]
async fn test_manager_initialize() {
    let (manager, _temp_dir) = create_test_manager().await;

    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_rule_via_manager() {
    let (manager, _temp_dir) = create_test_manager().await;
    manager.initialize().await.expect("Failed to initialize");

    let rule = create_test_rule("test_rule_1");

    let result = manager.add_rule(rule.clone()).await;
    assert!(result.is_ok());

    // Verify rule was added
    let retrieved = manager
        .get_rule("test_rule_1")
        .await
        .expect("Failed to get rule");
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_update_rule_via_manager() {
    let (manager, _temp_dir) = create_test_manager().await;
    manager.initialize().await.expect("Failed to initialize");

    let mut rule = create_test_rule("test_rule_1");
    manager.add_rule(rule.clone()).await.expect("Failed to add");

    // Update the rule
    rule.name = "Updated Name".to_string();
    let result = manager.update_rule(rule.clone()).await;
    assert!(result.is_ok());

    // Verify update
    let retrieved = manager
        .get_rule("test_rule_1")
        .await
        .expect("Failed to get")
        .expect("Rule should exist");
    assert_eq!(retrieved.name, "Updated Name");
}

#[tokio::test]
async fn test_remove_rule_via_manager() {
    let (manager, _temp_dir) = create_test_manager().await;
    manager.initialize().await.expect("Failed to initialize");

    let rule = create_test_rule("test_rule_1");
    manager.add_rule(rule).await.expect("Failed to add");

    let result = manager.remove_rule("test_rule_1").await;
    assert!(result.is_ok());

    // Verify removal
    let retrieved = manager
        .get_rule("test_rule_1")
        .await
        .expect("Failed to get");
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_get_all_rules_via_manager() {
    let (manager, _temp_dir) = create_test_manager().await;
    manager.initialize().await.expect("Failed to initialize");

    manager
        .add_rule(create_test_rule("rule_1"))
        .await
        .expect("Failed");
    manager
        .add_rule(create_test_rule("rule_2"))
        .await
        .expect("Failed");
    manager
        .add_rule(create_test_rule("rule_3"))
        .await
        .expect("Failed");

    let all_rules = manager.get_all_rules().await.expect("Failed to get all");
    assert_eq!(all_rules.len(), 3);
}

#[tokio::test]
async fn test_get_rules_by_category_via_manager() {
    let (manager, _temp_dir) = create_test_manager().await;
    manager.initialize().await.expect("Failed to initialize");

    let rule1 = Rule::new("rule_1").with_category("category_a");
    let rule2 = Rule::new("rule_2").with_category("category_a");
    let rule3 = Rule::new("rule_3").with_category("category_b");

    manager.add_rule(rule1).await.expect("Failed");
    manager.add_rule(rule2).await.expect("Failed");
    manager.add_rule(rule3).await.expect("Failed");

    let category_a_rules = manager
        .get_rules_by_category("category_a")
        .await
        .expect("Failed to get category rules");
    assert_eq!(category_a_rules.len(), 2);
}

#[tokio::test]
async fn test_manager_with_dependencies() {
    let (manager, _temp_dir) = create_test_manager().await;
    manager.initialize().await.expect("Failed to initialize");

    // Rule with dependencies
    let rule1 = Rule::new("rule_1").with_name("Base Rule");
    let rule2 = Rule::new("rule_2")
        .with_name("Dependent Rule")
        .with_dependency("rule_1");

    manager.add_rule(rule1).await.expect("Failed to add rule 1");
    manager.add_rule(rule2).await.expect("Failed to add rule 2");

    // Both rules should be present
    let all_rules = manager.get_all_rules().await.expect("Failed to get all");
    assert_eq!(all_rules.len(), 2);
}

#[tokio::test]
async fn test_manager_multiple_operations() {
    let (manager, _temp_dir) = create_test_manager().await;
    manager.initialize().await.expect("Failed to initialize");

    // Add multiple rules
    for i in 1..=5 {
        let rule = create_test_rule(&format!("rule_{i}"));
        manager.add_rule(rule).await.expect("Failed to add");
    }

    // Verify count
    let all_rules = manager.get_all_rules().await.expect("Failed to get all");
    assert_eq!(all_rules.len(), 5);

    // Remove some rules
    manager
        .remove_rule("rule_2")
        .await
        .expect("Failed to remove");
    manager
        .remove_rule("rule_4")
        .await
        .expect("Failed to remove");

    // Verify new count
    let all_rules = manager.get_all_rules().await.expect("Failed to get all");
    assert_eq!(all_rules.len(), 3);
}

#[tokio::test]
async fn test_manager_rule_execution_order() {
    let (manager, _temp_dir) = create_test_manager().await;
    manager.initialize().await.expect("Failed to initialize");

    // Add rules with different priorities
    let rule_high = Rule::new("rule_high").with_priority(1);
    let rule_mid = Rule::new("rule_mid").with_priority(50);
    let rule_low = Rule::new("rule_low").with_priority(100);

    manager.add_rule(rule_low).await.expect("Failed");
    manager.add_rule(rule_high).await.expect("Failed");
    manager.add_rule(rule_mid).await.expect("Failed");

    let all_rules = manager.get_all_rules().await.expect("Failed to get all");
    assert_eq!(all_rules.len(), 3);

    // Rules should all be present regardless of add order
    assert!(all_rules.iter().any(|r| r.id == "rule_high"));
    assert!(all_rules.iter().any(|r| r.id == "rule_mid"));
    assert!(all_rules.iter().any(|r| r.id == "rule_low"));
}
