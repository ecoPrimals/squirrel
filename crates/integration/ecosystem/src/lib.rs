// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::missing_docs_in_private_items)]
//! Squirrel Ecosystem Integration
//!
//! This crate provides integration points with the broader ecoPrimals ecosystem.
//! It is a **placeholder crate** for planned integration work—core types and
//! registration logic are defined here; full integration is implemented in
//! the main Squirrel crate via capability discovery.
//!
//! ## Integration Plan
//!
//! 1. **Songbird** (orchestration): Service mesh registration, health reporting,
//!    and capability routing. Squirrel will discover Songbird via capability
//!    registry and register MCP services.
//!
//! 2. **ToadStool** (compute): Task delegation, job submission, and resource
//!    allocation. Integration via universal adapter when compute capability
//!    is discovered.
//!
//! 3. **Cross-platform discovery**: Unix sockets (Linux/macOS), Named pipes
//!    (Windows), TCP fallback. Uses `universal-patterns` transport layer.
//!
//! ## Capabilities This Crate Will Provide
//!
//! - `ServiceRegistration` and `EcosystemIntegration` types
//! - `register_mcp_services()`: Register Squirrel MCP with service mesh
//! - Future: Songbird client, ToadStool client (when those crates exist)
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ecosystem service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    /// Unique identifier for this service instance
    pub service_id: String,
    /// Service type (e.g. "mcp", "ai-coordinator")
    pub service_type: String,
    /// Capabilities this service provides (e.g. "ai.query", "tool.execute")
    pub capabilities: Vec<String>,
    /// Socket or transport endpoints for this service
    pub endpoints: Vec<String>,
    /// Arbitrary key-value metadata for service discovery
    pub metadata: HashMap<String, String>,
}

/// Placeholder for ecosystem integration
pub struct EcosystemIntegration {
    // Future: Songbird client
    // Future: Toadstool client
}

impl EcosystemIntegration {
    /// Create a new ecosystem integration instance
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Register Squirrel MCP services with the ecosystem
    ///
    /// # Errors
    ///
    /// Returns an error if service registration fails.
    pub fn register_mcp_services(&self) -> Result<(), Box<dyn std::error::Error>> {
        // FUTURE: [Ecosystem-Integration] Implement service mesh registration via capability discovery
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // Use capability registry to discover service mesh, then register via Unix socket
        // FUTURE: [Ecosystem-Integration] Implement compute integration via capability discovery
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // Use capability registry to discover compute providers, then integrate via universal adapter
        Ok(())
    }
}

impl Default for EcosystemIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ServiceRegistration tests ---
    #[test]
    fn test_service_registration_serde() {
        let reg = ServiceRegistration {
            service_id: "squirrel-1".to_string(),
            service_type: "ai".to_string(),
            capabilities: vec!["inference".to_string(), "embedding".to_string()],
            endpoints: vec!["http://localhost:8080".to_string()],
            metadata: HashMap::from([("version".to_string(), "1.0.0".to_string())]),
        };
        let json = serde_json::to_string(&reg).unwrap();
        let deserialized: ServiceRegistration = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.service_id, "squirrel-1");
        assert_eq!(deserialized.capabilities.len(), 2);
        assert_eq!(deserialized.endpoints.len(), 1);
    }

    #[test]
    fn test_service_registration_clone() {
        let reg = ServiceRegistration {
            service_id: "svc-1".to_string(),
            service_type: "compute".to_string(),
            capabilities: vec![],
            endpoints: vec![],
            metadata: HashMap::new(),
        };
        let cloned = reg.clone();
        assert_eq!(cloned.service_id, reg.service_id);
    }

    // --- EcosystemIntegration tests ---
    #[test]
    fn test_ecosystem_integration_new() {
        let _integration = EcosystemIntegration::new();
    }

    #[test]
    fn test_ecosystem_integration_default() {
        let _integration = EcosystemIntegration::default();
    }

    #[test]
    fn test_register_mcp_services() {
        let integration = EcosystemIntegration::new();
        let result = integration.register_mcp_services();
        assert!(result.is_ok());
    }
}
