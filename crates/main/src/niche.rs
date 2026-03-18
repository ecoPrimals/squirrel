// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Niche self-knowledge for the Squirrel AI primal.
//!
//! Follows the ecoPrimals niche pattern established by groundSpring, wetSpring,
//! and airSpring. Every primal defines its self-knowledge in a single module so
//! that biomeOS, Songbird, and the Pathway Learner can reason about it without
//! hardcoded primal names or port numbers.
//!
//! This module holds:
//! - Identity (who am I?)
//! - Capabilities (what do I expose via biomeOS?)
//! - Semantic mappings (capability domain → handler methods)
//! - Consumed capabilities (what do I need from other primals?)
//! - Dependencies (what primals must be running for me to function?)
//! - Cost estimates (scheduling hints for biomeOS Pathway Learner)
//! - Operation dependencies (parallelization DAG for Pathway Learner)
//!
//! Other modules reference these constants rather than duplicating string
//! literals. Squirrel only knows itself — it discovers other primals at
//! runtime via capability-based discovery through Songbird.

/// Primal identity — used in all JSON-RPC, IPC, and biomeOS interactions.
pub const PRIMAL_ID: &str = "squirrel";

/// Human-readable description for biomeOS registration.
pub const PRIMAL_DESCRIPTION: &str = "Universal AI coordination and MCP routing primal";

/// Primary capability domain.
pub const DOMAIN: &str = "ai";

/// Primal version (tracks crate version).
pub const PRIMAL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// SPDX license identifier.
pub const LICENSE: &str = "AGPL-3.0-only";

/// IPC transport mechanism.
pub const TRANSPORT: &str = "unix_socket";

/// Wire protocol.
pub const PROTOCOL: &str = "jsonrpc_2.0";

/// All capabilities this primal exposes to biomeOS.
///
/// Each string is a fully qualified capability name (`{domain}.{method}`)
/// that biomeOS can route via `capability.call`.
pub const CAPABILITIES: &[&str] = &[
    // AI inference
    "ai.query",
    "ai.complete",
    "ai.chat",
    "ai.list_providers",
    // Capability routing
    "capability.announce",
    "capability.discover",
    "capability.list",
    // System monitoring
    "system.health",
    "system.status",
    "system.metrics",
    "system.ping",
    // Health probes (PRIMAL_IPC_PROTOCOL v3.0)
    "health.liveness",
    "health.readiness",
    // Peer discovery
    "discovery.peers",
    // Tool orchestration
    "tool.execute",
    "tool.list",
    // Context management
    "context.create",
    "context.update",
    "context.summarize",
    // Lifecycle (biomeOS)
    "lifecycle.register",
    "lifecycle.status",
];

/// Semantic mappings: short operation name → fully qualified capability.
///
/// biomeOS uses these during domain registration so
/// `capability.call { domain: "ai", operation: "query" }` routes to
/// the correct JSON-RPC method on our socket.
pub const SEMANTIC_MAPPINGS: &[(&str, &str)] = &[
    ("query", "ai.query"),
    ("complete", "ai.complete"),
    ("chat", "ai.chat"),
    ("list_providers", "ai.list_providers"),
    ("announce", "capability.announce"),
    ("discover", "capability.discover"),
    ("list_capabilities", "capability.list"),
    ("health", "system.health"),
    ("status", "system.status"),
    ("metrics", "system.metrics"),
    ("ping", "system.ping"),
    ("liveness", "health.liveness"),
    ("readiness", "health.readiness"),
    ("peers", "discovery.peers"),
    ("execute", "tool.execute"),
    ("list", "tool.list"),
    ("create", "context.create"),
    ("update", "context.update"),
    ("summarize", "context.summarize"),
    ("register", "lifecycle.register"),
];

/// Consumed capabilities — what Squirrel calls on other primals.
///
/// Squirrel discovers these at runtime via Songbird; it never hardcodes
/// which primal provides them. The Pathway Learner uses this list to
/// ensure required capabilities are available before routing to Squirrel.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    // Security (BearDog)
    "crypto.sign",
    "crypto.verify",
    "auth.validate_token",
    "secrets.store",
    "secrets.retrieve",
    "secrets.list",
    "secrets.delete",
    // Discovery (Songbird)
    "discovery.register",
    "discovery.find_primals",
    "discovery.query",
    // Compute delegation (ToadStool S158b)
    "compute.execute",
    "compute.submit",
    "compute.dispatch.submit",
    "compute.dispatch.status",
    "compute.dispatch.result",
    "compute.dispatch.capabilities",
    "compute.dispatch.cancel",
    "compute.hardware.observe",
    // Storage (NestGate)
    "storage.put",
    "storage.get",
    "storage.list",
    // Model cache (NestGate 4.1)
    "model.register",
    "model.locate",
    "model.metadata",
    "model.exists",
    // MCP tool discovery (domain springs)
    "mcp.tools.list",
    // Health probes (probe other primals before routing)
    "health.liveness",
    "health.readiness",
    // Relay coordination (BearDog)
    "relay.authorize",
    "relay.status",
    // DAG sessions (rhizoCrypt)
    "dag.session.create",
    "dag.event.append",
    "dag.vertex.query",
    // Provenance / attribution (sweetGrass)
    "anchoring.anchor",
    "anchoring.verify",
    "attribution.calculate_rewards",
];

/// Primal dependencies for deployment.
///
/// Each entry: `(primal_id, required, description)`.
/// `required = true` means Squirrel cannot function without it.
/// `required = false` means graceful degradation is supported.
pub const DEPENDENCIES: &[(&str, bool, &str)] = &[
    ("beardog", true, "cryptographic identity and trust"),
    ("songbird", true, "service discovery and IPC mesh"),
    (
        "toadstool",
        false,
        "GPU compute dispatch (graceful fallback to CPU-only inference)",
    ),
    (
        "nestgate",
        false,
        "persistent storage (graceful fallback to in-memory cache)",
    ),
];

/// Cost estimates for biomeOS Pathway Learner scheduling.
///
/// Each entry: `(capability, estimated_ms, gpu_beneficial)`.
/// Times are representative for typical workloads. The Pathway Learner
/// uses these to make intelligent routing decisions.
pub const COST_ESTIMATES: &[(&str, u32, bool)] = &[
    ("ai.query", 500, true),
    ("ai.complete", 500, true),
    ("ai.chat", 800, true),
    ("ai.list_providers", 1, false),
    ("capability.announce", 2, false),
    ("capability.discover", 1, false),
    ("capability.list", 1, false),
    ("system.health", 1, false),
    ("system.status", 1, false),
    ("system.metrics", 5, false),
    ("system.ping", 1, false),
    ("health.liveness", 1, false),
    ("health.readiness", 2, false),
    ("discovery.peers", 50, false),
    ("tool.execute", 200, false),
    ("tool.list", 1, false),
    ("context.create", 5, false),
    ("context.update", 5, false),
    ("context.summarize", 300, true),
    ("lifecycle.register", 10, false),
    ("lifecycle.status", 1, false),
];

/// Operation dependency hints for biomeOS Pathway Learner parallelization.
///
/// Maps each operation to the data inputs it requires, enabling the Pathway
/// Learner to build a DAG and parallelize independent operations.
#[must_use]
pub fn operation_dependencies() -> serde_json::Value {
    serde_json::json!({
        "ai.query": ["prompt"],
        "ai.complete": ["prompt"],
        "ai.chat": ["prompt"],
        "ai.list_providers": [],
        "capability.announce": ["capabilities", "primal"],
        "capability.discover": [],
        "capability.list": [],
        "system.health": [],
        "system.status": [],
        "system.metrics": [],
        "system.ping": [],
        "health.liveness": [],
        "health.readiness": [],
        "discovery.peers": [],
        "tool.execute": ["tool", "args"],
        "tool.list": [],
        "context.create": [],
        "context.update": ["id", "data"],
        "context.summarize": ["id"],
        "lifecycle.register": [],
        "lifecycle.status": [],
    })
}

/// Structured cost estimates as JSON for `capability.list` responses.
///
/// Richer than the static `COST_ESTIMATES` array — includes CPU load hints
/// and memory estimates for Pathway Learner scheduling.
#[must_use]
pub fn cost_estimates_json() -> serde_json::Value {
    serde_json::json!({
        "ai.query":              { "latency_ms": 500, "cpu": "low",    "memory_bytes": 8192,  "gpu_beneficial": true },
        "ai.complete":           { "latency_ms": 500, "cpu": "low",    "memory_bytes": 8192,  "gpu_beneficial": true },
        "ai.chat":               { "latency_ms": 800, "cpu": "low",    "memory_bytes": 16384, "gpu_beneficial": true },
        "ai.list_providers":     { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "capability.announce":   { "latency_ms": 2,   "cpu": "low",    "memory_bytes": 512,   "gpu_beneficial": false },
        "capability.discover":   { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "capability.list":       { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 512,   "gpu_beneficial": false },
        "system.health":         { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "system.status":         { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "system.metrics":        { "latency_ms": 5,   "cpu": "low",    "memory_bytes": 1024,  "gpu_beneficial": false },
        "system.ping":           { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 128,   "gpu_beneficial": false },
        "health.liveness":       { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 128,   "gpu_beneficial": false },
        "health.readiness":      { "latency_ms": 2,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "discovery.peers":       { "latency_ms": 50,  "cpu": "low",    "memory_bytes": 4096,  "gpu_beneficial": false },
        "tool.execute":          { "latency_ms": 200, "cpu": "medium", "memory_bytes": 16384, "gpu_beneficial": false },
        "tool.list":             { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "context.create":        { "latency_ms": 5,   "cpu": "low",    "memory_bytes": 512,   "gpu_beneficial": false },
        "context.update":        { "latency_ms": 5,   "cpu": "low",    "memory_bytes": 1024,  "gpu_beneficial": false },
        "context.summarize":     { "latency_ms": 300, "cpu": "medium", "memory_bytes": 32768, "gpu_beneficial": true },
        "lifecycle.register":    { "latency_ms": 10,  "cpu": "low",    "memory_bytes": 512,   "gpu_beneficial": false },
        "lifecycle.status":      { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
    })
}

/// Semantic mappings as JSON for biomeOS Neural API routing.
#[must_use]
pub fn semantic_mappings_json() -> serde_json::Value {
    serde_json::json!({
        "query":          "ai.query",
        "complete":       "ai.complete",
        "chat":           "ai.chat",
        "list_providers": "ai.list_providers",
        "announce":       "capability.announce",
        "discover":       "capability.discover",
        "health":         "system.health",
        "status":         "system.status",
        "metrics":        "system.metrics",
        "ping":           "system.ping",
        "liveness":       "health.liveness",
        "readiness":      "health.readiness",
        "peers":          "discovery.peers",
        "execute":        "tool.execute",
        "list":           "tool.list",
        "create":         "context.create",
        "update":         "context.update",
        "summarize":      "context.summarize",
        "register":       "lifecycle.register",
    })
}

/// Number of required dependencies.
#[must_use]
pub const fn required_dependency_count() -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < DEPENDENCIES.len() {
        if DEPENDENCIES[i].1 {
            count += 1;
        }
        i += 1;
    }
    count
}

/// Feature gates that expand primal capabilities.
pub const FEATURE_GATES: &[(&str, &str)] = &[
    (
        "direct-http",
        "Direct HTTP AI provider access (dev/testing)",
    ),
    ("marketplace", "Plugin marketplace integration"),
    ("monitoring", "Prometheus-compatible metrics export"),
    ("ecosystem", "Full ecosystem manager with federation"),
    ("nvml", "NVIDIA GPU detection via NVML"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_are_fully_qualified() {
        for cap in CAPABILITIES {
            assert!(
                cap.contains('.'),
                "capability {cap} must be domain.method format"
            );
        }
    }

    #[test]
    fn semantic_mappings_reference_valid_capabilities() {
        for (_, fqn) in SEMANTIC_MAPPINGS {
            assert!(
                CAPABILITIES.contains(fqn),
                "semantic mapping target {fqn} not in CAPABILITIES"
            );
        }
    }

    #[test]
    fn cost_estimates_cover_all_capabilities() {
        let costed: Vec<&str> = COST_ESTIMATES.iter().map(|(c, _, _)| *c).collect();
        for cap in CAPABILITIES {
            assert!(
                costed.contains(cap),
                "capability {cap} missing from COST_ESTIMATES"
            );
        }
    }

    #[test]
    fn required_dependency_count_is_correct() {
        let manual = DEPENDENCIES.iter().filter(|(_, req, _)| *req).count();
        assert_eq!(required_dependency_count(), manual);
    }

    #[test]
    fn operation_dependencies_covers_capabilities() {
        let deps = operation_dependencies();
        let map = deps.as_object().expect("should be an object");
        for cap in CAPABILITIES {
            assert!(
                map.contains_key(*cap),
                "capability {cap} missing from operation_dependencies()"
            );
        }
    }

    #[test]
    fn cost_estimates_json_covers_capabilities() {
        let costs = cost_estimates_json();
        let map = costs.as_object().expect("should be an object");
        for cap in CAPABILITIES {
            assert!(
                map.contains_key(*cap),
                "capability {cap} missing from cost_estimates_json()"
            );
        }
    }

    #[test]
    fn consumed_capabilities_are_fully_qualified() {
        for cap in CONSUMED_CAPABILITIES {
            assert!(
                cap.contains('.'),
                "consumed capability {cap} must be domain.method format"
            );
        }
    }

    #[test]
    fn primal_identity_constants() {
        assert_eq!(PRIMAL_ID, "squirrel");
        assert_eq!(DOMAIN, "ai");
        assert_eq!(TRANSPORT, "unix_socket");
        assert_eq!(PROTOCOL, "jsonrpc_2.0");
        assert_eq!(LICENSE, "AGPL-3.0-only");
    }

    #[test]
    fn identity_primal_domain_matches_niche_domain() {
        assert_eq!(
            universal_constants::identity::PRIMAL_DOMAIN,
            DOMAIN,
            "identity::PRIMAL_DOMAIN must match niche::DOMAIN"
        );
    }

    #[test]
    fn capability_registry_toml_sync() {
        let toml_str = include_str!("../../../capability_registry.toml");
        let toml: toml::Value = toml_str.parse().expect("valid TOML");
        let caps_table = toml
            .get("capabilities")
            .and_then(|v| v.as_table())
            .expect("capabilities table");

        let registry_methods: std::collections::BTreeSet<String> = caps_table
            .values()
            .filter_map(|v| v.get("method").and_then(|m| m.as_str()).map(String::from))
            .collect();

        let niche_methods: std::collections::BTreeSet<String> =
            CAPABILITIES.iter().map(|s| (*s).to_string()).collect();

        let missing_from_toml: Vec<_> = niche_methods.difference(&registry_methods).collect();
        let missing_from_niche: Vec<_> = registry_methods.difference(&niche_methods).collect();

        assert!(
            missing_from_toml.is_empty(),
            "niche::CAPABILITIES has methods not in capability_registry.toml: {missing_from_toml:?}"
        );
        assert!(
            missing_from_niche.is_empty(),
            "capability_registry.toml has methods not in niche::CAPABILITIES: {missing_from_niche:?}"
        );
    }
}
