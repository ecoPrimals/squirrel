// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Zero-Copy Security Types
//!
//! High-performance authentication and authorization types that minimize allocations
//! and eliminate unnecessary cloning in hot paths. Uses references, Arc, and Cow
//! to achieve zero-copy semantics where possible.

use async_trait::async_trait;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::PrimalResult;

/// Zero-copy credentials that use references and shared ownership
#[derive(Debug, Clone)]
pub struct ZeroCopyCredentials<'a> {
    /// Username as string slice or owned string
    pub username: Cow<'a, str>,
    /// Password as string slice or owned string
    pub password: Cow<'a, str>,
    /// Optional token, shared across multiple uses
    pub token: Option<Arc<str>>,
    /// Metadata shared across multiple uses
    pub metadata: Arc<HashMap<String, String>>,
}

impl<'a> ZeroCopyCredentials<'a> {
    /// Create credentials from borrowed strings (zero-copy)
    pub fn from_borrowed(
        username: &'a str,
        password: &'a str,
        token: Option<&str>,
        metadata: &HashMap<String, String>,
    ) -> Self {
        Self {
            username: Cow::Borrowed(username),
            password: Cow::Borrowed(password),
            token: token.map(Arc::from),
            metadata: Arc::new(metadata.clone()), // Only clone metadata if needed
        }
    }

    /// Create credentials from owned strings
    pub fn from_owned(
        username: String,
        password: String,
        token: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            username: Cow::Owned(username),
            password: Cow::Owned(password),
            token: token.map(Arc::from),
            metadata: Arc::new(metadata),
        }
    }

    /// Get username as string slice (zero allocation)
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Get password as string slice (zero allocation)
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Get token if present (zero allocation)
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Get metadata reference (zero allocation)
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

/// Zero-copy principal using Arc for expensive data
#[derive(Debug, Clone)]
pub struct ZeroCopyPrincipal {
    /// Principal ID (small, can be owned)
    pub id: Arc<str>,
    /// Principal name (can be large, use Arc)
    pub name: Arc<str>,
    /// Principal type (enum, cheap to copy)
    pub principal_type: PrincipalType,
    /// Roles (shared across many auth operations)
    pub roles: Arc<Vec<String>>,
    /// Permissions (shared across many auth operations)
    pub permissions: Arc<Vec<String>>,
    /// Metadata (shared, expensive to clone)
    pub metadata: Arc<HashMap<String, String>>,
}

/// Type of security principal in the system
///
/// Determines authentication requirements and permission scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalType {
    /// Human user with individual account
    User,
    /// Service account for system-to-system communication
    Service,
    /// System-level principal with elevated privileges
    System,
    /// Unauthenticated principal with minimal permissions
    Anonymous,
}

impl ZeroCopyPrincipal {
    /// Create a new principal with owned data
    pub fn new(
        id: String,
        name: String,
        principal_type: PrincipalType,
        roles: Vec<String>,
        permissions: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Arc::from(id),
            name: Arc::from(name),
            principal_type,
            roles: Arc::new(roles),
            permissions: Arc::new(permissions),
            metadata: Arc::new(metadata),
        }
    }

    /// Create principal from existing Arc data (zero-copy)
    pub fn from_arc(
        id: Arc<str>,
        name: Arc<str>,
        principal_type: PrincipalType,
        roles: Arc<Vec<String>>,
        permissions: Arc<Vec<String>>,
        metadata: Arc<HashMap<String, String>>,
    ) -> Self {
        Self {
            id,
            name,
            principal_type,
            roles,
            permissions,
            metadata,
        }
    }

    /// Get ID as string slice
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get name as string slice
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if principal has role (zero allocation)
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if principal has permission (zero allocation)
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Get metadata value (zero allocation)
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }
}

/// Zero-copy authentication request
#[derive(Debug)]
pub struct ZeroCopyAuthRequest<'a> {
    /// Service ID (can be borrowed)
    pub service_id: Cow<'a, str>,
    /// Credentials (zero-copy)
    pub credentials: ZeroCopyCredentials<'a>,
    /// Request timestamp (small, copy is fine)
    pub timestamp: std::time::SystemTime,
    /// Request ID (small, owned is fine)
    pub request_id: uuid::Uuid,
}

impl<'a> ZeroCopyAuthRequest<'a> {
    /// Create auth request with borrowed service ID
    pub fn new_borrowed(service_id: &'a str, credentials: ZeroCopyCredentials<'a>) -> Self {
        Self {
            service_id: Cow::Borrowed(service_id),
            credentials,
            timestamp: std::time::SystemTime::now(),
            request_id: uuid::Uuid::new_v4(),
        }
    }

    /// Create auth request with owned service ID
    pub fn new_owned(service_id: String, credentials: ZeroCopyCredentials<'a>) -> Self {
        Self {
            service_id: Cow::Owned(service_id),
            credentials,
            timestamp: std::time::SystemTime::now(),
            request_id: uuid::Uuid::new_v4(),
        }
    }

    /// Get service ID as string slice
    pub fn service_id(&self) -> &str {
        &self.service_id
    }
}

/// Zero-copy authorization request
#[derive(Debug)]
pub struct ZeroCopyAuthzRequest<'a> {
    /// Service ID
    pub service_id: Cow<'a, str>,
    /// Principal (shared)
    pub principal: Arc<ZeroCopyPrincipal>,
    /// Action being authorized
    pub action: Cow<'a, str>,
    /// Resource being accessed
    pub resource: Cow<'a, str>,
    /// Request context (shared)
    pub context: Arc<HashMap<String, String>>,
}

impl<'a> ZeroCopyAuthzRequest<'a> {
    /// Create authorization request with borrowed strings
    pub fn new_borrowed(
        service_id: &'a str,
        principal: Arc<ZeroCopyPrincipal>,
        action: &'a str,
        resource: &'a str,
        context: Arc<HashMap<String, String>>,
    ) -> Self {
        Self {
            service_id: Cow::Borrowed(service_id),
            principal,
            action: Cow::Borrowed(action),
            resource: Cow::Borrowed(resource),
            context,
        }
    }
}

/// Zero-copy authentication result
#[derive(Debug, Clone)]
pub struct ZeroCopyAuthResult {
    /// Success status
    pub success: bool,
    /// Authenticated principal (shared)
    pub principal: Option<Arc<ZeroCopyPrincipal>>,
    /// Authentication token (shared)
    pub token: Option<Arc<str>>,
    /// Session data (shared)
    pub session_data: Arc<HashMap<String, String>>,
    /// Error message if failed
    pub error: Option<Arc<str>>,
}

impl ZeroCopyAuthResult {
    /// Create successful auth result
    pub fn success(
        principal: Arc<ZeroCopyPrincipal>,
        token: Option<Arc<str>>,
        session_data: HashMap<String, String>,
    ) -> Self {
        Self {
            success: true,
            principal: Some(principal),
            token,
            session_data: Arc::new(session_data),
            error: None,
        }
    }

    /// Create failed auth result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            principal: None,
            token: None,
            session_data: Arc::new(HashMap::new()),
            error: Some(Arc::from(error)),
        }
    }

    /// Check if authentication was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get principal if authenticated
    pub fn principal(&self) -> Option<&ZeroCopyPrincipal> {
        self.principal.as_deref()
    }

    /// Get token if present
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Get error message if failed
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// Zero-copy security provider trait
#[async_trait]
pub trait ZeroCopySecurityProvider: Send + Sync {
    /// Authenticate using zero-copy credentials
    async fn authenticate_zero_copy<'a>(
        &self,
        request: ZeroCopyAuthRequest<'a>,
    ) -> PrimalResult<ZeroCopyAuthResult>;

    /// Authorize using zero-copy principal
    async fn authorize_zero_copy<'a>(
        &self,
        request: ZeroCopyAuthzRequest<'a>,
    ) -> PrimalResult<bool>;

    /// Validate token (zero-copy)
    async fn validate_token(&self, token: &str) -> PrimalResult<Option<Arc<ZeroCopyPrincipal>>>;
}

/// Principal cache for reusing expensive principal data
#[derive(Debug)]
pub struct PrincipalCache {
    /// Cached principals by ID
    cache: Arc<tokio::sync::RwLock<HashMap<String, Arc<ZeroCopyPrincipal>>>>,
    /// Cache statistics
    stats: Arc<tokio::sync::RwLock<CacheStats>>,
}

/// Principal cache performance statistics
///
/// Tracks cache hit rate and eviction patterns for optimization.
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Number of successful cache hits
    pub hits: u64,
    /// Number of cache misses requiring full lookup
    pub misses: u64,
    /// Number of principals evicted from cache
    pub evictions: u64,
}

impl Default for PrincipalCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PrincipalCache {
    /// Create new principal cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            stats: Arc::new(tokio::sync::RwLock::new(CacheStats::default())),
        }
    }

    /// Get principal from cache (zero-copy if found)
    pub async fn get(&self, id: &str) -> Option<Arc<ZeroCopyPrincipal>> {
        let cache = self.cache.read().await;
        if let Some(principal) = cache.get(id) {
            let mut stats = self.stats.write().await;
            stats.hits += 1;
            Some(principal.clone()) // Clone Arc, not the data
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            None
        }
    }

    /// Store principal in cache
    pub async fn store(&self, id: String, principal: Arc<ZeroCopyPrincipal>) {
        let mut cache = self.cache.write().await;
        cache.insert(id, principal);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        CacheStats {
            hits: stats.hits,
            misses: stats.misses,
            evictions: stats.evictions,
        }
    }

    /// Clear cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

/// Builder for creating zero-copy credentials efficiently
#[derive(Debug, Default)]
pub struct CredentialsBuilder {
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    metadata: HashMap<String, String>,
}

impl CredentialsBuilder {
    /// Create new credentials builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set username
    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    /// Set password
    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Set token
    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Build zero-copy credentials
    pub fn build(self) -> ZeroCopyCredentials<'static> {
        ZeroCopyCredentials::from_owned(
            self.username.unwrap_or_default(),
            self.password.unwrap_or_default(),
            self.token,
            self.metadata,
        )
    }
}

/// Macro for creating zero-copy credentials from string literals
#[macro_export]
macro_rules! zero_copy_creds {
    ($username:literal, $password:literal) => {
        ZeroCopyCredentials::from_borrowed(
            $username,
            $password,
            None,
            &std::collections::HashMap::new(),
        )
    };
    ($username:literal, $password:literal, $token:literal) => {
        ZeroCopyCredentials::from_borrowed(
            $username,
            $password,
            Some($token),
            &std::collections::HashMap::new(),
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_credentials() {
        let metadata = HashMap::new();
        let creds =
            ZeroCopyCredentials::from_borrowed("testuser", "testpass", Some("token123"), &metadata);

        assert_eq!(creds.username(), "testuser");
        assert_eq!(creds.password(), "testpass");
        assert_eq!(creds.token(), Some("token123"));
    }

    #[test]
    fn test_zero_copy_principal() {
        let principal = ZeroCopyPrincipal::new(
            "user123".to_string(),
            "Test User".to_string(),
            PrincipalType::User,
            vec!["admin".to_string()],
            vec!["read".to_string(), "write".to_string()],
            HashMap::new(),
        );

        assert_eq!(principal.id(), "user123");
        assert_eq!(principal.name(), "Test User");
        assert!(principal.has_role("admin"));
        assert!(principal.has_permission("read"));
        assert!(!principal.has_role("guest"));
    }

    #[test]
    fn test_credentials_builder() {
        let creds = CredentialsBuilder::new()
            .username("testuser".to_string())
            .password("testpass".to_string())
            .token("token123".to_string())
            .metadata("client".to_string(), "test".to_string())
            .build();

        assert_eq!(creds.username(), "testuser");
        assert_eq!(creds.token(), Some("token123"));
        assert_eq!(creds.metadata().get("client"), Some(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_principal_cache() {
        let cache = PrincipalCache::new();
        let principal = Arc::new(ZeroCopyPrincipal::new(
            "user123".to_string(),
            "Test User".to_string(),
            PrincipalType::User,
            vec![],
            vec![],
            HashMap::new(),
        ));

        // Cache miss
        assert!(cache.get("user123").await.is_none());

        // Store and retrieve
        cache.store("user123".to_string(), principal.clone()).await;
        let cached = cache.get("user123").await.unwrap();
        assert_eq!(cached.id(), "user123");

        // Check stats
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }
}
