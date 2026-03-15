// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Context representation in the MCP system
///
/// A Context is the primary data structure in the MCP system, representing
/// a piece of contextual information that can be synchronized across instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Unique identifier for the context
    pub id: Uuid,
    /// Human-readable name for the context
    pub name: String,
    /// Primary data content of the context
    pub data: serde_json::Value,
    /// Optional metadata associated with the context
    pub metadata: Option<serde_json::Value>,
    /// Optional parent context ID, for hierarchical relationships
    pub parent_id: Option<Uuid>,
    /// Timestamp when the context was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the context was last updated
    pub updated_at: DateTime<Utc>,
    /// Optional timestamp when the context should expire
    pub expires_at: Option<DateTime<Utc>>,
}

/// Validation rules and schema for context data
///
/// Contains the JSON schema and validation rules that are applied
/// to context data to ensure validity and consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextValidation {
    /// JSON schema for validating context data structure
    pub schema: serde_json::Value,
    /// List of validation rule identifiers to apply
    pub rules: Vec<String>,
}

/// Configuration for Context Manager
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Sync interval in seconds
    pub sync_interval: Option<u64>,
    /// Maximum retry attempts for sync operations
    pub max_retries: Option<u32>,
    /// Timeout for operations in milliseconds
    pub timeout_ms: Option<u64>,
    /// Days after which old data is cleaned up
    pub cleanup_older_than_days: Option<i64>,
    /// Central server URL
    pub sync_server_url: Option<String>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            sync_interval: Some(60),
            max_retries: Some(3),
            timeout_ms: Some(5000),
            cleanup_older_than_days: Some(30),
            sync_server_url: None,
        }
    }
}
