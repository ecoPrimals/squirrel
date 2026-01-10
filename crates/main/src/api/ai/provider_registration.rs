//! Provider registration API for Phase 6 - Ultimate Vision
//!
//! Allows external providers to self-register their capabilities dynamically.

use super::action_registry::{ActionProvider, ActionRegistry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use warp::{http::StatusCode, reply::json, Reply};

/// Request to register a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRegistrationRequest {
    /// Provider ID (unique identifier)
    pub provider_id: String,

    /// Provider display name
    pub provider_name: String,

    /// Advertised capabilities
    pub advertised_capabilities: Vec<CapabilityAdvertisement>,
}

/// A capability advertisement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAdvertisement {
    /// Action name (e.g., "image.generation")
    pub action: String,

    /// Input schema (JSON Schema format)
    pub input_schema: serde_json::Value,

    /// Output schema (JSON Schema format)
    pub output_schema: serde_json::Value,

    /// Cost per unit (USD, optional)
    pub cost_per_unit: Option<f64>,

    /// Average latency in milliseconds
    pub avg_latency_ms: u64,

    /// Quality tier ("low", "medium", "high", "premium")
    #[serde(default = "default_quality")]
    pub quality: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_quality() -> String {
    "medium".to_string()
}

/// Response from provider registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRegistrationResponse {
    pub success: bool,
    pub provider_id: String,
    pub actions_registered: usize,
    pub message: String,
}

/// Handler for `POST /api/v1/providers/register`
pub async fn handle_register_provider(
    request: ProviderRegistrationRequest,
    registry: Arc<ActionRegistry>,
) -> Result<impl Reply, warp::Rejection> {
    tracing::info!(
        "📝 Provider registration request: '{}' with {} capabilities",
        request.provider_id,
        request.advertised_capabilities.len()
    );

    // Register each capability
    let mut registered_count = 0;
    for capability in &request.advertised_capabilities {
        let action_provider = ActionProvider {
            provider_id: request.provider_id.clone(),
            provider_name: request.provider_name.clone(),
            action: capability.action.clone(),
            input_schema: capability.input_schema.clone(),
            output_schema: capability.output_schema.clone(),
            cost_per_unit: capability.cost_per_unit,
            avg_latency_ms: capability.avg_latency_ms,
            quality: capability.quality.clone(),
            reliability: 0.99, // Assume high until proven otherwise
            is_local: false,   // External providers are not local
            metadata: capability.metadata.clone(),
            registered_at: chrono::Utc::now(),
        };

        registry.register_action(action_provider).await;
        registered_count += 1;
    }

    let response = ProviderRegistrationResponse {
        success: true,
        provider_id: request.provider_id.clone(),
        actions_registered: registered_count,
        message: format!(
            "Successfully registered {} actions for provider '{}'",
            registered_count, request.provider_id
        ),
    };

    tracing::info!(
        "✅ Provider '{}' registered with {} actions",
        request.provider_id,
        registered_count
    );

    Ok(warp::reply::with_status(
        json(&response),
        StatusCode::CREATED,
    ))
}

/// Handler for `GET /api/v1/actions`
pub async fn handle_list_actions(
    registry: Arc<ActionRegistry>,
) -> Result<impl Reply, warp::Rejection> {
    let actions = registry.list_all_actions().await;
    let stats = registry.get_stats().await;

    let response = serde_json::json!({
        "actions": actions.iter().map(|action| {
            serde_json::json!({
                "action": action,
                "provider_count": registry.find_providers_for_action(action)
                    .now_or_never()
                    .flatten()
                    .map_or(0, |p| p.len()),
            })
        }).collect::<Vec<_>>(),
        "stats": stats,
    });

    Ok(warp::reply::with_status(json(&response), StatusCode::OK))
}

/// Handler for `GET /api/v1/providers`
pub async fn handle_list_providers(
    registry: Arc<ActionRegistry>,
) -> Result<impl Reply, warp::Rejection> {
    let providers = registry.list_all_providers().await;

    // Group by provider_id
    let mut provider_map: HashMap<String, Vec<String>> = HashMap::new();
    for provider in providers {
        provider_map
            .entry(provider.provider_id.clone())
            .or_default()
            .push(provider.action.clone());
    }

    let response = serde_json::json!({
        "providers": provider_map.iter().map(|(id, actions)| {
            serde_json::json!({
                "provider_id": id,
                "action_count": actions.len(),
                "actions": actions,
            })
        }).collect::<Vec<_>>(),
        "total_providers": provider_map.len(),
    });

    Ok(warp::reply::with_status(json(&response), StatusCode::OK))
}

/// Handler for `DELETE /api/v1/providers/:provider_id`
pub async fn handle_deregister_provider(
    provider_id: String,
    registry: Arc<ActionRegistry>,
) -> Result<impl Reply, warp::Rejection> {
    registry.deregister_provider(&provider_id).await;

    let response = serde_json::json!({
        "success": true,
        "provider_id": provider_id,
        "message": format!("Provider '{}' deregistered", provider_id),
    });

    Ok(warp::reply::with_status(json(&response), StatusCode::OK))
}

// Re-export for use with futures
use futures::FutureExt;
