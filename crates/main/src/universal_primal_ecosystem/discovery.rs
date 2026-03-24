// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem and well-known endpoint discovery.

use chrono::Utc;
use tracing::{debug, info};

use crate::universal::UniversalResult;

use super::UniversalPrimalEcosystem;
use super::types::{CapabilityMatch, CapabilityRequest, DiscoveredService, ServiceHealth};

impl UniversalPrimalEcosystem {
    /// Discover all available services and their capabilities
    pub async fn discover_ecosystem_services(&mut self) -> UniversalResult<()> {
        info!("Discovering ecosystem services through capability-based discovery");

        // This would typically query a service registry or perform network discovery
        // For now, we'll implement a basic discovery mechanism
        self.discover_via_well_known_endpoints().await?;
        self.discover_via_service_mesh().await?;

        Ok(())
    }

    /// Discover services via well-known endpoints (fallback)
    ///
    /// MODERN APPROACH: Environment-based discovery with capability queries
    /// No hardcoded primal-specific ports in code.
    async fn discover_via_well_known_endpoints(&mut self) -> UniversalResult<()> {
        use universal_constants::network;

        tracing::debug!("Attempting well-known endpoint discovery as fallback");

        // CAPABILITY-BASED: Build discovery list from environment configuration
        // Users can set SERVICE_DISCOVERY_PORTS="8080,8081,8082" to customize
        let discovery_ports = if let Ok(ports_str) = std::env::var("SERVICE_DISCOVERY_PORTS") {
            tracing::info!("Using custom service discovery ports from environment");
            ports_str
                .split(',')
                .filter_map(|s| s.trim().parse::<u16>().ok())
                .collect::<Vec<_>>()
        } else {
            // Default fallback: Common service ports discovered at runtime
            vec![
                network::get_service_port("http"), // Standard HTTP
                network::get_port_from_env("SERVICE_MESH_PORT", 8080), // Service mesh
                network::get_port_from_env("COMPUTE_SERVICE_PORT", 8081), // Compute
                network::get_port_from_env("SECURITY_SERVICE_PORT", 8082), // Security
                network::get_port_from_env("STORAGE_SERVICE_PORT", 8083), // Storage
                8500,                              // Consul/service mesh default
            ]
        };

        // Determine discovery host (default to localhost, configurable)
        let discovery_host = std::env::var("SERVICE_DISCOVERY_HOST")
            .unwrap_or_else(|_| network::DEFAULT_LOCALHOST.to_string());

        tracing::debug!(
            "Scanning {} ports on {} for service capabilities",
            discovery_ports.len(),
            discovery_host
        );

        let mut discovered_count = 0;

        for port in discovery_ports {
            let endpoint = network::http_url(&discovery_host, port, "");

            // Query for capabilities (not primal names)
            if let Ok(capabilities) = self.query_service_capabilities(&endpoint).await {
                if capabilities.is_empty() {
                    tracing::debug!("Port {} responded but advertised no capabilities", port);
                    continue;
                }

                discovered_count += 1;

                let service = DiscoveredService {
                    service_id: format!("service-{discovery_host}:{port}"),
                    instance_id: format!("instance-{discovery_host}:{port}"),
                    endpoint: endpoint.clone(),
                    capabilities: capabilities.clone(),
                    health: ServiceHealth::Healthy,
                    discovered_at: Utc::now(),
                    last_health_check: Some(Utc::now()),
                };

                tracing::info!(
                    "Discovered service at {}:{} with capabilities: {:?}",
                    discovery_host,
                    port,
                    capabilities
                );

                // Index by capability (capability-based architecture)
                let mut services = self.discovered_services.write().await;
                for capability in capabilities {
                    services
                        .entry(capability.clone())
                        .or_insert_with(Vec::new)
                        .push(service.clone());
                }
            }
        }

        if discovered_count > 0 {
            tracing::info!(
                "Well-known endpoint discovery completed: {} services found",
                discovered_count
            );
        } else {
            tracing::debug!("No services discovered via well-known endpoints");
        }

        Ok(())
    }

    /// Discover services via service mesh
    async fn discover_via_service_mesh(&mut self) -> UniversalResult<()> {
        if let Some(mesh_endpoint) = &self.service_mesh_endpoint {
            // Query service mesh for available services
            debug!(
                "Discovering services through service mesh: {}",
                mesh_endpoint
            );

            // Implementation would query the service mesh discovery API
            // This is a placeholder for the actual service mesh integration
        }

        Ok(())
    }

    /// Query service capabilities from an endpoint
    ///
    /// For Unix socket endpoints, uses the capability discovery system
    /// to probe the socket and retrieve its capabilities.
    pub(crate) async fn query_service_capabilities(
        &self,
        endpoint: &str,
    ) -> UniversalResult<Vec<String>> {
        // Parse endpoint to extract socket path (unix:// prefix)
        if let Some(socket_path) = endpoint.strip_prefix("unix://") {
            let path = std::path::PathBuf::from(socket_path);
            match crate::capabilities::discovery::probe_socket(&path).await {
                Ok(provider) => Ok(provider.capabilities),
                Err(e) => {
                    debug!("Failed to probe socket {}: {}", socket_path, e);
                    Ok(Vec::new())
                }
            }
        } else {
            // Non-socket endpoints not supported in primal architecture
            debug!("Non-socket endpoint, skipping: {}", endpoint);
            Ok(Vec::new())
        }
    }

    /// Find services by capability without caching (internal method)
    pub(crate) async fn find_services_by_capability_uncached(
        &self,
        request: &CapabilityRequest,
    ) -> UniversalResult<Vec<CapabilityMatch>> {
        debug!(
            "Finding services by capability (uncached): {:?}",
            request.required_capabilities
        );

        let mut matches = Vec::new();
        let services = self.discovered_services.read().await;

        // Search through all discovered services, regardless of their "primal type"
        for capability_services in services.values() {
            for service in capability_services {
                let mut matched_capabilities = Vec::new();
                let mut missing_capabilities = Vec::new();

                // Check required capabilities
                for required_cap in &request.required_capabilities {
                    if service.capabilities.contains(required_cap) {
                        matched_capabilities.push(required_cap.clone());
                    } else {
                        missing_capabilities.push(required_cap.clone());
                    }
                }

                // If all required capabilities are met, this is a valid match
                if missing_capabilities.is_empty() {
                    // Check optional capabilities for scoring
                    let optional_matches: usize = request
                        .optional_capabilities
                        .iter()
                        .filter(|cap| service.capabilities.contains(*cap))
                        .count();

                    // Calculate score based on capability matching
                    let total_optional = request.optional_capabilities.len();
                    let score = if total_optional > 0 {
                        (optional_matches as f64) / (total_optional as f64)
                    } else {
                        1.0
                    };

                    matches.push(CapabilityMatch {
                        service: service.clone(),
                        score,
                        matched_capabilities,
                        missing_capabilities,
                    });
                }
            }
        }

        // Sort by score (best matches first)
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(matches)
    }
}
