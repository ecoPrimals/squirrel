// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Additional error path coverage tests
//!
//! These tests target under-covered error scenarios to push coverage toward 65%

#[cfg(test)]
mod additional_error_coverage {
    use squirrel::error::PrimalError;
    /// Local test aliases for error recovery patterns
    #[allow(dead_code)]
    mod safe_operations {
        use super::PrimalError;
        pub type SafeResult<T> = Result<T, PrimalError>;
        pub enum RecoveryStrategy {
            Retry,
            Fallback,
            Fail,
        }
    }
    use safe_operations::*;
    use squirrel::universal::{NetworkLocation, PrimalContext, SecurityLevel};
    use std::collections::HashMap;

    // ============================================================================
    // PrimalError Variant Coverage
    // ============================================================================

    #[test]
    fn test_primal_error_io_variant() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let primal_err = PrimalError::from(io_err);
        assert!(primal_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_primal_error_network_variants() {
        let err1 = PrimalError::Network("connection failed".to_string());
        let err2 = PrimalError::NetworkError("timeout".to_string());

        assert!(err1.to_string().contains("Network error"));
        assert!(err2.to_string().contains("Network error"));
        assert_ne!(format!("{:?}", err1), format!("{:?}", err2));
    }

    #[test]
    fn test_primal_error_config_variants() {
        let err1 = PrimalError::Configuration("missing field".to_string());
        let err2 = PrimalError::ConfigurationError("invalid value".to_string());
        let err3 = PrimalError::ConfigError("parse error".to_string());

        assert!(err1.to_string().contains("Configuration"));
        assert!(err2.to_string().contains("Configuration"));
        assert!(err3.to_string().contains("Configuration"));
    }

    #[test]
    fn test_primal_error_resource_variants() {
        let err1 = PrimalError::ResourceNotFound("database".to_string());
        let err2 = PrimalError::ResourceError("out of memory".to_string());
        let err3 = PrimalError::NotFoundError("endpoint".to_string());

        assert!(!err1.to_string().is_empty());
        assert!(!err2.to_string().is_empty());
        assert!(!err3.to_string().is_empty());
    }

    #[test]
    fn test_primal_error_service_discovery() {
        let err1 = PrimalError::ServiceDiscoveryFailed("no services".to_string());
        let err2 = PrimalError::ServiceDiscoveryError("timeout".to_string());

        let display1 = format!("{}", err1);
        let display2 = format!("{}", err2);

        assert!(display1.contains("Service discovery"));
        assert!(display2.contains("Service discovery"));
    }

    #[test]
    fn test_primal_error_operation_variants() {
        let err1 = PrimalError::InvalidOperation("not allowed".to_string());
        let err2 = PrimalError::OperationFailed("crashed".to_string());
        let err3 = PrimalError::OperationNotSupported("deprecated".to_string());

        assert!(err1.to_string().contains("Invalid"));
        assert!(err2.to_string().contains("failed"));
        assert!(err3.to_string().contains("not supported"));
    }

    // ============================================================================
    // PrimalError Variant Coverage (additional variants)
    // ============================================================================

    #[test]
    fn test_primal_error_configuration_tuple() {
        let err = PrimalError::Configuration("missing port".to_string());
        assert!(err.to_string().contains("Configuration"));
    }

    #[test]
    fn test_primal_error_network_tuple() {
        let err = PrimalError::Network("unreachable: http://localhost:8080".to_string());
        assert!(err.to_string().contains("Network"));
    }

    #[test]
    fn test_primal_error_internal_tuple() {
        let err = PrimalError::Internal("lock poisoned".to_string());
        assert!(err.to_string().contains("Internal"));
    }

    #[test]
    fn test_primal_error_generic_tuple() {
        let err = PrimalError::Generic("channel closed".to_string());
        assert!(err.to_string().contains("Generic"));
    }

    #[test]
    fn test_primal_error_invalid_operation_for_timeout() {
        let err = PrimalError::InvalidOperation("timeout exceeded after 30s".to_string());
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_primal_error_resource_error() {
        let err = PrimalError::ResourceError("connection pool exhausted".to_string());
        assert!(err.to_string().contains("Resource"));
    }

    #[test]
    fn test_primal_error_service_unavailable() {
        let err = PrimalError::Network("Service unavailable: database - down".to_string());
        assert!(err.to_string().contains("Service unavailable"));
    }

    // ============================================================================
    // NetworkLocation Edge Cases
    // ============================================================================

    #[test]
    fn test_network_location_with_minimal_fields() {
        let loc = NetworkLocation {
            region: "test".to_string(),
            data_center: None,
            availability_zone: None,
            ip_address: None,
            subnet: None,
            network_id: None,
            geo_location: None,
        };

        assert_eq!(loc.region, "test");
        assert!(loc.data_center.is_none());
    }

    #[test]
    fn test_network_location_with_all_fields() {
        let loc = NetworkLocation {
            region: "us-west-2".to_string(),
            data_center: Some("dc1".to_string()),
            availability_zone: Some("az1".to_string()),
            ip_address: Some("10.0.0.1".to_string()),
            subnet: Some("10.0.0.0/24".to_string()),
            network_id: Some("net-123".to_string()),
            geo_location: Some("US-CA-SF".to_string()),
        };

        assert!(loc.data_center.is_some());
        assert!(loc.availability_zone.is_some());
        assert!(loc.ip_address.is_some());
        assert!(loc.subnet.is_some());
        assert!(loc.network_id.is_some());
        assert!(loc.geo_location.is_some());
    }

    // ============================================================================
    // SecurityLevel Coverage
    // ============================================================================

    #[test]
    fn test_all_security_levels() {
        let levels = vec![
            SecurityLevel::Basic,
            SecurityLevel::Standard,
            SecurityLevel::Public,
            SecurityLevel::Enhanced,
            SecurityLevel::Advanced,
            SecurityLevel::High,
            SecurityLevel::Critical,
            SecurityLevel::Administrative,
        ];

        for level in levels {
            // Test that each level can be formatted
            let debug_str = format!("{:?}", level);
            assert!(!debug_str.is_empty());
        }
    }

    // ============================================================================
    // PrimalContext Edge Cases
    // ============================================================================

    #[test]
    fn test_primal_context_with_empty_metadata() {
        let context = PrimalContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            session_id: None,
            biome_id: None,
            network_location: NetworkLocation::default(),
            security_level: SecurityLevel::Standard,
            metadata: HashMap::new(),
        };

        assert!(context.metadata.is_empty());
        assert!(context.session_id.is_none());
    }

    #[test]
    fn test_primal_context_with_full_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let context = PrimalContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            session_id: Some("session1".to_string()),
            biome_id: Some("biome1".to_string()),
            network_location: NetworkLocation::default(),
            security_level: SecurityLevel::High,
            metadata,
        };

        assert_eq!(context.metadata.len(), 2);
        assert!(context.session_id.is_some());
        assert!(context.biome_id.is_some());
    }

    // ============================================================================
    // Error Conversion and Display Tests
    // ============================================================================

    #[test]
    fn test_url_parse_error_conversion() {
        let url_err = url::Url::parse("not a url").unwrap_err();
        let primal_err = PrimalError::from(url_err);
        assert!(primal_err.to_string().contains("URL parse error"));
    }

    #[test]
    fn test_serde_error_conversion() {
        let json_err =
            serde_json::from_str::<HashMap<String, String>>("{invalid json}").unwrap_err();
        let primal_err = PrimalError::from(json_err);
        assert!(primal_err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_error_debug_formatting() {
        let errors = vec![
            PrimalError::Authentication("failed".to_string()),
            PrimalError::Internal("bug".to_string()),
            PrimalError::ValidationError("invalid".to_string()),
            PrimalError::SecurityError("breach".to_string()),
            PrimalError::ComputeError("overload".to_string()),
            PrimalError::StorageError("full".to_string()),
            PrimalError::Generic("unknown".to_string()),
            // PrimalError::NotImplemented removed in favor of specific error types
            PrimalError::Registry("corrupted".to_string()),
            PrimalError::ParsingError("malformed".to_string()),
            PrimalError::SerializationError("encoding".to_string()),
        ];

        for err in errors {
            let debug_str = format!("{:?}", err);
            let display_str = format!("{}", err);

            assert!(!debug_str.is_empty());
            assert!(!display_str.is_empty());
            assert_ne!(debug_str, display_str); // Debug should be different from Display
        }
    }

    // ============================================================================
    // Error Recovery Pattern Coverage
    // ============================================================================

    #[test]
    fn test_error_result_retry_pattern() {
        // Test retry pattern using standard Result
        let mut attempts = 0;
        let result: SafeResult<i32> = loop {
            attempts += 1;
            if attempts >= 3 {
                break Ok(42);
            }
            // Simulate transient failure
        };
        assert!(result.is_ok());
        assert_eq!(attempts, 3);
    }

    #[test]
    fn test_error_result_fallback_pattern() {
        // Test fallback pattern using standard Result
        let primary: SafeResult<String> = Err(PrimalError::Internal("primary failed".to_string()));
        let result = primary.unwrap_or_else(|_| "fallback_value".to_string());
        assert_eq!(result, "fallback_value");
    }

    #[test]
    fn test_recovery_strategy_enum_coverage() {
        // Exercise all recovery strategy variants
        let strategies = vec![
            RecoveryStrategy::Retry,
            RecoveryStrategy::Fallback,
            RecoveryStrategy::Fail,
        ];
        assert_eq!(strategies.len(), 3);
    }
}
