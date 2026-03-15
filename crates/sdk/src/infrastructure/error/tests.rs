// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the error handling system in the Squirrel Plugin SDK

#[cfg(test)]
mod tests {
    use super::super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_enhanced_error_system() {
        let error = core::PluginError::NetworkError {
            operation: "fetch_data".to_string(),
            message: "Connection timeout".to_string(),
        };

        let context = context::ErrorContext::new("network_operation")
            .with_module("http_client")
            .with_function("fetch_data");

        let enhanced = error.with_context(context);

        assert_eq!(enhanced.error.error_type(), "NetworkError");
        assert_eq!(enhanced.category, severity::ErrorCategory::Network);
        assert_eq!(enhanced.severity, severity::ErrorSeverity::High);
        assert!(enhanced.recoverable);
        assert!(!enhanced.recovery_suggestions.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_error_categorization() {
        use severity::PluginErrorClassification;

        let user_error = core::PluginError::MissingParameter {
            parameter: "name".to_string(),
        };
        assert_eq!(user_error.category(), severity::ErrorCategory::User);

        let network_error = core::PluginError::NetworkError {
            operation: "request".to_string(),
            message: "timeout".to_string(),
        };
        assert_eq!(network_error.category(), severity::ErrorCategory::Network);

        let security_error = core::PluginError::SecurityViolation {
            violation: "unauthorized access".to_string(),
        };
        assert_eq!(security_error.category(), severity::ErrorCategory::Security);
    }

    #[wasm_bindgen_test]
    fn test_error_severity() {
        use severity::PluginErrorClassification;

        let low_error = core::PluginError::Deprecated {
            feature: "old_api".to_string(),
            alternative: "new_api".to_string(),
        };
        assert_eq!(low_error.severity(), severity::ErrorSeverity::Low);

        let critical_error = core::PluginError::SecurityViolation {
            violation: "buffer overflow".to_string(),
        };
        assert_eq!(critical_error.severity(), severity::ErrorSeverity::Critical);
    }

    #[wasm_bindgen_test]
    fn test_validation_helpers() {
        let params = serde_json::json!({
            "name": "test",
            "count": 42,
            "enabled": true,
            "items": [1, 2, 3],
            "config": {"key": "value"}
        });

        assert_eq!(
            validation::validate_required_string(&params, "name").unwrap(),
            "test"
        );
        assert_eq!(
            validation::validate_required_number(&params, "count").unwrap(),
            42.0
        );
        assert_eq!(
            validation::validate_boolean(&params, "enabled", false).unwrap(),
            true
        );
        assert_eq!(
            validation::validate_array(&params, "items").unwrap().len(),
            3
        );
        assert_eq!(
            validation::validate_object(&params, "config")
                .unwrap()
                .len(),
            1
        );
    }

    #[wasm_bindgen_test]
    fn test_recovery_suggestions() {
        use severity::PluginErrorClassification;

        let network_error = core::PluginError::NetworkError {
            operation: "fetch".to_string(),
            message: "timeout".to_string(),
        };
        let suggestions = network_error.recovery_suggestions();
        assert!(suggestions.contains(&"Check network connectivity".to_string()));
        assert!(suggestions.contains(&"Retry the operation".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_error_context_creation() {
        let context = context::ErrorContext::new("test_operation")
            .with_module("test_module")
            .with_function("test_function")
            .with_line(42)
            .with_data("key", serde_json::json!("value"));

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.module, Some("test_module".to_string()));
        assert_eq!(context.function, Some("test_function".to_string()));
        assert_eq!(context.line, Some(42));
        assert_eq!(
            context.context_data.get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[wasm_bindgen_test]
    fn test_error_chaining() {
        let root_error = core::PluginError::NetworkError {
            operation: "connect".to_string(),
            message: "connection refused".to_string(),
        };

        let root_context = context::ErrorContext::new("network_connect");
        let root_enhanced = root_error.with_context(root_context);

        let chain_error = core::PluginError::ExecutionError {
            context: "api_call".to_string(),
            message: "failed to execute".to_string(),
        };

        let chain_context = context::ErrorContext::new("api_execution");
        let chained_enhanced = chain_error
            .with_context(chain_context)
            .with_source(root_enhanced);

        assert!(chained_enhanced.source.is_some());
        assert_eq!(
            chained_enhanced.source.as_ref().unwrap().error.error_type(),
            "NetworkError"
        );
    }

    #[wasm_bindgen_test]
    fn test_error_codes() {
        let error = core::PluginError::MissingParameter {
            parameter: "test".to_string(),
        };
        assert_eq!(error.error_code(), 1002);

        let error = core::PluginError::NetworkError {
            operation: "test".to_string(),
            message: "test".to_string(),
        };
        assert_eq!(error.error_code(), 2001);

        let error = core::PluginError::SecurityViolation {
            violation: "test".to_string(),
        };
        assert_eq!(error.error_code(), 6001);
    }

    #[wasm_bindgen_test]
    fn test_error_conversions() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());

        let plugin_error: core::PluginError = json_error.unwrap_err().into();
        assert_eq!(plugin_error.error_type(), "JsonError");
    }

    #[wasm_bindgen_test]
    fn test_validation_error_conversion() {
        let validation_error = validation::ValidationError::RequiredField {
            field: "name".to_string(),
        };

        let plugin_error: core::PluginError = validation_error.into();
        assert_eq!(plugin_error.error_type(), "MissingParameter");
    }

    #[wasm_bindgen_test]
    fn test_error_recoverability() {
        use severity::PluginErrorClassification;

        let recoverable_error = core::PluginError::NetworkError {
            operation: "fetch".to_string(),
            message: "timeout".to_string(),
        };
        assert!(recoverable_error.is_recoverable());

        let non_recoverable_error = core::PluginError::SecurityViolation {
            violation: "unauthorized access".to_string(),
        };
        assert!(!non_recoverable_error.is_recoverable());
    }

    #[wasm_bindgen_test]
    fn test_enhanced_error_display() {
        let error = core::PluginError::NetworkError {
            operation: "fetch".to_string(),
            message: "timeout".to_string(),
        };

        let context = context::ErrorContext::new("test_operation");
        let enhanced = error.with_context(context);

        let display_string = enhanced.to_string();
        assert!(display_string.contains("HIGH"));
        assert!(display_string.contains("NETWORK"));
        assert!(display_string.contains("Network error"));
        assert!(display_string.contains("test_operation"));
    }

    #[wasm_bindgen_test]
    fn test_validation_helpers_extended() {
        let params = serde_json::json!({
            "url": "https://example.com",
            "email": "test@example.com",
            "non_empty": "value",
            "items": [1, 2, 3, 4, 5]
        });

        assert!(validation::validate_url("https://example.com", "url").is_ok());
        assert!(validation::validate_url("invalid-url", "url").is_err());

        assert!(validation::validate_email("test@example.com", "email").is_ok());
        assert!(validation::validate_email("invalid-email", "email").is_err());

        assert!(validation::validate_non_empty_string("value", "non_empty").is_ok());
        assert!(validation::validate_non_empty_string("", "non_empty").is_err());

        let items = vec![
            serde_json::json!(1),
            serde_json::json!(2),
            serde_json::json!(3),
            serde_json::json!(4),
            serde_json::json!(5),
        ];
        assert!(validation::validate_array_length(&items, "items", 3, 10).is_ok());
        assert!(validation::validate_array_length(&items, "items", 10, 20).is_err());
    }

    #[wasm_bindgen_test]
    fn test_macro_helpers() {
        let error = crate::param_error!("test_param", "test reason");
        assert_eq!(error.error_type(), "InvalidParameter");

        let error = crate::missing_param!("required_param");
        assert_eq!(error.error_type(), "MissingParameter");

        let error = crate::network_error!("fetch", "connection timeout");
        assert_eq!(error.error_type(), "NetworkError");

        let error = crate::internal_error!("system failure");
        assert_eq!(error.error_type(), "InternalError");
    }
}
