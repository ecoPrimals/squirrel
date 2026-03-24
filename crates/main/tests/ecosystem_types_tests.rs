// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]
//! Comprehensive tests for Ecosystem Primal Types
//!
//! Tests ecosystem primal types and their properties.

#[expect(
    deprecated,
    reason = "Tests deprecated path for backward compatibility"
)]
// EcosystemPrimalType is deprecated but needed for backward compatibility in tests
use squirrel::ecosystem::EcosystemPrimalType;

#[test]
fn test_primal_type_toadstool() {
    let primal = EcosystemPrimalType::ToadStool;

    assert!(matches!(primal, EcosystemPrimalType::ToadStool));
}

#[test]
fn test_primal_type_songbird() {
    let primal = EcosystemPrimalType::Songbird;

    assert!(matches!(primal, EcosystemPrimalType::Songbird));
}

#[test]
fn test_primal_type_beardog() {
    let primal = EcosystemPrimalType::BearDog;

    assert!(matches!(primal, EcosystemPrimalType::BearDog));
}

#[test]
fn test_primal_type_nestgate() {
    let primal = EcosystemPrimalType::NestGate;

    assert!(matches!(primal, EcosystemPrimalType::NestGate));
}

#[test]
fn test_primal_type_squirrel() {
    let primal = EcosystemPrimalType::Squirrel;

    assert!(matches!(primal, EcosystemPrimalType::Squirrel));
}

#[test]
fn test_primal_type_biomeos() {
    let primal = EcosystemPrimalType::BiomeOS;

    assert!(matches!(primal, EcosystemPrimalType::BiomeOS));
}

#[test]
fn test_primal_type_clone() {
    let primal1 = EcosystemPrimalType::ToadStool;
    let primal2 = primal1;

    assert_eq!(primal1, primal2);
}

#[test]
fn test_primal_type_copy() {
    let primal1 = EcosystemPrimalType::Songbird;
    let primal2 = primal1; // Copy, not move

    assert_eq!(primal1, primal2);
}

#[test]
fn test_primal_type_debug() {
    let primal = EcosystemPrimalType::BearDog;
    let debug_str = format!("{primal:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("BearDog"));
}

#[test]
fn test_primal_type_serialization() {
    let primal = EcosystemPrimalType::NestGate;
    let serialized = serde_json::to_string(&primal);

    assert!(serialized.is_ok());
}

#[test]
fn test_primal_type_deserialization() {
    let json_str = r#""ToadStool""#;
    let deserialized: Result<EcosystemPrimalType, _> = serde_json::from_str(json_str);

    assert!(deserialized.is_ok());
}

#[test]
fn test_primal_type_roundtrip() {
    let primal = EcosystemPrimalType::Squirrel;
    let serialized = serde_json::to_string(&primal).expect("should succeed");
    let deserialized: EcosystemPrimalType =
        serde_json::from_str(&serialized).expect("should succeed");

    assert_eq!(primal, deserialized);
}

#[test]
fn test_primal_type_as_str_toadstool() {
    let primal = EcosystemPrimalType::ToadStool;

    assert_eq!(primal.as_str(), "toadstool");
}

#[test]
fn test_primal_type_as_str_songbird() {
    let primal = EcosystemPrimalType::Songbird;

    assert_eq!(primal.as_str(), "songbird");
}

#[test]
fn test_primal_type_as_str_beardog() {
    let primal = EcosystemPrimalType::BearDog;

    assert_eq!(primal.as_str(), "beardog");
}

#[test]
fn test_primal_type_as_str_nestgate() {
    let primal = EcosystemPrimalType::NestGate;

    assert_eq!(primal.as_str(), "nestgate");
}

#[test]
fn test_primal_type_as_str_squirrel() {
    let primal = EcosystemPrimalType::Squirrel;

    assert_eq!(primal.as_str(), "squirrel");
}

#[test]
fn test_primal_type_as_str_biomeos() {
    let primal = EcosystemPrimalType::BiomeOS;

    assert_eq!(primal.as_str(), "biomeos");
}

#[test]
fn test_primal_type_env_name_toadstool() {
    let primal = EcosystemPrimalType::ToadStool;

    assert_eq!(primal.env_name(), "TOADSTOOL");
}

#[test]
fn test_primal_type_env_name_songbird() {
    let primal = EcosystemPrimalType::Songbird;

    assert_eq!(primal.env_name(), "SONGBIRD");
}

#[test]
fn test_primal_type_env_name_beardog() {
    let primal = EcosystemPrimalType::BearDog;

    assert_eq!(primal.env_name(), "BEARDOG");
}

#[test]
fn test_primal_type_env_name_nestgate() {
    let primal = EcosystemPrimalType::NestGate;

    assert_eq!(primal.env_name(), "NESTGATE");
}

#[test]
fn test_primal_type_env_name_squirrel() {
    let primal = EcosystemPrimalType::Squirrel;

    assert_eq!(primal.env_name(), "SQUIRREL");
}

#[test]
fn test_primal_type_env_name_biomeos() {
    let primal = EcosystemPrimalType::BiomeOS;

    assert_eq!(primal.env_name(), "BIOMEOS");
}

#[test]
fn test_primal_type_all_variants() {
    let primals = [
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::BiomeOS,
    ];

    assert_eq!(primals.len(), 6);
}

#[test]
fn test_primal_type_equality() {
    let primal1 = EcosystemPrimalType::ToadStool;
    let primal2 = EcosystemPrimalType::ToadStool;
    let primal3 = EcosystemPrimalType::Songbird;

    assert_eq!(primal1, primal2);
    assert_ne!(primal1, primal3);
}

#[test]
fn test_primal_type_all_distinct() {
    let primals = [
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::BiomeOS,
    ];

    // All should be distinct
    for i in 0..primals.len() {
        for j in (i + 1)..primals.len() {
            assert_ne!(primals[i], primals[j]);
        }
    }
}

#[test]
fn test_primal_type_as_str_lowercase() {
    let primals = vec![
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::BiomeOS,
    ];

    for primal in primals {
        let as_str = primal.as_str();
        assert_eq!(as_str, as_str.to_lowercase());
    }
}

#[test]
fn test_primal_type_env_name_uppercase() {
    let primals = vec![
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::BiomeOS,
    ];

    for primal in primals {
        let env_name = primal.env_name();
        assert_eq!(env_name, env_name.to_uppercase());
    }
}

#[test]
fn test_primal_type_hash_consistency() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(EcosystemPrimalType::ToadStool);
    set.insert(EcosystemPrimalType::Songbird);
    set.insert(EcosystemPrimalType::BearDog);

    assert_eq!(set.len(), 3);
    assert!(set.contains(&EcosystemPrimalType::ToadStool));
}

#[test]
fn test_primal_type_partial_eq() {
    let primal = EcosystemPrimalType::Squirrel;

    assert_eq!(primal, EcosystemPrimalType::Squirrel);
    assert_ne!(primal, EcosystemPrimalType::BiomeOS);
}

#[test]
fn test_primal_type_match_pattern() {
    let primal = EcosystemPrimalType::ToadStool;

    let name = match primal {
        EcosystemPrimalType::ToadStool => "Storage",
        EcosystemPrimalType::Songbird => "Mesh",
        EcosystemPrimalType::BearDog => "Security",
        EcosystemPrimalType::NestGate => "Compute",
        EcosystemPrimalType::Squirrel => "AI",
        EcosystemPrimalType::BiomeOS => "Orchestrator",
    };

    assert_eq!(name, "Storage");
}

#[test]
fn test_primal_type_all_as_str() {
    let expected = vec![
        ("toadstool", EcosystemPrimalType::ToadStool),
        ("songbird", EcosystemPrimalType::Songbird),
        ("beardog", EcosystemPrimalType::BearDog),
        ("nestgate", EcosystemPrimalType::NestGate),
        ("squirrel", EcosystemPrimalType::Squirrel),
        ("biomeos", EcosystemPrimalType::BiomeOS),
    ];

    for (expected_str, primal) in expected {
        assert_eq!(primal.as_str(), expected_str);
    }
}

#[test]
fn test_primal_type_all_env_names() {
    let expected = vec![
        ("TOADSTOOL", EcosystemPrimalType::ToadStool),
        ("SONGBIRD", EcosystemPrimalType::Songbird),
        ("BEARDOG", EcosystemPrimalType::BearDog),
        ("NESTGATE", EcosystemPrimalType::NestGate),
        ("SQUIRREL", EcosystemPrimalType::Squirrel),
        ("BIOMEOS", EcosystemPrimalType::BiomeOS),
    ];

    for (expected_env, primal) in expected {
        assert_eq!(primal.env_name(), expected_env);
    }
}
