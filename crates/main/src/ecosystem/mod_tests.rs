// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for ecosystem module

use super::*;
use crate::monitoring::metrics::MetricsCollector;
use chrono::Utc;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

// ===== EcosystemPrimalType Tests =====

#[test]
fn test_ecosystem_primal_type_as_str() {
    assert_eq!(EcosystemPrimalType::ToadStool.as_str(), "toadstool");
    assert_eq!(EcosystemPrimalType::Songbird.as_str(), "songbird");
    assert_eq!(EcosystemPrimalType::BearDog.as_str(), "beardog");
    assert_eq!(EcosystemPrimalType::NestGate.as_str(), "nestgate");
    assert_eq!(EcosystemPrimalType::Squirrel.as_str(), "squirrel");
    assert_eq!(EcosystemPrimalType::BiomeOS.as_str(), "biomeos");
}

#[test]
fn test_ecosystem_primal_type_from_str_valid() {
    assert_eq!(
        EcosystemPrimalType::from_str("toadstool").unwrap(),
        EcosystemPrimalType::ToadStool
    );
    assert_eq!(
        EcosystemPrimalType::from_str("SONGBIRD").unwrap(),
        EcosystemPrimalType::Songbird
    );
    assert_eq!(
        EcosystemPrimalType::from_str("BeArDoG").unwrap(),
        EcosystemPrimalType::BearDog
    );
    assert_eq!(
        EcosystemPrimalType::from_str("nestgate").unwrap(),
        EcosystemPrimalType::NestGate
    );
    assert_eq!(
        EcosystemPrimalType::from_str("squirrel").unwrap(),
        EcosystemPrimalType::Squirrel
    );
    assert_eq!(
        EcosystemPrimalType::from_str("biomeos").unwrap(),
        EcosystemPrimalType::BiomeOS
    );
}

#[test]
fn test_ecosystem_primal_type_from_str_invalid() {
    assert!(EcosystemPrimalType::from_str("invalid").is_err());
    assert!(EcosystemPrimalType::from_str("").is_err());
    assert!(EcosystemPrimalType::from_str("unknown").is_err());
}

#[test]
fn test_ecosystem_primal_type_service_name() {
    assert_eq!(EcosystemPrimalType::ToadStool.service_name(), "toadstool");
    assert_eq!(EcosystemPrimalType::Songbird.service_name(), "songbird");
    assert_eq!(EcosystemPrimalType::BearDog.service_name(), "beardog");
    assert_eq!(EcosystemPrimalType::NestGate.service_name(), "nestgate");
    assert_eq!(EcosystemPrimalType::Squirrel.service_name(), "squirrel");
    assert_eq!(EcosystemPrimalType::BiomeOS.service_name(), "biomeos");
}

#[test]
fn test_ecosystem_primal_type_env_name() {
    assert_eq!(EcosystemPrimalType::ToadStool.env_name(), "TOADSTOOL");
    assert_eq!(EcosystemPrimalType::Songbird.env_name(), "SONGBIRD");
    assert_eq!(EcosystemPrimalType::BearDog.env_name(), "BEARDOG");
    assert_eq!(EcosystemPrimalType::NestGate.env_name(), "NESTGATE");
    assert_eq!(EcosystemPrimalType::Squirrel.env_name(), "SQUIRREL");
    assert_eq!(EcosystemPrimalType::BiomeOS.env_name(), "BIOMEOS");
}

#[test]
fn test_ecosystem_primal_type_display() {
    assert_eq!(format!("{}", EcosystemPrimalType::Squirrel), "squirrel");
    assert_eq!(format!("{}", EcosystemPrimalType::Songbird), "songbird");
}

// ===== EcosystemConfig Tests =====

#[test]
fn test_ecosystem_config_default() {
    let config = EcosystemConfig::default();

    assert!(!config.service_id.is_empty());
    assert!(config.service_id.starts_with("primal-squirrel-"));
    assert!(!config.service_name.is_empty());
    assert!(!config.service_host.is_empty());
    assert!(config.service_port > 0);
    assert!(!config.service_mesh_endpoint.is_empty());
}

#[test]
fn test_ecosystem_config_new() {
    let config = EcosystemConfig::new("test-service".to_string(), "127.0.0.1".to_string(), 9090);

    assert_eq!(config.service_name, "test-service");
    assert_eq!(config.service_host, "127.0.0.1");
    assert_eq!(config.service_port, 9090);
    assert!(!config.service_id.is_empty());
}

#[test]
fn test_ecosystem_config_validation() {
    let config = EcosystemConfig::default();
    assert!(config.validate().is_ok());

    // Test invalid configs
    let mut invalid = config.clone();
    invalid.service_name = String::new();
    assert!(invalid.validate().is_err());

    let mut invalid2 = config.clone();
    invalid2.service_host = String::new();
    assert!(invalid2.validate().is_err());

    let mut invalid3 = config.clone();
    invalid3.service_port = 0;
    assert!(invalid3.validate().is_err());
}

#[test]
fn test_ecosystem_config_service_url() {
    let config = EcosystemConfig::new("test".to_string(), "example.com".to_string(), 8080);
    assert_eq!(config.service_url(), "http://example.com:8080");
}

// ===== EcosystemManagerStatus Tests =====

#[test]
fn test_ecosystem_manager_status_creation() {
    let status = EcosystemManagerStatus {
        status: "initializing".to_string(),
        initialized_at: None,
        last_registration: None,
        active_registrations: Vec::new(),
        health_status: HealthStatus {
            health_score: 0.0,
            component_statuses: HashMap::new(),
            last_check: Utc::now(),
            health_errors: Vec::new(),
        },
        error_count: 0,
        last_error: None,
    };

    assert_eq!(status.status, "initializing");
    assert!(status.initialized_at.is_none());
    assert_eq!(status.active_registrations.len(), 0);
    assert_eq!(status.error_count, 0);
}

#[test]
fn test_ecosystem_manager_status_with_health() {
    let mut component_statuses = HashMap::new();
    component_statuses.insert(
        "database".to_string(),
        ComponentHealth {
            status: "healthy".to_string(),
            last_check: Utc::now(),
            error: None,
            metadata: HashMap::new(),
        },
    );

    let status = EcosystemManagerStatus {
        status: "initialized".to_string(),
        initialized_at: Some(Utc::now()),
        last_registration: Some(Utc::now()),
        active_registrations: vec![Arc::from("service-1")],
        health_status: HealthStatus {
            health_score: 1.0,
            component_statuses,
            last_check: Utc::now(),
            health_errors: Vec::new(),
        },
        error_count: 0,
        last_error: None,
    };

    assert_eq!(status.status, "initialized");
    assert!(status.initialized_at.is_some());
    assert_eq!(status.active_registrations.len(), 1);
    assert_eq!(status.health_status.health_score, 1.0);
}

// ===== EcosystemManager Tests =====

fn create_test_manager() -> EcosystemManager {
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    EcosystemManager::new(config, metrics)
}

#[test]
fn test_ecosystem_manager_creation() {
    let manager = create_test_manager();

    assert!(!manager.config.service_name.is_empty());
    assert!(!manager.config.service_id.is_empty());
    assert_eq!(manager.config.service_host, "localhost");
}

#[tokio::test]
async fn test_ecosystem_manager_initial_status() {
    let manager = create_test_manager();
    let status = manager.status.read().await;

    assert_eq!(status.status, "initializing");
    assert!(status.initialized_at.is_none());
    assert_eq!(status.active_registrations.len(), 0);
    assert_eq!(status.error_count, 0);
    assert!(status.last_error.is_none());
    assert_eq!(status.health_status.health_score, 0.0);
}

#[tokio::test]
async fn test_ecosystem_manager_initialize() {
    let mut manager = create_test_manager();

    let result = manager.initialize().await;
    assert!(result.is_ok());

    let status = manager.status.read().await;
    assert_eq!(status.status, "initialized");
    assert!(status.initialized_at.is_some());
}

#[tokio::test]
async fn test_ecosystem_manager_get_status() {
    let manager = create_test_manager();
    let status = manager.get_manager_status().await;

    assert_eq!(status.status, "initializing");
    assert_eq!(status.error_count, 0);
}

#[tokio::test]
async fn test_ecosystem_manager_with_custom_config() {
    let config = EcosystemConfig::new("custom-squirrel".to_string(), "0.0.0.0".to_string(), 9090);
    let metrics = Arc::new(MetricsCollector::new());
    let manager = EcosystemManager::new(config, metrics);

    assert_eq!(manager.config.service_name, "custom-squirrel");
    assert_eq!(manager.config.service_port, 9090);
}

#[tokio::test]
async fn test_ecosystem_manager_concurrent_status_access() {
    let manager = Arc::new(create_test_manager());

    let mut handles = vec![];
    for _ in 0..10 {
        let manager_clone = Arc::clone(&manager);
        handles.push(tokio::spawn(async move {
            let status = manager_clone.status.read().await;
            assert_eq!(status.status, "initializing");
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
