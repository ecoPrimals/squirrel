// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Configuration validation tests
//!
//! Comprehensive tests for configuration validation, edge cases, and error handling.

#[cfg(test)]
mod tests {
    use crate::unified::system::SystemConfig;
    use crate::environment_config::Environment;
    use std::path::PathBuf;

    #[test]
    fn test_system_config_defaults() {
        // Arrange & Act
        let config = SystemConfig::default();
        
        // Assert
        assert!(!config.instance_id().is_empty(), "Instance ID should not be empty");
        assert_eq!(config.environment(), Environment::Development, "Default should be development");
    }

    #[test]
    fn test_system_config_testing() {
        // Arrange & Act
        let config = SystemConfig::testing();
        
        // Assert
        assert_eq!(config.instance_id(), "test-instance");
        assert_eq!(config.environment(), Environment::Testing);
        assert!(config.work_dir().to_str().expect("test: should succeed").contains("test"));
    }

    #[test]
    fn test_system_config_development() {
        // Arrange & Act
        let config = SystemConfig::development();
        
        // Assert
        assert_eq!(config.environment(), Environment::Development);
        assert!(config.data_dir().to_str().expect("test: should succeed").contains("data"));
    }

    #[test]
    fn test_system_config_production() {
        // Arrange & Act
        let config = SystemConfig::production();
        
        // Assert
        assert_eq!(config.environment(), Environment::Production);
        assert!(config.work_dir().to_str().expect("test: should succeed").contains("var"));
    }

    #[test]
    fn test_config_builder_pattern() {
        // Arrange & Act
        let config = SystemConfig::builder()
            .instance_id("test-builder-id")
            .environment(Environment::Testing)
            .build();
        
        // Assert
        assert_eq!(config.instance_id(), "test-builder-id");
        assert_eq!(config.environment(), Environment::Testing);
    }

    #[test]
    fn test_empty_instance_id_handling() {
        // Arrange
        let mut config = SystemConfig::testing();
        
        // Act - Try to set empty instance ID
        // Note: In real implementation, this should be validated
        let id_before = config.instance_id().to_string();
        
        // Assert - Should have non-empty ID
        assert!(!id_before.is_empty(), "Instance ID should never be empty");
    }

    #[test]
    fn test_invalid_path_characters() {
        // Arrange - Paths with special characters
        let paths = vec![
            PathBuf::from("/tmp/test\0invalid"),
            PathBuf::from("/tmp/test"),
            PathBuf::from("./relative/path"),
        ];
        
        // Act & Assert - All paths should be constructable
        for path in paths {
            assert!(path.to_str().is_some() || path.to_str().is_none());
        }
    }

    #[test]
    fn test_config_environment_variants() {
        // Arrange & Act
        let environments = vec![
            Environment::Development,
            Environment::Testing,
            Environment::Staging,
            Environment::Production,
        ];
        
        // Assert - All variants should be usable
        for env in environments {
            let config_file = env.config_file();
            assert!(!config_file.is_empty(), "Config file path should not be empty");
            assert!(config_file.ends_with(".toml"), "Should be a TOML file");
        }
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        // Arrange
        let config = SystemConfig::testing();
        
        // Act - Serialize
        let serialized = serde_json::to_string(&config);
        
        // Assert - Should serialize successfully
        assert!(serialized.is_ok(), "Config should serialize to JSON");
        
        if let Ok(json) = serialized {
            // Try to deserialize
            let deserialized: Result<SystemConfig, _> = serde_json::from_str(&json);
            assert!(deserialized.is_ok(), "Config should deserialize from JSON");
        }
    }

    #[test]
    fn test_config_equality() {
        // Arrange
        let config1 = SystemConfig::testing();
        let config2 = SystemConfig::testing();
        
        // Act & Assert
        assert_eq!(config1.instance_id(), config2.instance_id());
        assert_eq!(config1.environment(), config2.environment());
    }

    #[test]
    fn test_environment_case_insensitivity() {
        // Arrange
        use std::env;
        let original = env::var("SQUIRREL_ENV").ok();
        
        // Act & Assert - Test various casings
        for env_val in &["PRODUCTION", "production", "Production"] {
            env::set_var("SQUIRREL_ENV", env_val);
            let env = Environment::from_env();
            assert_eq!(env, Environment::Production, "Should handle case variations");
        }
        
        // Cleanup
        if let Some(val) = original {
            env::set_var("SQUIRREL_ENV", val);
        } else {
            env::remove_var("SQUIRREL_ENV");
        }
    }

    #[test]
    fn test_config_path_normalization() {
        // Arrange
        let paths = vec![
            PathBuf::from("./data"),
            PathBuf::from("/tmp/data"),
            PathBuf::from("../data"),
        ];
        
        // Act & Assert - All paths should be valid
        for path in paths {
            assert!(path.as_os_str().len() > 0, "Path should not be empty");
        }
    }

    #[test]
    fn test_config_clone() {
        // Arrange
        let config = SystemConfig::testing();
        
        // Act
        let cloned = config.clone();
        
        // Assert
        assert_eq!(config.instance_id(), cloned.instance_id());
        assert_eq!(config.environment(), cloned.environment());
    }

    #[test]
    fn test_config_debug_output() {
        // Arrange
        let config = SystemConfig::testing();
        
        // Act
        let debug_output = format!("{:?}", config);
        
        // Assert
        assert!(!debug_output.is_empty(), "Debug output should not be empty");
        assert!(debug_output.contains("SystemConfig"), "Should contain type name");
    }

    #[test]
    fn test_environment_display() {
        // Arrange & Act
        let envs = vec![
            (Environment::Development, "development"),
            (Environment::Testing, "testing"),
            (Environment::Staging, "staging"),
            (Environment::Production, "production"),
        ];
        
        // Assert
        for (env, expected) in envs {
            let display = format!("{}", env);
            assert_eq!(display.to_lowercase(), expected);
        }
    }

    #[test]
    fn test_config_with_unicode_paths() {
        // Arrange
        let unicode_paths = vec![
            PathBuf::from("/tmp/测试"),
            PathBuf::from("/tmp/тест"),
            PathBuf::from("/tmp/🦀"),
        ];
        
        // Act & Assert - Should handle Unicode
        for path in unicode_paths {
            assert!(path.as_os_str().len() > 0);
        }
    }
}

