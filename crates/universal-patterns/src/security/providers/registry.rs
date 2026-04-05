// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability indexing and service discovery for universal security providers.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Public registry API for embedding; constructed by integrators"
    )
)]

use std::collections::HashMap;
use std::sync::Arc;

use super::super::errors::SecurityError;
use super::boxed::UniversalSecurityServiceBox;
use super::types::{SecurityCapability, UniversalSecurityService};

/// Universal security service registry
/// Services register themselves with their capabilities here
pub struct UniversalSecurityRegistry {
    services: HashMap<String, Arc<UniversalSecurityServiceBox>>,
    capabilities_index: HashMap<SecurityCapability, Vec<String>>,
}

impl UniversalSecurityRegistry {
    /// Create a new security service registry
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            capabilities_index: HashMap::new(),
        }
    }

    /// Register a security service (any service, regardless of name)
    pub async fn register_service(
        &mut self,
        service_id: String,
        service: Arc<UniversalSecurityServiceBox>,
    ) -> Result<(), SecurityError> {
        // Get service capabilities and index them
        let capabilities = service.get_capabilities();

        // Add service to registry
        self.services.insert(service_id.clone(), service);

        // Update capability index
        for capability in capabilities {
            self.capabilities_index
                .entry(capability)
                .or_default()
                .push(service_id.clone());
        }

        Ok(())
    }

    /// Find services by capability (agnostic discovery)
    pub fn find_by_capability(&self, capability: &SecurityCapability) -> Vec<String> {
        self.capabilities_index
            .get(capability)
            .cloned()
            .unwrap_or_default()
    }

    /// Get optimal service for specific security requirements
    pub async fn find_optimal_service(
        &self,
        requirements: Vec<SecurityCapability>,
    ) -> Result<String, SecurityError> {
        let mut candidates = Vec::new();

        // Find services that have all required capabilities
        for service_id in self.services.keys() {
            if let Some(service) = self.services.get(service_id) {
                let service_caps = service.get_capabilities();

                let has_all_requirements = requirements
                    .iter()
                    .all(|req| service_caps.iter().any(|cap| capabilities_match(req, cap)));

                if has_all_requirements {
                    candidates.push(service_id.clone());
                }
            }
        }

        if candidates.is_empty() {
            return Err(SecurityError::configuration(
                "No security services found matching requirements",
            ));
        }

        // For now, return first candidate (could implement scoring logic)
        Ok(candidates[0].clone())
    }

    /// Get service by ID
    pub fn get_service(&self, service_id: &str) -> Option<Arc<UniversalSecurityServiceBox>> {
        self.services.get(service_id).cloned()
    }

    /// List all registered services
    pub fn list_services(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }
}

/// Check if two security capabilities match
pub fn capabilities_match(required: &SecurityCapability, provided: &SecurityCapability) -> bool {
    use SecurityCapability::{
        Authentication, Authorization, Compliance, Cryptography, DataProtection, Identity,
        ThreatDetection,
    };

    match (required, provided) {
        (
            Authentication {
                methods: req_methods,
                ..
            },
            Authentication {
                methods: prov_methods,
                ..
            },
        ) => req_methods.iter().any(|req| prov_methods.contains(req)),
        (
            Authorization {
                rbac: rbac_mandatory,
                abac: abac_mandatory,
                ..
            },
            Authorization {
                rbac: rbac_available,
                abac: abac_available,
                ..
            },
        ) => (!rbac_mandatory || *rbac_available) && (!abac_mandatory || *abac_available),
        (
            Cryptography {
                algorithms: req_algs,
                ..
            },
            Cryptography {
                algorithms: prov_algs,
                ..
            },
        ) => req_algs.iter().any(|req| prov_algs.contains(req)),
        (
            Compliance {
                standards: req_stds,
                ..
            },
            Compliance {
                standards: prov_stds,
                ..
            },
        ) => req_stds.iter().any(|req| prov_stds.contains(req)),
        (ThreatDetection { .. }, ThreatDetection { .. }) => true,
        (Identity { .. }, Identity { .. }) => true,
        (DataProtection { .. }, DataProtection { .. }) => true,
        _ => false,
    }
}

/// Example registration function for any security service
/// This shows how a specific security service (like BearDog) would register
pub async fn register_security_service(
    registry: &mut UniversalSecurityRegistry,
    service: Arc<UniversalSecurityServiceBox>,
) -> Result<(), SecurityError> {
    let info = service.get_service_info();
    registry.register_service(info.service_id, service).await
}
