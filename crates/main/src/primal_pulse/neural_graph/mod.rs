// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Neural Graph Optimizer
//!
//! Analyzes coordination patterns between primals and suggests optimal graph
//! structures for biomeOS execution. Learns from execution patterns and evolves
//! coordination strategies over time.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single primal node in a coordination graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// The primal providing this capability
    pub primal_name: String,
    /// The capability being provided
    pub capability: String,
    /// Estimated execution latency in milliseconds
    pub estimated_latency_ms: u64,
    /// Estimated cost in USD
    pub estimated_cost_usd: f64,
    /// Reliability score (0.0 - 1.0)
    pub reliability: f64,
}

/// Type of edge connecting graph nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    /// Sequential execution (A then B)
    Sequential,
    /// Parallel execution (A and B simultaneously)
    Parallel,
    /// Conditional execution (A if condition)
    Conditional,
}

/// Represents a connection between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node
    pub from: String,
    /// Destination node
    pub to: String,
    /// Type of edge
    pub edge_type: EdgeType,
    /// Optional condition for conditional edges
    pub condition: Option<String>,
}

/// A coordination graph representing primal interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationGraph {
    /// All nodes in the graph
    pub nodes: Vec<GraphNode>,
    /// All edges connecting nodes
    pub edges: Vec<GraphEdge>,
    /// Graph metadata
    pub metadata: GraphMetadata,
}

/// Metadata about the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Purpose of this coordination
    pub purpose: String,
    /// Expected latency budget in milliseconds
    pub expected_latency_ms: Option<u64>,
    /// Cost budget in USD
    pub cost_budget_usd: Option<f64>,
    /// User-defined constraints
    pub constraints: Vec<String>,
}

/// Recognized graph patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphPattern {
    /// Sequential pipeline A → B → C
    Pipeline,
    /// Fan-out: A → [B, C, D]
    FanOut,
    /// Fan-in: [A, B, C] → D
    FanIn,
    /// Hub-spoke: Central coordinator
    HubSpoke,
    /// Circular: A → B → C → A
    Circular,
    /// Mesh: Fully connected
    Mesh,
    /// Unknown pattern
    Unknown,
}

/// Types of detected inefficiencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inefficiency {
    /// Type of inefficiency
    pub inefficiency_type: InefficiencyType,
    /// Description of the issue
    pub description: String,
    /// Severity (0.0 - 1.0, higher is worse)
    pub severity: f64,
    /// Nodes/edges involved
    pub affected_components: Vec<String>,
}

/// Type of detected inefficiency in a coordination graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InefficiencyType {
    /// Operations that could run in parallel are sequential
    UnnecessarySequential,
    /// High-latency node limiting throughput
    Bottleneck,
    /// Excessive communication between nodes
    ChattyCommunication,
    /// Circular dependency that may cause deadlocks
    CircularDependency,
    /// Redundant execution path
    RedundantPath,
    /// Node with many dependents but no redundancy
    SinglePointOfFailure,
}

/// Analysis results for a coordination graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysis {
    /// Graph depth (longest path)
    pub depth: usize,
    /// Graph width (max parallel nodes)
    pub width: usize,
    /// Detected bottleneck nodes
    pub bottlenecks: Vec<String>,
    /// Estimated total latency in milliseconds
    pub estimated_latency_ms: u64,
    /// Estimated total cost in USD
    pub estimated_cost_usd: f64,
    /// Estimated reliability (0.0 - 1.0)
    pub estimated_reliability: f64,
    /// Detected graph patterns
    pub detected_patterns: Vec<GraphPattern>,
    /// Identified inefficiencies
    pub inefficiencies: Vec<Inefficiency>,
}

/// Type of optimization recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Run independent operations in parallel
    Parallelization,
    /// Add caching to reduce redundant work
    Caching,
    /// Distribute load across multiple instances
    LoadBalancing,
    /// Skip unnecessary operations when possible
    ShortCircuiting,
    /// Use alternative execution path
    AlternativePath,
    /// Refactor graph pattern for better performance
    PatternRefactoring,
}

/// Expected performance improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// Latency reduction in milliseconds
    pub latency_reduction_ms: i64,
    /// Cost reduction in USD (can be negative for increase)
    pub cost_reduction_usd: f64,
    /// Reliability improvement (0.0 - 1.0)
    pub reliability_improvement: f64,
}

/// An optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    /// Human-readable description
    pub description: String,
    /// Expected improvement
    pub expected_improvement: PerformanceImprovement,
    /// Implementation details
    pub implementation: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Difficulty level (1-5, higher is harder)
    pub difficulty: u8,
}

/// Complete optimization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Analysis of current graph
    pub analysis: GraphAnalysis,
    /// Recommended optimizations
    pub recommendations: Vec<Optimization>,
    /// Optimized graph representation
    pub optimized_graph: Option<String>,
    /// Neural insights from pattern learning
    pub neural_insights: Option<String>,
}

impl CoordinationGraph {
    /// Create a new empty coordination graph
    #[must_use]
    pub const fn new(purpose: String) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            metadata: GraphMetadata {
                purpose,
                expected_latency_ms: None,
                cost_budget_usd: None,
                constraints: Vec::new(),
            },
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.push(node);
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    /// Get node by primal name
    #[must_use]
    pub fn get_node(&self, primal_name: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.primal_name == primal_name)
    }

    /// Calculate graph depth (longest path) using topological sort
    ///
    /// Performs Kahn's algorithm for topological ordering, then computes
    /// the longest path through the DAG. Handles both sequential and
    /// conditional edges. Returns 0 for empty graphs.
    #[must_use]
    pub fn calculate_depth(&self) -> usize {
        if self.nodes.is_empty() {
            return 0;
        }

        // Build adjacency list and in-degree map (sequential + conditional edges form the DAG)
        let node_names: Vec<&str> = self.nodes.iter().map(|n| n.primal_name.as_str()).collect();
        let node_set: std::collections::HashSet<&str> = node_names.iter().copied().collect();
        let mut in_degree: HashMap<&str, usize> = node_names.iter().map(|&n| (n, 0)).collect();
        let mut adj: HashMap<&str, Vec<&str>> =
            node_names.iter().map(|&n| (n, Vec::new())).collect();

        for edge in &self.edges {
            if edge.edge_type != EdgeType::Parallel {
                let from = edge.from.as_str();
                let to = edge.to.as_str();
                // Skip orphaned edges referencing unknown nodes
                if !node_set.contains(from) || !node_set.contains(to) {
                    continue;
                }
                adj.entry(from).or_default().push(to);
                *in_degree.entry(to).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm with longest-path tracking
        let mut queue: std::collections::VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(name, _)| *name)
            .collect();

        let mut dist: HashMap<&str, usize> = node_names.iter().map(|&n| (n, 0)).collect();

        while let Some(node) = queue.pop_front() {
            if let Some(neighbors) = adj.get(node) {
                for &next in neighbors {
                    let new_dist = dist[node] + 1;
                    if new_dist > dist[next] {
                        dist.insert(next, new_dist);
                    }
                    if let Some(deg) = in_degree.get_mut(next) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(next);
                        }
                    }
                }
            }
        }

        // Depth is the maximum distance + 1 (counting nodes, not edges)
        dist.values().copied().max().unwrap_or(0) + 1
    }

    /// Calculate graph width (max parallel branches)
    #[must_use]
    pub fn calculate_width(&self) -> usize {
        // Count maximum fan-out
        let mut max_width = 1;

        for node in &self.nodes {
            let outgoing = self
                .edges
                .iter()
                .filter(|e| e.from == node.primal_name && e.edge_type == EdgeType::Parallel)
                .count();

            if outgoing > max_width {
                max_width = outgoing;
            }
        }

        max_width
    }

    /// Estimate total latency using critical path analysis
    ///
    /// Computes the longest-weight path through the graph, which represents
    /// the minimum possible execution time. Sequential edges add latencies,
    /// parallel branches take the max of their sub-paths.
    #[must_use]
    pub fn estimate_latency(&self) -> u64 {
        if self.nodes.is_empty() {
            return 0;
        }

        // Build node latency lookup
        let node_latency: HashMap<&str, u64> = self
            .nodes
            .iter()
            .map(|n| (n.primal_name.as_str(), n.estimated_latency_ms))
            .collect();

        // Build adjacency list for sequential/conditional edges
        let node_names: Vec<&str> = self.nodes.iter().map(|n| n.primal_name.as_str()).collect();
        let node_set: std::collections::HashSet<&str> = node_names.iter().copied().collect();
        let mut adj: HashMap<&str, Vec<&str>> =
            node_names.iter().map(|&n| (n, Vec::new())).collect();
        let mut in_degree: HashMap<&str, usize> = node_names.iter().map(|&n| (n, 0)).collect();

        for edge in &self.edges {
            if edge.edge_type != EdgeType::Parallel {
                let from = edge.from.as_str();
                let to = edge.to.as_str();
                // Skip orphaned edges
                if !node_set.contains(from) || !node_set.contains(to) {
                    continue;
                }
                adj.entry(from).or_default().push(to);
                *in_degree.entry(to).or_insert(0) += 1;
            }
        }

        // Critical path: longest-weight path via topological order
        let mut earliest_finish: HashMap<&str, u64> = node_names
            .iter()
            .map(|&n| (n, *node_latency.get(n).unwrap_or(&0)))
            .collect();

        let mut queue: std::collections::VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(name, _)| *name)
            .collect();

        while let Some(node) = queue.pop_front() {
            let finish = earliest_finish[node];
            if let Some(neighbors) = adj.get(node) {
                for &next in neighbors {
                    let next_latency = *node_latency.get(next).unwrap_or(&0);
                    let new_finish = finish + next_latency;
                    if new_finish > earliest_finish[next] {
                        earliest_finish.insert(next, new_finish);
                    }
                    if let Some(deg) = in_degree.get_mut(next) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(next);
                        }
                    }
                }
            }
        }

        // Also account for parallel branches: nodes without sequential edges
        // still contribute their own latency
        let critical_path = earliest_finish.values().copied().max().unwrap_or(0);

        // If everything is parallel (no sequential edges), latency = max single node
        if self.edges.iter().all(|e| e.edge_type == EdgeType::Parallel) {
            return self
                .nodes
                .iter()
                .map(|n| n.estimated_latency_ms)
                .max()
                .unwrap_or(0);
        }

        critical_path
    }

    /// Estimate total cost
    #[must_use]
    pub fn estimate_cost(&self) -> f64 {
        self.nodes.iter().map(|n| n.estimated_cost_usd).sum()
    }

    /// Estimate overall reliability
    #[must_use]
    pub fn estimate_reliability(&self) -> f64 {
        // Product of individual reliabilities
        self.nodes.iter().map(|n| n.reliability).product()
    }
}

/// Analyzes coordination graphs for bottlenecks, patterns, and inefficiencies.
pub struct GraphAnalyzer;

impl GraphAnalyzer {
    /// Analyze a coordination graph
    #[must_use]
    pub fn analyze(graph: &CoordinationGraph) -> GraphAnalysis {
        let depth = graph.calculate_depth();
        let width = graph.calculate_width();
        let estimated_latency_ms = graph.estimate_latency();
        let estimated_cost_usd = graph.estimate_cost();
        let estimated_reliability = graph.estimate_reliability();

        let bottlenecks = Self::detect_bottlenecks(graph);
        let detected_patterns = Self::detect_patterns(graph);
        let inefficiencies = Self::detect_inefficiencies(graph);

        GraphAnalysis {
            depth,
            width,
            bottlenecks,
            estimated_latency_ms,
            estimated_cost_usd,
            estimated_reliability,
            detected_patterns,
            inefficiencies,
        }
    }

    /// Detect bottleneck nodes
    fn detect_bottlenecks(graph: &CoordinationGraph) -> Vec<String> {
        let mut bottlenecks = Vec::new();

        // Find nodes with high in-degree (many primals depend on them)
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for edge in &graph.edges {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
        }

        for (node, degree) in in_degree {
            if degree > 2 {
                // Arbitrary threshold
                bottlenecks.push(node);
            }
        }

        bottlenecks
    }

    /// Detect graph patterns including cycle detection
    fn detect_patterns(graph: &CoordinationGraph) -> Vec<GraphPattern> {
        let mut patterns = Vec::new();

        let sequential_count = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::Sequential)
            .count();
        let parallel_count = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::Parallel)
            .count();

        // Pipeline: all sequential, no parallel
        if sequential_count > 0 && parallel_count == 0 {
            patterns.push(GraphPattern::Pipeline);
        }

        // Fan-out: a node has multiple parallel outgoing edges
        if parallel_count > 0 {
            patterns.push(GraphPattern::FanOut);
        }

        // Fan-in: a node has multiple incoming edges
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for edge in &graph.edges {
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
        }
        if in_degree.values().any(|&d| d > 1) {
            patterns.push(GraphPattern::FanIn);
        }

        // Hub-spoke: one node connects to most others
        let mut out_degree: HashMap<&str, usize> = HashMap::new();
        for edge in &graph.edges {
            *out_degree.entry(edge.from.as_str()).or_insert(0) += 1;
        }
        let node_count = graph.nodes.len();
        if node_count > 2
            && out_degree
                .values()
                .any(|&d| d >= node_count.saturating_sub(1))
        {
            patterns.push(GraphPattern::HubSpoke);
        }

        // Cycle detection using DFS with three-color marking
        if Self::has_cycle(graph) {
            patterns.push(GraphPattern::Circular);
        }

        // Mesh: high connectivity (edge count >= n*(n-1)/2)
        let max_edges = node_count * node_count.saturating_sub(1) / 2;
        if max_edges > 0 && graph.edges.len() >= max_edges {
            patterns.push(GraphPattern::Mesh);
        }

        if patterns.is_empty() {
            patterns.push(GraphPattern::Unknown);
        }

        patterns
    }

    /// Detect cycles in the graph using DFS three-color marking
    ///
    /// White (unvisited) → Gray (in progress) → Black (finished)
    /// A back-edge to a gray node indicates a cycle.
    fn has_cycle(graph: &CoordinationGraph) -> bool {
        #[derive(Clone, Copy, PartialEq)]
        enum Color {
            White,
            Gray,
            Black,
        }

        let node_names: Vec<&str> = graph.nodes.iter().map(|n| n.primal_name.as_str()).collect();
        let node_set: std::collections::HashSet<&str> = node_names.iter().copied().collect();
        let mut color: HashMap<&str, Color> =
            node_names.iter().map(|&n| (n, Color::White)).collect();
        let mut adj: HashMap<&str, Vec<&str>> =
            node_names.iter().map(|&n| (n, Vec::new())).collect();

        for edge in &graph.edges {
            let from = edge.from.as_str();
            let to = edge.to.as_str();
            // Skip orphaned edges
            if !node_set.contains(from) || !node_set.contains(to) {
                continue;
            }
            adj.entry(from).or_default().push(to);
        }

        fn dfs<'a>(
            node: &'a str,
            adj: &HashMap<&'a str, Vec<&'a str>>,
            color: &mut HashMap<&'a str, Color>,
        ) -> bool {
            color.insert(node, Color::Gray);

            if let Some(neighbors) = adj.get(node) {
                for &next in neighbors {
                    match color.get(next) {
                        Some(Color::Gray) => return true, // Back edge → cycle
                        Some(Color::White) => {
                            if dfs(next, adj, color) {
                                return true;
                            }
                        }
                        _ => {} // Black = already fully explored
                    }
                }
            }

            color.insert(node, Color::Black);
            false
        }

        for &node in &node_names {
            if color[node] == Color::White && dfs(node, &adj, &mut color) {
                return true;
            }
        }

        false
    }

    /// Detect inefficiencies
    fn detect_inefficiencies(graph: &CoordinationGraph) -> Vec<Inefficiency> {
        let mut inefficiencies = Vec::new();

        // Check for unnecessary sequential execution
        if graph
            .edges
            .iter()
            .all(|e| e.edge_type == EdgeType::Sequential)
            && graph.nodes.len() > 2
        {
            inefficiencies.push(Inefficiency {
                inefficiency_type: InefficiencyType::UnnecessarySequential,
                description: "All operations are sequential, consider parallelization".to_string(),
                severity: 0.7,
                affected_components: graph.nodes.iter().map(|n| n.primal_name.clone()).collect(),
            });
        }

        // Check for high-latency bottlenecks
        for node in &graph.nodes {
            if node.estimated_latency_ms > 3000 {
                inefficiencies.push(Inefficiency {
                    inefficiency_type: InefficiencyType::Bottleneck,
                    description: format!(
                        "{} has high latency ({}ms)",
                        node.primal_name, node.estimated_latency_ms
                    ),
                    severity: 0.8,
                    affected_components: vec![node.primal_name.clone()],
                });
            }
        }

        // Check for circular dependencies
        if Self::has_cycle(graph) {
            inefficiencies.push(Inefficiency {
                inefficiency_type: InefficiencyType::CircularDependency,
                description: "Graph contains circular dependencies which may cause deadlocks"
                    .to_string(),
                severity: 0.9,
                affected_components: graph.nodes.iter().map(|n| n.primal_name.clone()).collect(),
            });
        }

        // Check for single points of failure (nodes with no redundancy but high fan-in)
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for edge in &graph.edges {
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
        }
        for node in &graph.nodes {
            let in_deg = in_degree
                .get(node.primal_name.as_str())
                .copied()
                .unwrap_or(0);
            if in_deg > 2 && node.reliability < 0.99 {
                inefficiencies.push(Inefficiency {
                    inefficiency_type: InefficiencyType::SinglePointOfFailure,
                    description: format!(
                        "{} is a single point of failure ({} dependents, {:.0}% reliability)",
                        node.primal_name,
                        in_deg,
                        node.reliability * 100.0
                    ),
                    severity: 0.85,
                    affected_components: vec![node.primal_name.clone()],
                });
            }
        }

        inefficiencies
    }
}

/// Generates optimization recommendations for coordination graphs.
pub struct GraphOptimizer;

impl GraphOptimizer {
    /// Generate optimization recommendations
    #[must_use]
    pub fn optimize(graph: &CoordinationGraph, analysis: &GraphAnalysis) -> Vec<Optimization> {
        let mut recommendations = Vec::new();

        // Suggest parallelization for sequential pipelines
        if analysis.detected_patterns.contains(&GraphPattern::Pipeline) && graph.nodes.len() > 2 {
            recommendations.push(Optimization {
                optimization_type: OptimizationType::Parallelization,
                description: "Convert sequential pipeline to parallel where possible".to_string(),
                expected_improvement: PerformanceImprovement {
                    latency_reduction_ms: (analysis.estimated_latency_ms / 3) as i64,
                    cost_reduction_usd: 0.0,
                    reliability_improvement: 0.0,
                },
                implementation: "Identify independent operations and execute in parallel"
                    .to_string(),
                confidence: 0.75,
                difficulty: 3,
            });
        }

        // Suggest load balancing for bottlenecks
        for bottleneck in &analysis.bottlenecks {
            recommendations.push(Optimization {
                optimization_type: OptimizationType::LoadBalancing,
                description: format!("Distribute load from {bottleneck} across multiple instances"),
                expected_improvement: PerformanceImprovement {
                    latency_reduction_ms: 1000,
                    cost_reduction_usd: 0.0,
                    reliability_improvement: 0.1,
                },
                implementation: format!(
                    "Use capability discovery to find multiple {bottleneck} providers"
                ),
                confidence: 0.65,
                difficulty: 4,
            });
        }

        recommendations
    }
}
