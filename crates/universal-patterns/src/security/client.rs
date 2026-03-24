// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal security client
//!
//! This module provides the main client interface for accessing security
//! services with automatic fallback capabilities.

use async_trait::async_trait;
use std::sync::Arc;

use crate::config::SecurityConfig;
use crate::traits::{AuthResult, Credentials, Principal};

use super::context::{SecurityContext, SecurityHealth};
use super::errors::SecurityError;
use super::providers::{BeardogSecurityProvider, LocalSecurityProvider};
use super::traits::UniversalSecurityProvider;

/// Universal security client for all primals
///
/// This client provides a unified interface to security services with automatic
/// fallback capabilities. It uses Beardog as the primary provider and can fall
/// back to a local provider if configured.
///
/// # Examples
///
/// ```rust,no_run
/// use universal_patterns::config::PrimalConfig;
/// use universal_patterns::security::UniversalSecurityClient;
/// use universal_patterns::traits::Credentials;
/// use universal_patterns::UniversalSecurityProvider;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = PrimalConfig::default().security.clone();
///     let client = UniversalSecurityClient::new(config).await?;
///
///     let credentials = Credentials::Test {
///         service_id: "test-service".to_string(),
///     };
///
///     let auth_result = client.authenticate(&credentials).await?;
///     println!("Authenticated user: {}", auth_result.principal.name);
///     Ok(())
/// }
/// ```
pub struct UniversalSecurityClient {
    /// Primary security provider (usually Beardog)
    primary: Arc<dyn UniversalSecurityProvider>,
    /// Fallback security provider (usually local)
    fallback: Option<Arc<dyn UniversalSecurityProvider>>,
    /// Security configuration
    config: SecurityConfig,
}

impl UniversalSecurityClient {
    /// Create a new universal security client
    ///
    /// This method creates a new client with the given configuration.
    /// It sets up the primary provider (Beardog) and optionally a fallback
    /// provider (local) based on the configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The security configuration to use
    ///
    /// # Returns
    ///
    /// Returns a new `UniversalSecurityClient` instance or an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use universal_patterns::config::PrimalConfig;
    /// use universal_patterns::security::UniversalSecurityClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = PrimalConfig::default().security.clone();
    ///     let _client = UniversalSecurityClient::new(config).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(config: SecurityConfig) -> Result<Self, SecurityError> {
        use super::providers::SecurityServiceConfig;

        // Convert SecurityConfig to SecurityServiceConfig for providers
        let service_config = SecurityServiceConfig {
            service_id: "beardog-security".to_string(),
            endpoint: config.beardog_endpoint.as_ref().map(|url| url.to_string()),
            timeout_seconds: Some(30),
            max_retries: Some(3),
            auth_config: None,
        };

        // Create primary Beardog provider
        let primary_provider = BeardogSecurityProvider::new(service_config.clone()).await?;
        let primary = Arc::new(primary_provider) as Arc<dyn UniversalSecurityProvider>;

        // Create fallback provider if enabled
        let fallback = if config.fallback.enable_local_fallback {
            let local_service_config = SecurityServiceConfig {
                service_id: "local-security".to_string(),
                endpoint: None,
                timeout_seconds: Some(30),
                max_retries: Some(3),
                auth_config: None,
            };
            let fallback_provider = LocalSecurityProvider::new(local_service_config).await?;
            Some(Arc::new(fallback_provider) as Arc<dyn UniversalSecurityProvider>)
        } else {
            None
        };

        Ok(Self {
            primary,
            fallback,
            config,
        })
    }

    /// Create a new universal security client with custom providers
    ///
    /// This method allows creating a client with custom primary and fallback
    /// providers for testing or advanced use cases.
    ///
    /// # Arguments
    ///
    /// * `primary` - The primary security provider
    /// * `fallback` - The optional fallback security provider
    /// * `config` - The security configuration
    ///
    /// # Returns
    ///
    /// Returns a new `UniversalSecurityClient` instance.
    pub fn with_providers(
        primary: Arc<dyn UniversalSecurityProvider>,
        fallback: Option<Arc<dyn UniversalSecurityProvider>>,
        config: SecurityConfig,
    ) -> Self {
        Self {
            primary,
            fallback,
            config,
        }
    }

    /// Get security provider with fallback
    ///
    /// This method returns the appropriate security provider to use.
    /// It first checks if the primary provider is healthy, and if not,
    /// falls back to the local provider if available.
    ///
    /// # Returns
    ///
    /// Returns the security provider to use.
    async fn get_provider(&self) -> Arc<dyn UniversalSecurityProvider> {
        // Check if primary is healthy (with configurable timeout)
        let health_check_result = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.fallback.fallback_timeout),
            self.primary.health_check(),
        )
        .await;

        if let Ok(Ok(_)) = health_check_result {
            return self.primary.clone();
        }

        // Fall back to local provider if available and enabled
        #[expect(clippy::collapsible_if, reason = "Readability; intentional structure")]
        if self.config.fallback.enable_local_fallback
            && let Some(fallback) = &self.fallback
        {
            if fallback.health_check().await.is_ok() {
                tracing::warn!("Falling back to local security provider");
                return fallback.clone();
            }
        }

        // Return primary even if unhealthy (will fail gracefully)
        self.primary.clone()
    }

    /// Check if fallback is enabled
    ///
    /// # Returns
    ///
    /// Returns true if fallback is enabled, false otherwise.
    pub fn is_fallback_enabled(&self) -> bool {
        self.config.fallback.enable_local_fallback
    }

    /// Get the current configuration
    ///
    /// # Returns
    ///
    /// Returns a reference to the security configuration.
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Test connectivity to the primary provider
    ///
    /// This method tests if the primary provider is reachable and healthy.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the primary provider is healthy, an error otherwise.
    pub async fn test_primary_connectivity(&self) -> Result<(), SecurityError> {
        self.primary.health_check().await?;
        Ok(())
    }

    /// Test connectivity to the fallback provider
    ///
    /// This method tests if the fallback provider is reachable and healthy.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the fallback provider is healthy, an error otherwise.
    pub async fn test_fallback_connectivity(&self) -> Result<(), SecurityError> {
        if let Some(fallback) = &self.fallback {
            fallback.health_check().await?;
            Ok(())
        } else {
            Err(SecurityError::Configuration(
                "Fallback provider not configured".to_string(),
            ))
        }
    }

    /// Get health status of all providers
    ///
    /// This method returns the health status of both primary and fallback providers.
    ///
    /// # Returns
    ///
    /// Returns a tuple of (primary_health, fallback_health).
    pub async fn get_providers_health(
        &self,
    ) -> (
        Result<SecurityHealth, SecurityError>,
        Option<Result<SecurityHealth, SecurityError>>,
    ) {
        let primary_health = self.primary.health_check().await;
        let fallback_health = if let Some(fallback) = &self.fallback {
            Some(fallback.health_check().await)
        } else {
            None
        };

        (primary_health, fallback_health)
    }
}

#[async_trait]
impl UniversalSecurityProvider for UniversalSecurityClient {
    /// Authenticate credentials
    ///
    /// This method authenticates the provided credentials using the selected
    /// security provider (primary or fallback).
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
        let provider = self.get_provider().await;
        provider.authenticate(credentials).await
    }

    /// Authorize an action
    ///
    /// This method authorizes an action for a principal using the selected
    /// security provider (primary or fallback).
    async fn authorize(
        &self,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError> {
        let provider = self.get_provider().await;
        provider.authorize(principal, action, resource).await
    }

    /// Encrypt data
    ///
    /// This method encrypts data using the selected security provider
    /// (primary or fallback).
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let provider = self.get_provider().await;
        provider.encrypt(data).await
    }

    /// Decrypt data
    ///
    /// This method decrypts data using the selected security provider
    /// (primary or fallback).
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let provider = self.get_provider().await;
        provider.decrypt(encrypted_data).await
    }

    /// Sign data
    ///
    /// This method signs data using the selected security provider
    /// (primary or fallback).
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let provider = self.get_provider().await;
        provider.sign(data).await
    }

    /// Verify signature
    ///
    /// This method verifies a signature using the selected security provider
    /// (primary or fallback).
    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        let provider = self.get_provider().await;
        provider.verify(data, signature).await
    }

    /// Audit log operation
    ///
    /// This method logs an audit event using the selected security provider
    /// (primary or fallback). Only logs if audit logging is enabled in the
    /// configuration.
    async fn audit_log(
        &self,
        operation: &str,
        context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        // Only perform audit logging if enabled in configuration
        if !self.config.audit_logging {
            return Ok(());
        }

        let provider = self.get_provider().await;
        provider.audit_log(operation, context).await
    }

    /// Health check
    ///
    /// This method performs a health check using the selected security provider
    /// (primary or fallback).
    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        let provider = self.get_provider().await;
        provider.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        AuthMethod, CredentialStorage, EncryptionAlgorithm, EncryptionConfig, KeyManagement,
        SecurityFallback,
    };
    // Note: PrincipalType and HashMap reserved for future test extensions
    use url::Url;

    /// Create a test encryption configuration
    fn test_encryption_config() -> EncryptionConfig {
        EncryptionConfig {
            enable_inter_primal: false,
            enable_at_rest: false,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_management: KeyManagement::Environment {
                var_name: "TEST_ENCRYPTION_KEY".to_string(),
            },
        }
    }

    /// Create a test security configuration
    fn test_security_config() -> SecurityConfig {
        SecurityConfig {
            beardog_endpoint: None,
            auth_method: AuthMethod::None,
            credential_storage: CredentialStorage::Memory,
            encryption: test_encryption_config(),
            audit_logging: false,
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 30,
            },
        }
    }

    #[tokio::test]
    async fn test_client_creation() {
        let mut config = test_security_config();
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        // Capability-based endpoint resolution (env: BEARDOG_ENDPOINT or SECURITY_SERVICE_PORT)
        let endpoint_str = std::env::var("BEARDOG_ENDPOINT").unwrap_or_else(|_| {
            universal_constants::deployment::endpoints::security_service_base()
        });
        config.beardog_endpoint =
            Some(Url::parse(&endpoint_str).expect("Failed to parse endpoint URL"));
        config.fallback.enable_local_fallback = true;
        config.audit_logging = true;

        let result = UniversalSecurityClient::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fallback_configuration() {
        let mut config = test_security_config();
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        // Capability-based endpoint resolution (env: BEARDOG_ENDPOINT or SECURITY_SERVICE_PORT)
        let endpoint_str = std::env::var("BEARDOG_ENDPOINT").unwrap_or_else(|_| {
            universal_constants::deployment::endpoints::security_service_base()
        });
        config.beardog_endpoint =
            Some(Url::parse(&endpoint_str).expect("Failed to parse endpoint URL"));
        config.fallback.enable_local_fallback = true;
        config.audit_logging = true;

        let client = UniversalSecurityClient::new(config)
            .await
            .expect("should succeed");
        assert!(client.is_fallback_enabled());
    }

    #[tokio::test]
    async fn test_client_with_custom_providers() {
        let mut config = test_security_config();
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        // Capability-based endpoint resolution (env: BEARDOG_ENDPOINT or SECURITY_SERVICE_PORT)
        let endpoint_str = std::env::var("BEARDOG_ENDPOINT").unwrap_or_else(|_| {
            universal_constants::deployment::endpoints::security_service_base()
        });
        config.beardog_endpoint =
            Some(Url::parse(&endpoint_str).expect("Failed to parse endpoint URL"));
        config.fallback.enable_local_fallback = false;
        config.audit_logging = false;

        let service_config = crate::security::providers::SecurityServiceConfig {
            service_id: "test-service".to_string(),
            endpoint: config.beardog_endpoint.as_ref().map(|u| u.to_string()),
            timeout_seconds: Some(30),
            max_retries: Some(3),
            auth_config: None,
        };

        let primary = Arc::new(
            LocalSecurityProvider::new(service_config)
                .await
                .expect("Failed to create local security provider for test"),
        );
        let client = UniversalSecurityClient::with_providers(primary, None, config);

        assert!(!client.is_fallback_enabled());
    }

    #[tokio::test]
    async fn test_authentication_with_fallback() {
        let mut config = test_security_config();
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        // Capability-based endpoint resolution (env: BEARDOG_ENDPOINT or SECURITY_SERVICE_PORT)
        let endpoint_str = std::env::var("BEARDOG_ENDPOINT").unwrap_or_else(|_| {
            universal_constants::deployment::endpoints::security_service_base()
        });
        config.beardog_endpoint =
            Some(Url::parse(&endpoint_str).expect("Failed to parse endpoint URL"));
        config.fallback.enable_local_fallback = true;
        config.fallback.fallback_timeout = 1; // Short timeout to trigger fallback
        config.audit_logging = false;

        let client = UniversalSecurityClient::new(config)
            .await
            .expect("should succeed");

        let credentials = Credentials::Test {
            service_id: "test-service".to_string(),
        };

        // This should fallback to local provider due to short timeout
        let result = client.authenticate(&credentials).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_provider_health_check() {
        let mut config = test_security_config();
        config.auth_method = AuthMethod::Beardog {
            service_id: "test-service".to_string(),
        };
        // Use port 1 (reserved, will always fail to connect)
        config.beardog_endpoint =
            Some(Url::parse("http://127.0.0.1:1").expect("Failed to parse endpoint URL"));
        config.fallback.enable_local_fallback = true;
        config.fallback.fallback_timeout = 1; // Fast timeout for primary to fail quickly
        config.audit_logging = false;

        let client = UniversalSecurityClient::new(config)
            .await
            .expect("should succeed");
        let (primary_health, fallback_health) = client.get_providers_health().await;

        // Check that we get health responses (may succeed or fail depending on system state)
        // The important thing is that the method returns without panicking
        // Primary may succeed if there's a beardog service running locally
        let _ = primary_health;

        // Fallback should be configured when fallback is enabled
        assert!(fallback_health.is_some(), "Fallback should be configured");

        // Verify fallback returns a health result
        let _ = fallback_health.expect("should succeed");
    }
}
