// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for capability-based discovery
//!
//! Tests the new environment variable and service discovery features

// EcosystemPrimalType is deprecated but needed for backward compatibility in tests
use squirrel::ecosystem::EcosystemPrimalType;
use std::str::FromStr;

#[test]
fn test_endpoint_env_prefix_all_primals() {
    assert_eq!(
        EcosystemPrimalType::Squirrel.endpoint_env_prefix(),
        "SQUIRREL"
    );
    assert_eq!(
        EcosystemPrimalType::Songbird.endpoint_env_prefix(),
        "SERVICE_MESH"
    );
    assert_eq!(
        EcosystemPrimalType::ToadStool.endpoint_env_prefix(),
        "COMPUTE"
    );
    assert_eq!(
        EcosystemPrimalType::BearDog.endpoint_env_prefix(),
        "SECURITY"
    );
    assert_eq!(
        EcosystemPrimalType::NestGate.endpoint_env_prefix(),
        "STORAGE"
    );
    assert_eq!(
        EcosystemPrimalType::BiomeOS.endpoint_env_prefix(),
        "ECOSYSTEM"
    );
}

#[test]
fn test_service_name_all_primals() {
    assert_eq!(EcosystemPrimalType::Squirrel.service_name(), "squirrel");
    assert_eq!(EcosystemPrimalType::Songbird.service_name(), "songbird");
    assert_eq!(EcosystemPrimalType::ToadStool.service_name(), "toadstool");
    assert_eq!(EcosystemPrimalType::BearDog.service_name(), "beardog");
    assert_eq!(EcosystemPrimalType::NestGate.service_name(), "nestgate");
    assert_eq!(EcosystemPrimalType::BiomeOS.service_name(), "biomeos");
}

#[test]
fn test_service_name_matches_as_str() {
    for primal in &[
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::BiomeOS,
    ] {
        assert_eq!(primal.service_name(), primal.as_str());
    }
}

#[test]
fn test_endpoint_env_prefix_format() {
    // Verify capability-derived prefixes are uppercase (and non-empty)
    for primal in &[
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::BiomeOS,
    ] {
        let prefix = primal.endpoint_env_prefix();
        assert_eq!(prefix, prefix.to_uppercase());
        assert!(!prefix.is_empty());
    }
}

#[test]
fn test_primal_type_roundtrip() {
    // Test that as_str() -> from_str() roundtrips correctly
    for primal in &[
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::BiomeOS,
    ] {
        let str_repr = primal.as_str();
        let parsed = EcosystemPrimalType::from_str(str_repr).expect("should succeed");
        assert_eq!(primal, &parsed);
    }
}

#[test]
fn test_from_str_case_insensitive() {
    // Test case insensitivity
    assert_eq!(
        EcosystemPrimalType::from_str("SONGBIRD").expect("should succeed"),
        EcosystemPrimalType::Songbird
    );
    assert_eq!(
        EcosystemPrimalType::from_str("SongBird").expect("should succeed"),
        EcosystemPrimalType::Songbird
    );
    assert_eq!(
        EcosystemPrimalType::from_str("songbird").expect("should succeed"),
        EcosystemPrimalType::Songbird
    );
}

#[test]
fn test_from_str_invalid() {
    assert!(EcosystemPrimalType::from_str("invalid").is_err());
    assert!(EcosystemPrimalType::from_str("").is_err());
    assert!(EcosystemPrimalType::from_str("unknown-primal").is_err());
}

#[test]
fn test_all_primals_unique_names() {
    use std::collections::HashSet;

    let primals = [
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::BiomeOS,
    ];

    // Check all service names are unique
    let service_names: HashSet<_> = primals
        .iter()
        .map(squirrel::ecosystem::EcosystemPrimalType::service_name)
        .collect();
    assert_eq!(service_names.len(), primals.len());

    // Check all capability-derived endpoint prefixes are unique
    let env_prefixes: HashSet<_> = primals
        .iter()
        .map(squirrel::ecosystem::EcosystemPrimalType::endpoint_env_prefix)
        .collect();
    assert_eq!(env_prefixes.len(), primals.len());
}

#[test]
fn test_endpoint_env_prefix_matches_capability_transform() {
    // Prefix is uppercase with hyphens from capability replaced by underscores
    for primal in &[
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::BiomeOS,
    ] {
        let cap = primal.capability();
        let expected = cap.replace('-', "_").to_uppercase();
        assert_eq!(primal.endpoint_env_prefix(), expected);
    }
}

#[test]
fn test_primal_type_hash_consistency() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(EcosystemPrimalType::Squirrel, "squirrel_data");
    map.insert(EcosystemPrimalType::Songbird, "songbird_data");

    // Test that we can retrieve values
    assert_eq!(
        map.get(&EcosystemPrimalType::Squirrel),
        Some(&"squirrel_data")
    );
    assert_eq!(
        map.get(&EcosystemPrimalType::Songbird),
        Some(&"songbird_data")
    );
}

#[test]
fn test_primal_type_clone_and_copy() {
    let primal = EcosystemPrimalType::Squirrel;
    let cloned = primal;
    let copied = primal;

    assert_eq!(primal, cloned);
    assert_eq!(primal, copied);
}
