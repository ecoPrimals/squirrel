// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive Error Path Coverage Tests
//!
//! Deep testing philosophy: Every error variant should be tested

#[cfg(test)]
mod error_path_tests {
    use super::super::PrimalError;

    // ===== Error Creation Tests =====

    #[test]
    fn test_network_error_creation() {
        let error = PrimalError::Network("Connection refused".to_string());
        assert!(error.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_authentication_error_creation() {
        let error = PrimalError::Authentication("Invalid credentials".to_string());
        assert!(error.to_string().contains("Invalid credentials"));
    }

    #[test]
    fn test_configuration_error_creation() {
        let error = PrimalError::Configuration("Missing port".to_string());
        assert!(error.to_string().contains("Missing port"));
    }

    #[test]
    fn test_not_implemented_error() {
        let error = PrimalError::OperationNotSupported("Feature X".to_string());
        assert!(error.to_string().contains("Feature X"));
    }

    #[test]
    fn test_service_discovery_error() {
        let error = PrimalError::ServiceDiscoveryFailed("No services found".to_string());
        assert!(error.to_string().contains("No services found"));
    }

    #[test]
    fn test_operation_failed_error() {
        let error = PrimalError::OperationFailed("Timeout".to_string());
        assert!(error.to_string().contains("Timeout"));
    }

    #[test]
    fn test_resource_not_found_error() {
        let error = PrimalError::ResourceNotFound("Plugin ID: 123".to_string());
        assert!(error.to_string().contains("Plugin ID: 123"));
    }

    #[test]
    fn test_validation_error() {
        let error = PrimalError::ValidationError("Invalid email format".to_string());
        assert!(error.to_string().contains("Invalid email"));
    }

    #[test]
    fn test_security_error() {
        let error = PrimalError::SecurityError("Unauthorized access".to_string());
        assert!(error.to_string().contains("Unauthorized"));
    }

    // ===== Error Propagation Tests =====

    #[test]
    fn test_error_propagation_in_result() {
        fn operation_that_fails() -> Result<(), PrimalError> {
            Err(PrimalError::Network("Failed".to_string()))
        }

        let result = operation_that_fails();
        assert!(result.is_err());
        match result {
            Err(PrimalError::Network(msg)) => assert_eq!(msg, "Failed"),
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_chain_propagation() {
        fn inner_operation() -> Result<(), PrimalError> {
            Err(PrimalError::Internal("Database error".to_string()))
        }

        fn outer_operation() -> Result<(), PrimalError> {
            inner_operation()?;
            Ok(())
        }

        let result = outer_operation();
        assert!(result.is_err());
    }

    // ===== Error Conversion Tests =====

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let primal_error: PrimalError = io_error.into();

        match primal_error {
            PrimalError::Io(_) => (),
            _ => panic!("Wrong error conversion"),
        }
    }

    #[test]
    fn test_serialization_error_conversion() {
        let json_str = "{invalid json";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json_str);

        if let Err(e) = result {
            let primal_error: PrimalError = e.into();
            match primal_error {
                PrimalError::Serialization(_) => (),
                _ => panic!("Wrong error conversion"),
            }
        }
    }

    // ===== Error Message Format Tests =====

    #[test]
    fn test_error_messages_are_descriptive() {
        let errors = vec![
            PrimalError::Network("test".to_string()),
            PrimalError::Authentication("test".to_string()),
            PrimalError::Configuration("test".to_string()),
            PrimalError::ValidationError("test".to_string()),
            PrimalError::SecurityError("test".to_string()),
        ];

        for error in errors {
            let message = error.to_string();
            assert!(!message.is_empty());
            assert!(message.len() > 5); // Meaningful message
        }
    }

    // ===== Error Debug Format Tests =====

    #[test]
    fn test_error_debug_format() {
        let error = PrimalError::Network("Connection failed".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Network"));
        assert!(debug_str.contains("Connection failed"));
    }

    // ===== Error Equality and Matching =====

    #[test]
    fn test_error_pattern_matching() {
        let error = PrimalError::OperationNotSupported("Feature Y".to_string());

        match error {
            PrimalError::OperationNotSupported(msg) => {
                assert_eq!(msg, "Feature Y");
            }
            _ => panic!("Pattern match failed"),
        }
    }

    // ===== Error with Empty Messages =====

    #[test]
    fn test_error_with_empty_message() {
        let error = PrimalError::Generic(String::new());
        assert!(!error.to_string().is_empty()); // Should have error type prefix
    }

    // ===== All Error Variants Coverage =====

    #[test]
    fn test_all_error_variants_can_be_created() {
        // Ensure all error variants can be instantiated
        let errors: Vec<PrimalError> = vec![
            PrimalError::Network("test".to_string()),
            PrimalError::NetworkError("test".to_string()),
            PrimalError::Authentication("test".to_string()),
            PrimalError::Configuration("test".to_string()),
            PrimalError::OperationNotSupported("test".to_string()),
            PrimalError::ConfigurationError("test".to_string()),
            PrimalError::ConfigError("test".to_string()),
            PrimalError::ParsingError("test".to_string()),
            PrimalError::InvalidOperation("test".to_string()),
            PrimalError::ServiceDiscoveryFailed("test".to_string()),
            PrimalError::ServiceDiscoveryError("test".to_string()),
            PrimalError::Registry("test".to_string()),
            PrimalError::Internal("test".to_string()),
            PrimalError::OperationFailed("test".to_string()),
            PrimalError::OperationNotSupported("test".to_string()),
            PrimalError::ResourceNotFound("test".to_string()),
            PrimalError::NotFoundError("test".to_string()),
            PrimalError::ResourceError("test".to_string()),
            PrimalError::General("test".to_string()),
            PrimalError::ValidationError("test".to_string()),
            PrimalError::SerializationError("test".to_string()),
            PrimalError::SecurityError("test".to_string()),
            PrimalError::ComputeError("test".to_string()),
            PrimalError::StorageError("test".to_string()),
            PrimalError::Generic("test".to_string()),
        ];

        // All should convert to strings without panic
        for error in errors {
            let _ = error.to_string();
        }
    }

    // ===== Error in Async Context =====

    #[tokio::test]
    async fn test_error_in_async_context() {
        async fn async_operation() -> Result<(), PrimalError> {
            Err(PrimalError::Network("Async failure".to_string()))
        }

        let result = async_operation().await;
        assert!(result.is_err());
    }

    // ===== Error Across Thread Boundaries =====

    #[test]
    fn test_error_send_sync() {
        // Verify PrimalError implements Send + Sync for concurrent use
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<PrimalError>();
        assert_sync::<PrimalError>();
    }

    // ===== Error in Option Context =====

    #[test]
    fn test_error_with_option_context() {
        fn operation() -> Result<Option<String>, PrimalError> {
            Ok(None)
        }

        let result = operation();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_error_from_option_conversion() {
        let maybe_value: Option<String> = None;
        let result: Result<String, PrimalError> =
            maybe_value.ok_or_else(|| PrimalError::NotFoundError("Value missing".to_string()));

        assert!(result.is_err());
    }
}
