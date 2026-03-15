// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for the Rules Evaluator
//!
//! These tests cover condition evaluation, pattern matching, value comparison,
//! and rule matching logic.

mod basic;
mod complex;
mod edge_cases;

use crate::rules::models::{Rule, RuleAction, RuleCondition, RuleMetadata};
use std::sync::Arc;

/// Helper to create a test rule
pub(crate) fn create_test_rule(
    id: &str,
    priority: i32,
    conditions: Vec<RuleCondition>,
) -> Arc<Rule> {
    Arc::new(Rule {
        id: id.to_string(),
        name: format!("Test Rule {}", id),
        description: "Test rule".to_string(),
        version: "1.0.0".to_string(),
        category: "test".to_string(),
        priority,
        patterns: vec!["test".to_string()],
        conditions,
        actions: vec![RuleAction::LogMessage {
            level: "info".to_string(),
            message: "test".to_string(),
        }],
        metadata: RuleMetadata::new(),
    })
}
