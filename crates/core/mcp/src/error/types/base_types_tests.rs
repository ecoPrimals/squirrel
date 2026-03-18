// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for base error types

use super::*;

#[test]
fn test_security_level_default() {
    let level = SecurityLevel::default();
    assert!(matches!(level, SecurityLevel::Medium));
}

#[test]
fn test_security_level_variants() {
    let low = SecurityLevel::Low;
    let medium = SecurityLevel::Medium;
    let high = SecurityLevel::High;
    let critical = SecurityLevel::Critical;

    // Verify all variants can be created
    assert!(matches!(low, SecurityLevel::Low));
    assert!(matches!(medium, SecurityLevel::Medium));
    assert!(matches!(high, SecurityLevel::High));
    assert!(matches!(critical, SecurityLevel::Critical));
}

#[test]
fn test_security_level_clone() {
    let level = SecurityLevel::High;
    let cloned = level;
    assert!(matches!(cloned, SecurityLevel::High));
}

#[test]
fn test_security_level_serialization() {
    let level = SecurityLevel::High;
    let json = serde_json::to_string(&level).unwrap();
    assert!(json.contains("High"));

    let deserialized: SecurityLevel = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized, SecurityLevel::High));
}

#[test]
fn test_wire_format_error_creation() {
    let error = WireFormatError {
        message: "test error".to_string(),
    };
    assert_eq!(error.message, "test error");
}

#[test]
fn test_wire_format_error_display() {
    let error = WireFormatError {
        message: "invalid format".to_string(),
    };
    let display = format!("{error}");
    assert!(display.contains("Wire format error"));
    assert!(display.contains("invalid format"));
}

#[test]
fn test_wire_format_error_is_error() {
    let error = WireFormatError {
        message: "test".to_string(),
    };
    // Verify it implements Error trait
    let _: &dyn std::error::Error = &error;
}

#[test]
fn test_wire_format_error_serialization() {
    let error = WireFormatError {
        message: "serialization test".to_string(),
    };
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("serialization test"));

    let deserialized: WireFormatError = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.message, "serialization test");
}

#[test]
fn test_wire_format_error_debug() {
    let error = WireFormatError {
        message: "debug test".to_string(),
    };
    let debug_str = format!("{error:?}");
    assert!(debug_str.contains("WireFormatError"));
    assert!(debug_str.contains("debug test"));
}
