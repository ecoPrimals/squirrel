// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Niche self-knowledge for the Squirrel AI primal.
//!
//! Follows the ecoPrimals niche pattern established by groundSpring, wetSpring,
//! and airSpring. Every primal defines its self-knowledge in a single module so
//! that biomeOS, the service mesh, and the Pathway Learner can reason about it without
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
//! runtime via capability-based discovery.

use universal_constants::primal_names;

/// Primal identity — used in all JSON-RPC, IPC, and biomeOS interactions.
pub const PRIMAL_ID: &str = "squirrel";

/// Human-readable description for biomeOS registration.
pub const PRIMAL_DESCRIPTION: &str = "Universal AI coordination and MCP routing primal";

/// Primary capability domain.
pub const DOMAIN: &str = "ai";

/// Primal version (tracks crate version).
pub const PRIMAL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// SPDX license identifier.
pub const LICENSE: &str = "AGPL-3.0-or-later";

/// IPC transport mechanism.
pub const TRANSPORT: &str = "unix_socket";

/// Wire protocol.
pub const PROTOCOL: &str = universal_constants::protocol::JSONRPC_PROTOCOL_ID;

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
    // Capability routing (capabilities.list is canonical per SEMANTIC_METHOD_NAMING_STANDARD v2.1)
    "capabilities.list",
    "capability.announce",
    "capability.discover",
    "capability.list",
    // Health probes — canonical per PRIMAL_IPC_PROTOCOL v3.0
    "health.check",
    "health.liveness",
    "health.readiness",
    // System monitoring (backward-compat aliases — prefer health.*)
    "system.health",
    "system.status",
    "system.metrics",
    "system.ping",
    // Identity (CAPABILITY_BASED_DISCOVERY_STANDARD v1.0)
    "identity.get",
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
    // Graph introspection (primalSpring BYOB)
    "graph.parse",
    "graph.validate",
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
    ("list_capabilities", "capabilities.list"),
    ("health_check", "health.check"),
    ("liveness", "health.liveness"),
    ("readiness", "health.readiness"),
    ("health", "system.health"),
    ("status", "system.status"),
    ("metrics", "system.metrics"),
    ("ping", "system.ping"),
    ("identity", "identity.get"),
    ("peers", "discovery.peers"),
    ("execute", "tool.execute"),
    ("list", "tool.list"),
    ("create", "context.create"),
    ("update", "context.update"),
    ("summarize", "context.summarize"),
    ("register", "lifecycle.register"),
    ("parse_graph", "graph.parse"),
    ("validate_graph", "graph.validate"),
];

/// Consumed capabilities — what Squirrel calls on other primals.
///
/// Squirrel discovers these at runtime via capability discovery; it never hardcodes
/// which primal provides them. The Pathway Learner uses this list to
/// ensure required capabilities are available before routing to Squirrel.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    // Security domain (crypto, auth, secrets)
    "crypto.sign",
    "crypto.verify",
    "auth.validate_token",
    "secrets.store",
    "secrets.retrieve",
    "secrets.list",
    "secrets.delete",
    // Discovery domain (service mesh)
    "discovery.register",
    "discovery.find_primals",
    "discovery.query",
    // Compute domain (GPU dispatch, hardware)
    "compute.execute",
    "compute.submit",
    "compute.dispatch.submit",
    "compute.dispatch.status",
    "compute.dispatch.result",
    "compute.dispatch.capabilities",
    "compute.dispatch.cancel",
    "compute.hardware.observe",
    // Storage domain (persistence, model cache)
    "storage.put",
    "storage.get",
    "storage.list",
    "model.register",
    "model.locate",
    "model.metadata",
    "model.exists",
    // MCP tool discovery
    "mcp.tools.list",
    // Health probes (verify peer readiness before routing)
    "health.liveness",
    "health.readiness",
    // Relay domain (authorization, status)
    "relay.authorize",
    "relay.status",
    // DAG domain (sessions, events, vertices)
    "dag.session.create",
    "dag.event.append",
    "dag.vertex.query",
    // Provenance domain (anchoring, attribution)
    "anchoring.anchor",
    "anchoring.verify",
    "attribution.calculate_rewards",
    // Coordination domain (composition, deployment)
    "coordination.validate_composition",
    "coordination.deploy_atomic",
    "composition.nucleus_health",
];

/// Primal dependencies for deployment.
///
/// Each entry: `(primal_id, required, description)`.
/// `required = true` means Squirrel cannot function without it.
/// `required = false` means graceful degradation is supported.
pub const DEPENDENCIES: &[(&str, bool, &str)] = &[
    (
        primal_names::BEARDOG,
        true,
        "cryptographic identity and trust",
    ),
    (
        primal_names::SONGBIRD,
        true,
        "service discovery and IPC mesh",
    ),
    (
        primal_names::TOADSTOOL,
        false,
        "GPU compute dispatch (graceful fallback to CPU-only inference)",
    ),
    (
        primal_names::NESTGATE,
        false,
        "persistent storage (graceful fallback to in-memory cache)",
    ),
    (
        primal_names::PRIMALSPRING,
        false,
        "coordination validation and BYOB graph execution",
    ),
    (
        primal_names::PETALTONGUE,
        false,
        "visualization and user interface rendering",
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
    ("capabilities.list", 1, false),
    ("capability.announce", 2, false),
    ("capability.discover", 1, false),
    ("capability.list", 1, false),
    ("health.check", 1, false),
    ("health.liveness", 1, false),
    ("health.readiness", 2, false),
    ("system.health", 1, false),
    ("system.status", 1, false),
    ("system.metrics", 5, false),
    ("system.ping", 1, false),
    ("identity.get", 1, false),
    ("discovery.peers", 50, false),
    ("tool.execute", 200, false),
    ("tool.list", 1, false),
    ("context.create", 5, false),
    ("context.update", 5, false),
    ("context.summarize", 300, true),
    ("lifecycle.register", 10, false),
    ("lifecycle.status", 1, false),
    ("graph.parse", 5, false),
    ("graph.validate", 50, false),
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
        "capabilities.list": [],
        "capability.announce": ["capabilities", "primal"],
        "capability.discover": [],
        "capability.list": [],
        "health.check": [],
        "health.liveness": [],
        "health.readiness": [],
        "system.health": [],
        "system.status": [],
        "system.metrics": [],
        "system.ping": [],
        "identity.get": [],
        "discovery.peers": [],
        "tool.execute": ["tool", "args"],
        "tool.list": [],
        "context.create": [],
        "context.update": ["id", "data"],
        "context.summarize": ["id"],
        "lifecycle.register": [],
        "lifecycle.status": [],
        "graph.parse": ["graph_toml"],
        "graph.validate": ["graph_toml"],
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
        "capabilities.list":     { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 512,   "gpu_beneficial": false },
        "capability.announce":   { "latency_ms": 2,   "cpu": "low",    "memory_bytes": 512,   "gpu_beneficial": false },
        "capability.discover":   { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "capability.list":       { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 512,   "gpu_beneficial": false },
        "health.check":          { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "health.liveness":       { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 128,   "gpu_beneficial": false },
        "health.readiness":      { "latency_ms": 2,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "system.health":         { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "system.status":         { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "system.metrics":        { "latency_ms": 5,   "cpu": "low",    "memory_bytes": 1024,  "gpu_beneficial": false },
        "system.ping":           { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 128,   "gpu_beneficial": false },
        "identity.get":          { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "discovery.peers":       { "latency_ms": 50,  "cpu": "low",    "memory_bytes": 4096,  "gpu_beneficial": false },
        "tool.execute":          { "latency_ms": 200, "cpu": "medium", "memory_bytes": 16384, "gpu_beneficial": false },
        "tool.list":             { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "context.create":        { "latency_ms": 5,   "cpu": "low",    "memory_bytes": 512,   "gpu_beneficial": false },
        "context.update":        { "latency_ms": 5,   "cpu": "low",    "memory_bytes": 1024,  "gpu_beneficial": false },
        "context.summarize":     { "latency_ms": 300, "cpu": "medium", "memory_bytes": 32768, "gpu_beneficial": true },
        "lifecycle.register":    { "latency_ms": 10,  "cpu": "low",    "memory_bytes": 512,   "gpu_beneficial": false },
        "lifecycle.status":      { "latency_ms": 1,   "cpu": "low",    "memory_bytes": 256,   "gpu_beneficial": false },
        "graph.parse":           { "latency_ms": 5,   "cpu": "low",    "memory_bytes": 2048,  "gpu_beneficial": false },
        "graph.validate":        { "latency_ms": 50,  "cpu": "low",    "memory_bytes": 4096,  "gpu_beneficial": false },
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
        "list_capabilities": "capabilities.list",
        "health_check":   "health.check",
        "liveness":       "health.liveness",
        "readiness":      "health.readiness",
        "health":         "system.health",
        "status":         "system.status",
        "metrics":        "system.metrics",
        "ping":           "system.ping",
        "identity":       "identity.get",
        "peers":          "discovery.peers",
        "execute":        "tool.execute",
        "list":           "tool.list",
        "create":         "context.create",
        "update":         "context.update",
        "summarize":      "context.summarize",
        "register":       "lifecycle.register",
        "parse_graph":    "graph.parse",
        "validate_graph": "graph.validate",
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
        assert_eq!(PROTOCOL, universal_constants::protocol::JSONRPC_PROTOCOL_ID);
        assert_eq!(LICENSE, "AGPL-3.0-or-later");
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

    #[test]
    fn semantic_mappings_json_matches_semantic_mappings_table() {
        let json = semantic_mappings_json();
        let map = json.as_object().expect("object");
        for (short, expected) in SEMANTIC_MAPPINGS {
            assert_eq!(
                map.get(*short).and_then(|v| v.as_str()),
                Some(*expected),
                "short key {short} should map to {expected}"
            );
        }
    }

    #[test]
    fn feature_gates_are_non_empty_descriptions() {
        for (gate, desc) in FEATURE_GATES {
            assert!(!gate.is_empty());
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn cost_estimates_json_has_latency_and_gpu_keys() {
        let costs = cost_estimates_json();
        let map = costs.as_object().expect("object");
        let q = map.get("ai.query").expect("ai.query");
        let obj = q.as_object().expect("inner object");
        assert!(obj.contains_key("latency_ms"));
        assert!(obj.contains_key("gpu_beneficial"));
    }

    #[test]
    fn consumed_capabilities_include_compute_and_storage() {
        let joined = CONSUMED_CAPABILITIES.join(" ");
        assert!(joined.contains("compute.execute"));
        assert!(joined.contains("storage.get"));
    }

    #[test]
    fn dependencies_name_beardog_and_songbird_required() {
        let beardog = DEPENDENCIES.iter().find(|(id, _, _)| *id == "beardog");
        let songbird = DEPENDENCIES.iter().find(|(id, _, _)| *id == "songbird");
        assert_eq!(beardog.map(|(_, r, _)| *r), Some(true));
        assert_eq!(songbird.map(|(_, r, _)| *r), Some(true));
    }
}
