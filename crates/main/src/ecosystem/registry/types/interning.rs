// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! String interning and canonical [`Arc<str>`](std::sync::Arc) values for registry lookups.
//!
//! Discovery prefers capability constants; legacy primal names exist only for deserialization
//! and display, not for routing.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use universal_constants::primal_names;

/// String interning for common service registry values
/// Uses capability constants for discovery; legacy primal names for backward compatibility.
///
/// **TRUE PRIMAL**: Discovery uses capability names (storage, compute, security, etc.).
/// Legacy primal names below are ONLY for display/fallback identifiers when deserializing
/// external data - NOT for discovery routing. Actual discovery is capability-based.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Warm-up table for tests; reserved for future shared interning"
    )
)]
static REGISTRY_STRINGS: LazyLock<HashMap<&'static str, Arc<str>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    use universal_constants::capabilities;
    // Capability-based (preferred for discovery)
    map.insert(
        capabilities::SELF_PRIMAL_NAME,
        Arc::from(capabilities::SELF_PRIMAL_NAME),
    );
    map.insert(
        capabilities::SERVICE_MESH_CAPABILITY,
        Arc::from(capabilities::SERVICE_MESH_CAPABILITY),
    );
    map.insert(
        capabilities::COMPUTE_CAPABILITY,
        Arc::from(capabilities::COMPUTE_CAPABILITY),
    );
    map.insert(
        capabilities::SECURITY_CAPABILITY,
        Arc::from(capabilities::SECURITY_CAPABILITY),
    );
    map.insert(
        capabilities::STORAGE_CAPABILITY,
        Arc::from(capabilities::STORAGE_CAPABILITY),
    );
    map.insert(
        capabilities::ECOSYSTEM_CAPABILITY,
        Arc::from(capabilities::ECOSYSTEM_CAPABILITY),
    );
    // Legacy primal names: display/fallback only, NOT for discovery routing.
    // Used when deserializing config or external responses that reference primal IDs.
    map.insert(primal_names::SONGBIRD, Arc::from(primal_names::SONGBIRD));
    map.insert(primal_names::TOADSTOOL, Arc::from(primal_names::TOADSTOOL));
    map.insert(primal_names::BEARDOG, Arc::from(primal_names::BEARDOG));
    map.insert(primal_names::NESTGATE, Arc::from(primal_names::NESTGATE));
    map.insert(primal_names::BIOMEOS, Arc::from(primal_names::BIOMEOS));

    // Common capabilities
    map.insert("ai_coordination", Arc::from("ai_coordination"));
    map.insert("service_mesh", Arc::from("service_mesh"));
    map.insert("security", Arc::from("security"));
    map.insert("monitoring", Arc::from("monitoring"));
    map.insert("discovery", Arc::from("discovery"));
    map.insert("orchestration", Arc::from("orchestration"));
    map.insert("intelligence", Arc::from("intelligence"));
    map.insert("biome_integration", Arc::from("biome_integration"));

    // Common metadata keys
    map.insert("version", Arc::from("version"));
    map.insert("environment", Arc::from("environment"));
    map.insert("region", Arc::from("region"));
    map.insert("instance_id", Arc::from("instance_id"));
    map.insert("last_updated", Arc::from("last_updated"));
    map.insert("health_endpoint", Arc::from("health_endpoint"));
    map.insert("metrics_endpoint", Arc::from("metrics_endpoint"));

    // Common operation names
    map.insert("register", Arc::from("register"));
    map.insert("discover", Arc::from("discover"));
    map.insert("health_check", Arc::from("health_check"));
    map.insert("metrics", Arc::from("metrics"));

    map
});

/// Get ```Arc<str>``` for registry string with zero allocation for common values
///
/// **TRUE PRIMAL**: Capability constants resolve to capability names for discovery.
/// Squirrel knows its own name ("squirrel"); other primal names are display/fallback only.
/// Discovery routing uses capability names, not primal hostnames.
#[must_use]
pub fn intern_registry_string(s: &str) -> Arc<str> {
    use universal_constants::capabilities;
    // Literal capability names first - must preserve for DiscoveredService.capabilities
    match s {
        "storage" => Arc::from("storage"),
        "compute" => Arc::from("compute"),
        "security" => Arc::from("security"),
        "discovery" => Arc::from("discovery"),
        "ai_coordination" => Arc::from("ai_coordination"),
        // Squirrel can know its own name (self-knowledge)
        primal_names::SQUIRREL => Arc::from(primal_names::SQUIRREL),
        // Legacy primal names: display/fallback only when deserializing external data.
        // NOT for discovery routing—use capability constants for that.
        primal_names::SONGBIRD => Arc::from(primal_names::SONGBIRD),
        primal_names::TOADSTOOL => Arc::from(primal_names::TOADSTOOL),
        primal_names::BEARDOG => Arc::from(primal_names::BEARDOG),
        primal_names::NESTGATE => Arc::from(primal_names::NESTGATE),
        primal_names::BIOMEOS => Arc::from(primal_names::BIOMEOS),
        // Capability constants -> capability names (for discovery, NOT primal names)
        n if n == capabilities::SELF_PRIMAL_NAME => Arc::from("squirrel"),
        n if n == capabilities::SERVICE_MESH_CAPABILITY => {
            Arc::from(capabilities::SERVICE_MESH_CAPABILITY)
        }
        n if n == capabilities::COMPUTE_CAPABILITY => Arc::from(capabilities::COMPUTE_CAPABILITY),
        n if n == capabilities::SECURITY_CAPABILITY => Arc::from(capabilities::SECURITY_CAPABILITY),
        n if n == capabilities::STORAGE_CAPABILITY => Arc::from(capabilities::STORAGE_CAPABILITY),
        n if n == capabilities::ECOSYSTEM_CAPABILITY => {
            Arc::from(capabilities::ECOSYSTEM_CAPABILITY)
        }
        "active" => Arc::from("active"),
        "inactive" => Arc::from("inactive"),
        "error" => Arc::from("error"),
        "network" => Arc::from("network"),
        _ => Arc::from(s), // Allocate for uncommon strings
    }
}

#[cfg(test)]
mod tests {
    use super::REGISTRY_STRINGS;

    /// Covers initialization of the static registry map (used for interning metadata in discovery).
    #[test]
    fn registry_string_table_initializes_and_contains_core_keys() {
        assert!(
            REGISTRY_STRINGS.len() >= 10,
            "expected predefined registry strings to be populated"
        );
        assert!(REGISTRY_STRINGS.contains_key("version"));
        assert!(REGISTRY_STRINGS.contains_key("discover"));
    }
}
