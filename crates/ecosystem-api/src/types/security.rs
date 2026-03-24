// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security-related types for ecosystem communication.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Security context for all requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Authentication token
    pub auth_token: Option<String>,

    /// User/service identity (`Arc<str>` for O(1) clone when shared)
    pub identity: Arc<str>,

    /// Permissions/capabilities
    pub permissions: Vec<String>,

    /// Security level required
    pub security_level: SecurityLevel,
}

/// Security level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum SecurityLevel {
    /// Publicly accessible
    Public,
    /// Internal ecosystem services only
    Internal,
    /// Restricted access
    Restricted,
    /// Confidential access
    Confidential,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication method
    pub auth_method: String,

    /// TLS enabled
    pub tls_enabled: bool,

    /// Mutual TLS required
    pub mtls_required: bool,

    /// Trust domain
    pub trust_domain: String,

    /// Security level
    pub security_level: SecurityLevel,

    /// Crypto lock enabled
    pub crypto_lock_enabled: bool,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            auth_token: None,
            identity: Arc::from("anonymous"),
            permissions: vec![],
            security_level: SecurityLevel::Public,
        }
    }
}
