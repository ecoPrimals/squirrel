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
