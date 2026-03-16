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
        use std::collections::{HashMap, HashSet, VecDeque};

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
        let mut visited = HashSet::new();

        while let Some(id) = queue.pop_front() {
            order.push(id.to_string());
            visited.insert(id);

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
}
