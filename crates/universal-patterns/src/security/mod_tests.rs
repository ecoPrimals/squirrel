// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for security module helper functions

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_version_constant() {
        assert_eq!(VERSION, "1.0.0");
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_get_module_info() {
        let info = get_module_info();

        assert_eq!(info.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(
            info.get("name"),
            Some(&"Universal Security Module".to_string())
        );
        assert_eq!(info.get("supports_beardog"), Some(&"true".to_string()));
        assert_eq!(
            info.get("supports_local_fallback"),
            Some(&"true".to_string())
        );
        assert_eq!(
            info.get("supports_audit_logging"),
            Some(&"true".to_string())
        );
        assert_eq!(
            info.get("supports_health_monitoring"),
            Some(&"true".to_string())
        );
        assert_eq!(info.get("supports_encryption"), Some(&"true".to_string()));
        assert_eq!(
            info.get("supports_digital_signatures"),
            Some(&"true".to_string())
        );
        assert_eq!(info.get("thread_safe"), Some(&"true".to_string()));

        assert!(info.contains_key("description"));
        assert!(info.len() >= 9);
    }

    #[tokio::test]
    async fn test_create_default_client() {
        let result = create_default_client().await;
        assert!(result.is_ok(), "Failed to create default security client");
    }

    #[tokio::test]
    async fn test_create_local_provider() {
        let result = create_local_provider().await;
        assert!(result.is_ok(), "Failed to create local security provider");
    }

    #[tokio::test]
    async fn test_validate_initialization() {
        let result = validate_initialization().await;
        assert!(
            result.is_ok(),
            "Security module initialization validation failed: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_create_beardog_client() {
        let endpoint = url::Url::parse("http://localhost:8080").unwrap();
        let service_id = "test-service".to_string();

        // Try with fallback enabled
        let result = create_beardog_client(endpoint.clone(), service_id.clone(), true).await;
        // This might fail if Beardog is not available, which is expected
        // We're mainly testing that the function can be called without panicking
        let _ = result;

        // Try with fallback disabled
        let result = create_beardog_client(endpoint, service_id, false).await;
        let _ = result;
    }

    #[test]
    fn test_module_exports() {
        // Test that all major types are exported and accessible
        use crate::security::*;

        // Verify major types are accessible
        let _ = SecurityError::Configuration("test".to_string());
        let _ = HealthStatus::Healthy;

        // These are just compile-time checks
        assert!(true);
    }

    #[tokio::test]
    async fn test_client_creation_with_different_configs() {
        // Test default client
        let client1 = create_default_client().await;
        assert!(client1.is_ok());

        // Test local provider
        let provider = create_local_provider().await;
        assert!(provider.is_ok());
    }

    #[test]
    fn test_version_format() {
        assert!(VERSION.contains('.'), "Version should be in semver format");
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "Version should have major.minor.patch format"
        );
    }

    #[test]
    fn test_module_info_completeness() {
        let info = get_module_info();

        // Essential keys must be present
        let essential_keys = vec![
            "version",
            "name",
            "description",
            "supports_beardog",
            "supports_local_fallback",
        ];

        for key in essential_keys {
            assert!(info.contains_key(key), "Module info missing key: {}", key);
        }
    }
}
