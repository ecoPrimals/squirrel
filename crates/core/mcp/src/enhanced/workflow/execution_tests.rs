// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for Workflow Execution Engine
//!
//! Tests cover the newly implemented functionality:
//! - Transform step (passthrough, extract, map, filter, merge)
//! - Parallel execution (real tokio-based parallelism)
//! - Sequential execution (context passing)
//! - Condition evaluation (6 operators)

use super::execution::*;
use super::types::*;
use crate::enhanced::coordinator::{AICoordinator, UniversalAIRequest};
use crate::enhanced::events::EventBroadcaster;
use crate::enhanced::service_composition::ServiceCompositionEngine;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Helper to create test execution context
fn create_test_context() -> ExecutionContext {
    ExecutionContext {
        workflow_id: "test-workflow".to_string(),
        execution_id: "test-exec".to_string(),
        variables: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        start_time: std::time::Instant::now(),
        retry_count: 0,
        timeout: Duration::from_secs(60),
    }
}

/// Helper to create test execution engine
fn create_test_engine() -> WorkflowExecutionEngine {
    WorkflowExecutionEngine::new(ExecutionEngineConfig {
        max_parallel_steps: 10,
        default_timeout: Duration::from_secs(60),
        enable_history: true,
        max_history_entries: 100,
    })
}

/// Helper to create test dependencies
fn create_test_deps() -> (Arc<AICoordinator>, Arc<ServiceCompositionEngine>, Arc<EventBroadcaster>) {
    let event_broadcaster = Arc::new(EventBroadcaster::new());
    let service_composition = Arc::new(ServiceCompositionEngine::new(
        crate::enhanced::service_composition::ServiceCompositionConfig::default(),
        event_broadcaster.clone(),
    ));
    let ai_coordinator = Arc::new(AICoordinator::new());
    
    (ai_coordinator, service_composition, event_broadcaster)
}

// ============================================================================
// Transform Step Tests
// ============================================================================

#[tokio::test]
async fn test_transform_passthrough() {
    let engine = create_test_engine();
    let context = create_test_context();
    
    let step = WorkflowStep {
        id: "transform-1".to_string(),
        name: "Passthrough Test".to_string(),
        description: "Test passthrough transformation".to_string(),
        step_type: "transform".to_string(),
        input: serde_json::json!({"data": "test value"}),
        config: {
            let mut config = HashMap::new();
            config.insert("transform_type".to_string(), serde_json::json!("passthrough"));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let result = engine.execute_transform_step(&step, &context).await.expect("should succeed");
    assert_eq!(result, serde_json::json!({"data": "test value"}));
}

#[tokio::test]
async fn test_transform_extract() {
    let engine = create_test_engine();
    let context = create_test_context();
    
    let step = WorkflowStep {
        id: "transform-2".to_string(),
        name: "Extract Test".to_string(),
        description: "Test field extraction".to_string(),
        step_type: "transform".to_string(),
        input: serde_json::json!({"name": "John", "age": 30, "city": "NYC"}),
        config: {
            let mut config = HashMap::new();
            config.insert("transform_type".to_string(), serde_json::json!("extract"));
            config.insert("field".to_string(), serde_json::json!("name"));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let result = engine.execute_transform_step(&step, &context).await.expect("should succeed");
    assert_eq!(result, serde_json::json!("John"));
}

#[tokio::test]
async fn test_transform_map() {
    let engine = create_test_engine();
    let context = create_test_context();
    
    let step = WorkflowStep {
        id: "transform-3".to_string(),
        name: "Map Test".to_string(),
        description: "Test map transformation".to_string(),
        step_type: "transform".to_string(),
        input: serde_json::json!([
            {"name": "Alice", "age": 25, "score": 95},
            {"name": "Bob", "age": 30, "score": 88},
            {"name": "Charlie", "age": 35, "score": 92}
        ]),
        config: {
            let mut config = HashMap::new();
            config.insert("transform_type".to_string(), serde_json::json!("map"));
            config.insert("map_fields".to_string(), serde_json::json!(["name", "score"]));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let result = engine.execute_transform_step(&step, &context).await.expect("should succeed");
    let expected = serde_json::json!([
        {"name": "Alice", "score": 95},
        {"name": "Bob", "score": 88},
        {"name": "Charlie", "score": 92}
    ]);
    assert_eq!(result, expected);
}

#[tokio::test]
async fn test_transform_filter() {
    let engine = create_test_engine();
    let context = create_test_context();
    
    let step = WorkflowStep {
        id: "transform-4".to_string(),
        name: "Filter Test".to_string(),
        description: "Test filter transformation".to_string(),
        step_type: "transform".to_string(),
        input: serde_json::json!([
            {"name": "Alice", "active": true},
            {"name": "Bob", "active": false},
            {"name": "Charlie", "active": true}
        ]),
        config: {
            let mut config = HashMap::new();
            config.insert("transform_type".to_string(), serde_json::json!("filter"));
            config.insert("filter_field".to_string(), serde_json::json!("active"));
            config.insert("filter_value".to_string(), serde_json::json!(true));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let result = engine.execute_transform_step(&step, &context).await.expect("should succeed");
    let expected = serde_json::json!([
        {"name": "Alice", "active": true},
        {"name": "Charlie", "active": true}
    ]);
    assert_eq!(result, expected);
}

#[tokio::test]
async fn test_transform_merge() {
    let engine = create_test_engine();
    let mut context = create_test_context();
    
    // Add merge data to context
    context.set_variable("extra_data", serde_json::json!({"city": "NYC", "country": "USA"})).expect("should succeed");
    
    let step = WorkflowStep {
        id: "transform-5".to_string(),
        name: "Merge Test".to_string(),
        description: "Test merge transformation".to_string(),
        step_type: "transform".to_string(),
        input: serde_json::json!({"name": "Alice", "age": 25}),
        config: {
            let mut config = HashMap::new();
            config.insert("transform_type".to_string(), serde_json::json!("merge"));
            config.insert("merge_with".to_string(), serde_json::json!("extra_data"));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let result = engine.execute_transform_step(&step, &context).await.expect("should succeed");
    let expected = serde_json::json!({
        "name": "Alice",
        "age": 25,
        "city": "NYC",
        "country": "USA"
    });
    assert_eq!(result, expected);
}

// ============================================================================
// Parallel Execution Tests
// ============================================================================

#[tokio::test]
async fn test_parallel_execution_success() {
    let engine = create_test_engine();
    let context = create_test_context();
    let (ai, svc, evt) = create_test_deps();
    
    let step = WorkflowStep {
        id: "parallel-1".to_string(),
        name: "Parallel Test".to_string(),
        description: "Test parallel execution".to_string(),
        step_type: "parallel".to_string(),
        input: serde_json::Value::Null,
        config: {
            let mut config = HashMap::new();
            config.insert("steps".to_string(), serde_json::json!([
                {
                    "id": "sub-1",
                    "name": "Sub Step 1",
                    "description": "First sub-step",
                    "step_type": "transform",
                    "input": {"value": 1},
                    "config": {"transform_type": "passthrough"}
                },
                {
                    "id": "sub-2",
                    "name": "Sub Step 2",
                    "description": "Second sub-step",
                    "step_type": "transform",
                    "input": {"value": 2},
                    "config": {"transform_type": "passthrough"}
                },
                {
                    "id": "sub-3",
                    "name": "Sub Step 3",
                    "description": "Third sub-step",
                    "step_type": "transform",
                    "input": {"value": 3},
                    "config": {"transform_type": "passthrough"}
                }
            ]));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let start = std::time::Instant::now();
    let result = engine.execute_parallel_step(&step, &context, &ai, &svc, &evt).await.expect("should succeed");
    let duration = start.elapsed();
    
    // All results should be present
    assert!(result.is_array());
    let results = result.as_array().expect("should succeed");
    assert_eq!(results.len(), 3);
    
    // Parallel execution should be fast (no significant delays)
    assert!(duration < Duration::from_millis(200));
}

#[tokio::test]
async fn test_parallel_execution_with_max_concurrency() {
    let engine = WorkflowExecutionEngine::new(ExecutionEngineConfig {
        max_parallel_steps: 2, // Limit to 2 concurrent steps
        default_timeout: Duration::from_secs(60),
        enable_history: true,
        max_history_entries: 100,
    });
    
    let context = create_test_context();
    let (ai, svc, evt) = create_test_deps();
    
    let step = WorkflowStep {
        id: "parallel-2".to_string(),
        name: "Limited Parallel Test".to_string(),
        description: "Test parallel execution with concurrency limit".to_string(),
        step_type: "parallel".to_string(),
        input: serde_json::Value::Null,
        config: {
            let mut config = HashMap::new();
            config.insert("steps".to_string(), serde_json::json!([
                {"id": "s1", "name": "Step 1", "description": "", "step_type": "transform", "input": {}, "config": {"transform_type": "passthrough"}},
                {"id": "s2", "name": "Step 2", "description": "", "step_type": "transform", "input": {}, "config": {"transform_type": "passthrough"}},
                {"id": "s3", "name": "Step 3", "description": "", "step_type": "transform", "input": {}, "config": {"transform_type": "passthrough"}},
                {"id": "s4", "name": "Step 4", "description": "", "step_type": "transform", "input": {}, "config": {"transform_type": "passthrough"}}
            ]));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let result = engine.execute_parallel_step(&step, &context, &ai, &svc, &evt).await.expect("should succeed");
    
    // All 4 steps should complete
    assert!(result.is_array());
    assert_eq!(result.as_array().expect("should succeed").len(), 4);
}

// ============================================================================
// Sequential Execution Tests
// ============================================================================

#[tokio::test]
async fn test_sequential_execution_with_context_passing() {
    let engine = create_test_engine();
    let context = create_test_context();
    let (ai, svc, evt) = create_test_deps();
    
    let step = WorkflowStep {
        id: "sequential-1".to_string(),
        name: "Sequential Test".to_string(),
        description: "Test sequential execution with context passing".to_string(),
        step_type: "sequential".to_string(),
        input: serde_json::Value::Null,
        config: {
            let mut config = HashMap::new();
            config.insert("steps".to_string(), serde_json::json!([
                {
                    "id": "seq-1",
                    "name": "First Step",
                    "description": "",
                    "step_type": "transform",
                    "input": {"value": "hello"},
                    "config": {"transform_type": "passthrough"}
                },
                {
                    "id": "seq-2",
                    "name": "Second Step",
                    "description": "",
                    "step_type": "transform",
                    "input": {"value": "world"},
                    "config": {"transform_type": "passthrough"}
                }
            ]));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let result = engine.execute_sequential_step(&step, &context, &ai, &svc, &evt).await.expect("should succeed");
    
    // Should return last result by default
    assert_eq!(result, serde_json::json!({"value": "world"}));
}

#[tokio::test]
async fn test_sequential_execution_return_all_results() {
    let engine = create_test_engine();
    let context = create_test_context();
    let (ai, svc, evt) = create_test_deps();
    
    let step = WorkflowStep {
        id: "sequential-2".to_string(),
        name: "Sequential Return All Test".to_string(),
        description: "Test sequential execution returning all results".to_string(),
        step_type: "sequential".to_string(),
        input: serde_json::Value::Null,
        config: {
            let mut config = HashMap::new();
            config.insert("return_all_results".to_string(), serde_json::json!(true));
            config.insert("steps".to_string(), serde_json::json!([
                {
                    "id": "seq-1",
                    "name": "First Step",
                    "description": "",
                    "step_type": "transform",
                    "input": {"value": 1},
                    "config": {"transform_type": "passthrough"}
                },
                {
                    "id": "seq-2",
                    "name": "Second Step",
                    "description": "",
                    "step_type": "transform",
                    "input": {"value": 2},
                    "config": {"transform_type": "passthrough"}
                },
                {
                    "id": "seq-3",
                    "name": "Third Step",
                    "description": "",
                    "step_type": "transform",
                    "input": {"value": 3},
                    "config": {"transform_type": "passthrough"}
                }
            ]));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let result = engine.execute_sequential_step(&step, &context, &ai, &svc, &evt).await.expect("should succeed");
    
    // Should return all results
    assert!(result.is_array());
    let results = result.as_array().expect("should succeed");
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], serde_json::json!({"value": 1}));
    assert_eq!(results[1], serde_json::json!({"value": 2}));
    assert_eq!(results[2], serde_json::json!({"value": 3}));
}

#[tokio::test]
async fn test_sequential_execution_continue_on_error() {
    let engine = create_test_engine();
    let context = create_test_context();
    let (ai, svc, evt) = create_test_deps();
    
    let step = WorkflowStep {
        id: "sequential-3".to_string(),
        name: "Sequential Continue on Error Test".to_string(),
        description: "Test sequential execution continuing after errors".to_string(),
        step_type: "sequential".to_string(),
        input: serde_json::Value::Null,
        config: {
            let mut config = HashMap::new();
            config.insert("continue_on_error".to_string(), serde_json::json!(true));
            config.insert("return_all_results".to_string(), serde_json::json!(true));
            config.insert("steps".to_string(), serde_json::json!([
                {
                    "id": "seq-1",
                    "name": "Good Step",
                    "description": "",
                    "step_type": "transform",
                    "input": {"value": "ok"},
                    "config": {"transform_type": "passthrough"}
                },
                {
                    "id": "seq-2",
                    "name": "Error Step",
                    "description": "",
                    "step_type": "transform",
                    "input": {},
                    "config": {
                        "transform_type": "extract",
                        "field": "nonexistent_field" // This will return null
                    }
                },
                {
                    "id": "seq-3",
                    "name": "Recovery Step",
                    "description": "",
                    "step_type": "transform",
                    "input": {"value": "recovered"},
                    "config": {"transform_type": "passthrough"}
                }
            ]));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let result = engine.execute_sequential_step(&step, &context, &ai, &svc, &evt).await.expect("should succeed");
    
    // Should continue and complete all steps despite errors
    assert!(result.is_array());
    let results = result.as_array().expect("should succeed");
    assert_eq!(results.len(), 3);
}

// ============================================================================
// Condition Evaluation Tests
// ============================================================================

#[tokio::test]
async fn test_condition_equals() {
    let engine = create_test_engine();
    let mut context = create_test_context();
    context.set_variable("status", serde_json::json!("active")).expect("should succeed");
    
    let result = engine.evaluate_condition("status == \"active\"", &context).await.expect("should succeed");
    assert!(result);
    
    let result = engine.evaluate_condition("status == \"inactive\"", &context).await.expect("should succeed");
    assert!(!result);
}

#[tokio::test]
async fn test_condition_not_equals() {
    let engine = create_test_engine();
    let mut context = create_test_context();
    context.set_variable("status", serde_json::json!("active")).expect("should succeed");
    
    let result = engine.evaluate_condition("status != \"inactive\"", &context).await.expect("should succeed");
    assert!(result);
    
    let result = engine.evaluate_condition("status != \"active\"", &context).await.expect("should succeed");
    assert!(!result);
}

#[tokio::test]
async fn test_condition_contains() {
    let engine = create_test_engine();
    let mut context = create_test_context();
    context.set_variable("message", serde_json::json!("Hello, world!")).expect("should succeed");
    
    let result = engine.evaluate_condition("message contains \"world\"", &context).await.expect("should succeed");
    assert!(result);
    
    let result = engine.evaluate_condition("message contains \"goodbye\"", &context).await.expect("should succeed");
    assert!(!result);
}

#[tokio::test]
async fn test_condition_greater_than() {
    let engine = create_test_engine();
    let mut context = create_test_context();
    context.set_variable("count", serde_json::json!(10)).expect("should succeed");
    
    let result = engine.evaluate_condition("count > 5", &context).await.expect("should succeed");
    assert!(result);
    
    let result = engine.evaluate_condition("count > 15", &context).await.expect("should succeed");
    assert!(!result);
}

#[tokio::test]
async fn test_condition_less_than() {
    let engine = create_test_engine();
    let mut context = create_test_context();
    context.set_variable("count", serde_json::json!(10)).expect("should succeed");
    
    let result = engine.evaluate_condition("count < 15", &context).await.expect("should succeed");
    assert!(result);
    
    let result = engine.evaluate_condition("count < 5", &context).await.expect("should succeed");
    assert!(!result);
}

#[tokio::test]
async fn test_condition_exists() {
    let engine = create_test_engine();
    let mut context = create_test_context();
    context.set_variable("enabled", serde_json::json!(true)).expect("should succeed");
    
    let result = engine.evaluate_condition("exists enabled", &context).await.expect("should succeed");
    assert!(result);
    
    let result = engine.evaluate_condition("exists nonexistent", &context).await.expect("should succeed");
    assert!(!result);
}

#[tokio::test]
async fn test_condition_boolean_variable() {
    let engine = create_test_engine();
    let mut context = create_test_context();
    context.set_variable("is_enabled", serde_json::json!(true)).expect("should succeed");
    context.set_variable("is_disabled", serde_json::json!(false)).expect("should succeed");
    
    let result = engine.evaluate_condition("is_enabled", &context).await.expect("should succeed");
    assert!(result);
    
    let result = engine.evaluate_condition("is_disabled", &context).await.expect("should succeed");
    assert!(!result);
}

#[tokio::test]
async fn test_condition_missing_variable() {
    let engine = create_test_engine();
    let context = create_test_context();
    
    // Missing variable should evaluate to false
    let result = engine.evaluate_condition("missing_var == \"value\"", &context).await.expect("should succeed");
    assert!(!result);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_complex_transform_pipeline() {
    let engine = create_test_engine();
    let mut context = create_test_context();
    
    // 1. Filter active users
    let filter_step = WorkflowStep {
        id: "filter".to_string(),
        name: "Filter Active".to_string(),
        description: "".to_string(),
        step_type: "transform".to_string(),
        input: serde_json::json!([
            {"name": "Alice", "active": true, "score": 95},
            {"name": "Bob", "active": false, "score": 88},
            {"name": "Charlie", "active": true, "score": 92}
        ]),
        config: {
            let mut config = HashMap::new();
            config.insert("transform_type".to_string(), serde_json::json!("filter"));
            config.insert("filter_field".to_string(), serde_json::json!("active"));
            config.insert("filter_value".to_string(), serde_json::json!(true));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let filtered = engine.execute_transform_step(&filter_step, &context).await.expect("should succeed");
    
    // 2. Map to extract names and scores
    let map_step = WorkflowStep {
        id: "map".to_string(),
        name: "Extract Fields".to_string(),
        description: "".to_string(),
        step_type: "transform".to_string(),
        input: filtered,
        config: {
            let mut config = HashMap::new();
            config.insert("transform_type".to_string(), serde_json::json!("map"));
            config.insert("map_fields".to_string(), serde_json::json!(["name", "score"]));
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let mapped = engine.execute_transform_step(&map_step, &context).await.expect("should succeed");
    
    // Verify final result
    let expected = serde_json::json!([
        {"name": "Alice", "score": 95},
        {"name": "Charlie", "score": 92}
    ]);
    assert_eq!(mapped, expected);
}

#[tokio::test]
async fn test_parallel_vs_sequential_performance() {
    let engine = create_test_engine();
    let context = create_test_context();
    let (ai, svc, evt) = create_test_deps();
    
    // Create 5 simple transform steps
    let steps = serde_json::json!([
        {"id": "s1", "name": "Step 1", "description": "", "step_type": "transform", "input": {"v": 1}, "config": {"transform_type": "passthrough"}},
        {"id": "s2", "name": "Step 2", "description": "", "step_type": "transform", "input": {"v": 2}, "config": {"transform_type": "passthrough"}},
        {"id": "s3", "name": "Step 3", "description": "", "step_type": "transform", "input": {"v": 3}, "config": {"transform_type": "passthrough"}},
        {"id": "s4", "name": "Step 4", "description": "", "step_type": "transform", "input": {"v": 4}, "config": {"transform_type": "passthrough"}},
        {"id": "s5", "name": "Step 5", "description": "", "step_type": "transform", "input": {"v": 5}, "config": {"transform_type": "passthrough"}}
    ]);
    
    // Test parallel execution
    let parallel_step = WorkflowStep {
        id: "parallel".to_string(),
        name: "Parallel".to_string(),
        description: "".to_string(),
        step_type: "parallel".to_string(),
        input: serde_json::Value::Null,
        config: {
            let mut config = HashMap::new();
            config.insert("steps".to_string(), steps.clone());
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let parallel_start = std::time::Instant::now();
    let _parallel_result = engine.execute_parallel_step(&parallel_step, &context, &ai, &svc, &evt).await.expect("should succeed");
    let parallel_duration = parallel_start.elapsed();
    
    // Test sequential execution
    let sequential_step = WorkflowStep {
        id: "sequential".to_string(),
        name: "Sequential".to_string(),
        description: "".to_string(),
        step_type: "sequential".to_string(),
        input: serde_json::Value::Null,
        config: {
            let mut config = HashMap::new();
            config.insert("steps".to_string(), steps);
            config
        },
        condition: None,
        timeout: None,
        retry: None,
        dependencies: vec![],
    };
    
    let sequential_start = std::time::Instant::now();
    let _sequential_result = engine.execute_sequential_step(&sequential_step, &context, &ai, &svc, &evt).await.expect("should succeed");
    let sequential_duration = sequential_start.elapsed();
    
    // Parallel should be comparable or faster (allowing for variance)
    // This is a smoke test - in production parallel would show more benefit
    println!("Parallel: {:?}, Sequential: {:?}", parallel_duration, sequential_duration);
    assert!(parallel_duration < Duration::from_secs(5));
    assert!(sequential_duration < Duration::from_secs(5));
}

