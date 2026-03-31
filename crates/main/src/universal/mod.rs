// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Primal Patterns for Squirrel AI Primal
//!
//! This module implements the universal, agnostic patterns that allow any primal
//! to be created, evolved, and integrated seamlessly within the ecoPrimals ecosystem.
//!
//! ## Universal Principles
//!
//! - **Agnostic**: Works across all computing environments and platforms
//! - **Extensible**: New primals can be added without breaking existing ones
//! - **Context-Aware**: Supports user/device-specific routing and multi-tenancy
//! - **Future-Proof**: Designed to evolve with new primal types and capabilities
//!
//! ## Module Organization
//!
//! - **`traits`**: Core trait definitions for primal providers
//! - **`types`**: Fundamental type definitions (primal types, capabilities)
//! - **`context`**: Context structures for operations
//! - **`health`**: Health monitoring and dependency management
//! - **`endpoints`**: Endpoint and port management
//! - **`messages`**: Inter-primal communication types
//! - **`service_mesh`**: Service mesh integration
//! - **`security`**: Security sessions and providers
//! - **`service`**: Service capabilities and registrations
//! - **`helpers`**: Utility functions

// Public modules
pub mod context;
pub mod endpoints;
pub mod health;
pub mod helpers;
pub mod messages;
pub mod security;
pub mod service;
pub mod service_mesh;
pub mod traits;
pub mod types;

// Re-export commonly used types
pub use context::{
    NetworkLocation, PrimalContext, SecurityLevel, UniversalSecurityContext,
    create_default_context, create_default_security_context,
};
pub use endpoints::{DynamicPortInfo, PortStatus, PortType, PrimalEndpoints};
pub use health::{HealthStatus, PrimalDependency, PrimalHealth};
pub use helpers::validate_capability_compatibility;
pub use messages::{
    EcosystemRequest, EcosystemResponse, PrimalRequest, PrimalResponse, ResponseStatus,
    create_ecosystem_request, create_error_response, create_success_response,
};
pub use security::UniversalSecuritySession;
pub use service::{ServiceCapability, ServiceEndpoint, UniversalServiceRegistration};
pub use service_mesh::{CircuitBreakerStatus, LoadBalancingStatus, ServiceMeshStatus};
pub use traits::{UniversalPrimalProvider, UniversalResult, UniversalSecurityProvider};
pub use types::{PrimalCapability, PrimalInfo, PrimalType, SquirrelCapability};

/// Re-export deployment configuration from universal-constants
pub use universal_constants::deployment;

use crate::error::PrimalError;

/// Universal system version
pub const VERSION: &str = "1.0.0";

/// Initialize the universal system
pub fn init() -> Result<(), PrimalError> {
    // Configure tracing via subscriber if RUST_LOG is unset, avoiding
    // `std::env::set_var` which is unsafe in edition 2024.
    if std::env::var("RUST_LOG").is_err() {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .try_init()
            .ok();
    }
    tracing::info!("Universal Primal System v{} initialized", VERSION);
    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_system_initialization() {
        let result = init();
        assert!(result.is_ok());
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_context_creation() {
        let context = create_default_context("test-user", "test-device");
        assert_eq!(context.user_id, "test-user");
        assert_eq!(context.device_id, "test-device");
        assert_eq!(context.security_level, SecurityLevel::Basic);
    }

    #[test]
    fn test_security_context_creation() {
        let security_context = create_default_security_context("test-user");
        assert_eq!(security_context.user_id, "test-user");
        assert_eq!(security_context.security_level, SecurityLevel::Basic);
    }

    #[test]
    fn test_ecosystem_requests() {
        let security_context = create_default_security_context("test-user");
        let request = create_ecosystem_request(
            "source-service",
            "target-service",
            "test-operation",
            serde_json::json!({"data": "test"}),
            security_context,
        );

        assert_eq!(request.source_service, "source-service");
        assert_eq!(request.target_service, "target-service");
        assert_eq!(request.operation, "test-operation");

        let response =
            create_success_response(request.request_id, serde_json::json!({"result": "success"}));

        assert_eq!(response.request_id, request.request_id);
        assert_eq!(response.status, ResponseStatus::Success);
        assert!(response.success);
        assert!(response.error_message.is_none());

        let error_response = create_error_response(request.request_id, "Test error message");

        assert_eq!(error_response.request_id, request.request_id);
        assert_eq!(error_response.status, ResponseStatus::Error);
        assert!(!error_response.success);
        assert_eq!(
            error_response.error_message.expect("should succeed"),
            "Test error message"
        );
    }
}
