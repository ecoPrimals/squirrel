//! Handler for neural.graph_optimize tool

use super::{CoordinationGraph, GraphAnalyzer, GraphNode, GraphOptimizer};
use crate::api::ai::router::AiRouter;
use crate::api::ai::types::{TextGenerationRequest, UniversalAiResponse};
use crate::error::PrimalError;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{info, warn};

/// Input for neural.graph_optimize
#[derive(Debug, Deserialize)]
pub struct GraphOptimizeInput {
    graph_description: String,
    purpose: String,
    #[serde(default)]
    expected_latency_ms: Option<u64>,
    #[serde(default)]
    cost_budget_usd: Option<f64>,
    #[serde(default)]
    constraints: Vec<String>,
}

/// Handle neural.graph_optimize action
pub async fn handle_neural_graph_optimize(
    input: Value,
    router: Arc<AiRouter>,
    constraints: Vec<String>,
) -> Result<UniversalAiResponse, PrimalError> {
    let params: GraphOptimizeInput = serde_json::from_value(input).map_err(|e| {
        PrimalError::ValidationError(format!("Invalid graph_optimize input: {}", e))
    })?;

    info!(
        "🧠 Optimizing coordination graph: {}",
        params.graph_description
    );

    // Parse the graph description
    let graph = parse_graph_description(
        &params.graph_description,
        &params.purpose,
        params.expected_latency_ms,
        params.cost_budget_usd,
        params.constraints.clone(),
    );

    // Analyze the graph
    let analysis = GraphAnalyzer::analyze(&graph);

    info!(
        "Graph analysis: depth={}, width={}, latency={}ms, cost=${:.4}",
        analysis.depth, analysis.width, analysis.estimated_latency_ms, analysis.estimated_cost_usd
    );

    // Generate optimizations
    let recommendations = GraphOptimizer::optimize(&graph, &analysis);

    info!(
        "Generated {} optimization recommendations",
        recommendations.len()
    );

    // Use AI to generate neural insights
    let ai_insights =
        generate_neural_insights(&params.graph_description, &analysis, router, constraints).await?;

    // Build output
    let output = json!({
        "analysis": {
            "depth": analysis.depth,
            "width": analysis.width,
            "estimated_latency_ms": analysis.estimated_latency_ms,
            "estimated_cost_usd": analysis.estimated_cost_usd,
            "estimated_reliability": analysis.estimated_reliability,
            "bottlenecks": analysis.bottlenecks,
            "inefficiencies": analysis.inefficiencies.iter().map(|i| i.description.clone()).collect::<Vec<_>>(),
            "detected_patterns": analysis.detected_patterns,
        },
        "recommendations": recommendations.iter().map(|r| {
            json!({
                "type": format!("{:?}", r.optimization_type),
                "description": r.description,
                "expected_improvement": {
                    "latency_reduction_ms": r.expected_improvement.latency_reduction_ms,
                    "cost_reduction_usd": r.expected_improvement.cost_reduction_usd,
                    "reliability_improvement": r.expected_improvement.reliability_improvement,
                },
                "implementation": r.implementation,
                "confidence": r.confidence,
                "difficulty": r.difficulty,
            })
        }).collect::<Vec<_>>(),
        "optimized_graph": generate_optimized_graph_description(&graph, &recommendations),
        "neural_insights": ai_insights,
    });

    Ok(UniversalAiResponse {
        action: "neural.graph_optimize".to_string(),
        output,
        metadata: crate::api::ai::types::ResponseMetadata {
            provider_id: "neural-optimizer".to_string(),
            provider_name: "PrimalPulse Neural Graph Optimizer".to_string(),
            cost_usd: Some(0.0),
            latency_ms: 0, // Will be filled by caller
            timestamp: chrono::Utc::now(),
            extras: std::collections::HashMap::new(),
        },
    })
}

/// Parse graph description into CoordinationGraph
fn parse_graph_description(
    description: &str,
    purpose: &str,
    expected_latency_ms: Option<u64>,
    cost_budget_usd: Option<f64>,
    constraints: Vec<String>,
) -> CoordinationGraph {
    let mut graph = CoordinationGraph::new(purpose.to_string());
    graph.metadata.expected_latency_ms = expected_latency_ms;
    graph.metadata.cost_budget_usd = cost_budget_usd;
    graph.metadata.constraints = constraints;

    // Simple parser: "primal1 -> primal2 -> primal3"
    // TODO: Support more complex graph descriptions (parallel, conditional, etc.)
    let parts: Vec<&str> = description.split("->").map(|s| s.trim()).collect();

    for (i, primal_name) in parts.iter().enumerate() {
        // Create node with estimated values
        // In production, these would come from actual primal metadata
        let node = GraphNode {
            primal_name: primal_name.to_string(),
            capability: format!("{}_capability", primal_name),
            estimated_latency_ms: 1000 + (i as u64 * 500), // Rough estimate
            estimated_cost_usd: if primal_name.contains("squirrel")
                || primal_name.contains("ollama")
            {
                0.0 // Local AI is free
            } else {
                0.001 * (i as f64 + 1.0)
            },
            reliability: 0.95,
        };

        graph.add_node(node);

        // Add edge to next node
        if i < parts.len() - 1 {
            graph.add_edge(super::GraphEdge {
                from: primal_name.to_string(),
                to: parts[i + 1].to_string(),
                edge_type: super::EdgeType::Sequential,
                condition: None,
            });
        }
    }

    graph
}

/// Generate neural insights using AI
async fn generate_neural_insights(
    graph_description: &str,
    analysis: &super::GraphAnalysis,
    router: Arc<AiRouter>,
    constraints: Vec<String>,
) -> Result<String, PrimalError> {
    let prompt = format!(
        "Analyze this ecoPrimals coordination graph and provide insights:\n\n\
        Graph: {}\n\
        Depth: {}\n\
        Width: {}\n\
        Estimated Latency: {}ms\n\
        Bottlenecks: {:?}\n\
        Patterns: {:?}\n\n\
        Provide 2-3 key insights about this coordination pattern, focusing on:\n\
        1. Potential optimization opportunities\n\
        2. Comparison to known efficient patterns\n\
        3. Risks or anti-patterns to avoid\n\n\
        Keep it concise (3-4 sentences).",
        graph_description,
        analysis.depth,
        analysis.width,
        analysis.estimated_latency_ms,
        analysis.bottlenecks,
        analysis.detected_patterns
    );

    let ai_request = TextGenerationRequest {
        prompt,
        system: None,
        max_tokens: 200,
        temperature: 0.7,
        model: None,
        constraints: vec![], // Empty constraints for now
        params: std::collections::HashMap::new(),
    };

    match router.generate_text(ai_request, None).await {
        Ok(response) => Ok(response.text),
        Err(e) => {
            warn!("Failed to generate neural insights: {}", e);
            Ok("Neural insights unavailable (AI generation failed)".to_string())
        }
    }
}

/// Generate optimized graph description
fn generate_optimized_graph_description(
    _graph: &CoordinationGraph,
    recommendations: &[super::Optimization],
) -> Option<String> {
    if recommendations.is_empty() {
        return None;
    }

    // For now, just suggest general improvements
    // TODO: Generate actual optimized graph structure
    Some(
        "Optimized graph with parallelization and load balancing (see recommendations)".to_string(),
    )
}
