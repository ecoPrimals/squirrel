// SPDX-License-Identifier: AGPL-3.0-or-later
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

mod option_arc_str_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::sync::Arc;

    #[expect(
        clippy::ref_option,
        reason = "serde with= contract requires &Option<T>"
    )]
    pub fn serialize<S>(opt: &Option<Arc<str>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match opt {
            Some(arc) => serializer.serialize_some(arc.as_ref()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Arc<str>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        Ok(opt.map(|s| Arc::from(s.as_str())))
    }
}

/// Ecosystem service registration for Squirrel AI primal (`Arc<str>` version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    #[serde(
        serialize_with = "ecosystem_serialize_arc_str",
        deserialize_with = "ecosystem_deserialize_arc_str"
    )]
    pub service_id: Arc<str>,
    /// Capability-based service identifier (TRUE PRIMAL: no primal names).
    #[serde(default, with = "option_arc_str_serde")]
    pub capability_id: Option<Arc<str>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecosystem::types::{
        EcosystemPrimalType, HealthCheckConfig, ResourceSpec, SecurityConfig, ServiceCapabilities,
        ServiceEndpoints,
    };

    fn minimal_registration() -> EcosystemServiceRegistration {
        EcosystemServiceRegistration {
            service_id: Arc::from("svc-test"),
            capability_id: Some(Arc::from("ai")),
            primal_type: EcosystemPrimalType::Squirrel,
            name: "Squirrel".into(),
            description: "test".into(),
            biome_id: None,
            version: "0.0.1".into(),
            capabilities: ServiceCapabilities::default(),
            endpoints: ServiceEndpoints {
                primary: "unix:///tmp/x.sock".into(),
                secondary: vec![],
                health: None,
            },
            dependencies: vec![],
            tags: vec!["t".into()],
            primal_provider: None,
            health_check: HealthCheckConfig::default(),
            security_config: SecurityConfig::default(),
            resource_requirements: ResourceSpec::default(),
            metadata: std::collections::HashMap::new(),
            registered_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn ecosystem_service_registration_serde_roundtrip() {
        let reg = minimal_registration();
        let json = serde_json::to_string(&reg).expect("should succeed");
        let back: EcosystemServiceRegistration =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(back.service_id.as_ref(), "svc-test");
        assert_eq!(back.primal_type, EcosystemPrimalType::Squirrel);
        assert_eq!(back.endpoints.primary, reg.endpoints.primary);
    }

    #[test]
    fn ecosystem_service_registration_arc_str_field() {
        let mut reg = minimal_registration();
        reg.service_id = Arc::from("other-id");
        let v = serde_json::to_value(&reg).expect("should succeed");
        assert_eq!(v["service_id"], serde_json::json!("other-id"));
    }
}
