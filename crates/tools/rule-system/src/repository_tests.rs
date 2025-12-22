//! Tests for rule repository

use super::*;
use crate::directory::RuleDirectoryManager;
use crate::models::Rule;
use crate::parser::RuleParser;
use tempfile::TempDir;

/// Helper to create a test repository
async fn create_test_repository() -> (RuleRepository, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let dir_config = crate::directory::RuleDirectoryConfig {
        root_directory: temp_dir.path().to_path_buf(),
        default_extension: "yaml".to_string(),
        include_patterns: vec![],
        exclude_patterns: vec![],
        watch_for_changes: false,
        recursion_depth: 10,
    };
    let dir_manager = RuleDirectoryManager::new(dir_config);

    let parser_config = crate::parser::ParserConfig::default();
    let parser = RuleParser::new(parser_config);

    let repo = RuleRepository::new(dir_manager, parser);
    (repo, temp_dir)
}

/// Helper to create a test rule
fn create_test_rule(id: &str) -> Rule {
    Rule::new(id)
        .with_name(format!("Test Rule {id}"))
        .with_description(format!("Description for {id}"))
        .with_category("test")
        .with_priority(100)
        .with_pattern("test_pattern")
}

#[tokio::test]
async fn test_repository_creation() {
    let (repo, _temp_dir) = create_test_repository().await;

    // Repository should be created successfully and empty
    let stats = repo.get_statistics().await.expect("Failed to get stats");
    assert_eq!(stats.total_rules, 0);

    let all_rules = repo.get_all_rules().await.expect("Failed to get rules");
    assert!(all_rules.is_empty());
}

#[tokio::test]
async fn test_add_rule() {
    let (repo, _temp_dir) = create_test_repository().await;
    let rule = create_test_rule("test_rule_1");

    // Add rule
    let result = repo.add_rule(rule.clone()).await;
    assert!(result.is_ok());

    // Verify rule was added
    let stats = repo.get_statistics().await.expect("Failed to get stats");
    assert_eq!(stats.total_rules, 1);

    let retrieved = repo
        .get_rule("test_rule_1")
        .await
        .expect("Failed to get rule");
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_add_duplicate_rule_fails() {
    let (repo, _temp_dir) = create_test_repository().await;
    let rule = create_test_rule("test_rule_1");

    // Add rule first time
    repo.add_rule(rule.clone())
        .await
        .expect("First add should succeed");

    // Try to add same rule again
    let result = repo.add_rule(rule).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_rule() {
    let (repo, _temp_dir) = create_test_repository().await;
    let rule = create_test_rule("test_rule_1");

    repo.add_rule(rule.clone())
        .await
        .expect("Failed to add rule");

    // Get rule by ID
    let retrieved = repo
        .get_rule("test_rule_1")
        .await
        .expect("Failed to get rule");
    assert!(retrieved.is_some());

    let retrieved_rule = retrieved.unwrap();
    assert_eq!(retrieved_rule.id, "test_rule_1");
    assert_eq!(retrieved_rule.name, "Test Rule test_rule_1");
}

#[tokio::test]
async fn test_get_nonexistent_rule() {
    let (repo, _temp_dir) = create_test_repository().await;

    let result = repo.get_rule("nonexistent").await.expect("Failed to query");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_all_rules() {
    let (repo, _temp_dir) = create_test_repository().await;

    // Add multiple rules
    repo.add_rule(create_test_rule("rule_1"))
        .await
        .expect("Failed to add rule 1");
    repo.add_rule(create_test_rule("rule_2"))
        .await
        .expect("Failed to add rule 2");
    repo.add_rule(create_test_rule("rule_3"))
        .await
        .expect("Failed to add rule 3");

    // Get all rules
    let all_rules = repo.get_all_rules().await.expect("Failed to get all rules");
    assert_eq!(all_rules.len(), 3);
}

#[tokio::test]
async fn test_get_rules_by_category() {
    let (repo, _temp_dir) = create_test_repository().await;

    // Add rules with different categories
    let rule1 = Rule::new("rule_1").with_category("category_a");
    let rule2 = Rule::new("rule_2").with_category("category_a");
    let rule3 = Rule::new("rule_3").with_category("category_b");

    repo.add_rule(rule1).await.expect("Failed to add rule 1");
    repo.add_rule(rule2).await.expect("Failed to add rule 2");
    repo.add_rule(rule3).await.expect("Failed to add rule 3");

    // Get rules by category
    let rules_in_category_a = repo
        .get_rules_by_category("category_a")
        .await
        .expect("Failed to get category_a rules");
    assert_eq!(rules_in_category_a.len(), 2);

    let rules_in_category_b = repo
        .get_rules_by_category("category_b")
        .await
        .expect("Failed to get category_b rules");
    assert_eq!(rules_in_category_b.len(), 1);
}

#[tokio::test]
async fn test_get_rules_by_pattern() {
    let (repo, _temp_dir) = create_test_repository().await;

    // Add rules with different patterns
    let rule1 = Rule::new("rule_1").with_pattern("pattern_x");
    let rule2 = Rule::new("rule_2").with_pattern("pattern_x");
    let rule3 = Rule::new("rule_3").with_pattern("pattern_y");

    repo.add_rule(rule1).await.expect("Failed to add rule 1");
    repo.add_rule(rule2).await.expect("Failed to add rule 2");
    repo.add_rule(rule3).await.expect("Failed to add rule 3");

    // Get rules by pattern
    let rules_with_pattern_x = repo
        .get_rules_by_pattern("pattern_x")
        .await
        .expect("Failed to get pattern_x rules");
    assert_eq!(rules_with_pattern_x.len(), 2);

    let rules_with_pattern_y = repo
        .get_rules_by_pattern("pattern_y")
        .await
        .expect("Failed to get pattern_y rules");
    assert_eq!(rules_with_pattern_y.len(), 1);
}

#[tokio::test]
async fn test_update_rule() {
    let (repo, _temp_dir) = create_test_repository().await;
    let mut rule = create_test_rule("test_rule_1");

    repo.add_rule(rule.clone())
        .await
        .expect("Failed to add rule");

    // Update rule
    rule.name = "Updated Name".to_string();
    let result = repo.update_rule(rule.clone()).await;
    assert!(result.is_ok());

    // Verify update
    let retrieved = repo
        .get_rule("test_rule_1")
        .await
        .expect("Failed to get rule")
        .expect("Rule should exist");
    assert_eq!(retrieved.name, "Updated Name");
}

#[tokio::test]
async fn test_remove_rule() {
    let (repo, _temp_dir) = create_test_repository().await;
    let rule = create_test_rule("test_rule_1");

    repo.add_rule(rule).await.expect("Failed to add rule");

    // Remove rule
    let result = repo.remove_rule("test_rule_1").await;
    assert!(result.is_ok());

    // Verify removal
    let retrieved = repo.get_rule("test_rule_1").await.expect("Failed to query");
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_remove_nonexistent_rule() {
    let (repo, _temp_dir) = create_test_repository().await;

    let result = repo.remove_rule("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_statistics() {
    let (repo, _temp_dir) = create_test_repository().await;

    // Initially empty
    let stats = repo.get_statistics().await.expect("Failed to get stats");
    assert_eq!(stats.total_rules, 0);

    // Add rules
    repo.add_rule(create_test_rule("rule_1"))
        .await
        .expect("Failed to add");
    repo.add_rule(create_test_rule("rule_2"))
        .await
        .expect("Failed to add");

    let stats = repo.get_statistics().await.expect("Failed to get stats");
    assert_eq!(stats.total_rules, 2);
}

#[tokio::test]
async fn test_get_categories() {
    let (repo, _temp_dir) = create_test_repository().await;

    // Add rules with various categories
    repo.add_rule(Rule::new("rule_1").with_category("cat_a"))
        .await
        .expect("Failed");
    repo.add_rule(Rule::new("rule_2").with_category("cat_a"))
        .await
        .expect("Failed");
    repo.add_rule(Rule::new("rule_3").with_category("cat_b"))
        .await
        .expect("Failed");

    // Get all categories
    let categories = repo
        .get_categories()
        .await
        .expect("Failed to get categories");
    assert_eq!(categories.len(), 2);
    assert!(categories.contains(&"cat_a".to_string()));
    assert!(categories.contains(&"cat_b".to_string()));
}

#[tokio::test]
async fn test_get_patterns() {
    let (repo, _temp_dir) = create_test_repository().await;

    // Add rules with various patterns
    let rule1 = Rule::new("rule_1")
        .with_pattern("pattern_a")
        .with_pattern("pattern_b");
    let rule2 = Rule::new("rule_2").with_pattern("pattern_a");

    repo.add_rule(rule1).await.expect("Failed");
    repo.add_rule(rule2).await.expect("Failed");

    // Get all patterns
    let patterns = repo.get_patterns().await.expect("Failed to get patterns");
    assert!(patterns.contains(&"pattern_a".to_string()));
    assert!(patterns.contains(&"pattern_b".to_string()));
}

#[tokio::test]
async fn test_repository_with_multiple_patterns() {
    let (repo, _temp_dir) = create_test_repository().await;

    let rule = Rule::new("rule_multi_pattern")
        .with_pattern("pattern_1")
        .with_pattern("pattern_2")
        .with_pattern("pattern_3");

    repo.add_rule(rule).await.expect("Failed to add");

    // Verify patterns are accessible via get_rules_by_pattern
    for i in 1..=3 {
        let pattern_key = format!("pattern_{i}");
        let rules = repo
            .get_rules_by_pattern(&pattern_key)
            .await
            .expect("Failed to get rules by pattern");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "rule_multi_pattern");
    }
}

#[tokio::test]
async fn test_repository_rule_priority_ordering() {
    let (repo, _temp_dir) = create_test_repository().await;

    // Add rules with different priorities
    repo.add_rule(Rule::new("rule_low").with_priority(100))
        .await
        .expect("Failed");
    repo.add_rule(Rule::new("rule_high").with_priority(1))
        .await
        .expect("Failed");
    repo.add_rule(Rule::new("rule_mid").with_priority(50))
        .await
        .expect("Failed");

    let all_rules = repo.get_all_rules().await.expect("Failed to get rules");

    // Verify all rules are present
    assert_eq!(all_rules.len(), 3);

    // Find each rule to verify they exist
    assert!(all_rules
        .iter()
        .any(|r| r.id == "rule_low" && r.priority == 100));
    assert!(all_rules
        .iter()
        .any(|r| r.id == "rule_high" && r.priority == 1));
    assert!(all_rules
        .iter()
        .any(|r| r.id == "rule_mid" && r.priority == 50));
}
