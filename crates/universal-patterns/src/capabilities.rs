// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Capability-Based Trait Definitions for Cross-Primal Integration
//!
//! This module defines capability traits that enable **runtime discovery and integration
//! WITHOUT compile-time dependencies** on specific primals. This is a core architectural
//! pattern that ensures loose coupling and independent evolution of primal services.
//!
//! ## 🎯 Core Architecture Principle: Primal Self-Knowledge
//!
//! **Each primal knows ONLY itself.** No primal has compile-time knowledge of other primals.
//! Instead, integration happens through a three-stage process:
//!
//! 1. **Capability Discovery** - Find services by capability, not by primal name
//! 2. **Runtime Binding** - Connect to services dynamically at runtime via Songbird
//! 3. **Standard Protocols** - HTTP/gRPC communication, not direct function calls
//!
//! This pattern enables:
//! - ✅ Zero vendor lock-in (any provider can implement a capability)
//! - ✅ Independent deployment and scaling of primals
//! - ✅ Runtime failover and load balancing
//! - ✅ Clean separation of concerns
//! - ✅ Capability-based security (fine-grained access control)
//!
//! ## 📖 Complete Usage Examples
//!
//! ### Example 1: Authentication (Basic Pattern)
//!
//! ```ignore,no_run
//! use universal_patterns::capabilities::AuthenticationCapability;
//! use squirrel::ecosystem::EcosystemManager;
//!
//! # async fn example(ecosystem: &EcosystemManager) -> Result<(), Box<dyn std::error::Error>> {
//! // ❌ BAD: Direct dependency (compile-time coupling)
//! // use beardog::AuthenticationService;
//! // let result = beardog.authenticate(credentials)?;
//!
//! // ✅ GOOD: Capability-based discovery
//! let auth_providers = ecosystem
//!     .discover_capability("authentication")
//!     .await?;
//!
//! // Select provider (could be BearDog, or any other auth service)
//! let auth_service = auth_providers
//!     .first()
//!     .ok_or("No authentication service available")?;
//!
//! // Authenticate via standard protocol
//! let token = ecosystem
//!     .call_capability::<AuthenticationCapability>(
//!         auth_service,
//!         |auth| async move {
//!             auth.authenticate("user@example.com", "password").await
//!         }
//!     )
//!     .await?;
//!
//! println!("Authenticated! Token: {}", token);
//! # Ok(())
//! # }
//! ```
//!
//! ### Example 2: GPU Computation (Advanced Pattern)
//!
//! ```ignore,no_run
//! use universal_patterns::capabilities::ComputeCapability;
//! use squirrel::ecosystem::EcosystemManager;
//!
//! # async fn example(ecosystem: &EcosystemManager) -> Result<(), Box<dyn std::error::Error>> {
//! // Discover GPU compute providers
//! let gpu_providers = ecosystem
//!     .discover_capability_with_requirements("compute.gpu", |provider| {
//!         provider.has_gpu() && provider.vram_gb() >= 8
//!     })
//!     .await?;
//!
//! // Select optimal provider (health, latency, cost)
//! let compute_service = ecosystem
//!     .select_optimal_provider(gpu_providers, |p| {
//!         // Scoring function
//!         p.health_score() * 0.5 +
//!         (1.0 / p.latency_ms()) * 0.3 +
//!         (1.0 / p.cost_per_hour()) * 0.2
//!     })
//!     .await?;
//!
//! // Execute compute task
//! let result = ecosystem
//!     .call_capability::<ComputeCapability>(
//!         &compute_service,
//!         |compute| async move {
//!             compute.execute_task("matrix_multiply", &task_data).await
//!         }
//!     )
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Example 3: Graceful Degradation (Resilience Pattern)
//!
//! ```ignore,no_run
//! use universal_patterns::capabilities::{StorageCapability, CacheCapability};
//! use squirrel::ecosystem::EcosystemManager;
//!
//! # async fn example(ecosystem: &EcosystemManager, key: &str) -> Result<String, Box<dyn std::error::Error>> {
//! // Try cache first (fast)
//! if let Ok(cache_providers) = ecosystem.discover_capability("cache").await {
//!     if let Some(cache) = cache_providers.first() {
//!         if let Ok(value) = ecosystem.call_capability::<CacheCapability>(
//!             cache,
//!             |cache| async move { cache.get(key).await }
//!         ).await {
//!             return Ok(value);
//!         }
//!     }
//! }
//!
//! // Fallback to persistent storage (slower but reliable)
//! let storage_providers = ecosystem
//!     .discover_capability("storage")
//!     .await?;
//!
//! let storage = storage_providers
//!     .first()
//!     .ok_or("No storage service available")?;
//!
//! let value = ecosystem
//!     .call_capability::<StorageCapability>(
//!         storage,
//!         |store| async move { store.read(key).await }
//!     )
//!     .await?;
//!
//! Ok(value)
//! # }
//! ```
//!
//! ## 🔒 Security Model
//!
//! Capabilities are protected through multiple layers:
//!
//! 1. **Discovery Security**: Only authorized services can register capabilities
//! 2. **Transport Security**: All communication via TLS (HTTP/gRPC)
//! 3. **Token-Based Auth**: OAuth2/JWT tokens for capability invocation
//! 4. **RBAC Integration**: Role-based access control via BearDog
//!
//! ## 🎨 Available Capabilities
//!
//! | Capability | Provider(s) | Description |
//! |------------|-------------|-------------|
//! | `authentication` | BearDog | User authentication, token validation |
//! | `authorization` | BearDog | Permission checks, RBAC |
//! | `compute.cpu` | ToadStool | CPU-bound computation |
//! | `compute.gpu` | ToadStool | GPU-accelerated computation |
//! | `storage.object` | NestGate | Object storage (S3-compatible) |
//! | `storage.block` | NestGate | Block storage (volume mounting) |
//! | `mesh.routing` | Songbird | Service mesh routing |
//! | `mesh.discovery` | Songbird | Service discovery |
//! | `ai.inference` | Squirrel | AI model inference |
//! | `ai.embedding` | Squirrel | Text/image embeddings |
//!
//! ## 🚀 Performance Considerations
//!
//! - **Discovery Caching**: Capability lookups are cached (TTL: 5 minutes)
//! - **Connection Pooling**: HTTP/gRPC connections are pooled and reused
//! - **Circuit Breakers**: Failed providers are temporarily excluded
//! - **Load Balancing**: Round-robin by default, weighted by health score
//!
//! ## 📐 Design Patterns
//!
//! This module implements several key patterns:
//!
//! - **Strategy Pattern**: Capabilities as interchangeable strategies
//! - **Service Locator**: Runtime discovery of service implementations  
//! - **Dependency Injection**: Services injected at runtime, not compile-time
//! - **Circuit Breaker**: Automatic failure detection and recovery
//!
//! ## 🔗 Related Modules
//!
//! - [`crate::ecosystem`] - Service discovery and ecosystem management
//! - [`crate::federation`] - Cross-primal federation and networking
//! - [`crate::security`] - Security hardening and RBAC integration

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Type alias for capability errors
pub type CapabilityError = anyhow::Error;

/// Authentication capability - Secure credential validation
///
/// Provides authentication services WITHOUT knowing which primal implements it.
/// Could be BearDog, or any other authentication provider.
///
/// # Example
///
/// ```ignore,no_run
/// use squirrel::capabilities::AuthenticationCapability;
///
/// # async fn example(auth: &dyn AuthenticationCapability) -> Result<(), Box<dyn std::error::Error>> {
/// // Authenticate user
/// let token = auth.authenticate("user@example.com", "password").await?;
///
/// // Validate token
/// let claims = auth.validate_token(&token).await?;
/// assert_eq!(claims.user_id, "user@example.com");
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait AuthenticationCapability: Send + Sync {
    /// Authenticate credentials and return an access token
    ///
    /// # Arguments
    ///
    /// * `user_id` - User identifier (email, username, etc.)
    /// * `credentials` - Authentication credentials (password, API key, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Access token on successful authentication
    /// * `Err(CapabilityError)` - Authentication failed
    ///
    /// # Errors
    ///
    /// * `CapabilityError::AuthenticationFailed` - Invalid credentials
    /// * `CapabilityError::ServiceUnavailable` - Authentication service unavailable
    /// * `CapabilityError::NetworkError` - Network communication failed
    async fn authenticate(
        &self,
        user_id: &str,
        credentials: &str,
    ) -> Result<String, CapabilityError>;

    /// Validate an access token and return claims
    ///
    /// # Arguments
    ///
    /// * `token` - Access token to validate
    ///
    /// # Returns
    ///
    /// * `Ok(TokenClaims)` - Valid token with user information
    /// * `Err(CapabilityError)` - Invalid or expired token
    ///
    /// # Errors
    ///
    /// * `CapabilityError::TokenExpired` - Token has expired
    /// * `CapabilityError::TokenInvalid` - Token signature invalid
    /// * `CapabilityError::ServiceUnavailable` - Validation service unavailable
    async fn validate_token(&self, token: &str) -> Result<TokenClaims, CapabilityError>;

    /// Refresh an access token before expiration
    ///
    /// # Arguments
    ///
    /// * `token` - Current access token
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - New access token
    /// * `Err(CapabilityError)` - Refresh failed
    async fn refresh_token(&self, token: &str) -> Result<String, CapabilityError>;

    /// Revoke an access token (logout)
    ///
    /// # Arguments
    ///
    /// * `token` - Access token to revoke
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Token successfully revoked
    /// * `Err(CapabilityError)` - Revocation failed
    async fn revoke_token(&self, token: &str) -> Result<(), CapabilityError>;
}

/// Token claims returned from authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// User identifier
    pub user_id: String,
    /// Token expiration timestamp (Unix epoch)
    pub expires_at: i64,
    /// Granted permissions/scopes
    pub permissions: Vec<String>,
    /// Issuing authority
    pub issuer: String,
}

/// GPU inference capability - Model loading and execution
///
/// Provides GPU-accelerated inference WITHOUT knowing which primal implements it.
/// Could be ToadStool, or any other GPU provider.
///
/// # Example
///
/// ```ignore,no_run
/// use squirrel::capabilities::GpuInferenceCapability;
///
/// # async fn example(gpu: &dyn GpuInferenceCapability) -> Result<(), Box<dyn std::error::Error>> {
/// // Load model
/// let model_handle = gpu.load_model("llama2:7b").await?;
///
/// // Run inference
/// let result = gpu.infer(&model_handle, "Hello, world!").await?;
/// println!("Response: {}", result.text);
///
/// // Unload when done
/// gpu.unload_model(&model_handle).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait GpuInferenceCapability: Send + Sync {
    /// Load a model onto GPU(s)
    ///
    /// # Arguments
    ///
    /// * `model_id` - Model identifier (e.g., "llama2:7b", "gpt-3.5-turbo")
    ///
    /// # Returns
    ///
    /// * `Ok(ModelHandle)` - Handle to loaded model
    /// * `Err(CapabilityError)` - Model loading failed
    ///
    /// # Errors
    ///
    /// * `CapabilityError::ModelNotFound` - Model doesn't exist
    /// * `CapabilityError::InsufficientVRAM` - Not enough GPU memory
    /// * `CapabilityError::ServiceUnavailable` - GPU service unavailable
    async fn load_model(&self, model_id: &str) -> Result<ModelHandle, CapabilityError>;

    /// Run inference on a loaded model
    ///
    /// # Arguments
    ///
    /// * `handle` - Model handle from `load_model`
    /// * `prompt` - Input prompt for generation
    ///
    /// # Returns
    ///
    /// * `Ok(InferenceResult)` - Generated output
    /// * `Err(CapabilityError)` - Inference failed
    async fn infer(
        &self,
        handle: &ModelHandle,
        prompt: &str,
    ) -> Result<InferenceResult, CapabilityError>;

    /// Unload a model from GPU(s)
    ///
    /// # Arguments
    ///
    /// * `handle` - Model handle to unload
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Model successfully unloaded
    /// * `Err(CapabilityError)` - Unload failed
    async fn unload_model(&self, handle: &ModelHandle) -> Result<(), CapabilityError>;

    /// Query available VRAM across all GPUs
    ///
    /// # Returns
    ///
    /// * `Ok(VramInfo)` - VRAM availability information
    /// * `Err(CapabilityError)` - Query failed
    async fn query_vram(&self) -> Result<VramInfo, CapabilityError>;
}

/// Handle to a loaded model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHandle {
    /// Unique model identifier
    pub id: String,
    /// Model metadata
    pub metadata: ModelMetadata,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model name
    pub name: String,
    /// Model size in GB
    pub size_gb: f32,
    /// Number of parameters
    pub parameters: u64,
    /// Supported operations
    pub capabilities: Vec<String>,
}

/// Inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    /// Generated text
    pub text: String,
    /// Tokens generated
    pub tokens: usize,
    /// Generation time in milliseconds
    pub latency_ms: u64,
}

/// VRAM information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VramInfo {
    /// Total VRAM in GB
    pub total_gb: f32,
    /// Available VRAM in GB
    pub available_gb: f32,
    /// Per-GPU breakdown
    pub gpus: Vec<GpuVramInfo>,
}

/// Per-GPU VRAM information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuVramInfo {
    /// GPU index
    pub index: u32,
    /// GPU model
    pub model: String,
    /// Total VRAM in GB
    pub total_gb: f32,
    /// Available VRAM in GB
    pub available_gb: f32,
}

/// Service mesh capability - Cross-primal routing
///
/// Provides service mesh coordination WITHOUT knowing which primal implements it.
/// Could be Songbird, or any other mesh provider.
#[async_trait]
pub trait ServiceMeshCapability: Send + Sync {
    /// Register a service with the mesh
    ///
    /// # Arguments
    ///
    /// * `service` - Service registration information
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Registration ID
    /// * `Err(CapabilityError)` - Registration failed
    async fn register_service(
        &self,
        service: &ServiceRegistration,
    ) -> Result<String, CapabilityError>;

    /// Discover services by capability
    ///
    /// # Arguments
    ///
    /// * `capability` - Capability to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ServiceInfo>)` - List of matching services
    /// * `Err(CapabilityError)` - Discovery failed
    async fn discover_services(
        &self,
        capability: &str,
    ) -> Result<Vec<ServiceInfo>, CapabilityError>;

    /// Route a request to a service
    ///
    /// # Arguments
    ///
    /// * `target` - Target service identifier
    /// * `request` - Request payload
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - Response payload
    /// * `Err(CapabilityError)` - Routing failed
    async fn route_request(&self, target: &str, request: &[u8])
        -> Result<Vec<u8>, CapabilityError>;
}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    /// Service name
    pub name: String,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service endpoint
    pub endpoint: String,
    /// Service metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service ID
    pub id: String,
    /// Service name
    pub name: String,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service endpoint
    pub endpoint: String,
    /// Service health status
    pub healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockAuthService;

    #[async_trait]
    impl AuthenticationCapability for MockAuthService {
        async fn authenticate(
            &self,
            _user_id: &str,
            _credentials: &str,
        ) -> Result<String, CapabilityError> {
            Ok("mock-token".to_string())
        }

        async fn validate_token(&self, _token: &str) -> Result<TokenClaims, CapabilityError> {
            Ok(TokenClaims {
                user_id: "test-user".to_string(),
                expires_at: 1234567890,
                permissions: vec!["read".to_string()],
                issuer: "mock".to_string(),
            })
        }

        async fn refresh_token(&self, _token: &str) -> Result<String, CapabilityError> {
            Ok("refreshed-token".to_string())
        }

        async fn revoke_token(&self, _token: &str) -> Result<(), CapabilityError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_auth_capability_trait() {
        let auth: &dyn AuthenticationCapability = &MockAuthService;
        let token = auth.authenticate("user", "pass").await.expect("should succeed");
        assert_eq!(token, "mock-token");

        let claims = auth.validate_token(&token).await.expect("should succeed");
        assert_eq!(claims.user_id, "test-user");
    }

    #[tokio::test]
    async fn test_auth_refresh_and_revoke() {
        let auth: &dyn AuthenticationCapability = &MockAuthService;
        let refreshed = auth.refresh_token("old-token").await.expect("should succeed");
        assert_eq!(refreshed, "refreshed-token");

        assert!(auth.revoke_token("any-token").await.is_ok());
    }

    #[test]
    fn test_token_claims_serde() {
        let claims = TokenClaims {
            user_id: "alice".to_string(),
            expires_at: 9999999999,
            permissions: vec!["admin".to_string(), "read".to_string()],
            issuer: "test-authority".to_string(),
        };
        let json = serde_json::to_string(&claims).expect("should succeed");
        let deser: TokenClaims = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deser.user_id, "alice");
        assert_eq!(deser.permissions.len(), 2);
    }

    #[test]
    fn test_model_handle_serde() {
        let handle = ModelHandle {
            id: "model-123".to_string(),
            metadata: ModelMetadata {
                name: "llama2".to_string(),
                size_gb: 3.5,
                parameters: 7_000_000_000,
                capabilities: vec!["text-generation".to_string()],
            },
        };
        let json = serde_json::to_string(&handle).expect("should succeed");
        let deser: ModelHandle = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deser.id, "model-123");
        assert_eq!(deser.metadata.parameters, 7_000_000_000);
    }

    #[test]
    fn test_inference_result_serde() {
        let result = InferenceResult {
            text: "Hello, world!".to_string(),
            tokens: 3,
            latency_ms: 42,
        };
        let json = serde_json::to_string(&result).expect("should succeed");
        let deser: InferenceResult = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deser.tokens, 3);
        assert_eq!(deser.latency_ms, 42);
    }

    #[test]
    fn test_vram_info_serde() {
        let info = VramInfo {
            total_gb: 24.0,
            available_gb: 16.5,
            gpus: vec![GpuVramInfo {
                index: 0,
                model: "RTX 4090".to_string(),
                total_gb: 24.0,
                available_gb: 16.5,
            }],
        };
        let json = serde_json::to_string(&info).expect("should succeed");
        let deser: VramInfo = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deser.gpus.len(), 1);
        assert!((deser.available_gb - 16.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_service_registration_serde() {
        let reg = ServiceRegistration {
            name: "ai-service".to_string(),
            capabilities: vec!["ai.inference".to_string(), "ai.embedding".to_string()],
            endpoint: "http://localhost:8080".to_string(),
            metadata: {
                let mut m = std::collections::HashMap::new();
                m.insert("version".to_string(), "1.0.0".to_string());
                m
            },
        };
        let json = serde_json::to_string(&reg).expect("should succeed");
        let deser: ServiceRegistration = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deser.capabilities.len(), 2);
    }

    #[test]
    fn test_service_info_serde() {
        let info = ServiceInfo {
            id: "svc-123".to_string(),
            name: "ai-inference".to_string(),
            capabilities: vec!["ai.query".to_string()],
            endpoint: "http://localhost:9090".to_string(),
            healthy: true,
        };
        let json = serde_json::to_string(&info).expect("should succeed");
        let deser: ServiceInfo = serde_json::from_str(&json).expect("should succeed");
        assert!(deser.healthy);
        assert_eq!(deser.id, "svc-123");
    }
}
