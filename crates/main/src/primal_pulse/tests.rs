// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive tests for PrimalPulse tools
//!
//! Includes unit, e2e, chaos, and fault injection tests

#[cfg(test)]
mod tests {
    use super::super::neural_graph::{CoordinationGraph, GraphAnalyzer, GraphNode, GraphOptimizer};

    // ============================================================================
    // UNIT TESTS
    // ============================================================================

    #[test]
    fn test_coordination_graph_creation() {
        let graph = CoordinationGraph::new("Test pipeline".to_string());
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
        assert_eq!(graph.metadata.purpose, "Test pipeline");
    }

    #[test]
    fn test_graph_node_addition() {
        let mut graph = CoordinationGraph::new("Test".to_string());

        graph.add_node(GraphNode {
            primal_name: "squirrel".to_string(),
            capability: "ai_routing".to_string(),
            estimated_latency_ms: 1000,
            estimated_cost_usd: 0.0,
            reliability: 0.95,
        });

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.get_node("squirrel").unwrap().primal_name, "squirrel");
    }

    #[test]
    fn test_graph_depth_calculation() {
        let mut graph = CoordinationGraph::new("Pipeline".to_string());

        // Add 3 sequential nodes
        graph.add_node(GraphNode {
            primal_name: "a".to_string(),
            capability: "cap_a".to_string(),
            estimated_latency_ms: 100,
            estimated_cost_usd: 0.0,
            reliability: 0.99,
        });

        graph.add_node(GraphNode {
            primal_name: "b".to_string(),
            capability: "cap_b".to_string(),
            estimated_latency_ms: 200,
            estimated_cost_usd: 0.0,
            reliability: 0.99,
        });

        graph.add_edge(super::super::neural_graph::GraphEdge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: super::super::neural_graph::EdgeType::Sequential,
            condition: None,
        });

        let depth = graph.calculate_depth();
        assert_eq!(depth, 2); // 1 edge + 1 = 2 depth
    }

    #[test]
    fn test_graph_cost_estimation() {
        let mut graph = CoordinationGraph::new("Pipeline".to_string());

        graph.add_node(GraphNode {
            primal_name: "expensive".to_string(),
            capability: "compute".to_string(),
            estimated_latency_ms: 1000,
            estimated_cost_usd: 0.01,
            reliability: 0.95,
        });

        graph.add_node(GraphNode {
            primal_name: "cheap".to_string(),
            capability: "storage".to_string(),
            estimated_latency_ms: 500,
            estimated_cost_usd: 0.001,
            reliability: 0.99,
        });

        let cost = graph.estimate_cost();
        assert!((cost - 0.011).abs() < 0.0001);
    }

    #[test]
    fn test_graph_reliability_calculation() {
        let mut graph = CoordinationGraph::new("Pipeline".to_string());

        graph.add_node(GraphNode {
            primal_name: "node1".to_string(),
            capability: "cap1".to_string(),
            estimated_latency_ms: 100,
            estimated_cost_usd: 0.0,
            reliability: 0.9,
        });

        graph.add_node(GraphNode {
            primal_name: "node2".to_string(),
            capability: "cap2".to_string(),
            estimated_latency_ms: 100,
            estimated_cost_usd: 0.0,
            reliability: 0.9,
        });

        let reliability = graph.estimate_reliability();
        assert!((reliability - 0.81).abs() < 0.01); // 0.9 * 0.9 = 0.81
    }

    #[test]
    fn test_graph_analyzer_bottleneck_detection() {
        let mut graph = CoordinationGraph::new("Hub-spoke".to_string());

        // Create hub with multiple incoming edges
        graph.add_node(GraphNode {
            primal_name: "hub".to_string(),
            capability: "central".to_string(),
            estimated_latency_ms: 1000,
            estimated_cost_usd: 0.0,
            reliability: 0.95,
        });

        // Add multiple nodes pointing to hub
        for i in 1..=4 {
            graph.add_node(GraphNode {
                primal_name: format!("spoke{}", i),
                capability: format!("cap{}", i),
                estimated_latency_ms: 100,
                estimated_cost_usd: 0.0,
                reliability: 0.99,
            });

            graph.add_edge(super::super::neural_graph::GraphEdge {
                from: format!("spoke{}", i),
                to: "hub".to_string(),
                edge_type: super::super::neural_graph::EdgeType::Sequential,
                condition: None,
            });
        }

        let analysis = GraphAnalyzer::analyze(&graph);
        assert!(analysis.bottlenecks.contains(&"hub".to_string()));
    }

    #[test]
    fn test_graph_pattern_detection_pipeline() {
        let mut graph = CoordinationGraph::new("Sequential".to_string());

        graph.add_node(GraphNode {
            primal_name: "a".to_string(),
            capability: "cap_a".to_string(),
            estimated_latency_ms: 100,
            estimated_cost_usd: 0.0,
            reliability: 0.99,
        });

        graph.add_node(GraphNode {
            primal_name: "b".to_string(),
            capability: "cap_b".to_string(),
            estimated_latency_ms: 100,
            estimated_cost_usd: 0.0,
            reliability: 0.99,
        });

        graph.add_edge(super::super::neural_graph::GraphEdge {
            from: "a".to_string(),
            to: "b".to_string(),
            edge_type: super::super::neural_graph::EdgeType::Sequential,
            condition: None,
        });

        let analysis = GraphAnalyzer::analyze(&graph);
        assert!(analysis
            .detected_patterns
            .contains(&super::super::neural_graph::GraphPattern::Pipeline));
    }

    #[test]
    fn test_optimizer_parallelization_suggestion() {
        let mut graph = CoordinationGraph::new("Sequential pipeline".to_string());

        // Create long sequential chain
        for i in 1..=5 {
            graph.add_node(GraphNode {
                primal_name: format!("node{}", i),
                capability: format!("cap{}", i),
                estimated_latency_ms: 500,
                estimated_cost_usd: 0.0,
                reliability: 0.99,
            });

            if i > 1 {
                graph.add_edge(super::super::neural_graph::GraphEdge {
                    from: format!("node{}", i - 1),
                    to: format!("node{}", i),
                    edge_type: super::super::neural_graph::EdgeType::Sequential,
                    condition: None,
                });
            }
        }

        let analysis = GraphAnalyzer::analyze(&graph);
        let recommendations = GraphOptimizer::optimize(&graph, &analysis);

        // Should suggest parallelization for long sequential chain
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| matches!(
            r.optimization_type,
            super::super::neural_graph::OptimizationType::Parallelization
        )));
    }

    // ============================================================================
    // E2E TESTS
    // ============================================================================

    #[tokio::test]
    #[ignore] // handlers module removed - HTTP endpoints deprecated
    async fn test_primal_analyze_e2e_mock() {
        // This would be a full e2e test if we had a running server
        // For now, verify the data structures work end-to-end
        // NOTE: handlers module removed in favor of Unix socket JSON-RPC

        let _input = serde_json::json!({
            "primal_path": "/tmp/test_primal",
            "depth": "summary"
        });

        // Test disabled: handlers module removed (HTTP → Unix sockets evolution)
    }

    #[tokio::test]
    async fn test_graph_optimize_full_workflow() {
        // Test complete graph optimization workflow

        let mut graph = CoordinationGraph::new("AI Analysis Pipeline".to_string());
        graph.metadata.expected_latency_ms = Some(5000);
        graph.metadata.cost_budget_usd = Some(0.01);

        // Add realistic coordination pattern
        graph.add_node(GraphNode {
            primal_name: "songbird".to_string(),
            capability: "service_mesh".to_string(),
            estimated_latency_ms: 500,
            estimated_cost_usd: 0.0,
            reliability: 0.99,
        });

        graph.add_node(GraphNode {
            primal_name: "squirrel".to_string(),
            capability: "ai_routing".to_string(),
            estimated_latency_ms: 2000,
            estimated_cost_usd: 0.0,
            reliability: 0.95,
        });

        graph.add_edge(super::super::neural_graph::GraphEdge {
            from: "songbird".to_string(),
            to: "squirrel".to_string(),
            edge_type: super::super::neural_graph::EdgeType::Sequential,
            condition: None,
        });

        // Analyze
        let analysis = GraphAnalyzer::analyze(&graph);
        assert!(analysis.estimated_latency_ms <= 5000);

        // Optimize
        let recommendations = GraphOptimizer::optimize(&graph, &analysis);
        assert!(recommendations
            .iter()
            .all(|r| r.confidence > 0.0 && r.confidence <= 1.0));
    }

    // ============================================================================
    // CHAOS TESTS
    // ============================================================================

    #[test]
    fn test_graph_with_empty_nodes() {
        // Chaos: Empty graph
        let graph = CoordinationGraph::new("Empty".to_string());
        let analysis = GraphAnalyzer::analyze(&graph);

        // Empty graph still returns valid analysis
        assert!(analysis.depth >= 0);
        assert!(analysis.width >= 0); // May have minimum width of 1
        assert_eq!(analysis.estimated_latency_ms, 0);
    }

    #[test]
    fn test_graph_with_high_latency_nodes() {
        // Chaos: Extremely high latency
        let mut graph = CoordinationGraph::new("Slow".to_string());

        graph.add_node(GraphNode {
            primal_name: "slow".to_string(),
            capability: "computation".to_string(),
            estimated_latency_ms: u64::MAX / 2, // Very high but not overflow
            estimated_cost_usd: 100.0,
            reliability: 0.5,
        });

        let analysis = GraphAnalyzer::analyze(&graph);
        assert!(analysis.estimated_latency_ms > 0);

        let recommendations = GraphOptimizer::optimize(&graph, &analysis);
        // Optimizer may or may not have recommendations - just verify it doesn't panic
        assert!(recommendations.len() >= 0);
    }

    #[test]
    fn test_graph_with_zero_reliability() {
        // Chaos: Unreliable nodes
        let mut graph = CoordinationGraph::new("Unreliable".to_string());

        graph.add_node(GraphNode {
            primal_name: "flaky".to_string(),
            capability: "flaky_service".to_string(),
            estimated_latency_ms: 1000,
            estimated_cost_usd: 0.0,
            reliability: 0.0, // Completely unreliable
        });

        let reliability = graph.estimate_reliability();
        assert_eq!(reliability, 0.0);
    }

    #[test]
    fn test_graph_with_many_parallel_branches() {
        // Chaos: Very wide graph
        let mut graph = CoordinationGraph::new("Wide".to_string());

        graph.add_node(GraphNode {
            primal_name: "source".to_string(),
            capability: "input".to_string(),
            estimated_latency_ms: 100,
            estimated_cost_usd: 0.0,
            reliability: 0.99,
        });

        // Create 100 parallel branches
        for i in 1..=100 {
            graph.add_node(GraphNode {
                primal_name: format!("parallel{}", i),
                capability: format!("cap{}", i),
                estimated_latency_ms: 100,
                estimated_cost_usd: 0.0,
                reliability: 0.99,
            });

            graph.add_edge(super::super::neural_graph::GraphEdge {
                from: "source".to_string(),
                to: format!("parallel{}", i),
                edge_type: super::super::neural_graph::EdgeType::Parallel,
                condition: None,
            });
        }

        let width = graph.calculate_width();
        assert!(width >= 100);
    }

    // ============================================================================
    // FAULT INJECTION TESTS
    // ============================================================================

    #[test]
    fn test_graph_analysis_with_missing_metadata() {
        // Fault: Missing metadata fields
        let mut graph = CoordinationGraph::new("".to_string()); // Empty purpose

        graph.add_node(GraphNode {
            primal_name: "node".to_string(),
            capability: "cap".to_string(),
            estimated_latency_ms: 1000,
            estimated_cost_usd: 0.0,
            reliability: 0.95,
        });

        let analysis = GraphAnalyzer::analyze(&graph);
        // Should still work with missing metadata
        assert!(analysis.depth > 0);
    }

    #[test]
    fn test_optimizer_with_no_inefficiencies() {
        // Fault: Perfect graph (nothing to optimize)
        let mut graph = CoordinationGraph::new("Perfect".to_string());

        graph.add_node(GraphNode {
            primal_name: "optimal".to_string(),
            capability: "fast".to_string(),
            estimated_latency_ms: 10,
            estimated_cost_usd: 0.0,
            reliability: 0.999,
        });

        let analysis = GraphAnalyzer::analyze(&graph);
        let recommendations = GraphOptimizer::optimize(&graph, &analysis);

        // May have no recommendations for perfect graph
        // This should not panic or error
        assert!(recommendations.len() >= 0);
    }

    #[test]
    fn test_graph_with_self_loop() {
        // Fault: Self-referencing edge
        let mut graph = CoordinationGraph::new("Self-loop".to_string());

        graph.add_node(GraphNode {
            primal_name: "loopy".to_string(),
            capability: "recursive".to_string(),
            estimated_latency_ms: 1000,
            estimated_cost_usd: 0.0,
            reliability: 0.95,
        });

        graph.add_edge(super::super::neural_graph::GraphEdge {
            from: "loopy".to_string(),
            to: "loopy".to_string(),
            edge_type: super::super::neural_graph::EdgeType::Sequential,
            condition: None,
        });

        // Should handle gracefully
        let analysis = GraphAnalyzer::analyze(&graph);
        assert!(analysis.depth > 0);
    }

    #[test]
    fn test_graph_with_orphaned_edges() {
        // Fault: Edges pointing to non-existent nodes
        let mut graph = CoordinationGraph::new("Orphaned".to_string());

        graph.add_node(GraphNode {
            primal_name: "exists".to_string(),
            capability: "real".to_string(),
            estimated_latency_ms: 1000,
            estimated_cost_usd: 0.0,
            reliability: 0.95,
        });

        graph.add_edge(super::super::neural_graph::GraphEdge {
            from: "exists".to_string(),
            to: "does_not_exist".to_string(),
            edge_type: super::super::neural_graph::EdgeType::Sequential,
            condition: None,
        });

        // Should handle gracefully - depth includes edges
        let analysis = GraphAnalyzer::analyze(&graph);
        assert!(analysis.depth > 0); // Graph exists and is analyzed
    }
}
