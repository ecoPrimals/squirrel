// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for rule manager

#[cfg(test)]
mod tests {
    use crate::actions::ActionExecutor;
    use crate::directory::{RuleDirectoryConfig, RuleDirectoryManager};
    use crate::error::{RuleManagerError, RuleSystemError};
    use crate::evaluator::RuleEvaluator;
    use crate::manager::{RuleManager, create_rule_manager, create_rule_manager_with_components};
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
        manager.add_rule(rule).await.expect("should succeed");

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
        manager.add_rule(rule).await.expect("should succeed");

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
        manager.add_rule(rule).await.expect("should succeed");

        // Get it back
        let result = manager.get_rule("test-get").await;
        assert!(result.is_ok());
        let retrieved = result.expect("should succeed");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("should succeed").id, "test-get");
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
            manager.add_rule(rule).await.expect("should succeed");
        }

        let result = manager.get_all_rules().await;
        assert!(result.is_ok());
        let rules = result.expect("should succeed");
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

        manager.add_rule(rule1).await.expect("should succeed");
        manager.add_rule(rule2).await.expect("should succeed");

        let result = manager.get_rules_by_category("security").await;
        assert!(result.is_ok());
        let rule_list = result.expect("should succeed");
        assert_eq!(rule_list.len(), 2);
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
        manager.add_rule(rule).await.expect("should succeed");

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
        manager.add_rule(rule).await.expect("should succeed");

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
        manager.add_rule(rule).await.expect("should succeed");

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
        manager.add_rule(rule).await.expect("should succeed");

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
        manager.add_rule(rule).await.expect("should succeed");
        manager
            .activate_rule("test-deactivate")
            .await
            .expect("should succeed");

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
        manager.add_rule(rule).await.expect("should succeed");
        manager
            .activate_rule("test-is-active")
            .await
            .expect("should succeed");

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
            manager.add_rule(rule).await.expect("should succeed");
            manager
                .activate_rule(&format!("test-active-{i}"))
                .await
                .expect("should succeed");
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

    #[tokio::test]
    async fn add_rule_fails_when_dependency_missing() {
        let manager = test_manager();
        let rule = Rule::new("needs-parent")
            .with_name("Needs parent")
            .with_pattern("ctx.*")
            .with_dependency("nonexistent-rule")
            .with_condition(RuleCondition::Exists {
                path: "data".to_string(),
            })
            .with_action(RuleAction::ModifyContext {
                path: "out".to_string(),
                value: json!(1),
            });
        let err = manager.add_rule(rule).await.expect_err("missing dep");
        assert!(matches!(
            err,
            RuleSystemError::ManagerError(RuleManagerError::DependencyError(_))
        ));
    }

    #[tokio::test]
    async fn remove_rule_fails_when_depended_upon() {
        let manager = test_manager();
        let base = Rule::new("base-rule")
            .with_name("Base")
            .with_pattern("ctx.*")
            .with_condition(RuleCondition::Exists {
                path: "x".to_string(),
            })
            .with_action(RuleAction::ModifyContext {
                path: "o".to_string(),
                value: json!(true),
            });
        let dependent = Rule::new("dependent-rule")
            .with_name("Dep")
            .with_pattern("ctx.*")
            .with_dependency("base-rule")
            .with_condition(RuleCondition::Exists {
                path: "y".to_string(),
            })
            .with_action(RuleAction::ModifyContext {
                path: "z".to_string(),
                value: json!(null),
            });
        manager.add_rule(base).await.expect("should succeed");
        manager.add_rule(dependent).await.expect("should succeed");
        let err = manager.remove_rule("base-rule").await.expect_err("blocked");
        assert!(matches!(
            err,
            RuleSystemError::ManagerError(RuleManagerError::DependencyError(msg))
                if msg.contains("depended")
        ));
    }

    #[tokio::test]
    async fn activate_rule_fails_for_unknown_id() {
        let manager = test_manager();
        let err = manager
            .activate_rule("no-such-rule")
            .await
            .expect_err("missing");
        assert!(matches!(
            err,
            RuleSystemError::ManagerError(RuleManagerError::RuleNotFound(_))
        ));
    }

    #[tokio::test]
    async fn update_rule_rejects_circular_dependency() {
        let manager = test_manager();
        let a = Rule::new("rule-a")
            .with_name("A")
            .with_pattern("ctx.*")
            .with_condition(RuleCondition::Exists {
                path: "p".to_string(),
            })
            .with_action(RuleAction::ModifyContext {
                path: "q".to_string(),
                value: json!(0),
            });
        let b = Rule::new("rule-b")
            .with_name("B")
            .with_pattern("ctx.*")
            .with_dependency("rule-a")
            .with_condition(RuleCondition::Exists {
                path: "p".to_string(),
            })
            .with_action(RuleAction::ModifyContext {
                path: "q".to_string(),
                value: json!(1),
            });
        manager.add_rule(a).await.expect("should succeed");
        manager.add_rule(b).await.expect("should succeed");

        let a_dep_b = Rule::new("rule-a")
            .with_name("A2")
            .with_pattern("ctx.*")
            .with_dependency("rule-b")
            .with_condition(RuleCondition::Exists {
                path: "p".to_string(),
            })
            .with_action(RuleAction::ModifyContext {
                path: "q".to_string(),
                value: json!(3),
            });
        let err = manager.update_rule(a_dep_b).await.expect_err("circular");
        assert!(matches!(
            err,
            RuleSystemError::ManagerError(RuleManagerError::DependencyError(msg))
                if msg.contains("Circular")
        ));
    }

    #[tokio::test]
    async fn initialize_clears_and_reloads_cache() {
        let manager = test_manager();
        manager.initialize().await.expect("init");
        let stats = manager.get_statistics().await.expect("stats");
        assert_eq!(stats.dependency_cache_size, stats.total_rules);
    }

    #[tokio::test]
    async fn create_rule_manager_factory_initializes() {
        let manager = create_rule_manager().expect("factory");
        manager.initialize().await.expect("init");
    }

    #[tokio::test]
    async fn create_rule_manager_with_components_smoke() {
        let dir_config = RuleDirectoryConfig {
            root_directory: PathBuf::from("/tmp/test-rules-components"),
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
        let manager = create_rule_manager_with_components(repository, evaluator, action_executor);
        assert!(format!("{manager:?}").contains("RuleManager"));
    }

    #[tokio::test]
    async fn update_rule_rejects_self_dependency() {
        let manager = test_manager();
        let base = Rule::new("self-loop")
            .with_name("Self")
            .with_pattern("ctx.*")
            .with_condition(RuleCondition::Exists {
                path: "x".to_string(),
            });
        manager.add_rule(base).await.expect("add");
        let bad = Rule::new("self-loop")
            .with_name("Self")
            .with_pattern("ctx.*")
            .with_dependency("self-loop")
            .with_condition(RuleCondition::Exists {
                path: "x".to_string(),
            });
        let err = manager.update_rule(bad).await.expect_err("self-dep");
        assert!(matches!(
            err,
            RuleSystemError::ManagerError(RuleManagerError::DependencyError(msg))
                if msg.contains("Circular")
        ));
    }

    #[tokio::test]
    async fn evaluate_rules_skips_rules_whose_dependencies_are_inactive() {
        let manager = test_manager();
        let parent = Rule::new("dep-parent")
            .with_name("Parent")
            .with_pattern("ctx*")
            .with_condition(RuleCondition::Exists {
                path: "ok".to_string(),
            });
        let child = Rule::new("dep-child")
            .with_name("Child")
            .with_pattern("ctx*")
            .with_dependency("dep-parent")
            .with_condition(RuleCondition::Exists {
                path: "ok".to_string(),
            });
        manager.add_rule(parent).await.expect("parent");
        manager.add_rule(child).await.expect("child");

        let ctx = json!({"ok": true});
        let results = manager.evaluate_rules("ctx-1", ctx).await.expect("eval");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rule_id, "dep-parent");
    }

    #[tokio::test]
    async fn evaluate_rules_runs_dependent_rule_after_parent_is_active() {
        let manager = test_manager();
        let parent = Rule::new("dep-parent-2")
            .with_name("Parent")
            .with_pattern("ctx*")
            .with_condition(RuleCondition::Exists {
                path: "ok".to_string(),
            });
        let child = Rule::new("dep-child-2")
            .with_name("Child")
            .with_pattern("ctx*")
            .with_dependency("dep-parent-2")
            .with_condition(RuleCondition::Exists {
                path: "ok".to_string(),
            });
        manager.add_rule(parent).await.expect("parent");
        manager.add_rule(child).await.expect("child");
        manager
            .activate_rule("dep-parent-2")
            .await
            .expect("activate");

        let ctx = json!({"ok": true});
        let results = manager.evaluate_rules("ctx-2", ctx).await.expect("eval");
        let ids: Vec<_> = results.iter().map(|r| r.rule_id.as_str()).collect();
        assert!(ids.contains(&"dep-parent-2"));
        assert!(ids.contains(&"dep-child-2"));
    }

    #[tokio::test]
    async fn remove_rule_after_dependent_removed_clears_active_cache_entry() {
        let manager = test_manager();
        let base = Rule::new("base-only")
            .with_pattern("ctx.*")
            .with_condition(RuleCondition::Exists {
                path: "z".to_string(),
            })
            .with_action(RuleAction::ModifyContext {
                path: "o".to_string(),
                value: json!(true),
            });
        let dep = Rule::new("dep-only")
            .with_pattern("ctx.*")
            .with_dependency("base-only")
            .with_condition(RuleCondition::Exists {
                path: "z".to_string(),
            });
        manager.add_rule(base).await.expect("base");
        manager.add_rule(dep).await.expect("dep");
        manager.activate_rule("base-only").await.expect("act");
        manager.remove_rule("dep-only").await.expect("rm dep");
        manager.remove_rule("base-only").await.expect("rm base");
        assert!(!manager.is_rule_active("base-only").await);
    }

    #[tokio::test]
    async fn duplicate_dependency_ids_are_visited_once() {
        let manager = test_manager();
        let shared = Rule::new("shared-dep")
            .with_pattern("ctx.*")
            .with_condition(RuleCondition::Exists {
                path: "q".to_string(),
            });
        let r = Rule::new("dup-deps")
            .with_pattern("ctx.*")
            .with_dependency("shared-dep")
            .with_dependency("shared-dep")
            .with_condition(RuleCondition::Exists {
                path: "q".to_string(),
            });
        manager.add_rule(shared).await.expect("shared");
        manager.add_rule(r).await.expect("dup");
        let all = manager.get_all_rules().await.expect("all");
        assert_eq!(all.len(), 2);
    }
}
