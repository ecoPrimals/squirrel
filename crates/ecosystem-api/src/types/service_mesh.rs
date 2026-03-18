// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service mesh status types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service mesh status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceMeshStatus {
    /// Is connected to service mesh
    pub connected: bool,

    /// Service mesh endpoint (capability-based, not primal-specific)
    pub service_mesh_endpoint: Option<String>,

    /// Registration timestamp
    pub registration_time: Option<DateTime<Utc>>,

    /// Last heartbeat timestamp
    pub last_heartbeat: Option<DateTime<Utc>>,

    /// Service mesh metadata
    pub metadata: HashMap<String, String>,
}
