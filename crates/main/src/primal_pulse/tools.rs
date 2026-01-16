//! Tool registration for PrimalPulse
//!
//! Registers all PrimalPulse tools with the ActionRegistry

use super::schemas;
use crate::api::ai::{action_registry::ActionProvider, ActionRegistry};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// Register all PrimalPulse tools with the ActionRegistry
pub async fn register_primal_pulse_tools(registry: Arc<ActionRegistry>) {
    info!("🌊 Registering PrimalPulse tools...");

    // Register primal.analyze
    let analyze_provider = ActionProvider {
        provider_id: "primalpulse-analyzer".to_string(),
        provider_name: "PrimalPulse Code Analyzer".to_string(),
        action: "primal.analyze".to_string(),
        input_schema: schemas::primal_analyze_input_schema(),
        output_schema: schemas::primal_analyze_output_schema(),
        cost_per_unit: Some(0.0), // FREE (uses local AI)
        avg_latency_ms: 2000,
        quality: "high".to_string(),
        reliability: 0.95,
        is_local: true, // Uses local Ollama
        metadata: {
            let mut map = HashMap::new();
            map.insert(
                "description".to_string(),
                serde_json::json!("Analyzes primal code structure using local AI for privacy"),
            );
            map.insert(
                "privacy_level".to_string(),
                serde_json::json!("100% local - code never leaves machine"),
            );
            map.insert(
                "recommended_constraints".to_string(),
                serde_json::json!(["require_local"]),
            );
            map
        },
        registered_at: chrono::Utc::now(),
    };
    registry.register_action(analyze_provider).await;

    // Register primal.audit_hardcoding
    let audit_provider = ActionProvider {
        provider_id: "primalpulse-auditor".to_string(),
        provider_name: "PrimalPulse Hardcoding Auditor".to_string(),
        action: "primal.audit_hardcoding".to_string(),
        input_schema: schemas::primal_audit_input_schema(),
        output_schema: schemas::primal_audit_output_schema(),
        cost_per_unit: Some(0.0), // FREE (uses local AI for bulk)
        avg_latency_ms: 3000,
        quality: "high".to_string(),
        reliability: 0.92,
        is_local: true,
        metadata: {
            let mut map = HashMap::new();
            map.insert(
                "description".to_string(),
                serde_json::json!("Audits code for TRUE PRIMAL violations using cost-optimized AI"),
            );
            map.insert(
                "privacy_level".to_string(),
                serde_json::json!("100% local by default"),
            );
            map.insert(
                "recommended_constraints".to_string(),
                serde_json::json!(["optimize_cost", "require_local"]),
            );
            map
        },
        registered_at: chrono::Utc::now(),
    };
    registry.register_action(audit_provider).await;

    // Register rootpulse.semantic_commit
    let commit_provider = ActionProvider {
        provider_id: "primalpulse-commit-gen".to_string(),
        provider_name: "PrimalPulse Semantic Commit Generator".to_string(),
        action: "rootpulse.semantic_commit".to_string(),
        input_schema: schemas::rootpulse_commit_input_schema(),
        output_schema: schemas::rootpulse_commit_output_schema(),
        cost_per_unit: Some(0.0001), // Low cost (short messages)
        avg_latency_ms: 1500,
        quality: "best".to_string(),
        reliability: 0.97,
        is_local: false, // Prefers OpenAI for quality
        metadata: {
            let mut map = HashMap::new();
            map.insert(
                "description".to_string(),
                serde_json::json!("Generates semantic commit messages using high-quality AI"),
            );
            map.insert(
                "privacy_level".to_string(),
                serde_json::json!(
                    "Cloud-based by default (can use local with require_local constraint)"
                ),
            );
            map.insert(
                "recommended_constraints".to_string(),
                serde_json::json!(["optimize_quality"]),
            );
            map
        },
        registered_at: chrono::Utc::now(),
    };
    registry.register_action(commit_provider).await;

    // 4. neural.graph_optimize
    let graph_provider = ActionProvider {
        provider_id: "primalpulse-graph-optimizer".to_string(),
        provider_name: "PrimalPulse Graph Optimizer".to_string(),
        action: "neural.graph_optimize".to_string(),
        input_schema: schemas::neural_graph_optimize_input_schema(),
        output_schema: schemas::neural_graph_optimize_output_schema(),
        cost_per_unit: Some(0.0),
        avg_latency_ms: 2000,
        quality: "high".to_string(),
        reliability: 0.98,
        is_local: true,
        metadata: {
            let mut map = HashMap::new();
            map.insert(
                "description".to_string(),
                serde_json::json!("Optimizes coordination graphs using neural pattern analysis"),
            );
            map.insert(
                "privacy_level".to_string(),
                serde_json::json!("100% local analysis"),
            );
            map
        },
        registered_at: chrono::Utc::now(),
    };
    registry.register_action(graph_provider).await;

    info!("✅ PrimalPulse tools registered: primal.analyze, primal.audit_hardcoding, rootpulse.semantic_commit, neural.graph_optimize");
}
