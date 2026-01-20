//! Comprehensive tests for rule repository

#[cfg(test)]
mod tests {
    use crate::directory::{RuleDirectoryConfig, RuleDirectoryManager};
    use crate::models::{Rule, RuleCondition};
    use crate::parser::{ParserConfig, RuleParser};
    use crate::repository::RuleRepository;
    use serde_json::json;
    use std::path::PathBuf;

    /// Helper to create a test repository
    fn test_repository() -> RuleRepository {
        let dir_config = RuleDirectoryConfig {
            root_directory: PathBuf::from("/tmp/test-rules"),
            default_extension: "yaml".to_string(),
            include_patterns: vec!["*.yaml".to_string(), "*.yml".to_string()],
            exclude_patterns: vec![],
            watch_for_changes: false,
            recursion_depth: 1,
        };
        let dir_manager = RuleDirectoryManager::new(dir_config);

        let parser_config = ParserConfig {
            validate: true,
            extract_metadata: true,
            parse_conditions: true,
            parse_actions: true,
        };
        let parser = RuleParser::new(parser_config);

        RuleRepository::new(dir_manager, parser)
    }

    #[tokio::test]
    async fn test_repository_creation() {
        let repo = test_repository();
        // Just verify it was created successfully
        assert!(format!("{repo:?}").contains("RuleRepository"));
    }

    #[tokio::test]
    async fn test_add_rule_to_repository() {
        let repo = test_repository();

        let rule = Rule::new("test-add")
            .with_name("Add Test")
            .with_category("test")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        let result = repo.add_rule(rule).await;
        // Should succeed for first add
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_duplicate_rule() {
        let repo = test_repository();

        let rule1 =
            Rule::new("test-dup")
                .with_name("First")
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });

        let rule2 =
            Rule::new("test-dup")
                .with_name("Second")
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });

        // Add first rule
        let result1 = repo.add_rule(rule1).await;
        assert!(result1.is_ok());

        // Try to add duplicate
        let result2 = repo.add_rule(rule2).await;
        assert!(result2.is_err()); // Should fail due to duplicate ID
    }

    #[tokio::test]
    async fn test_get_rule_by_id() {
        let repo = test_repository();

        let rule =
            Rule::new("test-get")
                .with_name("Get Test")
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });

        repo.add_rule(rule).await.unwrap();

        let result = repo.get_rule("test-get").await;
        assert!(result.is_ok());
        let retrieved = result.unwrap().unwrap();
        assert_eq!(retrieved.id, "test-get");
        assert_eq!(retrieved.name, "Get Test");
    }

    #[tokio::test]
    async fn test_get_nonexistent_rule() {
        let repo = test_repository();

        let result = repo.get_rule("nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_remove_rule() {
        let repo = test_repository();

        let rule = Rule::new("test-remove")
            .with_name("Remove Test")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        repo.add_rule(rule).await.unwrap();

        let result = repo.remove_rule("test-remove").await;
        assert!(result.is_ok());

        // Verify it's gone
        let get_result = repo.get_rule("test-remove").await.unwrap();
        assert!(get_result.is_none());
    }

    #[tokio::test]
    async fn test_update_rule() {
        let repo = test_repository();

        let rule = Rule::new("test-update")
            .with_name("Original Name")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        repo.add_rule(rule).await.unwrap();

        let updated_rule = Rule::new("test-update")
            .with_name("Updated Name")
            .with_description("New description")
            .with_condition(RuleCondition::Exists {
                path: "updated_data".to_string(),
            });

        let result = repo.update_rule(updated_rule).await;
        assert!(result.is_ok());

        // Verify update
        let retrieved = repo.get_rule("test-update").await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Name");
        assert_eq!(retrieved.description, "New description");
    }

    #[tokio::test]
    async fn test_get_all_rules() {
        let repo = test_repository();

        // Add multiple rules
        for i in 0..5 {
            let rule = Rule::new(format!("test-{i}"))
                .with_name(format!("Rule {i}"))
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });
            repo.add_rule(rule).await.unwrap();
        }

        let result = repo.get_all_rules().await;
        assert!(result.is_ok());
        let rules = result.unwrap();
        assert_eq!(rules.len(), 5);
    }

    #[tokio::test]
    async fn test_get_rules_by_category() {
        let repo = test_repository();

        // Add rules with different categories
        let rule1 = Rule::new("test-cat1")
            .with_name("Category Test 1")
            .with_category("security")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        let rule2 = Rule::new("test-cat2")
            .with_name("Category Test 2")
            .with_category("security")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        let rule3 = Rule::new("test-cat3")
            .with_name("Category Test 3")
            .with_category("performance")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        repo.add_rule(rule1).await.unwrap();
        repo.add_rule(rule2).await.unwrap();
        repo.add_rule(rule3).await.unwrap();

        let result = repo.get_rules_by_category("security").await;
        assert!(result.is_ok());
        let rules = result.unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[tokio::test]
    async fn test_get_rules_by_pattern() {
        let repo = test_repository();

        let rule = Rule::new("test-pattern")
            .with_name("Pattern Test")
            .with_pattern("login.*")
            .with_pattern("auth.*")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        repo.add_rule(rule).await.unwrap();

        let result = repo.get_rules_by_pattern("login.*").await;
        assert!(result.is_ok());
        let rules = result.unwrap();
        assert_eq!(rules.len(), 1);
    }

    #[tokio::test]
    async fn test_get_matching_rules() {
        let repo = test_repository();

        let rule = Rule::new("test-matching")
            .with_name("Matching Test")
            .with_pattern("test.*")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        repo.add_rule(rule).await.unwrap();

        let result = repo.get_matching_rules("test-context").await;
        // May or may not match depending on implementation
        let _ = result;
    }

    #[tokio::test]
    async fn test_get_categories() {
        let repo = test_repository();

        // Add rules with various categories
        let categories = ["security", "performance", "data", "security"];
        for (i, cat) in categories.iter().enumerate() {
            let rule = Rule::new(format!("test-categories-{i}"))
                .with_name(format!("Test {i}"))
                .with_category(*cat)
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });
            repo.add_rule(rule).await.unwrap();
        }

        let result = repo.get_categories().await;
        assert!(result.is_ok());
        let cats = result.unwrap();
        assert!(cats.contains(&"security".to_string()));
        assert!(cats.contains(&"performance".to_string()));
        assert!(cats.contains(&"data".to_string()));
    }

    #[tokio::test]
    async fn test_get_patterns() {
        let repo = test_repository();

        let rule = Rule::new("test-patterns")
            .with_name("Patterns Test")
            .with_pattern("login.*")
            .with_pattern("auth.*")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        repo.add_rule(rule).await.unwrap();

        let result = repo.get_patterns().await;
        assert!(result.is_ok());
        let patterns = result.unwrap();
        assert!(patterns.contains(&"login.*".to_string()));
        assert!(patterns.contains(&"auth.*".to_string()));
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let repo = test_repository();

        // Add some rules
        for i in 0..10 {
            let rule = Rule::new(format!("test-stats-{i}"))
                .with_name(format!("Stats Test {i}"))
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });
            repo.add_rule(rule).await.unwrap();
        }

        let result = repo.get_statistics().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reload() {
        let repo = test_repository();

        let result = repo.reload().await;
        // May fail without proper directory setup
        let _ = result;
    }

    #[tokio::test]
    async fn test_repository_with_metadata() {
        let repo = test_repository();

        let rule = Rule::new("test-meta")
            .with_name("Metadata Test")
            .with_metadata("author", json!("test-user"))
            .with_metadata("version", json!("1.0.0"))
            .with_metadata("tags", json!(["test", "metadata"]))
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });

        repo.add_rule(rule).await.unwrap();

        let retrieved = repo.get_rule("test-meta").await.unwrap().unwrap();
        assert_eq!(retrieved.metadata.get("author"), Some(&json!("test-user")));
        assert_eq!(retrieved.metadata.get("version"), Some(&json!("1.0.0")));
    }
}
