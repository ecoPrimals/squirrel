// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the graceful shutdown manager.

use super::shutdown::*;
use crate::shutdown::test_handlers::{FailingShutdownHandler, MockShutdownHandler};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_shutdown_phase_description() {
    assert_eq!(
        ShutdownPhase::StopAccepting.description(),
        "Stop accepting new requests"
    );
    assert_eq!(
        ShutdownPhase::DrainRequests.description(),
        "Drain existing requests"
    );
    assert_eq!(
        ShutdownPhase::CloseConnections.description(),
        "Close network connections"
    );
    assert_eq!(
        ShutdownPhase::CleanupResources.description(),
        "Cleanup resources"
    );
    assert_eq!(
        ShutdownPhase::ShutdownTasks.description(),
        "Shutdown background tasks"
    );
    assert_eq!(ShutdownPhase::FinalCleanup.description(), "Final cleanup");
}

#[test]
fn test_shutdown_phase_ordering() {
    assert!(ShutdownPhase::StopAccepting < ShutdownPhase::DrainRequests);
    assert!(ShutdownPhase::DrainRequests < ShutdownPhase::CloseConnections);
    assert!(ShutdownPhase::CloseConnections < ShutdownPhase::CleanupResources);
    assert!(ShutdownPhase::CleanupResources < ShutdownPhase::ShutdownTasks);
    assert!(ShutdownPhase::ShutdownTasks < ShutdownPhase::FinalCleanup);
}

#[tokio::test]
async fn test_shutdown_manager_new() {
    let manager = ShutdownManager::new();
    assert!(!manager.is_shutdown_complete().await);
    assert!(!manager.is_shutdown_requested());
}

#[tokio::test]
async fn test_shutdown_manager_default() {
    let manager = ShutdownManager::default();
    assert!(!manager.is_shutdown_complete().await);
    assert!(!manager.is_shutdown_requested());
}

#[tokio::test]
async fn test_register_handler() {
    let manager = ShutdownManager::new();
    let handler = Arc::new(MockShutdownHandler::new("test-component"));

    manager
        .register_handler(
            "test-component".to_string(),
            RegisteredShutdownHandler::TestMock(handler.clone()),
        )
        .await;

    assert!(manager.unregister_handler("test-component").await);
}

#[tokio::test]
async fn test_register_multiple_handlers() {
    let manager = ShutdownManager::new();
    let handler1 = Arc::new(MockShutdownHandler::new("component-1"));
    let handler2 = Arc::new(MockShutdownHandler::new("component-2"));
    let handler3 = Arc::new(MockShutdownHandler::new("component-3"));

    manager
        .register_handler(
            "component-1".to_string(),
            RegisteredShutdownHandler::TestMock(handler1),
        )
        .await;
    manager
        .register_handler(
            "component-2".to_string(),
            RegisteredShutdownHandler::TestMock(handler2),
        )
        .await;
    manager
        .register_handler(
            "component-3".to_string(),
            RegisteredShutdownHandler::TestMock(handler3),
        )
        .await;

    assert!(manager.unregister_handler("component-1").await);
    assert!(manager.unregister_handler("component-2").await);
    assert!(manager.unregister_handler("component-3").await);
}

#[tokio::test]
async fn test_unregister_nonexistent_handler() {
    let manager = ShutdownManager::new();
    assert!(!manager.unregister_handler("nonexistent").await);
}

#[tokio::test]
async fn test_request_shutdown() {
    let manager = ShutdownManager::new();

    let result = manager.request_shutdown().await;
    assert!(result.is_ok());
    assert!(manager.is_shutdown_requested());
}

#[tokio::test]
async fn test_shutdown_signal_clone() {
    let signal1 = ShutdownSignal::Graceful;
    let signal2 = signal1;

    assert!(matches!(signal2, ShutdownSignal::Graceful));
}

#[tokio::test]
async fn test_shutdown_signal_immediate() {
    let signal = ShutdownSignal::Immediate;
    assert!(matches!(signal, ShutdownSignal::Immediate));
}

#[tokio::test]
async fn test_shutdown_signal_timeout() {
    let signal = ShutdownSignal::Timeout(ShutdownPhase::DrainRequests);
    assert!(matches!(
        signal,
        ShutdownSignal::Timeout(ShutdownPhase::DrainRequests)
    ));
}

#[tokio::test]
async fn test_mock_handler_shutdown() {
    let handler = MockShutdownHandler::new("test");

    assert!(!handler.was_shutdown_called());
    assert!(!handler.is_shutdown_complete().await);

    let result = handler.shutdown(ShutdownPhase::StopAccepting).await;
    assert!(result.is_ok());
    assert!(handler.was_shutdown_called());
    assert!(handler.is_shutdown_complete().await);
}

#[tokio::test]
async fn test_mock_handler_estimated_time() {
    let handler = MockShutdownHandler::with_delay("test", Duration::from_millis(100));
    assert_eq!(
        handler.estimated_shutdown_time(),
        Duration::from_millis(100)
    );
}

#[tokio::test]
async fn test_failing_handler() {
    let handler = FailingShutdownHandler::new("failing");

    let result = handler.shutdown(ShutdownPhase::StopAccepting).await;
    assert!(result.is_err());
    assert!(!handler.is_shutdown_complete().await);
}

#[tokio::test]
async fn test_shutdown_phase_equality() {
    assert_eq!(ShutdownPhase::StopAccepting, ShutdownPhase::StopAccepting);
    assert_ne!(ShutdownPhase::StopAccepting, ShutdownPhase::DrainRequests);
}

#[tokio::test]
async fn test_shutdown_phase_hash() {
    use std::collections::HashSet;

    let mut phases = HashSet::new();
    phases.insert(ShutdownPhase::StopAccepting);
    phases.insert(ShutdownPhase::DrainRequests);
    phases.insert(ShutdownPhase::StopAccepting);

    assert_eq!(phases.len(), 2);
}

#[tokio::test]
async fn test_default_phase_timeouts() {
    let manager = ShutdownManager::new();

    assert_eq!(
        manager.phase_timeout(ShutdownPhase::StopAccepting),
        Some(Duration::from_secs(5))
    );
    assert_eq!(
        manager.phase_timeout(ShutdownPhase::DrainRequests),
        Some(Duration::from_secs(30))
    );
    assert_eq!(
        manager.phase_timeout(ShutdownPhase::CloseConnections),
        Some(Duration::from_secs(10))
    );
    assert_eq!(
        manager.phase_timeout(ShutdownPhase::CleanupResources),
        Some(Duration::from_secs(15))
    );
    assert_eq!(
        manager.phase_timeout(ShutdownPhase::ShutdownTasks),
        Some(Duration::from_secs(10))
    );
    assert_eq!(
        manager.phase_timeout(ShutdownPhase::FinalCleanup),
        Some(Duration::from_secs(5))
    );
}

#[tokio::test]
async fn test_shutdown_requested_flag() {
    let manager = ShutdownManager::new();

    assert!(!manager.is_shutdown_requested());

    manager.request_shutdown().await.expect("should succeed");

    assert!(manager.is_shutdown_requested());
}

#[tokio::test]
async fn test_multiple_handler_registration_same_name() {
    let manager = ShutdownManager::new();
    let handler1 = Arc::new(MockShutdownHandler::new("component"));
    let handler2 = Arc::new(MockShutdownHandler::new("component"));

    manager
        .register_handler(
            "component".to_string(),
            RegisteredShutdownHandler::TestMock(handler1),
        )
        .await;
    manager
        .register_handler(
            "component".to_string(),
            RegisteredShutdownHandler::TestMock(handler2),
        )
        .await;

    assert!(manager.unregister_handler("component").await);
    assert!(!manager.unregister_handler("component").await);
}

#[tokio::test]
async fn test_handler_component_name() {
    let handler = MockShutdownHandler::new("my-component");
    assert_eq!(handler.component_name(), "my-component");
}

#[tokio::test]
async fn test_shutdown_complete_flag() {
    let manager = ShutdownManager::new();
    assert!(!manager.is_shutdown_complete().await);
}

#[tokio::test]
async fn test_register_and_unregister_sequence() {
    let manager = ShutdownManager::new();
    let handler = Arc::new(MockShutdownHandler::new("test"));

    manager
        .register_handler(
            "test".to_string(),
            RegisteredShutdownHandler::TestMock(handler.clone()),
        )
        .await;

    assert!(manager.unregister_handler("test").await);

    assert!(!manager.unregister_handler("test").await);

    manager
        .register_handler(
            "test".to_string(),
            RegisteredShutdownHandler::TestMock(handler),
        )
        .await;

    assert!(manager.unregister_handler("test").await);
}

#[tokio::test]
async fn test_shutdown_handler_with_different_delays() {
    let fast_handler = MockShutdownHandler::with_delay("fast", Duration::from_millis(10));
    let slow_handler = MockShutdownHandler::with_delay("slow", Duration::from_millis(100));

    assert_eq!(
        fast_handler.estimated_shutdown_time(),
        Duration::from_millis(10)
    );
    assert_eq!(
        slow_handler.estimated_shutdown_time(),
        Duration::from_millis(100)
    );
}

#[test]
fn test_shutdown_phase_debug() {
    let phase = ShutdownPhase::DrainRequests;
    let debug_str = format!("{phase:?}");
    assert!(debug_str.contains("DrainRequests"));
}

#[test]
fn test_shutdown_signal_debug() {
    let signal = ShutdownSignal::Graceful;
    let debug_str = format!("{signal:?}");
    assert!(debug_str.contains("Graceful"));
}

#[tokio::test]
async fn test_coordinate_shutdown_graceful() {
    let manager = Arc::new(ShutdownManager::new());
    let handler = Arc::new(MockShutdownHandler::new("test"));
    manager
        .register_handler(
            "test".to_string(),
            RegisteredShutdownHandler::TestMock(handler.clone()),
        )
        .await;

    let manager_clone = Arc::clone(&manager);
    let coord_handle = tokio::spawn(async move { manager_clone.coordinate_shutdown().await });

    manager.request_shutdown().await.expect("should succeed");

    let result = coord_handle.await.expect("should succeed");
    assert!(result.is_ok());
    assert!(manager.is_shutdown_complete().await);
}

#[tokio::test]
async fn test_coordinate_shutdown_immediate() {
    let manager = Arc::new(ShutdownManager::new());
    let handler = Arc::new(MockShutdownHandler::new("c1"));
    manager
        .register_handler(
            "c1".to_string(),
            RegisteredShutdownHandler::TestMock(handler.clone()),
        )
        .await;

    let manager_clone = Arc::clone(&manager);
    let coord_handle = tokio::spawn(async move { manager_clone.coordinate_shutdown().await });

    manager
        .test_send_shutdown_signal(ShutdownSignal::Immediate)
        .await
        .expect("should succeed");

    let result = coord_handle.await.expect("should succeed");
    assert!(result.is_ok());
    assert!(manager.is_shutdown_complete().await);
}

#[tokio::test]
async fn test_coordinate_shutdown_timeout_signal_errors() {
    let manager = Arc::new(ShutdownManager::new());
    let manager_clone = Arc::clone(&manager);
    let coord_handle = tokio::spawn(async move { manager_clone.coordinate_shutdown().await });

    manager
        .test_send_shutdown_signal(ShutdownSignal::Timeout(ShutdownPhase::DrainRequests))
        .await
        .expect("should succeed");

    let result = coord_handle.await.expect("should succeed");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_coordinate_shutdown_second_call_errors_after_first_completes() {
    let manager = Arc::new(ShutdownManager::new());
    let m2 = Arc::clone(&manager);
    let coord = tokio::spawn(async move { m2.coordinate_shutdown().await });

    manager.request_shutdown().await.expect("should succeed");
    assert!(coord.await.expect("join").is_ok());

    let second = manager.coordinate_shutdown().await;
    assert!(second.is_err());
}

#[tokio::test]
async fn test_graceful_shutdown_fails_when_handler_returns_error() {
    let manager = Arc::new(ShutdownManager::new());
    manager
        .register_handler(
            "bad".to_string(),
            RegisteredShutdownHandler::TestFailing(Arc::new(FailingShutdownHandler::new("bad"))),
        )
        .await;

    let manager_clone = Arc::clone(&manager);
    let coord = tokio::spawn(async move { manager_clone.coordinate_shutdown().await });

    manager.request_shutdown().await.expect("should succeed");
    let result = coord.await.expect("join");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_graceful_shutdown_phase_times_out() {
    let mut overrides = HashMap::new();
    overrides.insert(ShutdownPhase::StopAccepting, Duration::from_millis(1));
    let manager = Arc::new(ShutdownManager::new_with_phase_overrides(overrides));
    let slow = Arc::new(MockShutdownHandler::with_delay(
        "slow",
        Duration::from_millis(200),
    ));
    manager
        .register_handler(
            "slow".to_string(),
            RegisteredShutdownHandler::TestMock(slow),
        )
        .await;

    let manager_clone = Arc::clone(&manager);
    let coord = tokio::spawn(async move { manager_clone.coordinate_shutdown().await });

    manager.request_shutdown().await.expect("should succeed");
    let result = coord.await.expect("join");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_immediate_shutdown_tolerates_failing_handler() {
    let manager = Arc::new(ShutdownManager::new());
    manager
        .register_handler(
            "bad".to_string(),
            RegisteredShutdownHandler::TestFailing(Arc::new(FailingShutdownHandler::new("bad"))),
        )
        .await;

    let manager_clone = Arc::clone(&manager);
    let coord = tokio::spawn(async move { manager_clone.coordinate_shutdown().await });

    manager
        .test_send_shutdown_signal(ShutdownSignal::Immediate)
        .await
        .expect("should succeed");

    let result = coord.await.expect("join");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_registered_shutdown_handler_test_mock_dispatch() {
    let h = Arc::new(MockShutdownHandler::new("x"));
    let reg = RegisteredShutdownHandler::TestMock(h.clone());
    assert_eq!(reg.component_name(), "x");
    assert!(reg.shutdown(ShutdownPhase::StopAccepting).await.is_ok());
    assert!(h.is_shutdown_complete().await);
}
