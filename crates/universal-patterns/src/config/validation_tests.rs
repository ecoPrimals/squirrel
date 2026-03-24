use super::*;
use std::fs;
use tempfile::tempdir;
use url::Url;

#[test]
fn test_validate_valid_config() {
    let config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();

    assert!(ConfigValidator::validate(&config).is_ok());
}

#[test]
fn test_validate_empty_name() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();

    config.info.name = String::new();

    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_invalid_version() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();

    config.info.version = "invalid-version".to_string();

    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_invalid_bind_address() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();

    config.network.bind_address = "invalid-address".to_string();

    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_zero_port() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();

    config.network.port = 0;

    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_missing_tls_files() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();

    config.network.tls = Some(TlsConfig {
        cert_file: PathBuf::from("/nonexistent/cert.pem"),
        key_file: PathBuf::from("/nonexistent/key.pem"),
        ca_file: None,
        require_client_cert: false,
    });

    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_cross_dependencies() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();

    // Enable orchestration without Songbird endpoint
    config.orchestration.enabled = true;
    config.orchestration.songbird_endpoint = None;

    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_semver() {
    assert!(ConfigValidator::is_valid_semver("1.0.0"));
    assert!(ConfigValidator::is_valid_semver("1.0"));
    assert!(ConfigValidator::is_valid_semver("1.2.3"));
    assert!(!ConfigValidator::is_valid_semver("1"));
    assert!(!ConfigValidator::is_valid_semver("1.0.0.0"));
    assert!(!ConfigValidator::is_valid_semver("1.0.0-alpha"));
    assert!(!ConfigValidator::is_valid_semver("invalid"));
}

#[test]
fn test_validate_hostname() {
    assert!(ConfigValidator::is_valid_hostname("example.com"));
    assert!(ConfigValidator::is_valid_hostname("test-host.example.com"));
    assert!(ConfigValidator::is_valid_hostname("host123.example.com"));
    assert!(!ConfigValidator::is_valid_hostname(""));
    assert!(!ConfigValidator::is_valid_hostname("-example.com"));
    assert!(!ConfigValidator::is_valid_hostname("example-.com"));
    assert!(!ConfigValidator::is_valid_hostname("example..com"));
}

#[test]
fn test_validate_resource_limits() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();

    config.environment.resources.max_cpu_percent = Some(150.0);

    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_reserved_custom_keys() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();

    config.custom.insert(
        "info".to_string(),
        serde_json::Value::String("test".to_string()),
    );

    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_invalid_primal_name_characters() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.info.name = "bad name".to_string();
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_empty_description() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.info.description.clear();
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_public_address_as_hostname() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.network.public_address = Some("node.example.com".to_string());
    assert!(ConfigValidator::validate(&config).is_ok());
}

#[test]
fn test_validate_invalid_public_address() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.network.public_address = Some("not an ip or host!".to_string());
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_connect_timeout_greater_than_request() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.network.timeouts.connect = 100;
    config.network.timeouts.request = 10;
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_zero_keep_alive_timeout() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.network.timeouts.keep_alive = 0;
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_connection_limits_zero() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.network.limits.max_connections = 0;
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_rate_limit_non_positive() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.network.limits.rate_limit = Some(0.0);
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_beardog_endpoint_bad_scheme() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.security.beardog_endpoint = Some(Url::parse("ftp://example.com").expect("url"));
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_songbird_endpoint_bad_scheme() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.orchestration.songbird_endpoint = Some(Url::parse("ftp://mesh").expect("url"));
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_token_auth_missing_file() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.security.auth_method = AuthMethod::Token {
        token_file: PathBuf::from("/nonexistent/token/file"),
    };
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_token_auth_existing_file() {
    let dir = tempdir().expect("tempdir");
    let token_path = dir.path().join("tok");
    fs::write(&token_path, b"x").expect("write");
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.security.auth_method = AuthMethod::Token {
        token_file: token_path,
    };
    assert!(ConfigValidator::validate(&config).is_ok());
}

#[test]
fn test_validate_cert_auth_files() {
    let dir = tempdir().expect("tempdir");
    let cert = dir.path().join("c.pem");
    let key = dir.path().join("k.pem");
    fs::write(&cert, b"c").expect("write");
    fs::write(&key, b"k").expect("write");
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.security.auth_method = AuthMethod::Certificate {
        cert_file: cert,
        key_file: key,
    };
    assert!(ConfigValidator::validate(&config).is_ok());
}

#[test]
fn test_validate_tls_with_ca_missing_file() {
    let dir = tempdir().expect("tempdir");
    let cert = dir.path().join("c.pem");
    let key = dir.path().join("k.pem");
    let ca = dir.path().join("ca.pem");
    fs::write(&cert, b"c").expect("write");
    fs::write(&key, b"k").expect("write");
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.network.tls = Some(TlsConfig {
        cert_file: cert,
        key_file: key,
        ca_file: Some(ca),
        require_client_cert: false,
    });
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_credential_storage_missing_parent_dir() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.security.credential_storage = CredentialStorage::File {
        path: PathBuf::from("/nonexistent/parent/creds.json"),
    };
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_key_management_file_missing_parent() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.security.encryption.key_management = KeyManagement::File {
        path: PathBuf::from("/nonexistent/keys/key.bin"),
    };
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_key_management_empty_env_var() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.security.encryption.key_management = KeyManagement::Environment {
        var_name: String::new(),
    };
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_health_check_invalid_when_enabled() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.orchestration.health_check = HealthCheckConfig {
        enabled: true,
        interval: 0,
        timeout: 5,
        endpoint: "/health".to_string(),
    };
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_health_check_timeout_not_less_than_interval() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.orchestration.health_check = HealthCheckConfig {
        enabled: true,
        interval: 10,
        timeout: 10,
        endpoint: "/health".to_string(),
    };
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_health_check_endpoint_must_start_with_slash() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.orchestration.health_check = HealthCheckConfig {
        enabled: true,
        interval: 10,
        timeout: 5,
        endpoint: "health".to_string(),
    };
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_service_discovery_dns_invalid_domain() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.orchestration.service_discovery = ServiceDiscoveryConfig {
        enabled: true,
        method: ServiceDiscoveryMethod::Dns {
            domain: "not a hostname!".to_string(),
        },
        ttl: 60,
    };
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_service_discovery_file_missing_parent() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.orchestration.service_discovery = ServiceDiscoveryConfig {
        enabled: true,
        method: ServiceDiscoveryMethod::File {
            path: PathBuf::from("/nonexistent/dir/services.json"),
        },
        ttl: 60,
    };
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_logging_requires_output() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.logging.outputs.clear();
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_log_file_parent_missing() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.logging.outputs = vec![LogOutput::File {
        path: PathBuf::from("/nonexistent/dir/app.log"),
    }];
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_resource_limits_memory_zero() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.environment.resources.max_memory_mb = Some(0);
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_resource_limits_disk_zero() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.environment.resources.max_disk_mb = Some(0);
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_resource_limits_fds_zero() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.environment.resources.max_file_descriptors = Some(0);
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_beardog_auth_requires_endpoint() {
    let mut config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .build_unchecked();
    config.security.auth_method = AuthMethod::Beardog {
        service_id: "svc".to_string(),
    };
    config.security.beardog_endpoint = None;
    assert!(ConfigValidator::validate(&config).is_err());
}

#[test]
fn test_validate_encryption_requires_env_key_when_enabled() {
    temp_env::with_var("UP_VALIDATION_ENC_KEY_TEST", None::<&str>, || {
        let mut config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();
        config.security.encryption.enable_inter_primal = true;
        config.security.encryption.key_management = KeyManagement::Environment {
            var_name: "UP_VALIDATION_ENC_KEY_TEST".to_string(),
        };
        assert!(ConfigValidator::validate(&config).is_err());
    });
}

#[test]
fn test_validate_encryption_ok_when_env_key_set() {
    temp_env::with_var(
        "UP_VALIDATION_ENC_KEY_TEST_OK",
        Some("secret-material"),
        || {
            let mut config = ConfigBuilder::new()
                .name("test-primal")
                .version("1.0.0")
                .build_unchecked();
            config.security.encryption.enable_at_rest = true;
            config.security.encryption.key_management = KeyManagement::Environment {
                var_name: "UP_VALIDATION_ENC_KEY_TEST_OK".to_string(),
            };
            assert!(ConfigValidator::validate(&config).is_ok());
        },
    );
}
