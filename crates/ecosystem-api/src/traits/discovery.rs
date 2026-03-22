// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service discovery query and result types used by [`super::ServiceMeshClient`].

use crate::types::{HealthStatus, PrimalType};
use serde::{Deserialize, Serialize};

/// Service query for service discovery
#[derive(Debug, Clone, Default)]
pub struct ServiceQuery {
    /// Service type filter
    pub service_type: Option<String>,

    /// Primal type filter
    pub primal_type: Option<PrimalType>,

    /// Required capabilities
    pub capabilities: Vec<String>,

    /// Health status filter
    pub health_status: Option<HealthStatus>,

    /// Metadata filters
    pub metadata: std::collections::HashMap<String, String>,
}

/// Service information from discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service ID
    pub id: String,

    /// Service name
    pub name: String,

    /// Service type
    pub service_type: String,

    /// Primal type
    pub primal_type: PrimalType,

    /// Service endpoint
    pub endpoint: String,

    /// Service capabilities
    pub capabilities: Vec<String>,

    /// Health status
    pub health_status: String,

    /// Service metadata
    pub metadata: std::collections::HashMap<String, String>,
}
