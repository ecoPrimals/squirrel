// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Environment variable-based discovery
//!
//! Highest priority discovery mechanism. Allows users to override
//! service locations via environment variables.
//!
//! ## Environment Variable Format
//!
//! `{CAPABILITY}_ENDPOINT` - Full endpoint URL
//! Example: `AI_ENDPOINT=http://192.168.1.100:9200`

use crate::discovery::types::DiscoveredService;
use std::collections::HashMap;
use std::env;
use tracing::debug;

/// Discover service via environment variable
///
/// Looks for `{CAPABILITY}_ENDPOINT` environment variable.
///
/// # Errors
///
/// Returns `None` if environment variable not set
pub fn discover_from_env(capability: &str) -> Option<DiscoveredService> {
    let env_var = format!("{}_ENDPOINT", capability.to_uppercase());

    if let Ok(endpoint) = env::var(&env_var) {
        debug!("✓ Found {} = {}", env_var, endpoint);

        Some(DiscoveredService {
            name: format!("{capability}-provider"),
            endpoint,
            capabilities: vec![capability.to_string()],
            metadata: HashMap::new(),
            discovered_at: std::time::SystemTime::now(),
            discovery_method: "environment_variable".to_string(),
            healthy: None,
            priority: 100, // Highest priority
        })
    } else {
        None
    }
}

/// Discover all services from environment variables
///
/// Scans all environment variables for `*_ENDPOINT` patterns.
pub fn discover_all_from_env() -> Vec<DiscoveredService> {
    let mut services = Vec::new();

    for (key, value) in env::vars() {
        if key.ends_with("_ENDPOINT") {
            // Extract capability name (remove _ENDPOINT suffix)
            if let Some(capability) = key.strip_suffix("_ENDPOINT") {
                let capability = capability.to_lowercase();

                services.push(DiscoveredService {
                    name: format!("{capability}-provider"),
                    endpoint: value,
                    capabilities: vec![capability.clone()],
                    metadata: HashMap::new(),
                    discovered_at: std::time::SystemTime::now(),
                    discovery_method: "environment_variable".to_string(),
                    healthy: None,
                    priority: 100,
                });
            }
        }
    }

    debug!("Discovered {} services from environment", services.len());
    services
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_from_env() {
        unsafe { env::set_var("TEST_CAPABILITY_ENDPOINT", "http://test.example.com:8080") };

        let service = discover_from_env("test_capability");
        assert!(service.is_some());

        let service = service.unwrap();
        assert_eq!(service.endpoint, "http://test.example.com:8080");
        assert_eq!(service.priority, 100);

        unsafe { env::remove_var("TEST_CAPABILITY_ENDPOINT") };
    }

    #[test]
    fn test_discover_not_found() {
        let service = discover_from_env("nonexistent_capability");
        assert!(service.is_none());
    }

    #[test]
    fn test_discover_all() {
        unsafe { env::set_var("AI_ENDPOINT", "http://ai.example.com:9200") };
        unsafe { env::set_var("STORAGE_ENDPOINT", "http://storage.example.com:8500") };

        let services = discover_all_from_env();

        // Should find at least our test services
        assert!(services.iter().any(|s| s.has_capability("ai")));
        assert!(services.iter().any(|s| s.has_capability("storage")));

        unsafe { env::remove_var("AI_ENDPOINT") };
        unsafe { env::remove_var("STORAGE_ENDPOINT") };
    }
}
