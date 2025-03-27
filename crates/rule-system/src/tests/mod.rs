//! Tests for the rule system

use std::path::PathBuf;
use tempfile::tempdir;
use serde_json::json;

use crate::models::{Rule, RuleCondition, RuleAction};
use crate::parser::{RuleParser, parse_rule_content};
use crate::utils;
use crate::directory::{RuleDirectoryManager, RuleDirectoryConfig};

mod models_tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = Rule::new("test-rule")
            .with_name("Test Rule")
            .with_description("Test rule description")
            .with_category("test")
            .with_priority(50)
            .with_pattern("context.*")
            .with_condition(RuleCondition::Exists { path: "data.value".to_string() })
            .with_action(RuleAction::ModifyContext { 
                path: "data.processed".to_string(), 
                value: json!(true) 
            });
        
        assert_eq!(rule.id, "test-rule");
        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.description, "Test rule description");
        assert_eq!(rule.category, "test");
        assert_eq!(rule.priority, 50);
        assert_eq!(rule.patterns.len(), 1);
        assert_eq!(rule.patterns[0], "context.*");
        assert_eq!(rule.conditions.len(), 1);
        assert_eq!(rule.actions.len(), 1);
    }
    
    #[test]
    fn test_rule_conditions() {
        let condition = RuleCondition::Equals {
            path: "data.value".to_string(),
            value: json!(42),
        };
        
        if let RuleCondition::Equals { path, value } = condition {
            assert_eq!(path, "data.value");
            assert_eq!(value, json!(42));
        } else {
            panic!("Unexpected condition type");
        }
        
        let nested_condition = RuleCondition::All {
            conditions: vec![
                RuleCondition::Exists { path: "data.value1".to_string() },
                RuleCondition::Exists { path: "data.value2".to_string() },
            ],
        };
        
        if let RuleCondition::All { conditions } = nested_condition {
            assert_eq!(conditions.len(), 2);
        } else {
            panic!("Unexpected condition type");
        }
    }
    
    #[test]
    fn test_rule_actions() {
        let action = RuleAction::ModifyContext {
            path: "data.value".to_string(),
            value: json!(42),
        };
        
        if let RuleAction::ModifyContext { path, value } = action {
            assert_eq!(path, "data.value");
            assert_eq!(value, json!(42));
        } else {
            panic!("Unexpected action type");
        }
        
        let recovery_action = RuleAction::CreateRecoveryPoint {
            description: "Before update".to_string(),
        };
        
        if let RuleAction::CreateRecoveryPoint { description } = recovery_action {
            assert_eq!(description, "Before update");
        } else {
            panic!("Unexpected action type");
        }
    }
}

mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_rule_content() {
        let content = r#"---
id: "test-rule"
name: "Test Rule"
description: "Test rule description"
version: "1.0.0"
category: "test"
priority: 50
patterns:
  - "context.*"
dependencies: []
---

## Conditions

- type: Exists
  config:
    path: "data.value"

## Actions

- type: ModifyContext
  config:
    path: "data.processed"
    value: true
"#;

        let rule = parse_rule_content(content).unwrap();
        
        assert_eq!(rule.id, "test-rule");
        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.description, "Test rule description");
        assert_eq!(rule.version, "1.0.0");
        assert_eq!(rule.category, "test");
        assert_eq!(rule.priority, 50);
        assert_eq!(rule.patterns.len(), 1);
        assert_eq!(rule.patterns[0], "context.*");
        assert_eq!(rule.conditions.len(), 1);
        assert_eq!(rule.actions.len(), 1);
    }
    
    #[test]
    fn test_parser_sections() {
        let content = r#"---
id: "test-rule"
name: "Test Rule"
patterns:
  - "test.*"
---

## Conditions

- type: Exists
  config:
    path: "data.value"

## Actions

- type: ModifyContext
  config:
    path: "data.processed"
    value: true

## Notes

This is a test rule.
"#;

        let parser = RuleParser::default();
        let rule = parser.parse_rule(content).unwrap();
        
        assert!(rule.metadata.contains_key("Notes"));
        assert_eq!(rule.metadata["Notes"].as_str().unwrap().trim(), "This is a test rule.");
    }
}

mod utils_tests {
    use super::*;

    #[test]
    fn test_rule_template() {
        let template = utils::create_rule_template("test-rule", "Test Rule", "test");
        
        assert!(template.contains("id: \"test-rule\""));
        assert!(template.contains("name: \"Test Rule\""));
        assert!(template.contains("category: \"test\""));
    }
    
    #[test]
    fn test_extract_value_by_path() {
        let data = json!({
            "user": {
                "name": "John",
                "age": 30,
                "addresses": [
                    {
                        "city": "New York",
                        "zipcode": "10001"
                    },
                    {
                        "city": "Boston",
                        "zipcode": "02108"
                    }
                ]
            }
        });
        
        // Simple path
        let name = utils::extract_value_by_path(&data, "user.name").unwrap();
        assert_eq!(name, json!("John"));
        
        // Array indexing
        let city = utils::extract_value_by_path(&data, "user.addresses[0].city").unwrap();
        assert_eq!(city, json!("New York"));
        
        // Non-existent path
        let missing = utils::extract_value_by_path(&data, "user.email");
        assert!(missing.is_none());
    }
    
    #[test]
    fn test_set_value_by_path() {
        let mut data = json!({
            "user": {
                "name": "John",
                "age": 30
            }
        });
        
        // Simple path
        utils::set_value_by_path(&mut data, "user.name", json!("Jane"));
        assert_eq!(data["user"]["name"], json!("Jane"));
        
        // Create new path
        utils::set_value_by_path(&mut data, "user.email", json!("jane@example.com"));
        assert_eq!(data["user"]["email"], json!("jane@example.com"));
        
        // Create nested path
        utils::set_value_by_path(&mut data, "user.address.city", json!("New York"));
        assert_eq!(data["user"]["address"]["city"], json!("New York"));
        
        // Create array
        utils::set_value_by_path(&mut data, "user.phones[0]", json!("123-456-7890"));
        assert_eq!(data["user"]["phones"][0], json!("123-456-7890"));
    }
    
    #[test]
    fn test_rule_matches_context() {
        let rule = Rule::new("test-rule")
            .with_pattern("context.*")
            .with_pattern("special.context");
        
        assert!(utils::rule_matches_context(&rule, "context.test"));
        assert!(utils::rule_matches_context(&rule, "context.another"));
        assert!(utils::rule_matches_context(&rule, "special.context"));
        assert!(!utils::rule_matches_context(&rule, "other.context"));
    }
}

#[tokio::test]
async fn test_rule_directory_manager() {
    // Create a temporary directory for testing
    let base_dir = tempdir().unwrap();
    
    // Create a rule directory manager with config
    let config = RuleDirectoryConfig {
        root_directory: base_dir.path().to_path_buf(),
        default_extension: "mdc".to_string(),
        include_patterns: vec!["**/*.mdc".to_string()],
        exclude_patterns: vec![],
        watch_for_changes: false,
        recursion_depth: -1,
    };
    
    let manager = RuleDirectoryManager::new(config);
    
    // Initialize the manager
    manager.initialize().await.unwrap();
    
    // Create a rule file
    let rule_content = utils::create_rule_template("test-rule", "Test Rule", "test");
    let file_path = manager.create_rule_file("test-rule", None::<String>, &rule_content).await.unwrap();
    
    // Check if the file exists
    assert!(file_path.exists());
    
    // Check if we can get all rule files
    let rule_files = manager.get_all_rule_files().await.unwrap();
    assert_eq!(rule_files.len(), 1);
    
    // Create a category
    let category_rule_content = utils::create_rule_template("category-rule", "Category Rule", "test-category");
    let category_file_path = manager.create_rule_file("category-rule", Some("test-category"), &category_rule_content).await.unwrap();
    
    // Check if the category file exists
    assert!(category_file_path.exists());
    
    // Get categories
    let categories = manager.get_categories().await.unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0], "test-category");
    
    // Delete a rule file
    manager.delete_rule_file("test-rule", None::<String>).await.unwrap();
    
    // Check if the file was deleted
    assert!(!PathBuf::from(&file_path).exists());
} 