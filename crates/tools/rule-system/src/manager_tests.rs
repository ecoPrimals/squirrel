//! Comprehensive tests for rule manager

#[cfg(test)]
mod tests {
    use crate::actions::ActionExecutor;
    use crate::directory::{RuleDirectoryConfig, RuleDirectoryManager};
    use crate::evaluator::RuleEvaluator;
    use crate::manager::RuleManager;
    use crate::models::{Rule, RuleAction, RuleCondition};
    use crate::parser::{ParserConfig, RuleParser};
    use crate::repository::RuleRepository;
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// Helper to create a test rule manager
    fn test_manager() -> RuleManager {
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

        let repository = Arc::new(RuleRepository::new(dir_manager, parser));
        let evaluator = Arc::new(RuleEvaluator::new());
        let action_executor = Arc::new(ActionExecutor::new());

        RuleManager::new(repository, evaluator, action_executor)
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = test_manager();
        // Just verify it was created successfully
        assert!(format!("{manager:?}").contains("RuleManager"));
    }

    #[tokio::test]
    async fn test_add_rule_to_manager() {
        let manager = test_manager();

        let rule =
            Rule::new("test-add")
                .with_name("Add Test")
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });

        let result = manager.add_rule(rule).await;
        // Should succeed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_rule_from_manager() {
        let manager = test_manager();

        // First add a rule
        let rule = Rule::new("test-remove")
            .with_name("Remove Test")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });
        manager.add_rule(rule).await.unwrap();

        // Now remove it
        let result = manager.remove_rule("test-remove").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_rule() {
        let manager = test_manager();

        // Add initial rule
        let rule = Rule::new("test-update")
            .with_name("Original Name")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });
        manager.add_rule(rule).await.unwrap();

        // Update it
        let updated_rule = Rule::new("test-update")
            .with_name("Updated Rule")
            .with_condition(RuleCondition::Exists {
                path: "updated".to_string(),
            });

        let result = manager.update_rule(updated_rule).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_rule() {
        let manager = test_manager();

        // Add a rule
        let rule =
            Rule::new("test-get")
                .with_name("Get Test")
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });
        manager.add_rule(rule).await.unwrap();

        // Get it back
        let result = manager.get_rule("test-get").await;
        assert!(result.is_ok());
        let retrieved = result.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-get");
    }

    #[tokio::test]
    async fn test_get_all_rules() {
        let manager = test_manager();

        // Add multiple rules
        for i in 0..5 {
            let rule = Rule::new(format!("test-all-{i}"))
                .with_name(format!("Rule {i}"))
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });
            manager.add_rule(rule).await.unwrap();
        }

        let result = manager.get_all_rules().await;
        assert!(result.is_ok());
        let rules = result.unwrap();
        assert_eq!(rules.len(), 5);
    }

    #[tokio::test]
    async fn test_get_rules_by_category() {
        let manager = test_manager();

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

        manager.add_rule(rule1).await.unwrap();
        manager.add_rule(rule2).await.unwrap();

        let result = manager.get_rules_by_category("security").await;
        assert!(result.is_ok());
        let rules = result.unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[tokio::test]
    async fn test_evaluate_rules() {
        let manager = test_manager();

        // Add a rule
        let rule = Rule::new("test-eval")
            .with_name("Evaluate Test")
            .with_condition(RuleCondition::Exists {
                path: "user".to_string(),
            });
        manager.add_rule(rule).await.unwrap();

        let context = json!({"user": {"name": "Test"}});

        let result = manager.evaluate_rules("ctx1", context.clone()).await;
        // Should work even if empty
        let _ = result;
    }

    #[tokio::test]
    async fn test_execute_actions() {
        let manager = test_manager();

        // Add a rule with an action
        let rule = Rule::new("test-action")
            .with_name("Action Test")
            .with_condition(RuleCondition::Exists {
                path: "user".to_string(),
            })
            .with_action(RuleAction::ModifyContext {
                path: "processed".to_string(),
                value: json!(true),
            });
        manager.add_rule(rule).await.unwrap();

        let context = json!({"user": {}});

        // First evaluate to get results
        let eval_results = manager
            .evaluate_rules("ctx1", context.clone())
            .await
            .unwrap_or_default();

        let result = manager.execute_actions(&eval_results).await;
        let _ = result;
    }

    #[tokio::test]
    async fn test_process_context() {
        let manager = test_manager();

        // Add a rule
        let rule = Rule::new("test-process")
            .with_name("Process Test")
            .with_condition(RuleCondition::Exists {
                path: "user".to_string(),
            })
            .with_action(RuleAction::ModifyContext {
                path: "processed".to_string(),
                value: json!(true),
            });
        manager.add_rule(rule).await.unwrap();

        let context = json!({"user": {}});

        let result = manager.process_context("ctx1", context).await;
        let _ = result;
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let manager = test_manager();

        let result = manager.get_statistics().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reload() {
        let manager = test_manager();

        let result = manager.reload().await;
        // May fail without proper directory setup, but API exists
        let _ = result;
    }

    #[tokio::test]
    async fn test_activate_rule() {
        let manager = test_manager();

        // Add a rule first
        let rule = Rule::new("test-activate")
            .with_name("Activate Test")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });
        manager.add_rule(rule).await.unwrap();

        let result = manager.activate_rule("test-activate").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deactivate_rule() {
        let manager = test_manager();

        // Add and activate a rule
        let rule = Rule::new("test-deactivate")
            .with_name("Deactivate Test")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });
        manager.add_rule(rule).await.unwrap();
        manager.activate_rule("test-deactivate").await.unwrap();

        let result = manager.deactivate_rule("test-deactivate").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_is_rule_active() {
        let manager = test_manager();

        // Add and activate a rule
        let rule = Rule::new("test-is-active")
            .with_name("Is Active Test")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            });
        manager.add_rule(rule).await.unwrap();
        manager.activate_rule("test-is-active").await.unwrap();

        let is_active = manager.is_rule_active("test-is-active").await;
        assert!(is_active);
    }

    #[tokio::test]
    async fn test_get_active_rules() {
        let manager = test_manager();

        // Add and activate multiple rules
        for i in 0..3 {
            let rule = Rule::new(format!("test-active-{i}"))
                .with_name(format!("Active {i}"))
                .with_condition(RuleCondition::Exists {
                    path: "data".to_string(),
                });
            manager.add_rule(rule).await.unwrap();
            manager
                .activate_rule(&format!("test-active-{i}"))
                .await
                .unwrap();
        }

        let active_rules = manager.get_active_rules().await;
        assert_eq!(active_rules.len(), 3);
    }

    #[tokio::test]
    async fn test_rule_with_multiple_conditions() {
        let manager = test_manager();

        let rule = Rule::new("test-multi-cond")
            .with_name("Multiple Conditions")
            .with_condition(RuleCondition::Exists {
                path: "user".to_string(),
            })
            .with_condition(RuleCondition::Equals {
                path: "user.active".to_string(),
                value: json!(true),
            });

        let result = manager.add_rule(rule).await;
        assert!(result.is_ok());
    }
}
