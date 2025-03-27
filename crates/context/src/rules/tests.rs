//! Tests for the rule system
use serde_json::json;
use tokio;
// No need to import error at the top since we'll use the Result type directly

use crate::rules::{
    Rule,
    RuleCondition,
    RuleAction,
    RuleMetadata,
};
// Add imports where needed in each specific test

// Helper function to create a test rule
#[allow(dead_code)]
fn create_test_rule(id: &str) -> Rule {
    Rule {
        id: id.to_string(),
        name: format!("Test Rule {}", id),
        description: format!("Test rule {}", id),
        version: "1.0.0".to_string(),
        category: "test".to_string(),
        priority: 100,
        patterns: vec!["test.*".to_string()],
        conditions: vec![
            RuleCondition::Exists {
                path: "user.name".to_string(),
            },
        ],
        actions: vec![
            RuleAction::ModifyContext {
                path: "processed_by".to_string(),
                value: json!(id),
            },
        ],
        metadata: RuleMetadata::default(),
    }
}

#[tokio::test]
async fn test_rule_directory_manager() -> crate::rules::error::Result<()> {
    use crate::rules::RuleDirectoryManager;
    
    let temp_path = std::env::temp_dir().join("rule_test_dir");
    let rules_dir = temp_path.join(".rules");
    
    // Clean up from any previous test runs
    let _ = std::fs::remove_dir_all(&temp_path);
    
    let dir_manager = RuleDirectoryManager::new(&rules_dir);
    
    // Create the directory
    RuleDirectoryManager::ensure_directory(&rules_dir).await?;
    assert!(rules_dir.exists());
    
    // Create a rule file
    let content = "---\nid: test-rule\nname: Test Rule\n---\n# Test Rule\n\nThis is a test rule.";
    dir_manager.create_rule_file("test", "test-rule", content).await?;
    
    // Check if file exists
    let file_path = rules_dir.join("test").join("test-rule.mdc");
    assert!(file_path.exists());
    
    // Update the rule file
    let updated_content = "---\nid: test-rule\nname: Updated Test Rule\n---\n# Updated Test Rule\n\nThis rule has been updated.";
    dir_manager.update_rule_file("test", "test-rule", updated_content).await?;
    
    // Read the updated content
    let read_content = tokio::fs::read_to_string(&file_path).await?;
    assert_eq!(read_content, updated_content);
    
    // List rule files
    let rule_files = dir_manager.list_rule_files(None).await?;
    assert_eq!(rule_files.len(), 1);
    assert_eq!(rule_files[0], file_path);
    
    // List rule files by category
    let category_rule_files = dir_manager.list_rule_files(Some("test")).await?;
    assert_eq!(category_rule_files.len(), 1);
    assert_eq!(category_rule_files[0], file_path);
    
    // Delete the rule file
    dir_manager.delete_rule_file("test", "test-rule").await?;
    assert!(!file_path.exists());
    
    // Clean up
    let _ = std::fs::remove_dir_all(&temp_path);
    
    Ok(())
}

#[tokio::test]
async fn test_rule_repository() -> crate::rules::error::Result<()> {
    use crate::rules::repository::RuleRepository;
    
    let temp_path = std::env::temp_dir().join("rule_repo_test_dir");
    let rules_dir = temp_path.join(".rules");
    
    // Clean up from any previous test runs
    let _ = std::fs::remove_dir_all(&temp_path);
    
    let repository = RuleRepository::new(rules_dir.to_string_lossy().to_string());
    repository.initialize().await?;
    
    // Create test rules
    let rule1 = create_test_rule("rule1");
    let mut rule2 = create_test_rule("rule2");
    rule2.priority = 5; // Higher priority (lower number)
    rule2.patterns = vec!["test.special".to_string()];
    let rule3 = create_test_rule("rule3");
    
    // Add rules
    repository.add_rule(rule1.clone()).await?;
    repository.add_rule(rule2.clone()).await?;
    repository.add_rule(rule3.clone()).await?;
    
    // Verify we have 3 rules now
    let all_rules = repository.get_all_rules().await?;
    assert_eq!(all_rules.len(), 3, "Expected 3 rules after adding them");
    
    // Get rule
    let rule = repository.get_rule("rule1").await?;
    assert!(rule.is_some(), "Should be able to find rule1");
    assert_eq!(rule.unwrap().id(), "rule1");
    
    // Get non-existent rule
    let rule = repository.get_rule("non-existent").await?;
    assert!(rule.is_none(), "Non-existent rule should return None");
    
    // Get rules by category
    let test_rules = repository.get_rules_by_category("test").await?;
    assert_eq!(test_rules.len(), 3, "Expected 3 rules in the 'test' category");
    
    // Get rules by pattern
    let matching_pattern_rules = repository.get_rules_by_pattern("test.*").await?;
    assert_eq!(matching_pattern_rules.len(), 2, "Expected 2 rules matching pattern 'test.*'");
    
    // Match pattern
    let matched_rules = repository.match_pattern("test.special").await?;
    assert_eq!(matched_rules.len(), 3, "Expected 3 rules matching 'test.special'");
    assert!(matched_rules.iter().any(|r| r.id() == "rule2"), "rule2 should be in the matching rules");
    
    // Update rule
    let mut updated_rule = rule2.clone();
    updated_rule.description = "Updated description".to_string();
    repository.update_rule(updated_rule).await?;
    
    // Get updated rule
    let updated_rule = repository.get_rule("rule2").await?.unwrap();
    assert_eq!(updated_rule.description(), "Updated description");
    
    // Remove rule
    repository.remove_rule("rule3").await?;
    
    // Verify removal
    let removed_rule = repository.get_rule("rule3").await?;
    assert!(removed_rule.is_none(), "rule3 should be removed");
    
    // Get all rules after removal
    let all_rules = repository.get_all_rules().await?;
    assert_eq!(all_rules.len(), 2, "Expected 2 rules after removing one");
    
    // Get all categories
    let categories = repository.get_all_categories().await?;
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0], "test");
    
    // Clean up
    let _ = std::fs::remove_dir_all(&temp_path);
    
    Ok(())
}

#[tokio::test]
async fn test_rule_evaluator() -> crate::rules::error::Result<()> {
    use crate::rules::repository::RuleRepository;
    use crate::rules::evaluator::RuleEvaluator;
    use std::sync::Arc;
    
    let temp_path = std::env::temp_dir().join("rule_eval_test_dir");
    let rules_dir = temp_path.join(".rules");
    
    // Clean up from any previous test runs
    let _ = std::fs::remove_dir_all(&temp_path);
    
    let repository = Arc::new(RuleRepository::new(rules_dir.to_string_lossy().to_string()));
    repository.initialize().await?;
    
    let evaluator = RuleEvaluator::new();
    
    // Create test rules
    let rule1 = create_test_rule("rule1");
    let mut rule2 = create_test_rule("rule2");
    rule2.priority = 5; // Higher priority (lower number)
    
    // Add rules
    repository.add_rule(rule1).await?;
    repository.add_rule(rule2.clone()).await?; // Clone rule2 before adding
    
    // Create context
    let context = json!({
        "user": {
            "name": "John Doe",
            "age": 30
        }
    });
    
    // Find matching rules
    let rules = repository.get_all_rules().await?;
    let matching_rules = evaluator.find_matching_rules(&rules, &context).await?;
    assert_eq!(matching_rules.len(), 2);
    
    // Rules should be sorted by priority (rule2 first, then rule1)
    assert_eq!(matching_rules[0].id(), "rule2");
    assert_eq!(matching_rules[1].id(), "rule1");
    
    // Clean up
    let _ = std::fs::remove_dir_all(&temp_path);
    
    Ok(())
}

#[tokio::test]
async fn test_action_executor() -> crate::rules::error::Result<()> {
    use crate::rules::plugin::RulePluginManager;
    use crate::rules::actions::ActionExecutor;
    use crate::rules::DummyPluginManager;
    use std::sync::Arc;
    
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(DummyPluginManager::default())));
    let action_executor = ActionExecutor::new(Arc::clone(&plugin_manager));
    
    // Create test context
    let mut context = json!({
        "user": {
            "name": "John Doe",
            "profile": {
                "status": "active"
            }
        }
    });
    
    // Test ModifyContext action
    let set_action = RuleAction::ModifyContext {
        path: "user.email".to_string(),
        value: json!("john@example.com"),
    };
    
    action_executor.execute_action(&set_action, &mut context).await?;
    assert_eq!(context["user"]["email"], "john@example.com");
    
    // Test CreateRecoveryPoint action
    let recovery_action = RuleAction::CreateRecoveryPoint {
        name: "test-point".to_string(),
        description: Some("Test recovery point".to_string()),
    };
    
    action_executor.execute_action(&recovery_action, &mut context).await?;
    
    // Test LogMessage action
    let log_action = RuleAction::LogMessage {
        level: "info".to_string(),
        message: "Test log message".to_string(),
    };
    
    action_executor.execute_action(&log_action, &mut context).await?;
    
    // Test NotifyUser action
    let notify_action = RuleAction::NotifyUser {
        title: "Test Notification".to_string(),
        message: "This is a test notification".to_string(),
        level: "info".to_string(),
    };
    
    action_executor.execute_action(&notify_action, &mut context).await?;
    
    // Test Custom action - should return an error
    let custom_action = RuleAction::Custom {
        id: "custom-action".to_string(),
        config: json!({}),
    };
    
    let result = action_executor.execute_action(&custom_action, &mut context).await;
    assert!(result.is_err());
    
    // Test with a test rule
    let rule = create_test_rule("test_action");
    let result = action_executor.execute_rule_actions(&rule, Some(&mut context)).await?;
    
    // Check that the processed_by field was set
    assert_eq!(result["processed_by"], "test_action");
    
    Ok(())
}

#[tokio::test]
async fn test_rule_manager() -> crate::rules::error::Result<()> {
    let temp_path = std::env::temp_dir().join("rule_manager_test_dir");
    let rules_dir = temp_path.join(".rules");
    
    // Clean up from any previous test runs
    let _ = std::fs::remove_dir_all(&temp_path);
    
    // Create rule directories
    std::fs::create_dir_all(&rules_dir).expect("Failed to create directories");
    
    // Create a RuleManager directly
    let rule_manager = crate::rules::RuleManager::new(&rules_dir);
    rule_manager.initialize().await?;
    
    // Create a simple test rule
    let rule = Rule {
        id: "test_rule".to_string(),
        name: "Test Rule".to_string(),
        description: "A simple test rule".to_string(),
        version: "1.0.0".to_string(),
        category: "test".to_string(),
        priority: 100,
        patterns: vec!["test.*".to_string()],
        conditions: vec![
            RuleCondition::Exists {
                path: "user.name".to_string(),
            },
        ],
        actions: vec![
            RuleAction::ModifyContext {
                path: "processed_by".to_string(),
                value: json!("test_rule"),
            },
        ],
        metadata: RuleMetadata::default(),
    };
    
    // Add rule to the manager
    rule_manager.add_or_update_rule(rule).await?;
    
    // Get the rule back to verify it was added
    let added_rule = rule_manager.get_rule("test_rule").await?;
    assert!(added_rule.is_some(), "Rule should have been added successfully");
    assert_eq!(added_rule.unwrap().id(), "test_rule");
    
    // Create context that matches the rule
    let mut context = json!({
        "user": {
            "name": "John Doe",
            "age": 30
        }
    });
    
    // Apply rules to the context
    let result = rule_manager.apply_rules(&mut context).await?;
    
    // Verify the rule was applied
    assert_eq!(result.rules_applied.len(), 1, "One rule should have been applied");
    assert_eq!(result.rules_applied[0].id, "test_rule");
    
    // Verify the context was modified as expected
    assert_eq!(context["processed_by"], "test_rule", "Rule action should have modified the context");
    
    // Clean up
    let _ = std::fs::remove_dir_all(&temp_path);
    
    Ok(())
}

#[tokio::test]
async fn test_rule_conditions() -> crate::rules::error::Result<()> {
    use crate::rules::evaluator::RuleEvaluator;
    
    let context = json!({
        "user": {
            "name": "John Doe",
            "age": 30,
            "email": "john@example.com",
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        },
        "session": {
            "id": "12345",
            "start_time": "2023-01-01T00:00:00Z"
        }
    });
    
    // Test the Exists condition
    let condition = RuleCondition::Exists {
        path: "user.name".to_string(),
    };
    
    let evaluator = RuleEvaluator::new();
    let result = evaluator.evaluate_condition(&condition, &context).await?;
    assert!(result);
    
    // Test the Match condition
    let condition = RuleCondition::Match {
        path: "user.name".to_string(),
        pattern: "John Doe".to_string(),
    };
    
    let result = evaluator.evaluate_condition(&condition, &context).await?;
    assert!(result);
    
    // Test non-matching pattern
    let condition = RuleCondition::Match {
        path: "user.name".to_string(),
        pattern: "Jane Doe".to_string(),
    };
    
    let result = evaluator.evaluate_condition(&condition, &context).await?;
    assert!(!result);
    
    // Test Compare condition with different values
    let condition = RuleCondition::Compare {
        path1: "user.age".to_string(),
        path2: "user.settings.theme".to_string(),
        operator: "eq".to_string(),
    };
    
    let result = evaluator.evaluate_condition(&condition, &context).await?;
    assert!(!result);
    
    // Test Compare condition with same values (comparing to itself)
    let condition = RuleCondition::Compare {
        path1: "user.age".to_string(),
        path2: "user.age".to_string(),
        operator: "eq".to_string(),
    };
    
    let result = evaluator.evaluate_condition(&condition, &context).await?;
    assert!(result);
    
    Ok(())
}

#[tokio::test]
async fn test_rule_actions() -> crate::rules::error::Result<()> {
    use crate::rules::plugin::RulePluginManager;
    use crate::rules::actions::ActionExecutor;
    use crate::rules::DummyPluginManager;
    use std::sync::Arc;
    
    let mut context = json!({
        "user": {
            "name": "John Doe",
            "age": 30
        },
        "session": {
            "id": "12345",
            "start_time": "2023-01-01T00:00:00Z"
        }
    });
    
    let plugin_manager = Arc::new(RulePluginManager::new(Arc::new(DummyPluginManager::default())));
    let executor = ActionExecutor::new(plugin_manager);
    
    // Test ModifyContext action
    let action = RuleAction::ModifyContext {
        path: "user.processed".to_string(),
        value: json!(true),
    };
    
    executor.execute_action(&action, &mut context).await?;
    let user = context.get("user").unwrap();
    assert!(user.get("processed").unwrap().as_bool().unwrap());
    
    // Test LogMessage action
    let action = RuleAction::LogMessage {
        level: "info".to_string(),
        message: "Test log message".to_string(),
    };
    
    executor.execute_action(&action, &mut context).await?;
    
    // Test with a test rule
    let rule = create_test_rule("test_action");
    let result = executor.execute_rule_actions(&rule, Some(&mut context)).await?;
    
    // Check that the processed_by field was set
    assert_eq!(result["processed_by"], "test_action");
    
    Ok(())
}

#[test]
fn test_pattern_matches() {
    use crate::rules::repository::RuleRepository;
    
    // Create a repository to test pattern_matches
    let repo = RuleRepository::new("dummy");
    
    // Test exact match
    assert!(repo.pattern_matches("test.special", "test.special"));
    
    // Test wildcard pattern
    assert!(repo.pattern_matches("anything", "*"));
    
    // Test suffix wildcard
    assert!(repo.pattern_matches("test.anything", "test.*"));
    assert!(!repo.pattern_matches("anything.test", "test.*"));
    
    // Test prefix wildcard
    assert!(repo.pattern_matches("anything.test", "*.test"));
    assert!(!repo.pattern_matches("test.anything", "*.test"));
    
    // Test wildcard with dots
    assert!(repo.pattern_matches("test.special", "*.*"));
    
    // Test complex cases
    assert!(!repo.pattern_matches("test.special", "test.other"));
    assert!(!repo.pattern_matches("other.special", "test.*"));
    assert!(repo.pattern_matches("test.anything.special", "test.*"));
}

// Add more tests as needed 