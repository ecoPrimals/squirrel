// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]
//! Comprehensive tests for Ecosystem Primal Types
//!
//! Tests ecosystem primal types and their properties.

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
fn test_primal_type_endpoint_env_prefix_toadstool() {
    let primal = EcosystemPrimalType::ToadStool;

    assert_eq!(primal.endpoint_env_prefix(), "COMPUTE");
}

#[test]
fn test_primal_type_endpoint_env_prefix_songbird() {
    let primal = EcosystemPrimalType::Songbird;

    assert_eq!(primal.endpoint_env_prefix(), "SERVICE_MESH");
}

#[test]
fn test_primal_type_endpoint_env_prefix_beardog() {
    let primal = EcosystemPrimalType::BearDog;

    assert_eq!(primal.endpoint_env_prefix(), "SECURITY");
}

#[test]
fn test_primal_type_endpoint_env_prefix_nestgate() {
    let primal = EcosystemPrimalType::NestGate;

    assert_eq!(primal.endpoint_env_prefix(), "STORAGE");
}

#[test]
fn test_primal_type_endpoint_env_prefix_squirrel() {
    let primal = EcosystemPrimalType::Squirrel;

    assert_eq!(primal.endpoint_env_prefix(), "SQUIRREL");
}

#[test]
fn test_primal_type_endpoint_env_prefix_biomeos() {
    let primal = EcosystemPrimalType::BiomeOS;

    assert_eq!(primal.endpoint_env_prefix(), "ECOSYSTEM");
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
fn test_primal_type_endpoint_env_prefix_uppercase() {
    let primals = vec![
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::BiomeOS,
    ];

    for primal in primals {
        let prefix = primal.endpoint_env_prefix();
        assert_eq!(prefix, prefix.to_uppercase());
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
fn test_primal_type_all_endpoint_env_prefixes() {
    let expected = vec![
        ("COMPUTE", EcosystemPrimalType::ToadStool),
        ("SERVICE_MESH", EcosystemPrimalType::Songbird),
        ("SECURITY", EcosystemPrimalType::BearDog),
        ("STORAGE", EcosystemPrimalType::NestGate),
        ("SQUIRREL", EcosystemPrimalType::Squirrel),
        ("ECOSYSTEM", EcosystemPrimalType::BiomeOS),
    ];

    for (expected_prefix, primal) in expected {
        assert_eq!(primal.endpoint_env_prefix(), expected_prefix);
    }
}
