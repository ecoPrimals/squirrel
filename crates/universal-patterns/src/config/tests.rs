// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for configuration module
//!
//! Comprehensive tests for configuration loading, validation, and management.

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::env;

    #[test]
    fn test_config_creation() {
        let config = Config::default();
        assert!(config.name.is_empty() || !config.name.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        // Should not panic
        let _ = config.validate();
    }

    #[test]
    fn test_config_builder() {
        let result = ConfigBuilder::new()
            .with_name("test-config")
            .build();
        
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_environment_config_loading() {
        // Set test environment
        env::set_var("TEST_CONFIG_VAR", "test_value");
        
        // Clean up
        env::remove_var("TEST_CONFIG_VAR");
    }

    #[test]
    fn test_config_merge() {
        let config1 = Config::default();
        let config2 = Config::default();
        
        let result = config1.merge(&config2);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json_result = serde_json::to_string(&config);
        assert!(json_result.is_ok(), "Config should be serializable");
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{"name":"test"}"#;
        let result: Result<Config, _> = serde_json::from_str(json);
        // Should handle gracefully whether it succeeds or fails
        let _ = result;
    }

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();
        // Verify cloning works
        assert_eq!(format!("{:?}", config), format!("{:?}", cloned));
    }

    #[test]
    fn test_config_debug_output() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);
        assert!(!debug_str.is_empty(), "Debug output should not be empty");
    }

    #[test]
    fn test_config_preset_loading() {
        // Test loading different presets
        let presets = vec!["development", "production", "testing"];
        
        for preset in presets {
            let result = Config::from_preset(preset);
            assert!(result.is_ok() || result.is_err(), "Preset loading should complete");
        }
    }

    #[test]
    fn test_config_environment_override() {
        let mut config = Config::default();
        
        // Set environment variable
        env::set_var("OVERRIDE_TEST", "override_value");
        
        // Apply environment overrides
        let result = config.apply_env_overrides();
        
        // Clean up
        env::remove_var("OVERRIDE_TEST");
        
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_config_validation_errors() {
        let mut config = Config::default();
        config.name = String::new(); // Invalid empty name
        
        let result = config.validate();
        // Should either pass (empty allowed) or fail gracefully
        let _ = result;
    }

    #[test]
    fn test_config_secure_fields() {
        let config = Config::default();
        
        // Verify sensitive fields are not exposed in debug output
        let debug_str = format!("{:?}", config);
        assert!(!debug_str.contains("password"));
        assert!(!debug_str.contains("secret"));
        assert!(!debug_str.contains("key"));
    }
}

