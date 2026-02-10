// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
    pub session_id: String,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub capabilities: Vec<ServiceCapability>,
    pub metadata: HashMap<String, String>,
}
