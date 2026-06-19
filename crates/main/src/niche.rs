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

use universal_constants::capabilities as cap_ids;

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
    // AI inference (legacy domain)
    "ai.query",
    "ai.complete",
    "ai.chat",
    "ai.list_providers",
    // Signal planning (Neural API composition collapse)
    "signal.plan",
    // Inference domain — vendor-agnostic wire standard (ecoPrimal)
    "inference.complete",
    "inference.embed",
    "inference.models",
    "inference.register_provider",
    "inference.unregister_provider",
    // Capability routing (capabilities.list is canonical per SEMANTIC_METHOD_NAMING_STANDARD v2.1)
    "capabilities.list",
    "capabilities.announce",
    "capability.announce",
    "capability.discover",
    "capability.list",
    // Self-registration (stadial standard)
    "primal.announce",
    // Health probes — canonical per PRIMAL_IPC_PROTOCOL v3.0
    "health",
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
    // Provider management (runtime registration)
    "provider.register",
    "provider.list",
    "provider.deregister",
    // BTSP binary transport negotiation
    "btsp.negotiate",
    // Lifecycle (biomeOS)
    "lifecycle.register",
    "lifecycle.status",
    // Graph introspection (primalSpring BYOB)
    "graph.parse",
    "graph.validate",
];

/// Human-readable descriptions for each capability domain group.
///
/// Used by `capabilities.list` responses so consumers understand what each
/// domain namespace provides without needing external documentation.
pub const CAPABILITY_GROUP_DESCRIPTIONS: &[(&str, &str)] = &[
    ("ai", "AI inference coordination and provider management"),
    ("inference", "Vendor-agnostic inference wire standard"),
    ("capabilities", "Capability introspection and routing"),
    ("capability", "Capability announcement and discovery"),
    ("primal", "Self-registration and announcements"),
    ("health", "Health probes (liveness, readiness)"),
    ("system", "System monitoring and diagnostics"),
    ("identity", "Primal identity resolution"),
    ("signal", "Neural API composition and signal planning"),
    ("discovery", "Peer discovery and mesh routing"),
    ("tool", "Tool orchestration and execution"),
    ("context", "Context lifecycle management"),
    ("provider", "Runtime provider registration and management"),
    (
        "btsp",
        "Binary Transport Specification Protocol negotiation",
    ),
    ("lifecycle", "Primal lifecycle registration"),
    ("graph", "Graph introspection and validation"),
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
    ("signal_plan", "signal.plan"),
    ("announce", "capability.announce"),
    ("capabilities_announce", "capabilities.announce"),
    ("primal_announce", "primal.announce"),
    ("discover", "capability.discover"),
    ("list_capabilities", "capabilities.list"),
    ("health_check", "health.check"),
    ("liveness", "health.liveness"),
    ("readiness", "health.readiness"),
    ("health", "health"),
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
    ("inference_complete", "inference.complete"),
    ("inference_embed", "inference.embed"),
    ("inference_models", "inference.models"),
    ("register_provider", "inference.register_provider"),
    ("unregister_provider", "inference.unregister_provider"),
    ("provider_register", "provider.register"),
    ("provider_list", "provider.list"),
    ("provider_deregister", "provider.deregister"),
    ("btsp_negotiate", "btsp.negotiate"),
    ("register", "lifecycle.register"),
    ("lifecycle_status", "lifecycle.status"),
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
    // IPC domain (peer registration and heartbeat)
    "ipc.register",
    "ipc.heartbeat",
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

/// Required capabilities for deployment (TRUE PRIMAL: capability-based, no primal names).
///
/// Each entry: `(capability_id, required, description)`.
/// `required = true` means Squirrel cannot function without a provider for this capability.
/// `required = false` means graceful degradation is supported.
///
/// Squirrel discovers providers at runtime via capability-based discovery — it never
/// hardcodes which primal supplies a given capability.
pub const REQUIRED_CAPABILITIES: &[(&str, bool, &str)] = &[
    (
        cap_ids::SECURITY_CAPABILITY,
        true,
        "cryptographic identity and trust",
    ),
    (
        cap_ids::SERVICE_MESH_CAPABILITY,
        true,
        "service discovery and IPC mesh",
    ),
    (
        cap_ids::COMPUTE_CAPABILITY,
        false,
        "GPU compute dispatch (graceful fallback to CPU-only inference)",
    ),
    (
        cap_ids::STORAGE_CAPABILITY,
        false,
        "persistent storage (graceful fallback to in-memory cache)",
    ),
    (
        cap_ids::COORDINATION_CAPABILITY,
        false,
        "coordination validation and BYOB graph execution",
    ),
    (
        cap_ids::UI_CAPABILITY,
        false,
        "visualization and user interface rendering",
    ),
];

/// Legacy primal dependencies for backward compatibility.
///
/// **Deprecated**: Use [`REQUIRED_CAPABILITIES`] instead. Squirrel discovers
/// providers at runtime by capability, not by primal name.
#[deprecated(
    since = "3.0.0",
    note = "Use REQUIRED_CAPABILITIES for capability-based discovery"
)]
pub const DEPENDENCIES: &[(&str, bool, &str)] = &[
    (
        universal_constants::primal_names::BEARDOG,
        true,
        "cryptographic identity and trust",
    ),
    (
        universal_constants::primal_names::SONGBIRD,
        true,
        "service discovery and IPC mesh",
    ),
    (
        universal_constants::primal_names::TOADSTOOL,
        false,
        "GPU compute dispatch (graceful fallback to CPU-only inference)",
    ),
    (
        universal_constants::primal_names::NESTGATE,
        false,
        "persistent storage (graceful fallback to in-memory cache)",
    ),
    (
        universal_constants::primal_names::PRIMALSPRING,
        false,
        "coordination validation and BYOB graph execution",
    ),
    (
        universal_constants::primal_names::PETALTONGUE,
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
    ("inference.complete", 500, true),
    ("inference.embed", 300, true),
    ("inference.models", 1, false),
    ("inference.register_provider", 5, false),
    ("inference.unregister_provider", 5, false),
    ("signal.plan", 1000, true),
    ("capabilities.list", 1, false),
    ("capabilities.announce", 2, false),
    ("capability.announce", 2, false),
    ("capability.discover", 1, false),
    ("capability.list", 1, false),
    ("primal.announce", 2, false),
    ("health", 1, false),
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
    ("provider.register", 5, false),
    ("provider.list", 1, false),
    ("provider.deregister", 5, false),
    ("btsp.negotiate", 10, false),
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
        "inference.complete": ["prompt"],
        "inference.embed": ["input"],
        "inference.models": [],
        "inference.register_provider": ["provider_id", "socket"],
        "inference.unregister_provider": ["provider_id"],
        "signal.plan": ["prompt", "tools"],
        "capabilities.list": [],
        "capabilities.announce": ["capabilities", "primal"],
        "capability.announce": ["capabilities", "primal"],
        "capability.discover": [],
        "capability.list": [],
        "primal.announce": ["capabilities", "primal"],
        "health": [],
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
        "provider.register": ["provider_id", "socket", "capabilities"],
        "provider.list": [],
        "provider.deregister": ["provider_id"],
        "btsp.negotiate": ["session_id", "preferred_cipher"],
        "lifecycle.register": [],
        "lifecycle.status": [],
        "graph.parse": ["graph_toml"],
        "graph.validate": ["graph_toml"],
    })
}

/// Structured cost estimates as JSON for `capability.list` responses.
///
/// Built from `COST_ESTIMATES` with enriched CPU/memory hints for the Pathway
/// Learner. GPU-beneficial operations get `"medium"` memory; others scale
/// from latency.
#[must_use]
pub fn cost_estimates_json() -> serde_json::Value {
    let mut map = serde_json::Map::with_capacity(COST_ESTIMATES.len());
    for &(cap, latency_ms, gpu) in COST_ESTIMATES {
        let cpu = if latency_ms >= 200 { "medium" } else { "low" };
        let memory_bytes: u64 = if gpu {
            if latency_ms >= 500 { 32768 } else { 8192 }
        } else if latency_ms >= 200 {
            16384
        } else if latency_ms >= 5 {
            1024
        } else {
            256
        };
        map.insert(
            cap.to_string(),
            serde_json::json!({
                "latency_ms": latency_ms,
                "cpu": cpu,
                "memory_bytes": memory_bytes,
                "gpu_beneficial": gpu,
            }),
        );
    }
    serde_json::Value::Object(map)
}

/// Semantic mappings as JSON for biomeOS Neural API routing.
#[must_use]
pub fn semantic_mappings_json() -> serde_json::Value {
    serde_json::json!({
        "query":          "ai.query",
        "complete":       "ai.complete",
        "chat":           "ai.chat",
        "list_providers": "ai.list_providers",
        "signal_plan":    "signal.plan",
        "announce":       "capability.announce",
        "capabilities_announce": "capabilities.announce",
        "primal_announce": "primal.announce",
        "discover":       "capability.discover",
        "list_capabilities": "capabilities.list",
        "health_check":   "health.check",
        "liveness":       "health.liveness",
        "readiness":      "health.readiness",
        "health":         "health",
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
        "inference_complete":    "inference.complete",
        "inference_embed":       "inference.embed",
        "inference_models":      "inference.models",
        "register_provider":     "inference.register_provider",
        "unregister_provider":   "inference.unregister_provider",
        "provider_register":     "provider.register",
        "provider_list":         "provider.list",
        "provider_deregister":   "provider.deregister",
        "btsp_negotiate":        "btsp.negotiate",
        "register":              "lifecycle.register",
        "lifecycle_status":      "lifecycle.status",
        "parse_graph":           "graph.parse",
        "validate_graph":        "graph.validate",
    })
}

/// Number of required capabilities.
#[must_use]
pub const fn required_capability_count() -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < REQUIRED_CAPABILITIES.len() {
        if REQUIRED_CAPABILITIES[i].1 {
            count += 1;
        }
        i += 1;
    }
    count
}

/// Legacy alias.
#[deprecated(since = "3.0.0", note = "Use required_capability_count()")]
#[must_use]
pub const fn required_dependency_count() -> usize {
    required_capability_count()
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
        // Wave 113 guidestone: bare "health" is the mandatory probe method
        // and is the sole exception to the domain.method naming rule.
        const BARE_METHOD_EXCEPTIONS: &[&str] = &["health"];

        for cap in CAPABILITIES {
            assert!(
                cap.contains('.') || BARE_METHOD_EXCEPTIONS.contains(cap),
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
    fn required_capability_count_is_correct() {
        let manual = REQUIRED_CAPABILITIES
            .iter()
            .filter(|(_, req, _)| *req)
            .count();
        assert_eq!(required_capability_count(), manual);
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
        let toml_str = include_str!("../../../config/capability_registry.toml");
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
    fn required_capabilities_include_security_and_mesh() {
        let security = REQUIRED_CAPABILITIES
            .iter()
            .find(|(id, _, _)| *id == cap_ids::SECURITY_CAPABILITY);
        let mesh = REQUIRED_CAPABILITIES
            .iter()
            .find(|(id, _, _)| *id == cap_ids::SERVICE_MESH_CAPABILITY);
        assert_eq!(security.map(|(_, r, _)| *r), Some(true));
        assert_eq!(mesh.map(|(_, r, _)| *r), Some(true));
    }

    #[test]
    fn required_capabilities_are_defined_in_universal_constants() {
        let known = [
            cap_ids::SECURITY_CAPABILITY,
            cap_ids::SERVICE_MESH_CAPABILITY,
            cap_ids::COMPUTE_CAPABILITY,
            cap_ids::STORAGE_CAPABILITY,
            cap_ids::COORDINATION_CAPABILITY,
            cap_ids::UI_CAPABILITY,
        ];
        for (cap_id, _, _) in REQUIRED_CAPABILITIES {
            assert!(
                known.contains(cap_id),
                "required capability '{cap_id}' is not a known capability constant"
            );
        }
    }
}
