// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Context and location types for primal operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::security::SecurityLevel;

/// Context for primal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalContext {
    /// User identifier (`Arc<str>` for O(1) clone when shared)
    pub user_id: Arc<str>,

    /// Device identifier (`Arc<str>` for O(1) clone when shared)
    pub device_id: Arc<str>,

    /// Session identifier (`Arc<str>` for O(1) clone when shared)
    pub session_id: Arc<str>,

    /// Network location information
    pub network_location: NetworkLocation,

    /// Security level
    pub security_level: SecurityLevel,

    /// Biome identifier (if applicable) (`Arc<str>` for O(1) clone when shared)
    pub biome_id: Option<Arc<str>>,

    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

/// Network location information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkLocation {
    /// IP address
    pub ip_address: Option<String>,

    /// Geographic region
    pub region: Option<String>,

    /// Availability zone
    pub zone: Option<String>,

    /// Network segment
    pub segment: Option<String>,
}

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            user_id: Arc::from("system"),
            device_id: Arc::from("unknown"),
            session_id: Arc::from(Uuid::new_v4().to_string()),
            network_location: NetworkLocation::default(),
            security_level: SecurityLevel::Internal,
            biome_id: None,
            metadata: HashMap::new(),
        }
    }
}
