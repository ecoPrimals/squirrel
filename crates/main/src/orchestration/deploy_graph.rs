// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Deployment graph types for biomeOS multi-primal composition.
//!
//! Wire-compatible with ludoSpring exp054 `DeploymentGraphDef` and the
//! biomeOS `graphs/*.toml` format. Squirrel uses these types to:
//!
//! - Parse deployment graphs received from biomeOS
//! - Understand node capability requirements
//! - Participate in multi-primal coordination workflows
//!
//! Zero chimeric dependencies: these are local protocol types, not imports
//! from ludoSpring. Both sides speak the same JSON-RPC / TOML wire format.

use serde::{Deserialize, Serialize};

/// A node in the deployment graph.
///
/// Matches the biomeOS `[[graph.nodes]]` TOML wire format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node identifier.
    pub id: String,
    /// Human-readable node name.
    pub name: String,
    /// Node IDs this node depends on (topological ordering).
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Capability to invoke (e.g. `"ai.query"`, `"game.player_input"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    /// Node ID to feed output back to on next tick (continuous graphs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_to: Option<String>,
    /// Per-tick budget in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_ms: Option<f64>,
}

/// Continuous tick configuration for real-time graphs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickConfig {
    /// Target tick rate in Hz (e.g. 20.0 for 50ms ticks).
    pub target_hz: f64,
    /// Maximum accumulator before frame-skipping (ms).
    pub max_accumulator_ms: f64,
    /// Log warning when a node exceeds this budget (ms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_warning_ms: Option<f64>,
}

/// Full deployment graph definition.
///
/// Matches the biomeOS `[graph]` TOML wire format and ludoSpring exp054
/// protocol. Coordination modes:
///
/// - `"sequential"` — nodes run in topological order
/// - `"parallel"` — independent nodes run concurrently
/// - `"pipeline"` — streaming data between stages
/// - `"continuous"` — tick-based with feedback loops (games, simulations)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentGraphDef {
    /// Graph identifier.
    pub id: String,
    /// Human-readable graph name.
    pub name: String,
    /// Version string (semver).
    pub version: String,
    /// Coordination mode.
    pub coordination: String,
    /// Tick configuration (only for `"continuous"` coordination).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick: Option<TickConfig>,
    /// Graph nodes in definition order.
    pub nodes: Vec<GraphNode>,
}

impl DeploymentGraphDef {
    /// Compute topological execution order based on `depends_on` edges.
    ///
    /// Returns node IDs in a valid execution order, or `Err` if the graph
    /// contains a cycle.
    pub fn execution_order(&self) -> Result<Vec<String>, String> {
        use std::collections::{HashMap, VecDeque};

        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        for node in &self.nodes {
            in_degree.entry(node.id.as_str()).or_insert(0);
            adjacency.entry(node.id.as_str()).or_default();
            for dep in &node.depends_on {
                adjacency.entry(dep.as_str()).or_default().push(&node.id);
                *in_degree.entry(node.id.as_str()).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());

        while let Some(id) = queue.pop_front() {
            order.push(id.to_string());

            if let Some(dependents) = adjacency.get(id) {
                for &dep in dependents {
                    if let Some(deg) = in_degree.get_mut(dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dep);
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err("cycle detected in deployment graph".into());
        }

        Ok(order)
    }

    /// Return nodes that require a specific capability.
    #[must_use]
    pub fn nodes_requiring(&self, capability: &str) -> Vec<&GraphNode> {
        self.nodes
            .iter()
            .filter(|n| n.capability.as_deref() == Some(capability))
            .collect()
    }

    /// Whether this graph references any AI capabilities that Squirrel provides.
    #[must_use]
    pub fn requires_squirrel(&self) -> bool {
        self.nodes.iter().any(|n| {
            n.capability
                .as_deref()
                .is_some_and(|c| c.starts_with("ai."))
        })
    }
}

// ── primalSpring-compatible deploy graph types ──────────────────────────────
//
// primalSpring uses `[graph]` + `[[graph.node]]` TOML for BYOB niche
// deployment. These types are wire-compatible with primalSpring's
// `deploy.rs` so Squirrel can both produce and consume these graphs.

/// A parsed primalSpring-compatible deploy graph (`[graph]` format).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NicheDeployGraph {
    /// Top-level graph metadata and nodes.
    pub graph: NicheGraphMeta,
}

/// Graph metadata in the primalSpring `[graph]` wire format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NicheGraphMeta {
    /// Graph name (e.g. `"squirrel_ai_coordination_niche"`).
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Semantic version.
    #[serde(default)]
    pub version: String,
    /// Coordination pattern: Sequential, Parallel, Pipeline, Continuous, ConditionalDag.
    #[serde(default)]
    pub coordination: Option<String>,
    /// Ordered list of primal/service nodes.
    #[serde(default)]
    pub node: Vec<NicheGraphNode>,
}

/// A single node in a primalSpring-compatible deploy graph.
///
/// Wire-compatible with primalSpring's `deploy::GraphNode`:
/// `name`, `binary`, `order`, `required`, `depends_on`, `health_method`,
/// `by_capability`, `capabilities`, `condition`, `skip_if`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NicheGraphNode {
    /// Primal name (e.g. `"squirrel"`).
    pub name: String,
    /// Binary to invoke (e.g. `"squirrel"`).
    pub binary: String,
    /// Startup order (1-indexed).
    pub order: u32,
    /// Whether the deployment fails if this node can't start.
    #[serde(default)]
    pub required: bool,
    /// Nodes that must be healthy before this one starts.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// JSON-RPC method name for health probing.
    #[serde(default)]
    pub health_method: String,
    /// Capability routing key (e.g. `"ai"`, `"crypto"`).
    #[serde(default)]
    pub by_capability: Option<String>,
    /// Capabilities this node provides.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Condition predicate for `ConditionalDag` execution.
    #[serde(default)]
    pub condition: Option<String>,
    /// Skip predicate for `ConditionalDag` execution.
    #[serde(default)]
    pub skip_if: Option<String>,
}

impl NicheDeployGraph {
    /// Load from a TOML string.
    ///
    /// # Errors
    ///
    /// Returns parse error description.
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        toml::from_str(toml_str).map_err(|e| format!("TOML parse error: {e}"))
    }

    /// Load from a file path.
    ///
    /// # Errors
    ///
    /// Returns IO or parse error description.
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_toml(&contents)
    }

    /// Structurally validate this graph. Returns a list of issues (empty = clean).
    #[must_use]
    pub fn structural_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let g = &self.graph;

        if g.name.is_empty() {
            issues.push("graph.name is empty".into());
        }
        if g.node.is_empty() {
            issues.push("graph has no nodes".into());
        }

        let names: Vec<&str> = g.node.iter().map(|n| n.name.as_str()).collect();
        for node in &g.node {
            if node.name.is_empty() {
                issues.push(format!("node at order {} has empty name", node.order));
            }
            if node.binary.is_empty() {
                issues.push(format!("node '{}' has empty binary", node.name));
            }
            if node.health_method.is_empty() {
                issues.push(format!("node '{}' has no health_method", node.name));
            }
            for dep in &node.depends_on {
                if !names.contains(&dep.as_str()) {
                    issues.push(format!(
                        "node '{}' depends on '{}' which is not in the graph",
                        node.name, dep
                    ));
                }
            }
        }

        let mut orders: Vec<u32> = g.node.iter().map(|n| n.order).collect();
        orders.sort_unstable();
        let unique_count = {
            orders.dedup();
            orders.len()
        };
        if unique_count != g.node.len() {
            issues.push("duplicate order values in graph nodes".into());
        }

        issues
    }

    /// Required node count.
    #[must_use]
    pub fn required_count(&self) -> usize {
        self.graph.node.iter().filter(|n| n.required).count()
    }

    /// Find nodes providing a specific capability.
    #[must_use]
    pub fn nodes_with_capability(&self, cap: &str) -> Vec<&NicheGraphNode> {
        self.graph
            .node
            .iter()
            .filter(|n| n.capabilities.iter().any(|c| c == cap))
            .collect()
    }

    /// Whether this graph includes Squirrel (via `by_capability = "ai"` or node name).
    #[must_use]
    pub fn includes_squirrel(&self) -> bool {
        self.graph
            .node
            .iter()
            .any(|n| n.name == "squirrel" || n.by_capability.as_deref() == Some("ai"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_graph() -> DeploymentGraphDef {
        DeploymentGraphDef {
            id: "test_graph".into(),
            name: "Test Graph".into(),
            version: "1.0.0".into(),
            coordination: "sequential".into(),
            tick: None,
            nodes: vec![
                GraphNode {
                    id: "input".into(),
                    name: "Input".into(),
                    depends_on: vec![],
                    capability: Some("game.player_input".into()),
                    feedback_to: None,
                    budget_ms: Some(2.0),
                },
                GraphNode {
                    id: "ai".into(),
                    name: "AI Coordinator".into(),
                    depends_on: vec!["input".into()],
                    capability: Some("ai.query".into()),
                    feedback_to: None,
                    budget_ms: Some(50.0),
                },
                GraphNode {
                    id: "render".into(),
                    name: "Renderer".into(),
                    depends_on: vec!["ai".into()],
                    capability: Some("visualization.render".into()),
                    feedback_to: None,
                    budget_ms: None,
                },
            ],
        }
    }

    #[test]
    fn execution_order_sequential() {
        let graph = sample_graph();
        let order = graph.execution_order().expect("test: no cycle");
        assert_eq!(order, vec!["input", "ai", "render"]);
    }

    #[test]
    fn execution_order_detects_cycle() {
        let graph = DeploymentGraphDef {
            id: "cycle".into(),
            name: "Cycle".into(),
            version: "1.0.0".into(),
            coordination: "sequential".into(),
            tick: None,
            nodes: vec![
                GraphNode {
                    id: "a".into(),
                    name: "A".into(),
                    depends_on: vec!["b".into()],
                    capability: None,
                    feedback_to: None,
                    budget_ms: None,
                },
                GraphNode {
                    id: "b".into(),
                    name: "B".into(),
                    depends_on: vec!["a".into()],
                    capability: None,
                    feedback_to: None,
                    budget_ms: None,
                },
            ],
        };
        assert!(graph.execution_order().is_err());
    }

    #[test]
    fn nodes_requiring_capability() {
        let graph = sample_graph();
        let ai_nodes = graph.nodes_requiring("ai.query");
        assert_eq!(ai_nodes.len(), 1);
        assert_eq!(ai_nodes[0].id, "ai");
    }

    #[test]
    fn requires_squirrel_true() {
        let graph = sample_graph();
        assert!(graph.requires_squirrel());
    }

    #[test]
    fn requires_squirrel_false() {
        let graph = DeploymentGraphDef {
            id: "no_ai".into(),
            name: "No AI".into(),
            version: "1.0.0".into(),
            coordination: "sequential".into(),
            tick: None,
            nodes: vec![GraphNode {
                id: "physics".into(),
                name: "Physics".into(),
                depends_on: vec![],
                capability: Some("compute.execute".into()),
                feedback_to: None,
                budget_ms: None,
            }],
        };
        assert!(!graph.requires_squirrel());
    }

    #[test]
    fn json_roundtrip() {
        let graph = sample_graph();
        let json = serde_json::to_string(&graph).expect("test: serialize");
        let decoded: DeploymentGraphDef = serde_json::from_str(&json).expect("test: deserialize");
        assert_eq!(graph, decoded);
    }

    #[test]
    fn continuous_graph_with_tick() {
        let graph = DeploymentGraphDef {
            id: "raid".into(),
            name: "2-Player Raid".into(),
            version: "1.0.0".into(),
            coordination: "continuous".into(),
            tick: Some(TickConfig {
                target_hz: 20.0,
                max_accumulator_ms: 100.0,
                budget_warning_ms: Some(10.0),
            }),
            nodes: vec![GraphNode {
                id: "server".into(),
                name: "Raid Server".into(),
                depends_on: vec![],
                capability: Some("game.raid_tick".into()),
                feedback_to: Some("server".into()),
                budget_ms: Some(40.0),
            }],
        };
        assert!(graph.tick.is_some());
        assert_eq!(graph.nodes[0].feedback_to.as_deref(), Some("server"));
    }

    // ── NicheDeployGraph (primalSpring-compatible) tests ────────────────

    #[test]
    fn parse_squirrel_ai_niche_graph() {
        let toml_str = include_str!("../../../../graphs/squirrel_ai_niche.toml");
        let graph =
            NicheDeployGraph::from_toml(toml_str).expect("squirrel_ai_niche.toml should parse");
        assert_eq!(graph.graph.name, "squirrel_ai_coordination_niche");
        assert_eq!(graph.graph.coordination.as_deref(), Some("Sequential"));
        assert!(!graph.graph.node.is_empty());
        assert!(graph.includes_squirrel());
    }

    #[test]
    fn squirrel_ai_niche_structural_validation() {
        let toml_str = include_str!("../../../../graphs/squirrel_ai_niche.toml");
        let graph = NicheDeployGraph::from_toml(toml_str).unwrap();
        let issues = graph.structural_issues();
        assert!(
            issues.is_empty(),
            "squirrel_ai_niche.toml has structural issues: {issues:?}"
        );
    }

    #[test]
    fn parse_ai_continuous_tick_graph() {
        let toml_str = include_str!("../../../../graphs/ai_continuous_tick.toml");
        let graph =
            NicheDeployGraph::from_toml(toml_str).expect("ai_continuous_tick.toml should parse");
        assert_eq!(graph.graph.name, "ai_continuous_tick");
        assert_eq!(graph.graph.coordination.as_deref(), Some("Continuous"));
        assert!(graph.includes_squirrel());
    }

    #[test]
    fn ai_continuous_tick_structural_validation() {
        let toml_str = include_str!("../../../../graphs/ai_continuous_tick.toml");
        let graph = NicheDeployGraph::from_toml(toml_str).unwrap();
        let issues = graph.structural_issues();
        assert!(
            issues.is_empty(),
            "ai_continuous_tick.toml has structural issues: {issues:?}"
        );
    }

    #[test]
    fn niche_graph_required_count() {
        let toml_str = include_str!("../../../../graphs/squirrel_ai_niche.toml");
        let graph = NicheDeployGraph::from_toml(toml_str).unwrap();
        assert!(
            graph.required_count() >= 2,
            "at least Tower Atomic (beardog + songbird) must be required"
        );
    }

    #[test]
    fn niche_graph_nodes_with_capability() {
        let toml_str = include_str!("../../../../graphs/squirrel_ai_niche.toml");
        let graph = NicheDeployGraph::from_toml(toml_str).unwrap();
        let ai_nodes = graph.nodes_with_capability("ai.query");
        assert_eq!(ai_nodes.len(), 1);
        assert_eq!(ai_nodes[0].name, "squirrel");
    }

    #[test]
    fn niche_graph_structural_detects_empty_name() {
        let graph = NicheDeployGraph {
            graph: NicheGraphMeta {
                name: String::new(),
                description: String::new(),
                version: String::new(),
                coordination: None,
                node: vec![],
            },
        };
        let issues = graph.structural_issues();
        assert!(issues.iter().any(|i| i.contains("name is empty")));
        assert!(issues.iter().any(|i| i.contains("no nodes")));
    }

    #[test]
    fn niche_graph_structural_detects_missing_dependency() {
        let graph = NicheDeployGraph {
            graph: NicheGraphMeta {
                name: "test".into(),
                description: String::new(),
                version: String::new(),
                coordination: None,
                node: vec![NicheGraphNode {
                    name: "alpha".into(),
                    binary: "alpha_primal".into(),
                    order: 1,
                    required: true,
                    depends_on: vec!["nonexistent".into()],
                    health_method: "health".into(),
                    by_capability: None,
                    capabilities: vec![],
                    condition: None,
                    skip_if: None,
                }],
            },
        };
        let issues = graph.structural_issues();
        assert!(issues.iter().any(|i| i.contains("nonexistent")));
    }

    #[test]
    fn niche_graph_json_roundtrip() {
        let toml_str = include_str!("../../../../graphs/squirrel_ai_niche.toml");
        let graph = NicheDeployGraph::from_toml(toml_str).unwrap();
        let json = serde_json::to_string(&graph).expect("test: serialize");
        let decoded: NicheDeployGraph = serde_json::from_str(&json).expect("test: deserialize");
        assert_eq!(graph, decoded);
    }

    #[test]
    fn all_squirrel_graphs_structurally_valid() {
        let graph_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../graphs");
        if !graph_dir.exists() {
            return;
        }
        for entry in std::fs::read_dir(&graph_dir).unwrap().flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "toml") {
                let contents = std::fs::read_to_string(&path).unwrap();
                let graph = NicheDeployGraph::from_toml(&contents).unwrap_or_else(|e| {
                    panic!("{} failed to parse: {e}", path.display());
                });
                let issues = graph.structural_issues();
                assert!(
                    issues.is_empty(),
                    "{} has structural issues: {issues:?}",
                    path.display()
                );
            }
        }
    }
}
