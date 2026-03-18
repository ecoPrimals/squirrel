// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem service registration and primal type definitions.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::types::{
    EcosystemPrimalType, HealthCheckConfig, ResourceSpec, SecurityConfig, ServiceCapabilities,
    ServiceEndpoints,
};

fn ecosystem_serialize_arc_str<S>(arc_str: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(arc_str)
}

fn ecosystem_deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}

/// Ecosystem service registration for Squirrel AI primal (`Arc<str>` version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    #[serde(
        serialize_with = "ecosystem_serialize_arc_str",
        deserialize_with = "ecosystem_deserialize_arc_str"
    )]
    pub service_id: Arc<str>,
    pub primal_type: EcosystemPrimalType,
    pub name: String,
    pub description: String,
    pub biome_id: Option<String>,
    pub version: String,
    pub capabilities: ServiceCapabilities,
    pub endpoints: ServiceEndpoints,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub primal_provider: Option<String>,
    pub health_check: HealthCheckConfig,
    pub security_config: SecurityConfig,
    pub resource_requirements: ResourceSpec,
    pub metadata: std::collections::HashMap<String, String>,
    pub registered_at: chrono::DateTime<chrono::Utc>,
}
