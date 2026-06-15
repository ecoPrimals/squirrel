// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use tempfile::NamedTempFile;

#[test]
fn test_builder_basic() {
    let config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .port(9000)
        .build()
        .expect("should succeed");

    assert_eq!(config.info.name, "test-primal");
    assert_eq!(config.info.version, "1.0.0");
    assert_eq!(config.network.port, 9000);
}

#[test]
fn test_builder_squirrel() {
    let config = ConfigBuilder::squirrel()
        .version("1.0.0")
        .build()
        .expect("should succeed");

    assert_eq!(config.info.name, "squirrel");
    assert_eq!(config.info.primal_type, PrimalType::Coordinator);
    assert_eq!(config.network.port, 8080);
}

#[test]
fn test_builder_security() {
    let config = ConfigBuilder::security()
        .name("test-security")
        .version("1.0.0")
        .build()
        .expect("should succeed");

    assert_eq!(config.info.name, "test-security");
    assert_eq!(config.info.primal_type, PrimalType::Security);
    assert_eq!(config.network.port, 8081);
}

#[test]
fn test_builder_orchestration() {
    let config = ConfigBuilder::orchestration()
        .version("1.0.0")
        .build()
        .expect("should succeed");

    assert_eq!(config.info.name, "orchestration");
    assert_eq!(config.info.primal_type, PrimalType::Orchestration);
    assert_eq!(config.network.port, 8082);
}

#[test]
fn test_builder_development() {
    let config = ConfigBuilder::development()
        .name("test-primal")
        .version("1.0.0")
        .build()
        .expect("should succeed");

    assert_eq!(config.environment.name, "development");
    assert_eq!(config.logging.level, LogLevel::Debug);
    assert!(config.logging.structured);
    assert!(config.logging.tracing);
    assert_eq!(config.environment.features.get("debug_mode"), Some(&true));
}

#[test]
fn test_builder_production() {
    temp_env::with_var(
        "PRIMAL_ENCRYPTION_KEY",
        Some("test_key_for_testing_12345678901234567890"),
        || {
            let config = ConfigBuilder::production()
                .name("test-primal")
                .version("1.0.0")
                .build()
                .expect("should succeed");

            assert_eq!(config.environment.name, "production");
            assert_eq!(config.logging.level, LogLevel::Info);
            assert_eq!(config.logging.format, LogFormat::Json);
            assert!(config.security.audit_logging);
            assert!(config.security.encryption.enable_inter_primal);
            assert!(config.security.encryption.enable_at_rest);
            assert_eq!(config.environment.features.get("debug_mode"), Some(&false));
        },
    );
}

#[test]
fn test_builder_tls() {
    let cert_file = NamedTempFile::new().expect("should succeed");
    let key_file = NamedTempFile::new().expect("should succeed");

    let config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .tls(
            cert_file.path().to_path_buf(),
            key_file.path().to_path_buf(),
        )
        .build()
        .expect("should succeed");

    assert!(config.network.tls.is_some());
    let tls = config.network.tls.expect("should succeed");
    assert_eq!(tls.cert_file, cert_file.path());
    assert_eq!(tls.key_file, key_file.path());
    assert!(!tls.require_client_cert);
}

#[test]
fn test_builder_custom_config() {
    let config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .add_custom("test_key", "test_value")
        .expect("should succeed")
        .build()
        .expect("should succeed");

    let custom_value: Option<String> = config.get_custom("test_key").expect("should succeed");
    assert_eq!(custom_value, Some("test_value".to_string()));
}

#[test]
fn test_builder_resource_limits() {
    let config = ConfigBuilder::new()
        .name("test-primal")
        .version("1.0.0")
        .max_memory_mb(1024)
        .max_cpu_percent(50.0)
        .build()
        .expect("should succeed");

    assert_eq!(config.environment.resources.max_memory_mb, Some(1024));
    assert_eq!(config.environment.resources.max_cpu_percent, Some(50.0));
}

#[test]
fn test_builder_mutual_tls() {
    let cert = NamedTempFile::new().expect("create");
    let key = NamedTempFile::new().expect("create");
    let ca = NamedTempFile::new().expect("create");

    let config = ConfigBuilder::new()
        .name("mtls-test")
        .version("1.0.0")
        .mutual_tls(
            cert.path().to_path_buf(),
            key.path().to_path_buf(),
            ca.path().to_path_buf(),
        )
        .build()
        .expect("build");

    let tls = config.network.tls.expect("tls present");
    assert!(tls.require_client_cert);
    assert!(tls.ca_file.is_some());
}

#[test]
fn test_builder_beardog_endpoint_valid() {
    let config = ConfigBuilder::new()
        .name("beardog-test")
        .version("1.0.0")
        .security_provider_endpoint("https://security.example.com")
        .expect("valid URL")
        .build()
        .expect("build");

    assert!(config.security.security_endpoint.is_some());
}

#[test]
fn test_builder_beardog_endpoint_invalid_url() {
    let result = ConfigBuilder::new().security_provider_endpoint("not-a-url");
    assert!(result.is_err());
}

#[test]
#[expect(
    deprecated,
    reason = "tests exercise beardog_endpoint_optional legacy alias"
)]
fn test_builder_beardog_endpoint_optional_valid() {
    let config = ConfigBuilder::new()
        .name("opt-test")
        .version("1.0.0")
        .beardog_endpoint_optional(Some("https://security.example.com".into()))
        .build_unchecked();

    assert!(config.security.security_endpoint.is_some());
}

#[test]
#[expect(
    deprecated,
    reason = "tests exercise beardog_endpoint_optional legacy alias"
)]
fn test_builder_beardog_endpoint_optional_invalid_ignored() {
    let config = ConfigBuilder::new()
        .beardog_endpoint_optional(Some("not-a-url".into()))
        .build_unchecked();

    assert!(config.security.security_endpoint.is_none());
}

#[test]
#[expect(
    deprecated,
    reason = "tests exercise beardog_endpoint_optional legacy alias"
)]
fn test_builder_beardog_endpoint_optional_none() {
    let config = ConfigBuilder::new()
        .beardog_endpoint_optional(None)
        .build_unchecked();

    assert!(config.security.security_endpoint.is_none());
}

#[test]
fn test_builder_auth_methods() {
    let config = ConfigBuilder::new()
        .security_provider_auth("my-service")
        .build_unchecked();
    assert!(matches!(
        config.security.auth_method,
        AuthMethod::SecurityProvider { .. }
    ));
    assert!(matches!(
        config.security.credential_storage,
        CredentialStorage::SecurityProvider
    ));

    let config = ConfigBuilder::new()
        .token_auth(PathBuf::from("/tmp/token"))
        .build_unchecked();
    assert!(matches!(
        config.security.auth_method,
        AuthMethod::Token { .. }
    ));

    let config = ConfigBuilder::new()
        .cert_auth(PathBuf::from("/tmp/cert"), PathBuf::from("/tmp/key"))
        .build_unchecked();
    assert!(matches!(
        config.security.auth_method,
        AuthMethod::Certificate { .. }
    ));
}

#[test]
fn test_builder_encryption_settings() {
    let config = ConfigBuilder::new()
        .enable_inter_primal_encryption()
        .enable_at_rest_encryption()
        .encryption_algorithm(EncryptionAlgorithm::Aes256Gcm)
        .build_unchecked();

    assert!(config.security.encryption.enable_inter_primal);
    assert!(config.security.encryption.enable_at_rest);
    assert!(matches!(
        config.security.encryption.algorithm,
        EncryptionAlgorithm::Aes256Gcm
    ));
}

#[test]
fn test_builder_fallback_settings() {
    let config = ConfigBuilder::new()
        .enable_local_fallback()
        .fallback_timeout(120)
        .build_unchecked();

    assert!(config.security.fallback.enable_local_fallback);
    assert_eq!(config.security.fallback.fallback_timeout, 120);

    let config = ConfigBuilder::new()
        .enable_local_fallback()
        .disable_fallback()
        .build_unchecked();

    assert!(!config.security.fallback.enable_local_fallback);
}

#[test]
fn test_builder_orchestration_settings() {
    let config = ConfigBuilder::new()
        .enable_orchestration()
        .orchestration_mode(OrchestrationMode::Managed)
        .health_check(true, 30, 10, "/health".to_string())
        .build_unchecked();

    assert!(config.orchestration.enabled);
    assert!(matches!(
        config.orchestration.mode,
        OrchestrationMode::Managed
    ));
    assert!(config.orchestration.health_check.enabled);
    assert_eq!(config.orchestration.health_check.interval, 30);
    assert_eq!(config.orchestration.health_check.timeout, 10);
    assert_eq!(config.orchestration.health_check.endpoint, "/health");
}

#[test]
fn test_builder_service_discovery() {
    let config = ConfigBuilder::new()
        .enable_service_discovery(
            ServiceDiscoveryMethod::Dns {
                domain: "example.com".to_string(),
            },
            300,
        )
        .build_unchecked();

    assert!(config.orchestration.service_discovery.enabled);
    assert_eq!(config.orchestration.service_discovery.ttl, 300);
}

#[test]
fn test_builder_discovery_endpoint_valid() {
    let config = ConfigBuilder::new()
        .discovery_endpoint("https://discovery.example.com")
        .expect("valid URL")
        .build_unchecked();

    assert!(config.orchestration.discovery_endpoint.is_some());
}

#[test]
fn test_builder_discovery_endpoint_invalid() {
    let result = ConfigBuilder::new().discovery_endpoint("not-valid");
    assert!(result.is_err());
}

#[test]
fn test_builder_logging_settings() {
    let config = ConfigBuilder::new()
        .log_level(LogLevel::Warn)
        .log_format(LogFormat::Json)
        .add_log_output(LogOutput::File {
            path: PathBuf::from("/tmp/log"),
        })
        .enable_structured_logging()
        .enable_tracing()
        .build_unchecked();

    assert_eq!(config.logging.level, LogLevel::Warn);
    assert_eq!(config.logging.format, LogFormat::Json);
    assert!(!config.logging.outputs.is_empty());
    assert!(config.logging.structured);
    assert!(config.logging.tracing);
}

#[test]
fn test_builder_environment_settings() {
    let config = ConfigBuilder::new()
        .environment("staging")
        .add_env_var("DB_HOST", "localhost")
        .add_feature("feature_x", true)
        .add_feature("feature_y", false)
        .build_unchecked();

    assert_eq!(config.environment.name, "staging");
    assert_eq!(
        config.environment.variables.get("DB_HOST"),
        Some(&"localhost".to_string())
    );
    assert_eq!(config.environment.features.get("feature_x"), Some(&true));
    assert_eq!(config.environment.features.get("feature_y"), Some(&false));
}

#[test]
fn test_builder_network_settings() {
    let config = ConfigBuilder::new()
        .bind_address("0.0.0.0")
        .public_address("example.com")
        .connect_timeout(30)
        .request_timeout(60)
        .max_connections(500)
        .rate_limit(100.0)
        .build_unchecked();

    assert_eq!(config.network.bind_address, "0.0.0.0");
    assert_eq!(
        config.network.public_address.as_deref(),
        Some("example.com")
    );
    assert_eq!(config.network.timeouts.connect, 30);
    assert_eq!(config.network.timeouts.request, 60);
    assert_eq!(config.network.limits.max_connections, 500);
    assert_eq!(config.network.limits.rate_limit, Some(100.0));
}

#[test]
fn test_builder_default_equals_new() {
    let d = ConfigBuilder::default().build_unchecked();
    let n = ConfigBuilder::new().build_unchecked();
    assert_eq!(d.info.name, n.info.name);
    assert_eq!(d.network.port, n.network.port);
}

#[test]
fn test_builder_enable_audit_logging() {
    let config = ConfigBuilder::new()
        .enable_audit_logging()
        .build_unchecked();
    assert!(config.security.audit_logging);
}
