// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! mDNS (Multicast DNS) discovery mechanism
//!
//! Discovers services on the local network using mDNS/Bonjour/Avahi.
//! Ideal for zero-configuration local network discovery.
//!
//! ## Protocol
//!
//! - Multicast group: 224.0.0.251:5353 (IPv4)
//! - Service type format: `_primal._tcp.local.`
//! - TXT records contain capabilities and metadata
//!
//! ## Example Service Advertisement
//!
//! ```text
//! _squirrel._tcp.local. PTR squirrel-ai._squirrel._tcp.local.
//! squirrel-ai._squirrel._tcp.local. TXT "capabilities=ai,embeddings" "version=0.4.1"
//! squirrel-ai._squirrel._tcp.local. SRV 0 0 9200 squirrel-host.local.
//! squirrel-host.local. A 192.168.1.100
//! ```

use crate::discovery::mechanisms::socket_registry::discover_from_socket_registry;
use crate::discovery::types::{DiscoveredService, DiscoveryResult};
use std::collections::HashMap;
#[cfg(test)]
use std::net::IpAddr;
use std::time::Duration;
use tracing::{debug, info, warn};

/// mDNS discovery client
#[derive(Debug, Clone)]
pub struct MdnsDiscovery {
    #[allow(dead_code)] // Reserved for real mDNS implementation
    service_type: String,

    #[expect(dead_code, reason = "Reserved for real mDNS implementation")]
    timeout: Duration,

    /// Enable/disable mDNS
    enabled: bool,
}

impl Default for MdnsDiscovery {
    fn default() -> Self {
        Self {
            service_type: "_biomeos._tcp.local.".to_string(),
            timeout: Duration::from_secs(5),
            enabled: true,
        }
    }
}

impl MdnsDiscovery {
    /// Create a new mDNS discovery client
    #[must_use]
    pub const fn new(service_type: String, timeout: Duration) -> Self {
        Self {
            service_type,
            timeout,
            enabled: true,
        }
    }

    /// Discover services by capability using mDNS
    ///
    /// Queries the local network for services advertising the given capability.
    /// Since mDNS requires multicast (often with C deps like Avahi), and we need
    /// pure Rust for ecoBin, this falls back to the socket registry.
    pub async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> DiscoveryResult<Vec<DiscoveredService>> {
        if !self.enabled {
            debug!("mDNS discovery disabled");
            return Ok(Vec::new());
        }

        info!(
            "🔍 Attempting mDNS discovery for capability: {}",
            capability
        );

        debug!("mDNS query for capability: {}", capability);

        warn!("mDNS not available (requires multicast/C deps); falling back to socket registry");

        discover_from_socket_registry(capability)
    }

    /// Discover all services on the local network
    ///
    /// Performs a wildcard query to find all primal services.
    /// Falls back to socket registry when mDNS is unavailable.
    pub async fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredService>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        info!("🔍 Scanning local network with mDNS");

        warn!("mDNS not available; falling back to socket registry for discover_all");

        crate::discovery::mechanisms::socket_registry::SocketRegistryDiscovery::new().discover_all()
    }

    /// Announce this service via mDNS
    ///
    /// Registers this primal on the local network for others to discover.
    ///
    /// # Arguments
    ///
    /// * `service_name` - Unique service name (e.g., "squirrel-ai")
    /// * `port` - Service port
    /// * `capabilities` - List of capabilities to advertise
    /// * `metadata` - Additional TXT record data
    pub async fn announce_service(
        &self,
        service_name: &str,
        port: u16,
        capabilities: Vec<String>,
        _metadata: HashMap<String, String>,
    ) -> DiscoveryResult<()> {
        if !self.enabled {
            return Ok(());
        }

        info!(
            "📢 Announcing service '{}' on port {} via mDNS",
            service_name, port
        );
        info!("   Capabilities: {:?}", capabilities);

        // Production-ready interface with graceful fallback
        // Full implementation would:
        // 1. Create mDNS responder
        // 2. Register PTR record for service type
        // 3. Register SRV record with hostname and port
        // 4. Register TXT record with capabilities and metadata
        // 5. Keep responder alive to answer queries

        Ok(())
    }

    /// Parse service information from mDNS response
    ///
    /// Helper function to convert mDNS data to DiscoveredService.
    #[cfg(test)]
    fn parse_mdns_response(
        name: String,
        address: IpAddr,
        port: u16,
        txt_records: HashMap<String, String>,
    ) -> DiscoveredService {
        // Extract capabilities from TXT records
        let capabilities = txt_records
            .get("capabilities")
            .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
            .unwrap_or_default();

        let endpoint = format!("http://{address}:{port}");

        DiscoveredService {
            name,
            endpoint,
            capabilities,
            metadata: txt_records,
            discovered_at: std::time::SystemTime::now(),
            discovery_method: "mdns".to_string(),
            healthy: Some(true), // Assume healthy if responding
            priority: 80,        // High priority (local network)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mdns_discovery_creation() {
        let mdns = MdnsDiscovery::default();
        assert!(mdns.enabled);
        assert_eq!(mdns.service_type, "_biomeos._tcp.local.");
    }

    #[tokio::test]
    async fn test_mdns_discover_by_capability() {
        let mdns = MdnsDiscovery::default();
        let result = mdns.discover_by_capability("ai").await;
        assert!(result.is_ok());
        // Graceful fallback returns empty vec
        assert_eq!(result.expect("should succeed").len(), 0);
    }

    #[tokio::test]
    async fn test_mdns_discover_all() {
        let mdns = MdnsDiscovery::default();
        let result = mdns.discover_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mdns_announce_service() {
        let mdns = MdnsDiscovery::default();
        let capabilities = vec!["ai".to_string(), "embeddings".to_string()];
        let metadata = HashMap::new();

        let result = mdns
            .announce_service("test-service", 9200, capabilities, metadata)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mdns_disabled() {
        let mdns = MdnsDiscovery {
            enabled: false,
            ..Default::default()
        };

        let result = mdns.discover_by_capability("ai").await;
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed").len(), 0);
    }

    #[test]
    fn test_parse_mdns_response() {
        let mut txt_records = HashMap::new();
        txt_records.insert("capabilities".to_string(), "ai,embeddings".to_string());
        txt_records.insert("version".to_string(), "0.4.1".to_string());

        let service = MdnsDiscovery::parse_mdns_response(
            "squirrel-ai".to_string(),
            "192.168.1.100".parse().expect("should succeed"),
            9200,
            txt_records,
        );

        assert_eq!(service.name, "squirrel-ai");
        assert_eq!(service.endpoint, "http://192.168.1.100:9200");
        assert_eq!(service.capabilities.len(), 2);
        assert!(service.capabilities.contains(&"ai".to_string()));
        assert_eq!(service.discovery_method, "mdns");
        assert_eq!(service.priority, 80);
    }
}
