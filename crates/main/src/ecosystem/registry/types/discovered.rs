// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! [`DiscoveredService`] and discovery-time metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::ecosystem::EcosystemPrimalType;

use super::arc_serde::{
    deserialize_arc_str, deserialize_arc_str_map, deserialize_arc_str_vec, serialize_arc_str,
    serialize_arc_str_map, serialize_arc_str_vec,
};
use super::health::ServiceHealthStatus;
use super::interning::intern_registry_string;

/// Discovered service information with `Arc<str>` optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Service ID as `Arc<str>` for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub service_id: Arc<str>,
    /// Type of primal providing this service
    pub primal_type: EcosystemPrimalType,
    /// Endpoint as `Arc<str>` for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub endpoint: Arc<str>,
    /// Health endpoint as `Arc<str>`
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub health_endpoint: Arc<str>,
    /// API version as `Arc<str>`
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub api_version: Arc<str>,
    /// Capabilities as `Arc<str>` for efficient sharing
    #[serde(
        serialize_with = "serialize_arc_str_vec",
        deserialize_with = "deserialize_arc_str_vec"
    )]
    pub capabilities: Vec<Arc<str>>,
    /// Metadata with `Arc<str>` keys and values for zero-copy
    #[serde(
        serialize_with = "serialize_arc_str_map",
        deserialize_with = "deserialize_arc_str_map"
    )]
    pub metadata: HashMap<Arc<str>, Arc<str>>,
    /// When the service was first discovered.
    pub discovered_at: DateTime<Utc>,
    /// Timestamp of the last health check, if performed.
    pub last_health_check: Option<DateTime<Utc>>,
    /// Current health status of the service.
    pub health_status: ServiceHealthStatus,
}

impl DiscoveredService {
    /// Create new `DiscoveredService` with string interning optimization
    pub fn new(
        service_id: &str,
        primal_type: EcosystemPrimalType,
        endpoint: &str,
        health_endpoint: &str,
        api_version: &str,
        capabilities: Vec<&str>,
        metadata: HashMap<&str, &str>,
    ) -> Self {
        Self {
            service_id: intern_registry_string(service_id),
            primal_type,
            endpoint: Arc::from(endpoint),
            health_endpoint: Arc::from(health_endpoint),
            api_version: intern_registry_string(api_version),
            capabilities: capabilities
                .into_iter()
                .map(intern_registry_string)
                .collect(),
            metadata: metadata
                .into_iter()
                .map(|(k, v)| (intern_registry_string(k), Arc::from(v)))
                .collect(),
            discovered_at: Utc::now(),
            last_health_check: None,
            health_status: ServiceHealthStatus::Unknown,
        }
    }

    /// Efficient lookup of metadata without allocation
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&Arc<str>> {
        self.metadata
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }

    /// Check if service has capability without allocation
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|cap| cap.as_ref() == capability)
    }
}
