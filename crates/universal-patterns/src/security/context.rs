// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security context and health types
//!
//! This module defines the security context and health monitoring types
//! used throughout the security system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::traits::{AuthResult, Principal};

/// Security context for operations
///
/// This struct contains the security context for operations, including
/// the principal, authentication token, permissions, and metadata.
///
/// # Examples
///
/// ```ignore
/// use universal_patterns::security::SecurityContext;
/// use universal_patterns::traits::{AuthResult, Principal, PrincipalType};
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let principal = Principal {
///     id: "user123".to_string(),
///     name: "John Doe".to_string(),
///     principal_type: PrincipalType::User,
///     roles: vec!["admin".to_string()],
///     permissions: vec!["read".to_string(), "write".to_string()],
///     metadata: HashMap::new(),
/// };
///
/// let context = SecurityContext::from_principal(&principal);
/// println!("Context for user: {}", context.principal.name);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Principal performing the operation
    pub principal: Principal,
    /// Authentication token
    pub token: String,
    /// Token expiration
    pub expires_at: DateTime<Utc>,
    /// Granted permissions
    pub permissions: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SecurityContext {
    /// Create security context from auth result
    ///
    /// This method creates a security context from an authentication result.
    ///
    /// # Arguments
    ///
    /// * `auth_result` - The authentication result to create context from
    ///
    /// # Returns
    ///
    /// Returns a new `SecurityContext` instance.
    pub fn from_auth_result(auth_result: &AuthResult) -> Self {
        Self {
            principal: auth_result.principal.clone(),
            token: auth_result.token.clone(),
            expires_at: auth_result.expires_at,
            permissions: auth_result.permissions.clone(),
            metadata: auth_result.metadata.clone(),
        }
    }

    /// Create security context from principal
    ///
    /// This method creates a security context from a principal with default
    /// values for token and expiration.
    ///
    /// # Arguments
    ///
    /// * `principal` - The principal to create context from
    ///
    /// # Returns
    ///
    /// Returns a new `SecurityContext` instance.
    pub fn from_principal(principal: &Principal) -> Self {
        Self {
            principal: principal.clone(),
            token: String::new(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            permissions: principal.permissions.clone(),
            metadata: HashMap::new(),
        }
    }

    /// Check if the context is expired
    ///
    /// # Returns
    ///
    /// Returns `true` if the context has expired, `false` otherwise.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the context has a specific permission
    ///
    /// # Arguments
    ///
    /// * `permission` - The permission to check for
    ///
    /// # Returns
    ///
    /// Returns `true` if the context has the permission, `false` otherwise.
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Get the remaining time before expiration
    ///
    /// # Returns
    ///
    /// Returns the remaining duration before expiration, or zero if expired.
    pub fn time_until_expiration(&self) -> Duration {
        let now = Utc::now();
        if self.expires_at > now {
            (self.expires_at - now).to_std().unwrap_or(Duration::ZERO)
        } else {
            Duration::ZERO
        }
    }

    /// Add metadata to the context
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    /// * `value` - The metadata value
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata from the context
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    ///
    /// # Returns
    ///
    /// Returns the metadata value if present, `None` otherwise.
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Security health information
///
/// This struct contains information about the health status of a security provider.
///
/// # Examples
///
/// ```ignore
/// use universal_patterns::security::{SecurityHealth, HealthStatus};
/// use std::time::Duration;
/// use std::collections::HashMap;
/// use chrono::Utc;
///
/// let health = SecurityHealth {
///     status: HealthStatus::Healthy,
///     latency: Duration::from_millis(100),
///     last_check: Utc::now(),
///     details: HashMap::new(),
/// };
///
/// println!("Security provider is: {:?}", health.status);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHealth {
    /// Health status
    pub status: HealthStatus,
    /// Response latency
    pub latency: Duration,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
    /// Additional details
    pub details: HashMap<String, String>,
}

impl SecurityHealth {
    /// Create a new healthy security health instance
    ///
    /// # Arguments
    ///
    /// * `latency` - The response latency
    ///
    /// # Returns
    ///
    /// Returns a new `SecurityHealth` instance with healthy status.
    pub fn healthy(latency: Duration) -> Self {
        Self {
            status: HealthStatus::Healthy,
            latency,
            last_check: Utc::now(),
            details: HashMap::new(),
        }
    }

    /// Create a new unhealthy security health instance
    ///
    /// # Arguments
    ///
    /// * `reason` - The reason for being unhealthy
    ///
    /// # Returns
    ///
    /// Returns a new `SecurityHealth` instance with unhealthy status.
    pub fn unhealthy(reason: String) -> Self {
        let mut details = HashMap::new();
        details.insert("reason".to_string(), reason);

        Self {
            status: HealthStatus::Unhealthy,
            latency: Duration::ZERO,
            last_check: Utc::now(),
            details,
        }
    }

    /// Check if the health status is healthy
    ///
    /// # Returns
    ///
    /// Returns `true` if the status is healthy, `false` otherwise.
    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }

    /// Check if the health check is recent
    ///
    /// # Arguments
    ///
    /// * `max_age` - Maximum age for a health check to be considered recent
    ///
    /// # Returns
    ///
    /// Returns `true` if the health check is recent, `false` otherwise.
    pub fn is_recent(&self, max_age: Duration) -> bool {
        let age = Utc::now() - self.last_check;
        age.to_std().unwrap_or(Duration::MAX) < max_age
    }

    /// Add a detail to the health information
    ///
    /// # Arguments
    ///
    /// * `key` - The detail key
    /// * `value` - The detail value
    pub fn add_detail(&mut self, key: String, value: String) {
        self.details.insert(key, value);
    }
}

/// Health status enum
///
/// This enum represents the health status of a security provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy and operational
    Healthy,
    /// Service is unhealthy or non-operational
    Unhealthy,
    /// Service health status is unknown
    Unknown,
}

impl HealthStatus {
    /// Check if the status is healthy
    ///
    /// # Returns
    ///
    /// Returns `true` if the status is healthy, `false` otherwise.
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Check if the status is unhealthy
    ///
    /// # Returns
    ///
    /// Returns `true` if the status is unhealthy, `false` otherwise.
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy)
    }

    /// Check if the status is unknown
    ///
    /// # Returns
    ///
    /// Returns `true` if the status is unknown, `false` otherwise.
    pub fn is_unknown(&self) -> bool {
        matches!(self, HealthStatus::Unknown)
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
            HealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::PrincipalType;

    #[test]
    fn test_security_context_from_principal() {
        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string()],
            metadata: HashMap::new(),
        };

        let context = SecurityContext::from_principal(&principal);
        assert_eq!(context.principal.id, "test-user");
        assert_eq!(context.principal.name, "Test User");
        assert!(context.has_permission("read"));
        assert!(!context.has_permission("write"));
    }

    #[test]
    fn test_security_context_expiration() {
        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let mut context = SecurityContext::from_principal(&principal);
        assert!(!context.is_expired());

        // Set expiration to past
        context.expires_at = Utc::now() - chrono::Duration::hours(1);
        assert!(context.is_expired());
    }

    #[test]
    fn test_security_health() {
        let health = SecurityHealth::healthy(Duration::from_millis(100));
        assert!(health.is_healthy());
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.latency, Duration::from_millis(100));

        let unhealthy = SecurityHealth::unhealthy("Connection failed".to_string());
        assert!(!unhealthy.is_healthy());
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert_eq!(
            unhealthy.details.get("reason"),
            Some(&"Connection failed".to_string())
        );
    }

    #[test]
    fn test_health_status() {
        let healthy = HealthStatus::Healthy;
        assert!(healthy.is_healthy());
        assert!(!healthy.is_unhealthy());
        assert!(!healthy.is_unknown());

        let unhealthy = HealthStatus::Unhealthy;
        assert!(!unhealthy.is_healthy());
        assert!(unhealthy.is_unhealthy());
        assert!(!unhealthy.is_unknown());

        let unknown = HealthStatus::Unknown;
        assert!(!unknown.is_healthy());
        assert!(!unknown.is_unhealthy());
        assert!(unknown.is_unknown());
    }

    #[test]
    fn test_metadata_operations() {
        let principal = Principal {
            id: "test-user".to_string(),
            name: "Test User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let mut context = SecurityContext::from_principal(&principal);
        assert_eq!(context.get_metadata("key"), None);

        context.add_metadata("key".to_string(), "value".to_string());
        assert_eq!(context.get_metadata("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_security_context_from_auth_result() {
        let auth_result = AuthResult {
            principal: Principal {
                id: "user1".to_string(),
                name: "User One".to_string(),
                principal_type: PrincipalType::User,
                roles: vec!["admin".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
                metadata: HashMap::new(),
            },
            token: "tok-123".to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(2),
            permissions: vec!["read".to_string(), "write".to_string()],
            metadata: {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "local".to_string());
                m
            },
        };
        let ctx = SecurityContext::from_auth_result(&auth_result);
        assert_eq!(ctx.principal.id, "user1");
        assert_eq!(ctx.token, "tok-123");
        assert!(ctx.has_permission("read"));
        assert!(ctx.has_permission("write"));
        assert!(!ctx.has_permission("delete"));
        assert_eq!(ctx.metadata.get("source"), Some(&"local".to_string()));
    }

    #[test]
    fn test_security_context_time_until_expiration() {
        let principal = Principal {
            id: "user".to_string(),
            name: "User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };
        let context = SecurityContext::from_principal(&principal);
        // Should have ~1 hour until expiration
        let time_left = context.time_until_expiration();
        assert!(time_left.as_secs() > 3500); // ~1 hour minus small delta
    }

    #[test]
    fn test_security_context_expired_time_until_expiration() {
        let principal = Principal {
            id: "user".to_string(),
            name: "User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec![],
            metadata: HashMap::new(),
        };
        let mut context = SecurityContext::from_principal(&principal);
        context.expires_at = Utc::now() - chrono::Duration::hours(1);
        assert_eq!(context.time_until_expiration(), Duration::ZERO);
    }

    #[test]
    fn test_security_context_serde() {
        let principal = Principal {
            id: "user".to_string(),
            name: "User".to_string(),
            principal_type: PrincipalType::User,
            roles: vec![],
            permissions: vec!["read".to_string()],
            metadata: HashMap::new(),
        };
        let context = SecurityContext::from_principal(&principal);
        let json = serde_json::to_string(&context).unwrap();
        let deserialized: SecurityContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.principal.id, "user");
        assert!(deserialized.has_permission("read"));
    }

    #[test]
    fn test_security_health_is_recent() {
        let health = SecurityHealth::healthy(Duration::from_millis(50));
        // Just created, should be recent
        assert!(health.is_recent(Duration::from_secs(60)));
    }

    #[test]
    fn test_security_health_add_detail() {
        let mut health = SecurityHealth::healthy(Duration::from_millis(100));
        health.add_detail("component".to_string(), "auth-service".to_string());
        assert_eq!(
            health.details.get("component"),
            Some(&"auth-service".to_string())
        );
    }

    #[test]
    fn test_security_health_serde() {
        let health = SecurityHealth::healthy(Duration::from_millis(100));
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: SecurityHealth = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_healthy());
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "Healthy");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "Unhealthy");
        assert_eq!(HealthStatus::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_health_status_serde() {
        for status in [
            HealthStatus::Healthy,
            HealthStatus::Unhealthy,
            HealthStatus::Unknown,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, status);
        }
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use crate::traits::PrincipalType;
    use proptest::prelude::*;

    fn principal_strategy() -> impl Strategy<Value = Principal> {
        (
            any::<String>(),
            any::<String>(),
            prop_oneof![
                Just(PrincipalType::User),
                Just(PrincipalType::Service),
                Just(PrincipalType::Client),
                Just(PrincipalType::System),
            ],
            proptest::collection::vec(any::<String>(), 0..5),
            proptest::collection::vec(any::<String>(), 0..5),
        )
            .prop_map(|(id, name, pt, roles, perms)| Principal {
                id,
                name,
                principal_type: pt,
                roles,
                permissions: perms,
                metadata: HashMap::new(),
            })
    }

    proptest! {
        #[test]
        fn security_context_round_trip_serde(principal in principal_strategy(), token in any::<String>()) {
            let original = SecurityContext {
                principal: principal.clone(),
                token,
                expires_at: Utc::now() + chrono::Duration::hours(1),
                permissions: principal.permissions,
                metadata: HashMap::new(),
            };
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: SecurityContext = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(original.principal.id, deserialized.principal.id);
            prop_assert_eq!(original.principal.name, deserialized.principal.name);
            prop_assert_eq!(original.token, deserialized.token);
            prop_assert_eq!(original.permissions, deserialized.permissions);
        }
    }
}
