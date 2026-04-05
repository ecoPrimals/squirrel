// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;

#[test]
fn test_environment_from_string() {
    assert_eq!(
        Environment::from_str("development").expect("should succeed"),
        Environment::Development
    );
    assert_eq!(
        Environment::from_str("production").expect("should succeed"),
        Environment::Production
    );
    assert!(Environment::from_str("invalid").is_err());
}

#[test]
fn test_environment_from_str_all_variants() {
    assert_eq!(
        Environment::from_str("dev").expect("should succeed"),
        Environment::Development
    );
    assert_eq!(
        Environment::from_str("test").expect("should succeed"),
        Environment::Testing
    );
    assert_eq!(
        Environment::from_str("staging").expect("should succeed"),
        Environment::Staging
    );
    assert_eq!(
        Environment::from_str("prod").expect("should succeed"),
        Environment::Production
    );
}

#[test]
fn test_environment_is_development() {
    assert!(Environment::Development.is_development());
    assert!(!Environment::Production.is_development());
}

#[test]
fn test_environment_is_production() {
    assert!(Environment::Production.is_production());
    assert!(!Environment::Development.is_production());
}

#[test]
fn test_environment_config_suffix() {
    assert_eq!(Environment::Development.config_suffix(), "dev");
    assert_eq!(Environment::Testing.config_suffix(), "test");
    assert_eq!(Environment::Staging.config_suffix(), "staging");
    assert_eq!(Environment::Production.config_suffix(), "prod");
}

#[test]
fn test_environment_from_env_default() {
    temp_env::with_var_unset("MCP_ENV", || {
        let env_type = Environment::from_env();
        assert!(matches!(
            env_type,
            Environment::Development | Environment::Testing
        ));
    });
}

#[test]
fn test_environment_from_env_production() {
    temp_env::with_var("MCP_ENV", Some("production"), || {
        let env_type = Environment::from_env();
        assert_eq!(env_type, Environment::Production);
    });
}

#[test]
fn test_environment_error_display() {
    let err = EnvironmentError::MissingVariable("TEST_VAR".to_string());
    assert!(err.to_string().contains("TEST_VAR"));

    let err = EnvironmentError::InvalidValue {
        variable: "PORT".to_string(),
        value: "abc".to_string(),
    };
    assert!(err.to_string().contains("PORT"));
}

#[test]
fn test_environment_config_validation() {
    let mut config = test_env_config();

    config.network.port = 0;
    assert!(config.validate().is_err());

    config.network.port = 8080;
    config.network.request_timeout_ms = 0;
    assert!(config.validate().is_err());
}

fn test_network_config() -> NetworkConfig {
    NetworkConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        cors_origins: vec![],
        request_timeout_ms: 30000,
        max_connections: 100,
    }
}

fn test_ai_provider_config() -> AIProviderConfig {
    AIProviderConfig {
        openai_api_key: None,
        openai_endpoint: "https://api.openai.com/v1".to_string(),
        anthropic_api_key: None,
        anthropic_endpoint: "https://api.anthropic.com/v1".to_string(),
        local_server_endpoint: "http://localhost:11434".to_string(),
        default_model: "gpt-3.5-turbo".to_string(),
        request_timeout_ms: 30000,
    }
}

fn test_database_config() -> DatabaseConfig {
    DatabaseConfig {
        connection_string: "sqlite::memory:".to_string(),
        max_connections: 5,
        timeout_seconds: 30,
    }
}

fn test_ecosystem_config() -> EcosystemConfig {
    EcosystemConfig {
        storage_endpoint: "discovered://storage".to_string(),
        security_endpoint: "discovered://security".to_string(),
        compute_endpoint: "discovered://compute".to_string(),
        service_mesh_endpoint: "discovered://service-mesh".to_string(),
        service_timeout_ms: 5000,
    }
}

fn test_env_config() -> EnvironmentConfig {
    EnvironmentConfig {
        environment: Environment::Testing,
        network: test_network_config(),
        ai_providers: test_ai_provider_config(),
        database: test_database_config(),
        ecosystem: test_ecosystem_config(),
    }
}

#[test]
fn test_environment_config_validation_empty_database() {
    let mut config = test_env_config();
    config.database.connection_string = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn test_environment_config_validation_empty_openai_endpoint() {
    let mut config = test_env_config();
    config.ai_providers.openai_endpoint = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn test_network_config_fields() {
    let config = test_network_config();
    assert!(config.port > 0);
    assert!(config.request_timeout_ms > 0);
    assert!(config.max_connections > 0);
}

#[test]
fn test_ai_provider_config_fields() {
    let config = test_ai_provider_config();
    assert!(config.request_timeout_ms > 0);
    assert!(!config.openai_endpoint.is_empty());
}

#[test]
fn test_ecosystem_config_fields() {
    let config = test_ecosystem_config();
    assert!(!config.storage_endpoint.is_empty());
    assert!(!config.security_endpoint.is_empty());
}

#[test]
fn test_environment_config_validate_ok() {
    let config = test_env_config();
    assert!(config.validate().is_ok());
}

#[test]
fn test_network_config_from_env_defaults() {
    temp_env::with_vars(
        [
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = NetworkConfig::from_env().expect("should load");
            assert!(config.port > 0);
            assert_eq!(config.request_timeout_ms, 5000);
            assert_eq!(config.max_connections, 50);
        },
    );
}

#[test]
fn test_network_config_from_env_invalid_timeout() {
    temp_env::with_var("MCP_REQUEST_TIMEOUT_MS", Some("invalid"), || {
        assert!(NetworkConfig::from_env().is_err());
    });
}

#[test]
fn test_ai_provider_config_from_env() {
    temp_env::with_var("AI_REQUEST_TIMEOUT_MS", Some("15000"), || {
        let config = AIProviderConfig::from_env().expect("should load");
        assert_eq!(config.request_timeout_ms, 15000);
    });
}

#[test]
fn test_ai_provider_config_from_env_invalid() {
    temp_env::with_var("AI_REQUEST_TIMEOUT_MS", Some("not_a_number"), || {
        assert!(AIProviderConfig::from_env().is_err());
    });
}

#[test]
fn test_ecosystem_config_from_env() {
    temp_env::with_vars(
        [
            ("NESTGATE_ENDPOINT", Some("http://127.0.0.1:8444")),
            ("BEARDOG_ENDPOINT", Some("http://127.0.0.1:8443")),
            ("TOADSTOOL_ENDPOINT", Some("http://127.0.0.1:8445")),
            ("SERVICE_MESH_ENDPOINT", Some("http://127.0.0.1:8446")),
        ],
        || {
            let config = EcosystemConfig::from_env().expect("should load");
            assert!(config.storage_endpoint.contains("127.0.0.1"));
        },
    );
}

#[test]
fn test_environment_config_snake_case_serialization() {
    let env = Environment::Development;
    assert_eq!(env.config_suffix(), "dev");
}

// ========== Environment::from_env additional coverage ==========
#[test]
fn test_environment_from_env_staging() {
    temp_env::with_var("MCP_ENV", Some("staging"), || {
        let env_type = Environment::from_env();
        assert_eq!(env_type, Environment::Staging);
    });
}

#[test]
fn test_environment_from_env_testing() {
    temp_env::with_var("MCP_ENV", Some("testing"), || {
        let env_type = Environment::from_env();
        assert_eq!(env_type, Environment::Testing);
    });
}

#[test]
fn test_environment_from_str_parse_error() {
    let err = Environment::from_str("unknown").unwrap_err();
    assert!(matches!(err, EnvironmentError::InvalidValue { .. }));
    assert!(err.to_string().contains("MCP_ENV"));
}

#[test]
fn test_environment_error_parse_error_display() {
    let err = EnvironmentError::ParseError {
        variable: "MCP_PORT".to_string(),
        error: "invalid digit".to_string(),
    };
    assert!(err.to_string().contains("MCP_PORT"));
    assert!(err.to_string().contains("invalid digit"));
}

// ========== NetworkConfig::from_env comprehensive ==========
#[test]
fn test_network_config_from_env_host_production() {
    temp_env::with_vars(
        [
            ("MCP_ENVIRONMENT", Some("production")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = NetworkConfig::from_env().expect("should load");
            assert_eq!(config.host, "0.0.0.0");
        },
    );
}

#[test]
fn test_network_config_from_env_host_explicit() {
    temp_env::with_vars(
        [
            ("MCP_HOST", Some("192.168.1.1")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = NetworkConfig::from_env().expect("should load");
            assert_eq!(config.host, "192.168.1.1");
        },
    );
}

#[test]
fn test_network_config_from_env_port_explicit() {
    temp_env::with_vars(
        [
            ("MCP_PORT", Some("9090")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = NetworkConfig::from_env().expect("should load");
            assert_eq!(config.port, 9090);
        },
    );
}

#[test]
fn test_network_config_from_env_port_invalid_fallback() {
    temp_env::with_vars(
        [
            ("MCP_PORT", Some("not_a_number")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = NetworkConfig::from_env().expect("should load");
            assert_eq!(config.port, 8080); // Fallback
        },
    );
}

#[test]
fn test_network_config_from_env_cors_origins() {
    temp_env::with_vars(
        [
            ("MCP_CORS_ORIGINS", Some("http://a.com, http://b.com")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = NetworkConfig::from_env().expect("should load");
            assert_eq!(config.cors_origins.len(), 2);
            assert!(config.cors_origins.contains(&"http://a.com".to_string()));
            assert!(config.cors_origins.contains(&"http://b.com".to_string()));
        },
    );
}

#[test]
fn test_network_config_from_env_cors_origins_web_ui_port() {
    temp_env::with_vars(
        [
            ("WEB_UI_PORT", Some("4000")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = NetworkConfig::from_env().expect("should load");
            assert!(config.cors_origins.iter().any(|o| o.contains("4000")));
        },
    );
}

#[test]
fn test_network_config_from_env_invalid_max_connections() {
    temp_env::with_var("MCP_MAX_CONNECTIONS", Some("invalid"), || {
        assert!(NetworkConfig::from_env().is_err());
    });
}

// ========== DatabaseConfig - Testing/Development only (avoid exit) ==========
#[test]
fn test_database_config_from_env_testing() {
    temp_env::with_vars(
        [
            ("MCP_ENV", Some("testing")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = DatabaseConfig::from_env().expect("should load");
            assert_eq!(config.connection_string, "sqlite::memory:");
        },
    );
}

#[test]
fn test_database_config_from_env_development_default() {
    temp_env::with_vars(
        [
            ("MCP_ENV", Some("development")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = DatabaseConfig::from_env().expect("should load");
            assert_eq!(config.connection_string, "sqlite::memory:");
        },
    );
}

#[test]
fn test_database_config_from_env_development_with_dev_url() {
    temp_env::with_vars(
        [
            ("MCP_ENV", Some("development")),
            ("DATABASE_URL_DEV", Some("postgres://localhost/dev")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = DatabaseConfig::from_env().expect("should load");
            assert_eq!(config.connection_string, "postgres://localhost/dev");
        },
    );
}

#[test]
fn test_database_config_from_env_with_database_url() {
    temp_env::with_vars(
        [
            ("MCP_ENV", Some("development")),
            ("DATABASE_URL", Some("postgres://prod/db")),
            ("DATABASE_MAX_CONNECTIONS", Some("20")),
            ("DATABASE_TIMEOUT_SECS", Some("60")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = DatabaseConfig::from_env().expect("should load");
            assert_eq!(config.connection_string, "postgres://prod/db");
            assert_eq!(config.max_connections, 20);
            assert_eq!(config.timeout_seconds, 60);
        },
    );
}

#[test]
fn test_database_config_max_connections_invalid_fallback() {
    temp_env::with_vars(
        [
            ("MCP_ENV", Some("testing")),
            ("DATABASE_MAX_CONNECTIONS", Some("invalid")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
        ],
        || {
            let config = DatabaseConfig::from_env().expect("should load");
            assert_eq!(config.max_connections, 10); // Fallback
        },
    );
}

// ========== AIProviderConfig comprehensive ==========
#[test]
fn test_ai_provider_config_from_env_full() {
    temp_env::with_vars(
        [
            ("OPENAI_API_KEY", Some("sk-test")),
            ("OPENAI_ENDPOINT", Some("https://custom.openai.com/v1")),
            ("ANTHROPIC_API_KEY", Some("sk-ant-test")),
            (
                "ANTHROPIC_ENDPOINT",
                Some("https://custom.anthropic.com/v1"),
            ),
            ("LOCAL_AI_ENDPOINT", Some("http://localhost:9999")),
            ("MCP_DEFAULT_MODEL", Some("gpt-4")),
            ("AI_REQUEST_TIMEOUT_MS", Some("60000")),
        ],
        || {
            let config = AIProviderConfig::from_env().expect("should load");
            assert_eq!(config.openai_api_key.as_deref(), Some("sk-test"));
            assert_eq!(config.openai_endpoint, "https://custom.openai.com/v1");
            assert_eq!(config.anthropic_api_key.as_deref(), Some("sk-ant-test"));
            assert_eq!(config.anthropic_endpoint, "https://custom.anthropic.com/v1");
            assert_eq!(config.local_server_endpoint, "http://localhost:9999");
            assert_eq!(config.default_model, "gpt-4");
            assert_eq!(config.request_timeout_ms, 60000);
        },
    );
}

#[test]
fn test_ai_provider_config_local_endpoint_ollama_fallback() {
    temp_env::with_vars(
        [
            ("OLLAMA_ENDPOINT", Some("http://ollama:11434")),
            ("AI_REQUEST_TIMEOUT_MS", Some("5000")),
        ],
        || {
            let config = AIProviderConfig::from_env().expect("should load");
            assert_eq!(config.local_server_endpoint, "http://ollama:11434");
        },
    );
}

#[test]
fn test_ai_provider_config_local_endpoint_toadstool_not_a_local_ai_fallback() {
    temp_env::with_vars(
        [
            ("TOADSTOOL_ENDPOINT", Some("http://toadstool:8445")),
            ("AI_REQUEST_TIMEOUT_MS", Some("5000")),
        ],
        || {
            let config = AIProviderConfig::from_env().expect("should load");
            assert_ne!(
                config.local_server_endpoint, "http://toadstool:8445",
                "TOADSTOOL_ENDPOINT is compute, not local AI"
            );
        },
    );
}

#[test]
fn test_ai_provider_config_local_port_override() {
    temp_env::with_vars(
        [
            ("LOCAL_AI_PORT", Some("12345")),
            ("AI_REQUEST_TIMEOUT_MS", Some("5000")),
        ],
        || {
            let config = AIProviderConfig::from_env().expect("should load");
            assert_eq!(config.local_server_endpoint, "http://localhost:12345");
        },
    );
}

#[test]
fn test_ai_provider_config_ollama_port_override() {
    temp_env::with_vars(
        [
            ("OLLAMA_PORT", Some("9999")),
            ("AI_REQUEST_TIMEOUT_MS", Some("5000")),
        ],
        || {
            let config = AIProviderConfig::from_env().expect("should load");
            assert_eq!(config.local_server_endpoint, "http://localhost:9999");
        },
    );
}

// ========== EcosystemConfig comprehensive ==========
#[test]
fn test_ecosystem_config_from_env_production_discovered() {
    temp_env::with_vars(
        [
            ("MCP_ENVIRONMENT", Some("production")),
            ("ECOSYSTEM_SERVICE_TIMEOUT_MS", Some("8000")),
        ],
        || {
            let config = EcosystemConfig::from_env().expect("should load");
            assert!(config.storage_endpoint.starts_with("discovered://"));
            assert!(config.security_endpoint.starts_with("discovered://"));
            assert!(config.compute_endpoint.starts_with("discovered://"));
            assert!(config.service_mesh_endpoint.starts_with("discovered://"));
            assert_eq!(config.service_timeout_ms, 8000);
        },
    );
}

#[test]
fn test_ecosystem_config_from_env_port_overrides() {
    temp_env::with_vars(
        [
            ("NESTGATE_PORT", Some("9444")),
            ("SECURITY_AUTHENTICATION_PORT", Some("9443")),
            ("TOADSTOOL_PORT", Some("9445")),
            ("BIOMEOS_PORT", Some("9446")),
            ("ECOSYSTEM_SERVICE_TIMEOUT_MS", Some("3000")),
        ],
        || {
            let config = EcosystemConfig::from_env().expect("should load");
            assert!(config.storage_endpoint.contains("9444"));
            assert!(config.security_endpoint.contains("9443"));
            assert!(config.compute_endpoint.contains("9445"));
            assert!(config.service_mesh_endpoint.contains("9446"));
            assert_eq!(config.service_timeout_ms, 3000);
        },
    );
}

#[test]
fn test_ecosystem_config_default() {
    temp_env::with_vars_unset(
        [
            "NESTGATE_ENDPOINT",
            "BEARDOG_ENDPOINT",
            "TOADSTOOL_ENDPOINT",
            "SERVICE_MESH_ENDPOINT",
            "BIOMEOS_ENDPOINT",
        ],
        || {
            let config = EcosystemConfig::default();
            assert!(config.storage_endpoint.contains("localhost"));
            assert!(config.security_endpoint.contains("localhost"));
            assert!(config.compute_endpoint.contains("localhost"));
            assert!(config.service_mesh_endpoint.contains("localhost"));
            assert_eq!(config.service_timeout_ms, 5000);
        },
    );
}

#[test]
fn test_ecosystem_config_service_mesh_biomeos_precedence() {
    temp_env::with_vars(
        [
            ("BIOMEOS_ENDPOINT", Some("http://biomeos:8446")),
            ("SERVICE_MESH_ENDPOINT", Some("http://mesh:8447")),
            ("ECOSYSTEM_SERVICE_TIMEOUT_MS", Some("1000")),
        ],
        || {
            let config = EcosystemConfig::from_env().expect("should load");
            // BIOMEOS_ENDPOINT takes precedence (checked first)
            assert_eq!(config.service_mesh_endpoint, "http://biomeos:8446");
        },
    );
}

// ========== EnvironmentConfig ==========
#[test]
fn test_environment_config_from_env_full() {
    temp_env::with_vars(
        [
            ("MCP_ENV", Some("testing")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
            ("AI_REQUEST_TIMEOUT_MS", Some("10000")),
        ],
        || {
            let config = EnvironmentConfig::from_env().expect("should load");
            assert_eq!(config.environment, Environment::Testing);
            assert_eq!(config.network.request_timeout_ms, 5000);
            assert_eq!(config.ai_providers.request_timeout_ms, 10000);
        },
    );
}

#[test]
fn test_environment_config_load_and_validate() {
    temp_env::with_vars(
        [
            ("MCP_ENV", Some("testing")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("5000")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
            ("AI_REQUEST_TIMEOUT_MS", Some("10000")),
        ],
        || {
            let config = EnvironmentConfig::load_and_validate().expect("should load");
            assert!(config.validate().is_ok());
        },
    );
}

#[test]
fn test_environment_config_load_and_validate_fails_on_invalid() {
    temp_env::with_vars(
        [
            ("MCP_ENV", Some("testing")),
            ("MCP_REQUEST_TIMEOUT_MS", Some("0")),
            ("MCP_MAX_CONNECTIONS", Some("50")),
            ("AI_REQUEST_TIMEOUT_MS", Some("10000")),
        ],
        || {
            let result = EnvironmentConfig::load_and_validate();
            assert!(result.is_err());
        },
    );
}
