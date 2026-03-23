// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Endpoint handling for web plugins
//!
//! This module provides structs and traits for handling HTTP endpoints in web plugins.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::web::http::HttpMethod;

/// Represents an HTTP endpoint provided by a web plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebEndpoint {
    /// Unique identifier for the endpoint
    pub id: Uuid,
    /// The endpoint path, relative to the API base
    pub path: String,
    /// The HTTP method for this endpoint
    pub method: HttpMethod,
    /// Human-readable description of what the endpoint does
    pub description: String,
    /// Required permissions to access this endpoint
    pub permissions: Vec<String>,
    /// Whether this endpoint is public (no authentication required)
    pub is_public: bool,
    /// Whether this endpoint requires admin privileges
    pub is_admin: bool,
    /// Tags for categorizing endpoints
    pub tags: Vec<String>,
}

impl WebEndpoint {
    /// Create a new web endpoint
    #[must_use]
    pub const fn new(id: Uuid, path: String, method: HttpMethod, description: String) -> Self {
        Self {
            id,
            path,
            method,
            description,
            permissions: vec![],
            is_public: false,
            is_admin: false,
            tags: vec![],
        }
    }

    /// Add a required permission to access this endpoint
    #[must_use]
    pub fn with_permission(mut self, permission: &str) -> Self {
        self.permissions.push(permission.to_string());
        self
    }

    /// Make this endpoint public (no authentication required)
    #[must_use]
    pub const fn make_public(mut self) -> Self {
        self.is_public = true;
        self
    }

    /// Make this endpoint admin-only
    #[must_use]
    pub const fn make_admin(mut self) -> Self {
        self.is_admin = true;
        self
    }

    /// Add a tag to this endpoint
    #[must_use]
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Check if a specific path and method match this endpoint
    #[must_use]
    pub fn matches(&self, path: &str, method: HttpMethod) -> bool {
        self.path == path && self.method == method
    }

    /// Check if the user has permission to access this endpoint
    #[must_use]
    pub fn check_permission(&self, user_permissions: &[String], is_admin: bool) -> bool {
        // Public endpoints are accessible to all
        if self.is_public {
            return true;
        }

        // Admin endpoints require admin privileges
        if self.is_admin && !is_admin {
            return false;
        }

        // If no specific permissions are required, just need authentication
        if self.permissions.is_empty() {
            return true;
        }

        // Check if user has at least one of the required permissions
        self.permissions
            .iter()
            .any(|p| user_permissions.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::http::HttpMethod;
    use uuid::Uuid;

    #[test]
    fn web_endpoint_builder_and_matches() {
        let id = Uuid::new_v4();
        let ep = WebEndpoint::new(
            id,
            "/api/foo".to_string(),
            HttpMethod::Get,
            "desc".to_string(),
        )
        .with_permission("read")
        .with_tag("t")
        .make_public()
        .make_admin();

        assert_eq!(ep.path, "/api/foo");
        assert_eq!(ep.method, HttpMethod::Get);
        assert_eq!(ep.permissions, vec!["read"]);
        assert!(ep.is_public);
        assert!(ep.is_admin);
        assert_eq!(ep.tags, vec!["t"]);
        assert!(ep.matches("/api/foo", HttpMethod::Get));
        assert!(!ep.matches("/api/bar", HttpMethod::Get));
        assert!(!ep.matches("/api/foo", HttpMethod::Post));
    }

    #[test]
    fn web_endpoint_check_permission() {
        let id = Uuid::nil();
        let public = WebEndpoint::new(id, "/p".to_string(), HttpMethod::Get, "x".to_string())
            .make_public()
            .with_permission("never_used"); // public wins first

        assert!(public.check_permission(&[], false));

        let admin_only = WebEndpoint::new(
            Uuid::new_v4(),
            "/a".to_string(),
            HttpMethod::Post,
            "x".to_string(),
        )
        .make_admin();

        assert!(!admin_only.check_permission(&[], false));
        assert!(admin_only.check_permission(&[], true));

        let needs_perm = WebEndpoint::new(
            Uuid::new_v4(),
            "/r".to_string(),
            HttpMethod::Get,
            "x".to_string(),
        )
        .with_permission("plugin.read");

        assert!(!needs_perm.check_permission(&[], false));
        assert!(needs_perm.check_permission(&["plugin.read".to_string()], false));
    }

    #[test]
    fn web_endpoint_serde_roundtrip() {
        let ep = WebEndpoint::new(
            Uuid::new_v4(),
            "/z".to_string(),
            HttpMethod::Patch,
            "d".to_string(),
        )
        .with_tag("z");
        let json = serde_json::to_string(&ep).unwrap();
        let back: WebEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, ep.path);
        assert_eq!(back.method, ep.method);
        assert_eq!(back.tags, ep.tags);
    }
}
