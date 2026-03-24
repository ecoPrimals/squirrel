// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Modern authentication service with capability discovery and standalone fallback
//!
//! Uses universal adapter pattern for capability discovery - no hardcoded primal dependencies.
//! Discovers any primal providing security/auth capabilities through network effects.

mod discovery;
mod operations;

#[cfg(test)]
mod tests;

use crate::session::SessionManager;
use crate::types::{AuthProvider, User};
#[cfg(feature = "http-auth")]
use reqwest::Client;
use std::collections::HashMap;

/// Modern authentication service supporting capability discovery and standalone fallback
#[derive(Debug)]
pub struct AuthService {
    /// HTTP client for external auth requests
    #[cfg(feature = "http-auth")]
    client: Client,
    /// Session manager for handling user sessions
    session_manager: SessionManager,
    /// Current authentication provider configuration
    auth_provider: AuthProvider,
    /// In-memory user store (for standalone mode)
    users: HashMap<String, User>,
}

pub(super) fn default_admin_user() -> User {
    let mut user = User::new("admin", "admin@squirrel.local");
    user.roles.push("admin".to_string());
    user.roles.push("user".to_string());
    user
}

pub(super) fn default_user() -> User {
    let mut user = User::new("user", "user@squirrel.local");
    user.roles.push("user".to_string());
    user
}
