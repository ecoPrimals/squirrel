//! Tests for system configuration types

use super::{LogLevel, Environment, SystemConfig};

#[test]
fn test_log_level_variants() {
    let levels = vec![
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
    ];
    
    for level in levels {
        let cloned = level.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_log_level_partial_eq() {
    assert_eq!(LogLevel::Info, LogLevel::Info);
    assert_ne!(LogLevel::Info, LogLevel::Debug);
}

#[test]
fn test_log_level_serialization() {
    let level = LogLevel::Info;
    let serialized = serde_json::to_string(&level).expect("Failed to serialize");
    assert_eq!(serialized, "\"info\"");
}

#[test]
fn test_log_level_deserialization() {
    let json = "\"debug\"";
    let level: LogLevel = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(level, LogLevel::Debug);
}

#[test]
fn test_log_level_all_variants_serialization() {
    let variants = vec![
        (LogLevel::Trace, "trace"),
        (LogLevel::Debug, "debug"),
        (LogLevel::Info, "info"),
        (LogLevel::Warn, "warn"),
        (LogLevel::Error, "error"),
    ];
    
    for (level, expected) in variants {
        let serialized = serde_json::to_string(&level).expect("test: should succeed");
        assert!(serialized.contains(expected));
    }
}

#[test]
fn test_environment_variants() {
    let envs = vec![
        Environment::Development,
        Environment::Testing,
        Environment::Staging,
        Environment::Production,
    ];
    
    for env in envs {
        let cloned = env.clone();
        let debug_str = format!("{:?}", cloned);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_environment_partial_eq() {
    assert_eq!(Environment::Production, Environment::Production);
    assert_ne!(Environment::Production, Environment::Development);
}

#[test]
fn test_environment_serialization() {
    let env = Environment::Production;
    let serialized = serde_json::to_string(&env).expect("Failed to serialize");
    assert!(serialized.contains("production"));
}

#[test]
fn test_environment_deserialization() {
    let json = "\"development\"";
    let env: Environment = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(env, Environment::Development);
}

#[test]
fn test_system_config_default() {
    let config = SystemConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("SystemConfig"));
}

#[test]
fn test_system_config_clone() {
    let config = SystemConfig::default();
    let cloned = config.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_system_config_serialization() {
    let config = SystemConfig::default();
    let serialized = serde_json::to_string(&config).expect("Failed to serialize");
    let _deserialized: SystemConfig = serde_json::from_str(&serialized).expect("Failed to deserialize");
}

#[test]
fn test_log_level_copy() {
    let level = LogLevel::Info;
    let copied = level;
    assert_eq!(level, copied);
}

#[test]
fn test_environment_copy() {
    let env = Environment::Production;
    let copied = env;
    assert_eq!(env, copied);
}

#[test]
fn test_log_level_eq_trait() {
    let level1 = LogLevel::Debug;
    let level2 = LogLevel::Debug;
    assert!(level1 == level2);
}

#[test]
fn test_environment_eq_trait() {
    let env1 = Environment::Staging;
    let env2 = Environment::Staging;
    assert!(env1 == env2);
}

