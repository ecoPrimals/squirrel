// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors
#![allow(deprecated)]

//! # Production Security System
//!
//! This module provides comprehensive enterprise-grade security including:
//! - Authentication and authorization with `BearDog` integration
//! - Rate limiting and `DoS` protection
//! - Input validation and injection attack prevention
//! - Real-time security monitoring and threat detection
//! - Automated security response and incident handling
//! - Security audit logging and compliance

pub mod beardog_coordinator;
pub mod config;
pub mod health;
pub mod input_validator;
pub mod monitoring;
pub mod orchestrator;
pub mod policy;
pub mod rate_limiter;
pub mod session;
pub mod traits;
pub mod types;

#[cfg(test)]
mod session_tests;

// Re-export core types and components
pub use beardog_coordinator::BeardogSecurityCoordinator;
pub use config::{AuthMethod, RetryConfig, SecurityProviderConfig};
pub use input_validator::{
    InputType, InputValidationConfig, ProductionInputValidator, RiskLevel as InputRiskLevel,
    SecurityViolation, ValidationResult,
};
pub use monitoring::{
    AlertType, EventSeverity, SecurityAlert, SecurityEvent, SecurityEventType,
    SecurityMonitoringConfig, SecurityMonitoringSystem,
};
pub use orchestrator::{
    ResponseType, RiskLevel, SecurityCheckRequest, SecurityCheckResult,
    SecurityOrchestrationConfig, SecurityOrchestrator, SecurityResponse,
};
pub use policy::{PolicyRule, PolicyType, SecurityPolicy};
pub use rate_limiter::{
    EndpointType, ProductionRateLimiter, RateLimitConfig, RateLimitResult, RateLimitStatistics,
};
pub use session::SecuritySession;
pub use traits::SecurityCoordinator;
pub use types::{
    AuthorizationLevel, SecurityCapability, SecurityContext, SecurityLevel, SecurityRequest,
    SecurityRequestType, SecurityResponse as SecurityApiResponse,
};

/// Convenience re-exports from core auth system
pub use squirrel_mcp_auth::{
    AuthContext, AuthError, LoginRequest, LoginResponse, Permission, Session, User,
};

use crate::error::PrimalError;
use std::sync::Arc;

/// Complete security system factory
pub struct SecuritySystemBuilder {
    orchestration_config: Option<SecurityOrchestrationConfig>,
    enable_beardog_integration: bool,
}

impl SecuritySystemBuilder {
    /// Create a new security system builder
    #[must_use]
    pub const fn new() -> Self {
        Self {
            orchestration_config: None,
            enable_beardog_integration: false,
        }
    }

    /// Configure the security orchestration system
    #[must_use]
    pub fn with_orchestration_config(mut self, config: SecurityOrchestrationConfig) -> Self {
        self.orchestration_config = Some(config);
        self
    }

    /// Enable `BearDog` security integration
    #[must_use]
    pub const fn with_beardog_integration(mut self, enable: bool) -> Self {
        self.enable_beardog_integration = enable;
        self
    }

    /// Build the complete security system
    pub async fn build(self) -> Result<Arc<SecurityOrchestrator>, PrimalError> {
        let config = self.orchestration_config.unwrap_or_default();

        let orchestrator = SecurityOrchestrator::new(config).await?;

        tracing::info!(
            beardog_integration = self.enable_beardog_integration,
            operation = "security_system_built",
            "Production security system initialized successfully"
        );

        Ok(Arc::new(orchestrator))
    }
}

impl Default for SecuritySystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Production security system facade for easy integration
pub struct ProductionSecuritySystem {
    orchestrator: Arc<SecurityOrchestrator>,
    beardog_coordinator: Option<Arc<BeardogSecurityCoordinator>>,
}

impl ProductionSecuritySystem {
    /// Create a new production security system
    pub async fn new(config: SecurityOrchestrationConfig) -> Result<Self, PrimalError> {
        let orchestrator = SecurityOrchestrator::new(config).await?;

        Ok(Self {
            orchestrator: Arc::new(orchestrator),
            beardog_coordinator: None,
        })
    }

    /// Get the security orchestrator
    #[must_use]
    pub fn orchestrator(&self) -> Arc<SecurityOrchestrator> {
        Arc::clone(&self.orchestrator)
    }

    /// Get `BearDog` coordinator if available
    #[must_use]
    pub fn beardog_coordinator(&self) -> Option<Arc<BeardogSecurityCoordinator>> {
        self.beardog_coordinator.clone()
    }

    /// Perform comprehensive security check
    pub async fn check_security(&self, request: SecurityCheckRequest) -> SecurityCheckResult {
        self.orchestrator.check_security(request).await
    }

    /// Get comprehensive security statistics
    pub async fn get_security_statistics(&self) -> orchestrator::SecurityStatistics {
        self.orchestrator.get_security_statistics().await
    }
}

#[cfg(test)]
mod security_mod_tests {
    use super::*;

    #[test]
    fn test_security_system_builder_new() {
        let builder = SecuritySystemBuilder::new();
        assert!(!builder.enable_beardog_integration);
        assert!(builder.orchestration_config.is_none());
    }

    #[test]
    fn test_security_system_builder_default() {
        let builder = SecuritySystemBuilder::default();
        assert!(!builder.enable_beardog_integration);
    }

    #[test]
    fn test_security_system_builder_with_beardog() {
        let builder = SecuritySystemBuilder::new().with_beardog_integration(true);
        assert!(builder.enable_beardog_integration);
    }

    #[test]
    fn test_security_system_builder_with_config() {
        let config = SecurityOrchestrationConfig::default();
        let builder = SecuritySystemBuilder::new().with_orchestration_config(config);
        assert!(builder.orchestration_config.is_some());
    }

    #[test]
    fn test_security_system_builder_chaining() {
        let config = SecurityOrchestrationConfig::default();
        let builder = SecuritySystemBuilder::new()
            .with_orchestration_config(config)
            .with_beardog_integration(true);
        assert!(builder.orchestration_config.is_some());
        assert!(builder.enable_beardog_integration);
    }

    #[tokio::test]
    async fn test_security_system_builder_build() {
        let result = SecuritySystemBuilder::new().build().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_production_security_system_new() {
        let config = SecurityOrchestrationConfig::default();
        let result = ProductionSecuritySystem::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_production_security_system_orchestrator() {
        let config = SecurityOrchestrationConfig::default();
        let system = ProductionSecuritySystem::new(config).await.unwrap();
        let orchestrator = system.orchestrator();
        // Just verify we get an Arc back
        assert!(Arc::strong_count(&orchestrator) >= 1);
    }

    #[tokio::test]
    async fn test_production_security_system_no_beardog() {
        let config = SecurityOrchestrationConfig::default();
        let system = ProductionSecuritySystem::new(config).await.unwrap();
        assert!(system.beardog_coordinator().is_none());
    }
}
