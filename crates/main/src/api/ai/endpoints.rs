//! AI API endpoint handlers
//!
//! Modern async handlers for universal AI capability endpoints.

use super::router::AiRouter;
use super::types::{
    AiErrorResponse, ImageGenerationRequest, ResponseMetadata, TextGenerationRequest,
    UniversalAiRequest, UniversalAiResponse,
};
use std::sync::Arc;
use warp::{http::StatusCode, reply::json, Reply};

/// Handler for `/ai/generate-image` endpoint
pub async fn handle_generate_image(
    request: ImageGenerationRequest,
    router: Arc<AiRouter>,
) -> Result<impl Reply, warp::Rejection> {
    match router.generate_image(request, None).await {
        Ok(response) => Ok(warp::reply::with_status(json(&response), StatusCode::OK)),
        Err(e) => {
            let error = AiErrorResponse::new(
                "image_generation_failed",
                format!("Image generation failed: {e}"),
            )
            .retryable();

            Ok(warp::reply::with_status(
                json(&error),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// Handler for `/ai/generate-text` endpoint
pub async fn handle_generate_text(
    request: TextGenerationRequest,
    router: Arc<AiRouter>,
) -> Result<impl Reply, warp::Rejection> {
    match router.generate_text(request, None).await {
        Ok(response) => Ok(warp::reply::with_status(json(&response), StatusCode::OK)),
        Err(e) => {
            let error = AiErrorResponse::new(
                "text_generation_failed",
                format!("Text generation failed: {e}"),
            )
            .retryable();

            Ok(warp::reply::with_status(
                json(&error),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// Handler for `/ai/execute` universal endpoint
pub async fn handle_execute_ai(
    request: UniversalAiRequest,
    router: Arc<AiRouter>,
) -> Result<impl Reply, warp::Rejection> {
    // Route based on action type
    match request.action.as_str() {
        "image.generation" => {
            // Convert to ImageGenerationRequest
            let image_request: ImageGenerationRequest = serde_json::from_value(request.input)
                .map_err(|e| {
                    warp::reject::custom(ApiError::BadRequest(format!(
                        "Invalid image generation request: {e}"
                    )))
                })?;

            match router
                .generate_image(image_request, request.requirements)
                .await
            {
                Ok(response) => {
                    // Evolution: Proper error handling instead of unwrap_or_default
                    let output = serde_json::to_value(&response).map_err(|e| {
                        warp::reject::custom(ApiError::InternalError(format!(
                            "Failed to serialize response: {e}"
                        )))
                    })?;

                    let universal_response = UniversalAiResponse {
                        action: request.action.clone(),
                        output,
                        metadata: ResponseMetadata {
                            provider_id: response.provider_id.clone(),
                            provider_name: response.provider_id.clone(),
                            cost_usd: response.cost_usd,
                            latency_ms: response.latency_ms,
                            timestamp: chrono::Utc::now(),
                            extras: std::collections::HashMap::new(),
                        },
                    };

                    Ok(warp::reply::with_status(
                        json(&universal_response),
                        StatusCode::OK,
                    ))
                }
                Err(e) => {
                    let error =
                        AiErrorResponse::new("action_failed", format!("Action failed: {e}"))
                            .retryable();

                    Ok(warp::reply::with_status(
                        json(&error),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        "text.generation" => {
            // Convert to TextGenerationRequest
            let text_request: TextGenerationRequest = serde_json::from_value(request.input)
                .map_err(|e| {
                    warp::reject::custom(ApiError::BadRequest(format!(
                        "Invalid text generation request: {e}"
                    )))
                })?;

            match router
                .generate_text(text_request, request.requirements)
                .await
            {
                Ok(response) => {
                    // Evolution: Proper error handling instead of unwrap_or_default
                    let output = serde_json::to_value(&response).map_err(|e| {
                        warp::reject::custom(ApiError::InternalError(format!(
                            "Failed to serialize response: {e}"
                        )))
                    })?;

                    let universal_response = UniversalAiResponse {
                        action: request.action.clone(),
                        output,
                        metadata: ResponseMetadata {
                            provider_id: response.provider_id.clone(),
                            provider_name: response.model.clone(),
                            cost_usd: response.cost_usd,
                            latency_ms: response.latency_ms,
                            timestamp: chrono::Utc::now(),
                            extras: std::collections::HashMap::new(),
                        },
                    };

                    Ok(warp::reply::with_status(
                        json(&universal_response),
                        StatusCode::OK,
                    ))
                }
                Err(e) => {
                    let error =
                        AiErrorResponse::new("action_failed", format!("Action failed: {e}"))
                            .retryable();

                    Ok(warp::reply::with_status(
                        json(&error),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        // PrimalPulse custom actions
        action
            if action.starts_with("primal.")
                || action.starts_with("rootpulse.")
                || action.starts_with("neural.") =>
        {
            handle_primalpulse_action(request, router).await
        }
        _ => {
            let error = AiErrorResponse::new(
                "unsupported_action",
                format!(
                    "Action '{}' is not supported. Available: image.generation, text.generation, primal.analyze, primal.audit_hardcoding, rootpulse.semantic_commit, neural.graph_optimize",
                    request.action
                ),
            );

            Ok(warp::reply::with_status(
                json(&error),
                StatusCode::BAD_REQUEST,
            ))
        }
    }
}

/// Handler for PrimalPulse custom actions
async fn handle_primalpulse_action(
    request: UniversalAiRequest,
    router: Arc<AiRouter>,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, warp::Rejection> {
    use crate::primal_pulse::handlers;

    let constraints = vec![]; // TODO: Extract from request.requirements

    let result = match request.action.as_str() {
        "primal.analyze" => {
            handlers::handle_primal_analyze(
                request.input.clone(),
                router.clone(),
                constraints.clone(),
            )
            .await
        }
        "primal.audit_hardcoding" => {
            handlers::handle_primal_audit_hardcoding(
                request.input.clone(),
                router.clone(),
                constraints.clone(),
            )
            .await
        }
        "rootpulse.semantic_commit" => {
            handlers::handle_rootpulse_semantic_commit(
                request.input.clone(),
                router.clone(),
                constraints.clone(),
            )
            .await
        }
        "neural.graph_optimize" => {
            crate::primal_pulse::neural_graph::handle_neural_graph_optimize(
                request.input.clone(),
                router.clone(),
                constraints,
            )
            .await
        }
        _ => {
            return Ok(warp::reply::with_status(
                json(&AiErrorResponse::new(
                    "unknown_action",
                    format!("Unknown PrimalPulse action: {}", request.action),
                )),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    match result {
        Ok(response) => Ok(warp::reply::with_status(json(&response), StatusCode::OK)),
        Err(e) => {
            let error = AiErrorResponse::new(
                "primalpulse_action_failed",
                format!("PrimalPulse action failed: {}", e),
            )
            .retryable();
            Ok(warp::reply::with_status(
                json(&error),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// Handler for `/api/v1/capabilities` endpoint
pub async fn handle_query_capabilities(
    router: Arc<AiRouter>,
) -> Result<impl Reply, warp::Rejection> {
    let providers = router.list_providers().await;

    let capabilities = serde_json::json!({
        "capabilities": [
            {
                "action": "image.generation",
                "providers": providers.iter()
                    .filter(|p| p.capabilities.contains(&"image.generation".to_string()))
                    .count(),
                "available": providers.iter()
                    .any(|p| p.capabilities.contains(&"image.generation".to_string()) && p.is_available),
            },
            {
                "action": "text.generation",
                "providers": providers.iter()
                    .filter(|p| p.capabilities.contains(&"text.generation".to_string()))
                    .count(),
                "available": providers.iter()
                    .any(|p| p.capabilities.contains(&"text.generation".to_string()) && p.is_available),
            }
        ],
        "providers": providers.iter().map(|p| {
            serde_json::json!({
                "id": p.provider_id,
                "name": p.provider_name,
                "capabilities": p.capabilities,
                "available": p.is_available,
                "quality": format!("{:?}", p.quality_tier),
                "cost_per_unit": p.cost_per_unit,
                "avg_latency_ms": p.avg_latency_ms,
            })
        }).collect::<Vec<_>>(),
    });

    Ok(warp::reply::with_status(
        json(&capabilities),
        StatusCode::OK,
    ))
}

/// Custom API error type for rejections
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    ProviderNotFound(String),
    ProviderError(String),
    InvalidRequest(String),
    InternalError(String),
}

impl warp::reject::Reject for ApiError {}
