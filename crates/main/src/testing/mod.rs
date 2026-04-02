// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Testing utilities for Squirrel main crate
//!
//! This module provides comprehensive testing utilities including:
//! - Test fixtures and builders
//! - Helper functions for common test scenarios
//! - Modern concurrent test utilities (event-driven, no sleeps)

pub mod concurrent_test_utils;
//! - Integration test utilities

use std::sync::Arc;
use std::collections::HashMap;

/// Test helper for creating mock primal contexts
pub mod context_helpers {
    use super::*;
    use crate::universal::PrimalContext;
    
    /// Create a test context with default values
    pub fn create_test_context() -> PrimalContext {
        PrimalContext {
            instance_id: "test-instance".to_string(),
            biome_id: "test-biome".to_string(),
            user_id: Some("test-user".to_string()),
            session_id: Some("test-session".to_string()),
            device_id: None,
            security_context: None,
        }
    }
    
    /// Create a test context with custom instance ID
    pub fn create_test_context_with_instance(instance_id: &str) -> PrimalContext {
        PrimalContext {
            instance_id: instance_id.to_string(),
            biome_id: "test-biome".to_string(),
            user_id: Some("test-user".to_string()),
            session_id: Some("test-session".to_string()),
            device_id: None,
            security_context: None,
        }
    }
    
    /// Create a standalone test context (no biome)
    pub fn create_standalone_context() -> PrimalContext {
        PrimalContext {
            instance_id: "standalone-instance".to_string(),
            biome_id: "standalone".to_string(),
            user_id: None,
            session_id: None,
            device_id: None,
            security_context: None,
        }
    }
}

/// Test helpers for service endpoints
pub mod endpoint_helpers {
    use crate::universal::deployment;
    
    /// Get test API gateway endpoint
    pub fn test_api_gateway() -> String {
        format!("http://{}:{}", deployment::hosts::localhost(), deployment::ports::api_gateway())
    }
    
    /// Get test websocket endpoint
    pub fn test_websocket() -> String {
        format!("ws://{}:{}", deployment::hosts::localhost(), deployment::ports::websocket())
    }
    
    /// Get test service mesh endpoint
    pub fn test_service_mesh() -> String {
        format!("http://{}:{}", deployment::hosts::localhost(), deployment::ports::service_mesh())
    }
}

/// Test helpers for creating test responses
/// 
/// ⚠️ Note: These are TEST HELPERS ONLY. They create simple HashMap responses
/// for use in unit tests. Real production code uses proper types and error handling.
pub mod response_helpers {
    use std::collections::HashMap;
    
    /// Create a successful test response (for unit tests only)
    /// 
    /// # Testing Only
    /// This is a simplified helper for unit tests. Production code should use
    /// proper response types with full error handling.
    pub fn mock_success_response(data: &str) -> HashMap<String, String> {
        let mut response = HashMap::new();
        response.insert("status".to_string(), "success".to_string());
        response.insert("data".to_string(), data.to_string());
        response
    }
    
    /// Create an error test response (for unit tests only)
    /// 
    /// # Testing Only
    /// This is a simplified helper for unit tests. Production code should use
    /// proper error types with full context.
    pub fn mock_error_response(error: &str) -> HashMap<String, String> {
        let mut response = HashMap::new();
        response.insert("status".to_string(), "error".to_string());
        response.insert("error".to_string(), error.to_string());
        response
    }
}

/// Test assertions for common patterns (test-only; panics on wrong variant)
pub mod assertions {
    /// Assert that a result is `Ok` and return the value.
    ///
    /// # Panics
    ///
    /// Panics if the result is `Err`.
    pub fn assert_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {e:?}"),
        }
    }

    /// Assert that a result is `Err`.
    ///
    /// # Panics
    ///
    /// Panics if the result is `Ok`.
    pub fn assert_err<T: std::fmt::Debug, E>(result: Result<T, E>) {
        match result {
            Ok(val) => panic!("Expected Err, got Ok: {val:?}"),
            Err(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_helpers::*;
    use endpoint_helpers::*;
    use response_helpers::*;
    use assertions::*;
    
    #[test]
    fn test_create_test_context() {
        let ctx = create_test_context();
        assert_eq!(ctx.instance_id, "test-instance");
        assert_eq!(ctx.biome_id, "test-biome");
        assert_eq!(ctx.user_id, Some("test-user".to_string()));
    }
    
    #[test]
    fn test_create_test_context_with_instance() {
        let ctx = create_test_context_with_instance("custom-id");
        assert_eq!(ctx.instance_id, "custom-id");
        assert_eq!(ctx.biome_id, "test-biome");
    }
    
    #[test]
    fn test_create_standalone_context() {
        let ctx = create_standalone_context();
        assert_eq!(ctx.biome_id, "standalone");
        assert_eq!(ctx.user_id, None);
        assert_eq!(ctx.session_id, None);
    }
    
    #[test]
    fn test_endpoint_helpers() {
        assert!(test_api_gateway().starts_with("http://"));
        assert!(test_websocket().starts_with("ws://"));
        assert!(test_service_mesh().contains("8500"));
    }
    
    #[test]
    fn test_mock_success_response() {
        let response = mock_success_response("test-data");
        assert_eq!(response.get("status"), Some(&"success".to_string()));
        assert_eq!(response.get("data"), Some(&"test-data".to_string()));
    }
    
    #[test]
    fn test_mock_error_response() {
        let response = mock_error_response("test-error");
        assert_eq!(response.get("status"), Some(&"error".to_string()));
        assert_eq!(response.get("error"), Some(&"test-error".to_string()));
    }
    
    #[test]
    fn test_assert_ok() {
        let result: Result<i32, String> = Ok(42);
        let value = assert_ok(result);
        assert_eq!(value, 42);
    }
    
    #[test]
    #[should_panic(expected = "Expected Err, got Ok:")]
    fn test_assert_err_with_ok() {
        let result: Result<i32, String> = Ok(42);
        assert_err(result);
    }
    
    #[test]
    fn test_assert_err_with_err() {
        let result: Result<i32, String> = Err("error".to_string());
        assert_err(result);
    }
}

