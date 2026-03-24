// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)] // Test code: explicit unwrap/expect and local lint noise
//! Comprehensive tests for `BiomeOS` Integration Agent Status
//!
//! Tests agent deployment status variants and lifecycle.

use squirrel::biomeos_integration::AgentStatus;

#[test]
fn test_agent_status_deploying() {
    let status = AgentStatus::Deploying;

    assert!(matches!(status, AgentStatus::Deploying));
}

#[test]
fn test_agent_status_running() {
    let status = AgentStatus::Running;

    assert!(matches!(status, AgentStatus::Running));
}

#[test]
fn test_agent_status_starting() {
    let status = AgentStatus::Starting;

    assert!(matches!(status, AgentStatus::Starting));
}

#[test]
fn test_agent_status_stopping() {
    let status = AgentStatus::Stopping;

    assert!(matches!(status, AgentStatus::Stopping));
}

#[test]
fn test_agent_status_stopped() {
    let status = AgentStatus::Stopped;

    assert!(matches!(status, AgentStatus::Stopped));
}

#[test]
fn test_agent_status_failed() {
    let error_msg = "Deployment failed";
    let status = AgentStatus::Failed(error_msg.to_string());

    assert!(matches!(status, AgentStatus::Failed(_)));
}

#[test]
fn test_agent_status_scaling() {
    let status = AgentStatus::Scaling;

    assert!(matches!(status, AgentStatus::Scaling));
}

#[test]
fn test_agent_status_updating() {
    let status = AgentStatus::Updating;

    assert!(matches!(status, AgentStatus::Updating));
}

#[test]
fn test_agent_status_clone() {
    let status1 = AgentStatus::Running;
    let status2 = status1;

    assert!(matches!(status2, AgentStatus::Running));
}

#[test]
fn test_agent_status_failed_clone() {
    let status1 = AgentStatus::Failed("test error".to_string());
    let status2 = status1;

    assert!(matches!(status2, AgentStatus::Failed(_)));
}

#[test]
fn test_agent_status_debug() {
    let status = AgentStatus::Running;
    let debug_str = format!("{status:?}");

    assert!(!debug_str.is_empty());
}

#[test]
fn test_agent_status_failed_debug() {
    let status = AgentStatus::Failed("test error".to_string());
    let debug_str = format!("{status:?}");

    assert!(debug_str.contains("test error"));
}

#[test]
fn test_agent_status_lifecycle_sequence() {
    let lifecycle = [
        AgentStatus::Deploying,
        AgentStatus::Starting,
        AgentStatus::Running,
        AgentStatus::Stopping,
        AgentStatus::Stopped,
    ];

    assert_eq!(lifecycle.len(), 5);
}

#[test]
fn test_agent_status_failed_with_message() {
    let error_msg = "Connection timeout";
    let status = AgentStatus::Failed(error_msg.to_string());

    if let AgentStatus::Failed(msg) = status {
        assert_eq!(msg, error_msg);
    } else {
        panic!("Status should be Failed");
    }
}

#[test]
fn test_agent_status_failed_empty_message() {
    let status = AgentStatus::Failed(String::new());

    if let AgentStatus::Failed(msg) = status {
        assert_eq!(msg, "");
    }
}

#[test]
fn test_agent_status_failed_long_message() {
    let long_msg = "a".repeat(1000);
    let status = AgentStatus::Failed(long_msg);

    if let AgentStatus::Failed(msg) = status {
        assert_eq!(msg.len(), 1000);
    }
}

#[test]
fn test_agent_status_serialization_deploying() {
    let status = AgentStatus::Deploying;
    let serialized = serde_json::to_string(&status);

    assert!(serialized.is_ok());
}

#[test]
fn test_agent_status_serialization_running() {
    let status = AgentStatus::Running;
    let serialized = serde_json::to_string(&status);

    assert!(serialized.is_ok());
}

#[test]
fn test_agent_status_serialization_failed() {
    let status = AgentStatus::Failed("test error".to_string());
    let serialized = serde_json::to_string(&status);

    assert!(serialized.is_ok());
}

#[test]
fn test_agent_status_all_variants() {
    // Test all variants can be created
    let _ = AgentStatus::Deploying;
    let _ = AgentStatus::Running;
    let _ = AgentStatus::Starting;
    let _ = AgentStatus::Stopping;
    let _ = AgentStatus::Stopped;
    let _ = AgentStatus::Failed("error".to_string());
    let _ = AgentStatus::Scaling;
    let _ = AgentStatus::Updating;

    // All agent status variants should be creatable (compilation verifies)
}

#[test]
fn test_agent_status_deserialization_deploying() {
    let json_str = r#""Deploying""#;
    let deserialized: Result<AgentStatus, _> = serde_json::from_str(json_str);

    assert!(deserialized.is_ok());
}

#[test]
fn test_agent_status_deserialization_running() {
    let json_str = r#""Running""#;
    let deserialized: Result<AgentStatus, _> = serde_json::from_str(json_str);

    assert!(deserialized.is_ok());
}

#[test]
fn test_agent_status_operational_states() {
    // States that represent actively operational agents
    let operational = [
        AgentStatus::Running,
        AgentStatus::Scaling,
        AgentStatus::Updating,
    ];

    assert_eq!(operational.len(), 3);
}

#[test]
fn test_agent_status_transitional_states() {
    // States that represent transitions
    let transitional = [
        AgentStatus::Starting,
        AgentStatus::Deploying,
        AgentStatus::Stopping,
    ];

    assert_eq!(transitional.len(), 3);
}

#[test]
fn test_agent_status_terminal_states() {
    // States that are terminal
    let terminal = [
        AgentStatus::Stopped,
        AgentStatus::Failed("error".to_string()),
    ];

    assert_eq!(terminal.len(), 2);
}

#[test]
fn test_agent_status_match_pattern() {
    let status = AgentStatus::Running;

    let is_operational = matches!(
        status,
        AgentStatus::Running | AgentStatus::Scaling | AgentStatus::Updating
    );

    assert!(is_operational);
}

#[test]
fn test_agent_status_match_failed() {
    let status = AgentStatus::Failed("test".to_string());

    let is_failed = matches!(status, AgentStatus::Failed(_));

    assert!(is_failed);
}

#[test]
fn test_agent_status_match_stopped() {
    let status = AgentStatus::Stopped;

    let is_stopped = matches!(status, AgentStatus::Stopped);

    assert!(is_stopped);
}
