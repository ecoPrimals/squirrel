//! Security providers implementations
//!
//! This module contains implementations of security providers including
//! Beardog integration, local fallback provider, and legacy integration.

use async_trait::async_trait;
use base64::prelude::*;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use crate::config::{AuthMethod, SecurityConfig, CredentialStorage, EncryptionConfig, EncryptionAlgorithm, KeyManagement, SecurityFallback};
use crate::traits::{AuthResult, Credentials, Principal, PrincipalType};

use super::context::{HealthStatus, SecurityContext, SecurityHealth};
use super::errors::SecurityError;
use super::traits::{SecurityProvider, UniversalSecurityProvider};
use super::types::*;

/// Beardog security provider implementation
///
/// This provider integrates with the Beardog security service for authentication,
/// authorization, cryptographic operations, and audit logging.
///
/// # Examples
///
/// ```no_run
/// use universal_patterns::security::providers::BeardogSecurityProvider;
/// use universal_patterns::config::{SecurityConfig, AuthMethod};
/// use url::Url;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut config = SecurityConfig::default();
/// config.auth_method = AuthMethod::Beardog {
///     service_id: "my-service".to_string(),
/// };
/// config.beardog_endpoint = Some(Url::parse("https://beardog.example.com")?);
///
/// let provider = BeardogSecurityProvider::new(config).await?;
/// # Ok(())
/// # }
/// ```
pub struct BeardogSecurityProvider {
    /// HTTP client for making requests
    client: reqwest::Client,
    /// Beardog endpoint URL
    endpoint: Url,
    /// Service ID for this provider
    service_id: String,
}

impl BeardogSecurityProvider {
    /// Create a new Beardog security provider
    ///
    /// # Arguments
    ///
    /// * `config` - Security configuration containing Beardog settings
    ///
    /// # Returns
    ///
    /// Returns a new `BeardogSecurityProvider` instance or an error.
    pub async fn new(config: SecurityConfig) -> Result<Self, SecurityError> {
        let endpoint = config.beardog_endpoint.ok_or_else(|| {
            SecurityError::Configuration("Beardog endpoint not configured".to_string())
        })?;

        let service_id = match config.auth_method {
            AuthMethod::Beardog { service_id } => service_id,
            _ => {
                return Err(SecurityError::Configuration(
                    "Invalid auth method for Beardog".to_string(),
                ))
            }
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        Ok(Self {
            client,
            endpoint,
            service_id,
        })
    }

    /// Get the service ID
    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    /// Get the endpoint URL
    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }
}

#[async_trait]
impl UniversalSecurityProvider for BeardogSecurityProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
        let url = self
            .endpoint
            .join("/api/v1/auth/authenticate")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let request = AuthRequest {
            service_id: self.service_id.clone(),
            credentials: credentials.clone(),
            timestamp: Utc::now(),
        };

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let auth_result: AuthResult = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;

            // Audit log authentication
            self.audit_log(
                "authenticate",
                &SecurityContext::from_auth_result(&auth_result),
            )
            .await?;

            Ok(auth_result)
        } else {
            Err(SecurityError::Authentication(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    async fn authorize(
        &self,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError> {
        let url = self
            .endpoint
            .join("/api/v1/auth/authorize")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let request = AuthorizationRequest {
            service_id: self.service_id.clone(),
            principal: principal.clone(),
            action: action.to_string(),
            resource: resource.to_string(),
            timestamp: Utc::now(),
        };

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: AuthorizationResult = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;

            // Audit log authorization
            self.audit_log(
                &format!("authorize:{action}"),
                &SecurityContext::from_principal(principal),
            )
            .await?;

            Ok(result.authorized)
        } else {
            Err(SecurityError::Authorization(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let url = self
            .endpoint
            .join("/api/v1/crypto/encrypt")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let request = EncryptionRequest {
            service_id: self.service_id.clone(),
            data: BASE64_STANDARD.encode(data),
        };

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: EncryptionResult = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;

            BASE64_STANDARD
                .decode(result.encrypted_data)
                .map_err(|e| SecurityError::Encryption(e.to_string()))
        } else {
            Err(SecurityError::Encryption(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let url = self
            .endpoint
            .join("/api/v1/crypto/decrypt")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let request = DecryptionRequest {
            service_id: self.service_id.clone(),
            encrypted_data: BASE64_STANDARD.encode(encrypted_data),
        };

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: DecryptionResult = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;

            BASE64_STANDARD
                .decode(result.decrypted_data)
                .map_err(|e| SecurityError::Encryption(e.to_string()))
        } else {
            Err(SecurityError::Encryption(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let url = self
            .endpoint
            .join("/api/v1/crypto/sign")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let request = SigningRequest {
            service_id: self.service_id.clone(),
            data: BASE64_STANDARD.encode(data),
        };

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: SigningResult = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;

            BASE64_STANDARD
                .decode(result.signature)
                .map_err(|e| SecurityError::Encryption(e.to_string()))
        } else {
            Err(SecurityError::Encryption(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        let url = self
            .endpoint
            .join("/api/v1/crypto/verify")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let request = VerificationRequest {
            service_id: self.service_id.clone(),
            data: BASE64_STANDARD.encode(data),
            signature: BASE64_STANDARD.encode(signature),
        };

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: VerificationResult = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;

            Ok(result.valid)
        } else {
            Err(SecurityError::Encryption(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    async fn audit_log(
        &self,
        operation: &str,
        context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        let url = self
            .endpoint
            .join("/api/v1/audit/log")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let request = AuditLogRequest {
            service_id: self.service_id.clone(),
            operation: operation.to_string(),
            context: context.clone(),
            timestamp: Utc::now(),
        };

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if !response.status().is_success() {
            tracing::warn!("Audit log failed: HTTP {}", response.status());
        }

        Ok(())
    }

    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        let url = self
            .endpoint
            .join("/api/v1/health")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let start_time = std::time::Instant::now();

        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        let latency = start_time.elapsed();

        if response.status().is_success() {
            Ok(SecurityHealth {
                status: HealthStatus::Healthy,
                latency,
                last_check: Utc::now(),
                details: HashMap::new(),
            })
        } else {
            Err(SecurityError::Network(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }
}

/// Local security provider for fallback
///
/// This provider implements basic security operations locally without
/// external dependencies. It's used as a fallback when the primary
/// provider is unavailable.
///
/// # Examples
///
/// ```no_run
/// use universal_patterns::security::providers::LocalSecurityProvider;
/// use universal_patterns::config::SecurityConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = SecurityConfig::default();
/// let provider = LocalSecurityProvider::new(config).await?;
/// # Ok(())
/// # }
/// ```
pub struct LocalSecurityProvider {
    /// Configuration for the local provider
    config: SecurityConfig,
}

impl LocalSecurityProvider {
    /// Create a new local security provider
    ///
    /// # Arguments
    ///
    /// * `config` - Security configuration
    ///
    /// # Returns
    ///
    /// Returns a new `LocalSecurityProvider` instance or an error.
    pub async fn new(config: SecurityConfig) -> Result<Self, SecurityError> {
        Ok(Self { config })
    }

    /// Get the configuration
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }
}

#[async_trait]
impl UniversalSecurityProvider for LocalSecurityProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
        // Simple local authentication for fallback
        match credentials {
            Credentials::Test { .. } => Ok(AuthResult {
                principal: Principal {
                    id: "test-user".to_string(),
                    name: "Test User".to_string(),
                    principal_type: PrincipalType::User,
                    roles: vec!["user".to_string()],
                    permissions: vec!["read".to_string()],
                    metadata: HashMap::new(),
                },
                token: "local-fallback-token".to_string(),
                expires_at: Utc::now() + chrono::Duration::hours(1),
                permissions: vec!["read".to_string()],
                metadata: HashMap::new(),
            }),
            _ => Err(SecurityError::Authentication(
                "Local auth only supports test credentials".to_string(),
            )),
        }
    }

    async fn authorize(
        &self,
        _principal: &Principal,
        _action: &str,
        _resource: &str,
    ) -> Result<bool, SecurityError> {
        // Allow all operations in local fallback mode
        Ok(true)
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Simple XOR encryption for local fallback
        let key = b"local-fallback-key";
        let encrypted: Vec<u8> = data
            .iter()
            .enumerate()
            .map(|(i, byte)| byte ^ key[i % key.len()])
            .collect();
        Ok(encrypted)
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Simple XOR decryption for local fallback
        let key = b"local-fallback-key";
        let decrypted: Vec<u8> = encrypted_data
            .iter()
            .enumerate()
            .map(|(i, byte)| byte ^ key[i % key.len()])
            .collect();
        Ok(decrypted)
    }

    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // Simple hash-based signing for local fallback
        use sha256::digest;
        let hash = digest(data);
        Ok(hash.into_bytes())
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        // Simple hash-based verification for local fallback
        use sha256::digest;
        let hash = digest(data);
        Ok(hash.as_bytes() == signature)
    }

    async fn audit_log(
        &self,
        operation: &str,
        context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        // Only log if audit logging is enabled in configuration
        if self.config.audit_logging {
            // Log to local system for fallback
            tracing::info!("Audit: {} by {}", operation, context.principal.name);
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<SecurityHealth, SecurityError> {
        Ok(SecurityHealth {
            status: HealthStatus::Healthy,
            latency: Duration::from_millis(1),
            last_check: Utc::now(),
            details: HashMap::new(),
        })
    }
}

/// Beardog integration for security (legacy interface)
///
/// This provides a legacy interface for Beardog integration that implements
/// the basic SecurityProvider trait rather than the extended UniversalSecurityProvider.
///
/// # Examples
///
/// ```no_run
/// use universal_patterns::security::providers::BeardogIntegration;
/// use url::Url;
///
/// let endpoint = Url::parse("https://beardog.example.com").unwrap();
/// let integration = BeardogIntegration::new(endpoint, "my-service".to_string());
/// ```
#[derive(Debug)]
pub struct BeardogIntegration {
    /// Beardog endpoint URL
    endpoint: Url,
    /// HTTP client for making requests
    client: reqwest::Client,
    /// Service ID for this integration
    service_id: String,
}

impl BeardogIntegration {
    /// Create a new Beardog integration
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The Beardog endpoint URL
    /// * `service_id` - The service ID for this integration
    ///
    /// # Returns
    ///
    /// Returns a new `BeardogIntegration` instance.
    pub fn new(endpoint: Url, service_id: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
            service_id,
        }
    }

    /// Get the service ID
    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    /// Get the endpoint
    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }
}

#[async_trait]
impl SecurityProvider for BeardogIntegration {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
        let url = self
            .endpoint
            .join("/auth/authenticate")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let response = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "service_id": self.service_id,
                "credentials": credentials
            }))
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let auth_result: AuthResult = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            Ok(auth_result)
        } else {
            Err(SecurityError::Authentication(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }

    async fn authorize(
        &self,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError> {
        let url = self
            .endpoint
            .join("/auth/authorize")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let response = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "service_id": self.service_id,
                "principal": principal,
                "action": action,
                "resource": resource
            }))
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            Ok(result
                .get("authorized")
                .and_then(|v| v.as_bool())
                .unwrap_or(false))
        } else {
            Err(SecurityError::Authorization(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let url = self
            .endpoint
            .join("/crypto/encrypt")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let response = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "service_id": self.service_id,
                "data": BASE64_STANDARD.encode(data)
            }))
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            if let Some(encrypted_data) = result.get("encrypted_data").and_then(|v| v.as_str()) {
                BASE64_STANDARD
                    .decode(encrypted_data)
                    .map_err(|e| SecurityError::Encryption(e.to_string()))
            } else {
                Err(SecurityError::Encryption(
                    "Invalid response format".to_string(),
                ))
            }
        } else {
            Err(SecurityError::Encryption(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let url = self
            .endpoint
            .join("/crypto/decrypt")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let response = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "service_id": self.service_id,
                "encrypted_data": BASE64_STANDARD.encode(encrypted_data)
            }))
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            if let Some(decrypted_data) = result.get("decrypted_data").and_then(|v| v.as_str()) {
                BASE64_STANDARD
                    .decode(decrypted_data)
                    .map_err(|e| SecurityError::Encryption(e.to_string()))
            } else {
                Err(SecurityError::Encryption(
                    "Invalid response format".to_string(),
                ))
            }
        } else {
            Err(SecurityError::Encryption(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }

    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let url = self
            .endpoint
            .join("/crypto/sign")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let response = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "service_id": self.service_id,
                "data": BASE64_STANDARD.encode(data)
            }))
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            if let Some(signature) = result.get("signature").and_then(|v| v.as_str()) {
                BASE64_STANDARD
                    .decode(signature)
                    .map_err(|e| SecurityError::Encryption(e.to_string()))
            } else {
                Err(SecurityError::Encryption(
                    "Invalid response format".to_string(),
                ))
            }
        } else {
            Err(SecurityError::Encryption(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        let url = self
            .endpoint
            .join("/crypto/verify")
            .map_err(|e| SecurityError::Configuration(e.to_string()))?;

        let response = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "service_id": self.service_id,
                "data": BASE64_STANDARD.encode(data),
                "signature": BASE64_STANDARD.encode(signature)
            }))
            .send()
            .await
            .map_err(|e| SecurityError::Network(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| SecurityError::Network(e.to_string()))?;
            Ok(result
                .get("valid")
                .and_then(|v| v.as_bool())
                .unwrap_or(false))
        } else {
            Err(SecurityError::Encryption(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AuthMethod;
    use crate::traits::PrincipalType;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_beardog_provider_creation() {
        let config = SecurityConfig {
            auth_method: AuthMethod::Beardog {
                service_id: "test-service".to_string(),
            },
            beardog_endpoint: Some(Url::parse("http://localhost:8443").unwrap()),
            credential_storage: CredentialStorage::Beardog,
            encryption: EncryptionConfig {
                enable_inter_primal: true,
                enable_at_rest: true,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_management: KeyManagement::Beardog,
            },
            fallback: SecurityFallback {
                enable_local_fallback: false,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 5,
            },
            audit_logging: false,
        };

        let result = BeardogSecurityProvider::new(config).await;
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.service_id(), "test-service");
    }

    #[tokio::test]
    async fn test_local_provider_creation() {
        let config = SecurityConfig {
            auth_method: AuthMethod::None,
            beardog_endpoint: None,
            credential_storage: CredentialStorage::Memory,
            encryption: EncryptionConfig {
                enable_inter_primal: false,
                enable_at_rest: false,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_management: KeyManagement::Environment {
                    var_name: "TEST_KEY".to_string(),
                },
            },
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 5,
            },
            audit_logging: false,
        };

        let result = LocalSecurityProvider::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_local_provider_authentication() {
        let config = SecurityConfig {
            auth_method: AuthMethod::None,
            beardog_endpoint: None,
            credential_storage: CredentialStorage::Memory,
            encryption: EncryptionConfig {
                enable_inter_primal: false,
                enable_at_rest: false,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_management: KeyManagement::Environment {
                    var_name: "TEST_KEY".to_string(),
                },
            },
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 5,
            },
            audit_logging: false,
        };

        let provider = LocalSecurityProvider::new(config).await.unwrap();

        let credentials = Credentials::Test {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        let result = provider.authenticate(&credentials).await;
        assert!(result.is_ok());

        let auth_result = result.unwrap();
        assert_eq!(auth_result.principal.id, "test-user");
        assert_eq!(auth_result.principal.name, "Test User");
    }

    #[tokio::test]
    async fn test_local_provider_encryption() {
        let config = SecurityConfig {
            auth_method: AuthMethod::None,
            beardog_endpoint: None,
            credential_storage: CredentialStorage::Memory,
            encryption: EncryptionConfig {
                enable_inter_primal: false,
                enable_at_rest: false,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_management: KeyManagement::Environment {
                    var_name: "TEST_KEY".to_string(),
                },
            },
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 5,
            },
            audit_logging: false,
        };

        let provider = LocalSecurityProvider::new(config).await.unwrap();

        let data = b"test data";
        let encrypted = provider.encrypt(data).await.unwrap();
        let decrypted = provider.decrypt(&encrypted).await.unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_local_provider_signing() {
        let config = SecurityConfig {
            auth_method: AuthMethod::None,
            beardog_endpoint: None,
            credential_storage: CredentialStorage::Memory,
            encryption: EncryptionConfig {
                enable_inter_primal: false,
                enable_at_rest: false,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_management: KeyManagement::Environment {
                    var_name: "TEST_KEY".to_string(),
                },
            },
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 5,
            },
            audit_logging: false,
        };

        let provider = LocalSecurityProvider::new(config).await.unwrap();

        let data = b"test data";
        let signature = provider.sign(data).await.unwrap();
        let valid = provider.verify(data, &signature).await.unwrap();

        assert!(valid);
    }

    #[tokio::test]
    async fn test_local_provider_health_check() {
        let config = SecurityConfig {
            auth_method: AuthMethod::None,
            beardog_endpoint: None,
            credential_storage: CredentialStorage::Memory,
            encryption: EncryptionConfig {
                enable_inter_primal: false,
                enable_at_rest: false,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_management: KeyManagement::Environment {
                    var_name: "TEST_KEY".to_string(),
                },
            },
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 5,
            },
            audit_logging: false,
        };

        let provider = LocalSecurityProvider::new(config).await.unwrap();
        let health = provider.health_check().await.unwrap();

        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.latency.as_millis() < 10);
    }

    #[tokio::test]
    async fn test_beardog_integration_creation() {
        let endpoint = Url::parse("http://localhost:8443").unwrap();
        let integration = BeardogIntegration::new(endpoint.clone(), "test-service".to_string());

        assert_eq!(integration.service_id(), "test-service");
        assert_eq!(integration.endpoint(), &endpoint);
    }

    #[tokio::test]
    async fn test_local_provider_authorization() {
        let config = SecurityConfig {
            auth_method: AuthMethod::None,
            beardog_endpoint: None,
            credential_storage: CredentialStorage::Memory,
            encryption: EncryptionConfig {
                enable_inter_primal: false,
                enable_at_rest: false,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_management: KeyManagement::Environment {
                    var_name: "TEST_KEY".to_string(),
                },
            },
            fallback: SecurityFallback {
                enable_local_fallback: true,
                local_auth_method: AuthMethod::None,
                fallback_timeout: 5,
            },
            audit_logging: false,
        };

        let provider = LocalSecurityProvider::new(config).await.unwrap();

        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let authorized = provider
            .authorize(&principal, "read", "resource")
            .await
            .unwrap();
        assert!(authorized); // Local provider allows all operations
    }
}
