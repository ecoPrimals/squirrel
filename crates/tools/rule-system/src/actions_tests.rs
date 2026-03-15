// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for action executor

use crate::actions::ActionExecutor;
use std::sync::Arc;

#[tokio::test]
async fn test_executor_creation() {
    let executor = ActionExecutor::new();
    // Executor should be created successfully
}

#[tokio::test]
async fn test_executor_initialize() {
    let executor = ActionExecutor::new();
    let result = executor.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_executor_concurrent_creation() {
    // Test that multiple executors can be created
    let executor1 = ActionExecutor::new();
    let executor2 = ActionExecutor::new();
    let executor3 = ActionExecutor::new();

    // All should initialize independently
    assert!(executor1.initialize().await.is_ok());
    assert!(executor2.initialize().await.is_ok());
    assert!(executor3.initialize().await.is_ok());
}

#[tokio::test]
async fn test_executor_reinitialization() {
    let executor = ActionExecutor::new();

    // Initialize multiple times should be safe
    assert!(executor.initialize().await.is_ok());
    assert!(executor.initialize().await.is_ok());
    assert!(executor.initialize().await.is_ok());
}

#[tokio::test]
async fn test_executor_arc_wrapping() {
    let executor = Arc::new(ActionExecutor::new());
    executor.initialize().await.expect("Failed to initialize");

    // Test that Arc-wrapped executor works
    let executor_clone = Arc::clone(&executor);
    let result = executor_clone.initialize().await;
    assert!(result.is_ok());
}
