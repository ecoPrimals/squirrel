// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Json, IntoResponse},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::router::{AIRouter, RequestContext, RoutingHint};
use crate::common::{ChatRequest, ChatResponse, AIClient};
use crate::common::capability::{AITask, AICapabilities};
use crate::Result;

/// API state containing the router and other shared resources
#[derive(Clone)]
pub struct ApiState {
    pub router: Arc<AIRouter>,
    pub squirrel_registry: Arc<SquirrelRegistry>,
}

/// Registry for managing hierarchical squirrel nodes
#[derive(Debug)]
pub struct SquirrelRegistry {
    nodes: tokio::sync::RwLock<HashMap<String, SquirrelNode>>,
}

/// Represents a squirrel node in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquirrelNode {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub capabilities: HashMap<String, AICapabilities>,
    pub status: NodeStatus,
    pub priority: u8,
    pub region: Option<String>,
    pub tags: Vec<String>,
}

/// Status of a squirrel node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Degraded,
    Maintenance,
}

/// Request for routing AI tasks
#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub request_id: Option<String>,
    pub squirrel_id: Option<String>,
    pub task: AITask,
    pub routing_hint: Option<RoutingHint>,
    pub chat_request: ChatRequest,
    pub user_preference: Option<UserPreference>,
}

/// User preferences for provider selection
#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreference {
    pub preferred_provider: Option<String>,
    pub preferred_model: Option<String>,
    pub cost_preference: Option<CostPreference>,
    pub privacy_level: Option<PrivacyLevel>,
    pub performance_preference: Option<PerformancePreference>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CostPreference {
    Free,
    Low,
    Medium,
    High,
    NoLimit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Public,      // Can use any provider
    Restricted,  // Prefer local/private providers
    Private,     // Only local providers
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PerformancePreference {
    Fast,        // Prioritize speed
    Quality,     // Prioritize output quality
    Balanced,    // Balance speed and quality
}

/// Response for routing requests
#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingResponse {
    pub request_id: String,
    pub provider_used: String,
    pub model_used: Option<String>,
    pub squirrel_used: Option<String>,
    pub response: ChatResponse,
    pub routing_metadata: RoutingMetadata,
}

/// Metadata about the routing decision
#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingMetadata {
    pub routing_strategy: String,
    pub candidates_considered: u32,
    pub routing_time_ms: u64,
    pub cost_estimate: Option<f64>,
    pub privacy_level: String,
}

/// Provider information for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub status: ProviderStatus,
    pub capabilities: AICapabilities,
    pub supported_models: Vec<String>,
    pub cost_tier: String,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProviderType {
    Cloud,
    Local,
    Hybrid,
    Remote,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProviderStatus {
    Available,
    Unavailable,
    RateLimited,
    Error,
}

impl SquirrelRegistry {
    pub fn new() -> Self {
        Self {
            nodes: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_node(&self, node: SquirrelNode) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.id.clone(), node);
        Ok(())
    }

    pub async fn get_node(&self, id: &str) -> Option<SquirrelNode> {
        let nodes = self.nodes.read().await;
        nodes.get(id).cloned()
    }

    pub async fn list_nodes(&self) -> Vec<SquirrelNode> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    pub async fn find_capable_nodes(&self, task: &AITask) -> Vec<SquirrelNode> {
        let nodes = self.nodes.read().await;
        nodes.values()
            .filter(|node| self.node_can_handle_task(node, task))
            .cloned()
            .collect()
    }

    fn node_can_handle_task(&self, node: &SquirrelNode, task: &AITask) -> bool {
        // Check if any provider on this node can handle the task
        node.capabilities.values().any(|caps| {
            caps.supported_task_types.contains(&task.task_type)
        })
    }
}

/// Create the API router
pub fn create_api_router(state: ApiState) -> Router {
    Router::new()
        .route("/ai/route", axum::routing::post(route_request))
        .route("/ai/route/stream", axum::routing::post(route_request_stream))
        .route("/ai/providers", axum::routing::get(list_providers))
        .route("/ai/providers/:provider_id", axum::routing::get(get_provider_info))
        .route("/squirrels", axum::routing::get(list_squirrels))
        .route("/squirrels", axum::routing::post(register_squirrel))
        .route("/squirrels/:squirrel_id", axum::routing::get(get_squirrel))
        .route("/squirrels/:squirrel_id/route", axum::routing::post(route_to_squirrel))
        .route("/squirrels/:squirrel_id/capabilities", axum::routing::get(get_squirrel_capabilities))
        .route("/routing/strategies", axum::routing::get(list_routing_strategies))
        .route("/routing/health", axum::routing::get(routing_health_check))
        .with_state(state)
}

/// Route an AI request
async fn route_request(
    State(state): State<ApiState>,
    Json(request): Json<RoutingRequest>,
) -> Result<Json<RoutingResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    let request_id = request.request_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Create request context
    let mut context = RequestContext::new(request.task);
    context.request_id = Uuid::parse_str(&request_id).unwrap_or_else(|_| Uuid::new_v4());
    
    if let Some(hint) = request.routing_hint {
        context = context.with_routing_hint(hint);
    }

    // Apply user preferences to routing hint
    if let Some(prefs) = request.user_preference {
        context = apply_user_preferences(context, prefs);
    }

    // Route to specific squirrel if requested
    if let Some(squirrel_id) = &request.squirrel_id {
        return route_to_specific_squirrel(&state, &request_id, squirrel_id, request.chat_request, context, start_time).await;
    }

    // Route using the main router
    match state.router.process_request(request.chat_request, context.clone()).await {
        Ok(response) => {
            let routing_time = start_time.elapsed().as_millis() as u64;
            
            // Get model used from the response
            let model_used = response.model.clone();
            
            // Calculate cost estimate based on response usage
            let cost_estimate = if let Some(usage) = &response.usage {
                Some((usage.total_tokens as f64) * 0.001) // Simple estimation: $0.001 per token
            } else {
                None
            };
            
            Ok(Json(RoutingResponse {
                request_id,
                provider_used: "auto-selected".to_string(), // Router doesn't track this currently
                model_used: Some(model_used),
                squirrel_used: None,
                response,
                routing_metadata: RoutingMetadata {
                    routing_strategy: format!("{:?}", state.router.config.routing_strategy),
                    candidates_considered: 1, // Router doesn't track this currently
                    routing_time_ms: routing_time,
                    cost_estimate,
                    privacy_level: if context.task.security_requirements.contains_sensitive_data {
                        "private".to_string()
                    } else {
                        "public".to_string()
                    },
                },
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Route a streaming AI request
async fn route_request_stream(
    State(state): State<ApiState>,
    Json(request): Json<RoutingRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    use futures::StreamExt;
    use axum::response::sse::{Event, Sse};
    
    let request_id = request.request_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut context = RequestContext::new(request.task);
    
    if let Some(hint) = request.routing_hint {
        context.routing_hint = Some(hint);
    }
    
    if let Some(prefs) = request.user_preference {
        context = apply_user_preferences(context, prefs);
    }
    
    // Process streaming request
    match state.router.process_stream_request(request.chat_request, context).await {
        Ok(stream) => {
            let sse_stream = stream.map(|chunk_result| {
                match chunk_result {
                    Ok(chunk) => {
                        let json = serde_json::to_string(&chunk).unwrap_or_default();
                        Ok(Event::default().data(json))
                    }
                    Err(e) => {
                        let error_json = serde_json::json!({
                            "error": e.to_string()
                        });
                        Ok(Event::default().data(error_json.to_string()))
                    }
                }
            });
            
            Ok(Sse::new(sse_stream))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// List available providers
async fn list_providers(
    State(state): State<ApiState>,
) -> Result<Json<Vec<ProviderInfo>>, StatusCode> {
    let provider_ids = state.router.registry().list_providers();
    let mut providers = Vec::new();
    
    for provider_id in provider_ids {
        if let Some(provider) = state.router.registry().get_provider(&provider_id) {
            let capabilities = provider.capabilities();
            let supported_models = provider.list_models().await.unwrap_or_default();
            
            // Determine provider type based on provider name
            let provider_type = match provider_id.as_str() {
                "local" | "local-server" | "native" => ProviderType::Local,
                "openai" | "anthropic" | "gemini" => ProviderType::Cloud,
                _ => ProviderType::Hybrid,
            };
            
            // Determine cost tier based on capabilities
            let cost_tier = if capabilities.cost_metrics.is_free {
                "free".to_string()
            } else if let Some(cost) = capabilities.cost_metrics.cost_per_1k_input_tokens {
                if cost > 0.01 {
                    "high".to_string()
                } else if cost > 0.002 {
                    "medium".to_string()
                } else {
                    "low".to_string()
                }
            } else {
                "medium".to_string()
            };
            
            providers.push(ProviderInfo {
                id: provider_id.clone(),
                name: provider.provider_name().to_string(),
                provider_type,
                status: if provider.is_available().await {
                    ProviderStatus::Available
                } else {
                    ProviderStatus::Unavailable
                },
                capabilities,
                supported_models,
                cost_tier,
                region: None, // Could be extracted from provider configuration
            });
        }
    }
    
    Ok(Json(providers))
}

/// Get information about a specific provider
async fn get_provider_info(
    State(state): State<ApiState>,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderInfo>, StatusCode> {
    if let Some(provider) = state.router.registry().get_provider(&provider_id) {
        let capabilities = provider.capabilities();
        let supported_models = provider.list_models().await.unwrap_or_default();
        
        // Determine provider type based on provider name
        let provider_type = match provider_id.as_str() {
            "local" | "local-server" | "native" => ProviderType::Local,
            "openai" | "anthropic" | "gemini" => ProviderType::Cloud,
            _ => ProviderType::Hybrid,
        };
        
        // Determine cost tier based on capabilities
        let cost_tier = if capabilities.cost_metrics.is_free {
            "free".to_string()
        } else if let Some(cost) = capabilities.cost_metrics.cost_per_1k_input_tokens {
            if cost > 0.01 {
                "high".to_string()
            } else if cost > 0.002 {
                "medium".to_string()
            } else {
                "low".to_string()
            }
        } else {
            "medium".to_string()
        };
        
        let provider_info = ProviderInfo {
            id: provider_id.clone(),
            name: provider.provider_name().to_string(),
            provider_type,
            status: if provider.is_available().await {
                ProviderStatus::Available
            } else {
                ProviderStatus::Unavailable
            },
            capabilities,
            supported_models,
            cost_tier,
            region: None,
        };
        
        Ok(Json(provider_info))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// List registered squirrels
async fn list_squirrels(
    State(state): State<ApiState>,
) -> Result<Json<Vec<SquirrelNode>>, StatusCode> {
    let nodes = state.squirrel_registry.list_nodes().await;
    Ok(Json(nodes))
}

/// Register a new squirrel node
async fn register_squirrel(
    State(state): State<ApiState>,
    Json(node): Json<SquirrelNode>,
) -> Result<Json<SquirrelNode>, StatusCode> {
    match state.squirrel_registry.register_node(node.clone()).await {
        Ok(_) => Ok(Json(node)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get information about a specific squirrel
async fn get_squirrel(
    State(state): State<ApiState>,
    Path(squirrel_id): Path<String>,
) -> Result<Json<SquirrelNode>, StatusCode> {
    match state.squirrel_registry.get_node(&squirrel_id).await {
        Some(node) => Ok(Json(node)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Route a request to a specific squirrel
async fn route_to_squirrel(
    State(state): State<ApiState>,
    Path(squirrel_id): Path<String>,
    Json(request): Json<RoutingRequest>,
) -> Result<Json<RoutingResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    let request_id = request.request_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    route_to_specific_squirrel(&state, &request_id, &squirrel_id, request.chat_request, 
                              RequestContext::new(request.task), start_time).await
}

/// Get capabilities of a specific squirrel
async fn get_squirrel_capabilities(
    State(state): State<ApiState>,
    Path(squirrel_id): Path<String>,
) -> Result<Json<HashMap<String, AICapabilities>>, StatusCode> {
    match state.squirrel_registry.get_node(&squirrel_id).await {
        Some(node) => Ok(Json(node.capabilities)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// List available routing strategies
async fn list_routing_strategies() -> Json<Vec<String>> {
    Json(vec![
        "FirstMatch".to_string(),
        "HighestPriority".to_string(),
        "LowestLatency".to_string(),
        "LowestCost".to_string(),
        "BestFit".to_string(),
        "RoundRobin".to_string(),
    ])
}

/// Health check for routing system
async fn routing_health_check(
    State(_state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Helper functions

async fn route_to_specific_squirrel(
    state: &ApiState,
    request_id: &str,
    squirrel_id: &str,
    chat_request: ChatRequest,
    context: RequestContext,
    start_time: std::time::Instant,
) -> Result<Json<RoutingResponse>, StatusCode> {
    match state.squirrel_registry.get_node(squirrel_id).await {
        Some(node) => {
            let routing_time = start_time.elapsed().as_millis() as u64;
            
            // For now, simulate remote routing by processing locally
            // In a real implementation, this would make an HTTP/MCP call to the remote squirrel
            match state.router.process_request(chat_request, context.clone()).await {
                Ok(response) => {
                    let cost_estimate = if let Some(usage) = &response.usage {
                        Some((usage.total_tokens as f64) * 0.001)
                    } else {
                        None
                    };
                    
                    Ok(Json(RoutingResponse {
                        request_id: request_id.to_string(),
                        provider_used: format!("remote-squirrel-{}", squirrel_id),
                        model_used: Some(response.model.clone()),
                        squirrel_used: Some(node.name.clone()),
                        response,
                        routing_metadata: RoutingMetadata {
                            routing_strategy: "RemoteSquirrel".to_string(),
                            candidates_considered: 1,
                            routing_time_ms: routing_time,
                            cost_estimate,
                            privacy_level: if context.task.security_requirements.contains_sensitive_data {
                                "private".to_string()
                            } else {
                                "public".to_string()
                            },
                        },
                    }))
                }
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

fn apply_user_preferences(mut context: RequestContext, prefs: UserPreference) -> RequestContext {
    let mut hint = context.routing_hint.unwrap_or_default();
    
    if let Some(provider) = prefs.preferred_provider {
        hint.preferred_provider = Some(provider);
    }
    
    if let Some(model) = prefs.preferred_model {
        hint.preferred_model = Some(model);
    }
    
    // Apply privacy preferences
    if let Some(privacy) = prefs.privacy_level {
        match privacy {
            PrivacyLevel::Private => {
                hint.allow_remote = Some(false);
                context.task.security_requirements.contains_sensitive_data = true;
            }
            PrivacyLevel::Restricted => {
                // Prefer local but allow remote if necessary
            }
            PrivacyLevel::Public => {
                hint.allow_remote = Some(true);
            }
        }
    }
    
    context.with_routing_hint(hint)
}

impl Default for RoutingHint {
    fn default() -> Self {
        Self {
            preferred_provider: None,
            preferred_model: None,
            allow_remote: None,
            max_latency_ms: None,
            max_cost_tier: None,
            priority: None,
        }
    }
} 