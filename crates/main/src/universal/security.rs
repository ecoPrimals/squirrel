// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security types and session management
//!
//! This module defines types for managing security sessions and
//! universal service registrations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::service::ServiceCapability;

/// Universal security session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSecuritySession {
    /// Unique session identifier.
    pub session_id: String,
    /// User identifier, if authenticated.
    pub user_id: Option<String>,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// When the session expires.
    pub expires_at: DateTime<Utc>,
    /// Capabilities granted to this session.
    pub capabilities: Vec<ServiceCapability>,
    /// Additional session metadata.
    pub metadata: HashMap<String, String>,
}
