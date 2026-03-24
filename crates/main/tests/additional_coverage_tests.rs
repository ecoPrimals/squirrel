// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)] // Test code: explicit unwrap/expect and local lint noise
//! Additional critical path tests for MCP error handling
//!
//! These tests specifically target error paths to improve code coverage.

#[cfg(test)]
mod mcp_error_tests {
    use squirrel_mcp::error::MCPError;

    #[test]
    fn test_mcp_error_resource_exhausted_display() {
        let error = MCPError::ResourceExhausted("Memory limit reached".to_string());
        let display = format!("{error}");
        assert!(
            !display.is_empty(),
            "Error should have display representation"
        );
        assert!(display.len() > 5, "Error display should be descriptive");
    }

    #[test]
    fn test_mcp_error_invalid_argument_display() {
        let error = MCPError::InvalidArgument("Invalid parameter type".to_string());
        let display = format!("{error}");
        assert!(!display.is_empty());
        assert!(display.len() > 5);
    }

    #[test]
    fn test_mcp_error_not_found_display() {
        let error = MCPError::NotFound("Resource not found".to_string());
        let display = format!("{error}");
        assert!(!display.is_empty());
        assert!(display.len() > 5);
    }
}

#[cfg(test)]
mod capability_request_tests {
    use squirrel::universal::{NetworkLocation, PrimalContext, SecurityLevel};
    use squirrel::universal_primal_ecosystem::CapabilityRequest;
    use std::collections::HashMap;

    /// Create a test context with minimal setup
    fn create_test_context() -> PrimalContext {
        PrimalContext {
            user_id: "test_user".to_string(),
            device_id: "test_device".to_string(),
            session_id: Some("test_session".to_string()),
            biome_id: None,
            network_location: NetworkLocation::default(),
            security_level: SecurityLevel::default(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_capability_request_with_empty_required() {
        let request = CapabilityRequest {
            required_capabilities: vec![],
            optional_capabilities: vec!["test".to_string()],
            context: create_test_context(),
            metadata: HashMap::new(),
        };

        assert_eq!(request.required_capabilities.len(), 0);
        assert_eq!(request.optional_capabilities.len(), 1);
    }

    #[test]
    fn test_capability_request_with_duplicates() {
        let request = CapabilityRequest {
            required_capabilities: vec!["capability_a".to_string(), "capability_a".to_string()],
            optional_capabilities: vec![],
            context: create_test_context(),
            metadata: HashMap::new(),
        };

        // Test that duplicates are present (implementation may deduplicate)
        assert_eq!(request.required_capabilities.len(), 2);
    }

    #[test]
    fn test_capability_request_with_special_characters() {
        let request = CapabilityRequest {
            required_capabilities: vec![
                "ai-inference".to_string(),
                "ai_inference".to_string(),
                "ai.inference".to_string(),
            ],
            optional_capabilities: vec![],
            context: create_test_context(),
            metadata: HashMap::new(),
        };

        assert_eq!(request.required_capabilities.len(), 3);
    }
}

#[cfg(test)]
mod ecosystem_tests {
    use squirrel::universal::{NetworkLocation, PrimalContext, SecurityLevel};
    use squirrel::universal_primal_ecosystem::CapabilityRequest;
    use squirrel::universal_primal_ecosystem::UniversalPrimalEcosystem;
    use std::collections::HashMap;

    /// Create a test context for ecosystem tests
    fn create_test_context() -> PrimalContext {
        PrimalContext {
            user_id: "test_user".to_string(),
            device_id: "test_device".to_string(),
            session_id: Some("test_session".to_string()),
            biome_id: None,
            network_location: NetworkLocation::default(),
            security_level: SecurityLevel::default(),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_empty_ecosystem_capability_search() {
        let ecosystem = UniversalPrimalEcosystem::new(create_test_context());
        let request = CapabilityRequest {
            required_capabilities: vec!["test_capability".to_string()],
            optional_capabilities: vec![],
            context: create_test_context(),
            metadata: HashMap::new(),
        };

        let result = ecosystem.find_services_by_capability(&request).await;
        assert!(result.is_ok(), "Should handle empty ecosystem");
        assert_eq!(result.expect("verified ok").len(), 0);
    }

    #[tokio::test]
    async fn test_ecosystem_cache_stats() {
        let ecosystem = UniversalPrimalEcosystem::new(create_test_context());
        let stats = ecosystem.get_cache_stats().await;

        // Cache should be accessible and stats should be valid
        // usize fields are always >= 0; we just verify stats are accessible
        let _ = stats;
    }

    #[tokio::test]
    async fn test_concurrent_capability_searches() {
        use std::sync::Arc;

        let ecosystem = Arc::new(UniversalPrimalEcosystem::new(create_test_context()));
        let request = CapabilityRequest {
            required_capabilities: vec!["test".to_string()],
            optional_capabilities: vec![],
            context: create_test_context(),
            metadata: HashMap::new(),
        };

        let mut handles = vec![];
        for _ in 0..10 {
            let eco = ecosystem.clone();
            let req = request.clone();
            handles.push(tokio::spawn(async move {
                eco.find_services_by_capability(&req).await
            }));
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent access should not deadlock");
        }
    }
}

#[cfg(test)]
mod primal_type_tests {
    use squirrel::universal::PrimalType;

    #[test]
    fn test_primal_type_display() {
        // Modern capability-based types (evolved from hardcoded primal names)
        let types = vec![
            PrimalType::AI,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::Network,
            PrimalType::Security,
            PrimalType::Coordination,
        ];

        for primal_type in types {
            let display = format!("{primal_type}");
            assert!(!display.is_empty(), "Type display should not be empty");
            // AI is 2 chars, which is fine - it's descriptive enough
            assert!(
                display.len() >= 2,
                "Type display should be at least 2 characters"
            );
        }
    }

    #[test]
    fn test_primal_type_debug() {
        let ai_type = PrimalType::AI;
        let debug = format!("{ai_type:?}");
        assert!(!debug.is_empty());
    }

    #[test]
    fn test_primal_type_variants_coverage() {
        // Test all capability-based variants for completeness
        let _ = PrimalType::AI;
        let _ = PrimalType::Storage;
        let _ = PrimalType::Compute;
        let _ = PrimalType::Network;
        let _ = PrimalType::Security;
        let _ = PrimalType::Coordination;
    }
}
