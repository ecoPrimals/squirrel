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
//! Comprehensive tests for capability-based discovery
//!
//! Tests the new environment variable and service discovery features

// EcosystemPrimalType is deprecated but needed for backward compatibility in tests
use squirrel::ecosystem::EcosystemPrimalType;
use std::str::FromStr;

#[test]
fn test_env_name_all_primals() {
    assert_eq!(EcosystemPrimalType::Squirrel.env_name(), "SQUIRREL");
    assert_eq!(EcosystemPrimalType::Songbird.env_name(), "SONGBIRD");
    assert_eq!(EcosystemPrimalType::ToadStool.env_name(), "TOADSTOOL");
    assert_eq!(EcosystemPrimalType::BearDog.env_name(), "BEARDOG");
    assert_eq!(EcosystemPrimalType::NestGate.env_name(), "NESTGATE");
    assert_eq!(EcosystemPrimalType::BiomeOS.env_name(), "BIOMEOS");
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
fn test_env_name_format() {
    // Verify all env names are uppercase
    for primal in &[
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::BiomeOS,
    ] {
        let env_name = primal.env_name();
        assert_eq!(env_name, env_name.to_uppercase());
        assert!(!env_name.is_empty());
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
        .map(squirrel::EcosystemPrimalType::service_name)
        .collect();
    assert_eq!(service_names.len(), primals.len());

    // Check all env names are unique
    let env_names: HashSet<_> = primals
        .iter()
        .map(squirrel::EcosystemPrimalType::env_name)
        .collect();
    assert_eq!(env_names.len(), primals.len());
}

#[test]
fn test_env_name_uppercase_consistency() {
    // Verify env_name is uppercase version of service_name
    for primal in &[
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::NestGate,
        EcosystemPrimalType::BiomeOS,
    ] {
        assert_eq!(primal.env_name().to_lowercase(), primal.service_name());
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
