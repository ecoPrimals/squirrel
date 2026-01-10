//! AI Model Management API
//!
//! Provides endpoints for model discovery, compatibility checking, and loading.
//! Maintains primal self-knowledge - only manages THIS instance's models.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::Reply;

use crate::ecosystem::EcosystemManager;
use crate::hardware::detect_local_gpus;

/// Model information from a provider (Ollama, `HuggingFace`, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier (e.g., "llama2:7b", "mistral:7b")
    pub id: String,
    /// Model name
    pub name: String,
    /// Model provider (ollama, huggingface, openai)
    pub provider: String,
    /// Estimated VRAM requirement in GB
    pub vram_gb: u32,
    /// Model size in GB (on disk)
    pub size_gb: f32,
    /// Model parameters (e.g., "7B", "13B", "70B")
    pub parameters: String,
    /// Model capabilities
    pub capabilities: Vec<String>,
}

/// Compatible model response
#[derive(Debug, Serialize)]
pub struct CompatibleModelsResponse {
    /// Models that can run on THIS instance alone
    pub local_models: Vec<ModelInfo>,
    /// Models that require splitting across towers
    pub split_required: Vec<ModelSplitInfo>,
    /// Total available VRAM on THIS instance
    pub local_vram_gb: u32,
    /// Total available VRAM across all discovered towers
    pub mesh_vram_gb: u32,
    /// Number of towers in mesh (including THIS instance)
    pub tower_count: usize,
}

/// Model split information with performance prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSplitInfo {
    /// Model information
    pub model: ModelInfo,
    /// Required number of towers
    pub required_towers: usize,
    /// Minimum VRAM per tower
    pub min_vram_per_tower: u32,
    /// Can be split with current mesh
    pub splittable: bool,
    /// Performance prediction (if splittable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_prediction: Option<PerformancePredictionResponse>,
}

/// Performance prediction response (simplified for API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePredictionResponse {
    /// Expected tokens per second
    pub tokens_per_sec: f32,
    /// Confidence in prediction (0.0-1.0)
    pub confidence: f32,
    /// Total power consumption (watts)
    pub power_watts: u32,
    /// Efficiency (tokens/sec/watt)
    pub efficiency: f32,
    /// Network overhead (ms per token)
    pub network_overhead_ms: u32,
    /// Time to generate 100 tokens (seconds)
    pub time_for_100_tokens: f32,
    /// Quality rating
    pub quality_rating: QualityRatingResponse,
    /// Per-tower breakdown
    pub tower_breakdown: Vec<TowerPerformanceResponse>,
}

/// Quality rating for the configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRatingResponse {
    pub speed: String,      // "excellent", "good", "acceptable", "slow"
    pub efficiency: String, // "excellent", "good", "acceptable", "poor"
    pub cost: String,       // "excellent", "good", "acceptable", "expensive"
}

/// Performance breakdown for a single tower
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerPerformanceResponse {
    pub tower_id: String,
    pub gpu_model: String,
    pub assigned_layers: u32,
    pub tokens_per_sec: f32,
    pub power_watts: u32,
    pub efficiency: f32,
    pub is_bottleneck: bool,
}

/// Model load request
#[derive(Debug, Deserialize)]
pub struct ModelLoadRequest {
    /// Model ID to load
    pub model_id: String,
    /// Provider (ollama, huggingface, etc.)
    pub provider: String,
    /// Whether to allow splitting
    pub allow_split: bool,
}

/// Model load response
#[derive(Debug, Serialize)]
pub struct ModelLoadResponse {
    /// Whether model was loaded successfully
    pub success: bool,
    /// Model ID that was loaded
    pub model_id: String,
    /// Whether model was split
    pub split: bool,
    /// Towers participating in split (if applicable)
    pub towers: Vec<String>,
    /// VRAM used on THIS instance
    pub vram_used_gb: u32,
    /// Error message if failed
    pub error: Option<String>,
}

/// Handle compatible models endpoint
///
/// Discovers available models from providers and determines which can run
/// on THIS instance vs which require splitting across towers.
///
/// # Primal Self-Knowledge
///
/// - Detects THIS instance's GPU capabilities
/// - Queries Songbird for other towers' capabilities
/// - Does NOT have direct knowledge of other towers
///
/// GET /api/ai/models/compatible
pub async fn handle_compatible_models(
    ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    tracing::info!("Checking compatible models for this instance and mesh");

    // Detect THIS instance's GPU capabilities
    let local_gpu = detect_local_gpus().await.map_err(|e| {
        tracing::error!("Failed to detect local GPU: {}", e);
        warp::reject::reject()
    })?;

    let local_vram_gb = local_gpu.as_ref().map_or(0, |gpu| gpu.total_vram_gb);

    // Discover other towers through Songbird
    let discovered_services = ecosystem_manager
        .find_services_by_type(crate::ecosystem::EcosystemPrimalType::Squirrel)
        .await
        .map_err(|e| {
            tracing::error!("Failed to discover towers: {}", e);
            warp::reject::reject()
        })?;

    // Calculate total mesh VRAM
    let this_tower_id = &ecosystem_manager.config.service_id;
    let mut mesh_vram_gb = local_vram_gb;

    for service in &discovered_services {
        // Skip THIS instance
        if service.service_id.as_ref() == this_tower_id {
            continue;
        }

        // Add other towers' VRAM
        if let Some(vram_str) = service.metadata.get("vram_total_gb") {
            if let Ok(vram) = vram_str.parse::<u32>() {
                mesh_vram_gb += vram;
            }
        }
    }

    tracing::info!(
        "Mesh VRAM: {}GB across {} tower(s) (local: {}GB)",
        mesh_vram_gb,
        discovered_services.len(),
        local_vram_gb
    );

    // Get available models from providers
    let available_models = discover_available_models().await;

    // Categorize models
    let mut local_models = Vec::new();
    let mut split_required = Vec::new();

    for model in available_models {
        if model.vram_gb <= local_vram_gb {
            // Model fits on THIS instance
            local_models.push(model);
        } else {
            // Model requires splitting
            let required_towers = calculate_required_towers(model.vram_gb, local_vram_gb);
            let min_vram_per_tower = model.vram_gb / required_towers as u32;
            let splittable =
                mesh_vram_gb >= model.vram_gb && discovered_services.len() >= required_towers;

            // Generate performance prediction if splittable
            let performance_prediction = if splittable {
                generate_performance_prediction(&model, local_vram_gb, &discovered_services)
                    .await
                    .ok()
            } else {
                None
            };

            split_required.push(ModelSplitInfo {
                model,
                required_towers,
                min_vram_per_tower,
                splittable,
                performance_prediction,
            });
        }
    }

    let response = CompatibleModelsResponse {
        local_models,
        split_required,
        local_vram_gb,
        mesh_vram_gb,
        tower_count: discovered_services.len(),
    };

    Ok(warp::reply::json(&response))
}

/// Handle model load endpoint
///
/// Loads a model on THIS instance or coordinates split loading across towers.
///
/// # Primal Self-Knowledge
///
/// - Loads model on THIS instance
/// - Coordinates with other towers through Songbird (if split required)
/// - Does NOT directly communicate with other towers
///
/// POST /api/ai/model/load
pub async fn handle_model_load(
    request: ModelLoadRequest,
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, warp::Rejection> {
    tracing::info!(
        "Loading model: {} from {} (allow_split: {})",
        request.model_id,
        request.provider,
        request.allow_split
    );

    // Detect THIS instance's GPU
    let local_gpu = detect_local_gpus().await.map_err(|e| {
        tracing::error!("Failed to detect local GPU: {}", e);
        warp::reject::reject()
    })?;

    let local_vram_gb = local_gpu.as_ref().map_or(0, |gpu| gpu.total_vram_gb);

    // For Phase 2, we'll implement mock loading
    // Phase 3 will add real model loading and tensor communication

    // Mock: Calculate required VRAM
    let model_vram = estimate_model_vram(&request.model_id);

    if model_vram <= local_vram_gb {
        // Model fits locally
        tracing::info!(
            "Model {} fits on local GPU ({}GB <= {}GB)",
            request.model_id,
            model_vram,
            local_vram_gb
        );

        let response = ModelLoadResponse {
            success: true,
            model_id: request.model_id,
            split: false,
            towers: vec![],
            vram_used_gb: model_vram,
            error: None,
        };

        Ok(warp::reply::json(&response))
    } else if request.allow_split {
        // Model requires split - Phase 3 will implement this
        tracing::warn!(
            "Model {} requires split ({}GB > {}GB) - split loading not yet implemented",
            request.model_id,
            model_vram,
            local_vram_gb
        );

        let response = ModelLoadResponse {
            success: false,
            model_id: request.model_id,
            split: false,
            towers: vec![],
            vram_used_gb: 0,
            error: Some("Split model loading not yet implemented (Phase 3)".to_string()),
        };

        Ok(warp::reply::json(&response))
    } else {
        // Model doesn't fit and split not allowed
        tracing::error!(
            "Model {} doesn't fit on local GPU and split not allowed",
            request.model_id
        );

        let response = ModelLoadResponse {
            success: false,
            model_id: request.model_id,
            split: false,
            towers: vec![],
            vram_used_gb: 0,
            error: Some(format!(
                "Model requires {model_vram}GB VRAM but only {local_vram_gb}GB available. Enable split mode."
            )),
        };

        Ok(warp::reply::json(&response))
    }
}

/// Discover available models from providers
///
/// # Implementation Note
///
/// This is a mock implementation for Phase 2. Phase 3 will query actual providers.
async fn discover_available_models() -> Vec<ModelInfo> {
    // Mock models for testing
    vec![
        ModelInfo {
            id: "llama2:7b".to_string(),
            name: "Llama 2 7B".to_string(),
            provider: "ollama".to_string(),
            vram_gb: 8,
            size_gb: 3.8,
            parameters: "7B".to_string(),
            capabilities: vec!["text_generation".to_string(), "chat".to_string()],
        },
        ModelInfo {
            id: "llama2:13b".to_string(),
            name: "Llama 2 13B".to_string(),
            provider: "ollama".to_string(),
            vram_gb: 16,
            size_gb: 7.3,
            parameters: "13B".to_string(),
            capabilities: vec!["text_generation".to_string(), "chat".to_string()],
        },
        ModelInfo {
            id: "llama2:70b".to_string(),
            name: "Llama 2 70B".to_string(),
            provider: "ollama".to_string(),
            vram_gb: 80,
            size_gb: 38.0,
            parameters: "70B".to_string(),
            capabilities: vec!["text_generation".to_string(), "chat".to_string()],
        },
        ModelInfo {
            id: "mixtral:8x7b".to_string(),
            name: "Mixtral 8x7B".to_string(),
            provider: "ollama".to_string(),
            vram_gb: 45,
            size_gb: 26.0,
            parameters: "8x7B".to_string(),
            capabilities: vec!["text_generation".to_string(), "chat".to_string()],
        },
    ]
}

/// Calculate required number of towers for a model
fn calculate_required_towers(model_vram: u32, tower_vram: u32) -> usize {
    if tower_vram == 0 {
        return 0;
    }

    // Add 10% overhead for safety
    let required_vram = (model_vram as f32 * 1.1) as u32;
    required_vram.div_ceil(tower_vram).max(2) as usize
}

/// Estimate model VRAM requirement from model ID
fn estimate_model_vram(model_id: &str) -> u32 {
    // Convert to lowercase for case-insensitive matching
    let id_lower = model_id.to_lowercase();

    // Simple heuristic based on parameter count
    if id_lower.contains("8x7b") {
        45 // Mixtral 8x7B
    } else if id_lower.contains("7b") {
        8
    } else if id_lower.contains("13b") {
        16
    } else if id_lower.contains("33b") {
        40
    } else if id_lower.contains("70b") {
        80
    } else {
        // Default: assume 10GB
        10
    }
}

/// Estimate number of layers for a model based on parameters
fn estimate_model_layers(parameters: &str) -> u32 {
    // Rough estimates for common model sizes
    if parameters.contains("70") || parameters.contains("70B") {
        80
    } else if parameters.contains("33") || parameters.contains("33B") {
        60
    } else if parameters.contains("13") || parameters.contains("13B") {
        40
    } else if parameters.contains('7') || parameters.contains("7B") {
        32
    } else {
        32 // Default
    }
}

/// Generate performance prediction for a splittable model
///
/// NOTE: This is a simplified estimation. For actual model splitting and performance
/// prediction, delegate to `ToadStool` via capability discovery.
async fn generate_performance_prediction(
    model: &ModelInfo,
    local_vram_gb: u32,
    discovered_services: &[crate::ecosystem::registry::types::DiscoveredService],
) -> Result<PerformancePredictionResponse, String> {
    // TODO: Evolve to capability-based discovery of ToadStool's model analysis service
    // For now, provide simplified estimation

    let tower_count = 1 + discovered_services.len();
    let total_vram = local_vram_gb
        + discovered_services
            .iter()
            .filter_map(|s| s.metadata.get("vram_total_gb")?.parse::<u32>().ok())
            .sum::<u32>();

    // Simplified performance estimation
    // In production: Query ToadStool's performance prediction API
    let tokens_per_sec = if model.vram_gb <= local_vram_gb {
        100.0 // Local execution
    } else {
        50.0 / (tower_count as f32) // Split penalty
    };

    let confidence = if model.vram_gb <= local_vram_gb {
        0.9
    } else {
        0.6
    };
    let power_watts = 350 * tower_count as u32;
    let efficiency = tokens_per_sec / power_watts as f32;
    let network_overhead_ms = if tower_count > 1 {
        10 * (tower_count - 1) as u32
    } else {
        0
    };
    let time_for_100_tokens = 100.0 / tokens_per_sec;

    // Convert to API response format
    Ok(PerformancePredictionResponse {
        tokens_per_sec,
        confidence,
        power_watts,
        efficiency,
        network_overhead_ms,
        time_for_100_tokens,
        quality_rating: QualityRatingResponse {
            speed: if tokens_per_sec > 80.0 {
                "excellent".to_string()
            } else if tokens_per_sec > 50.0 {
                "good".to_string()
            } else {
                "acceptable".to_string()
            },
            efficiency: if efficiency > 0.2 {
                "excellent".to_string()
            } else {
                "good".to_string()
            },
            cost: if tower_count == 1 {
                "excellent".to_string()
            } else {
                "good".to_string()
            },
        },
        tower_breakdown: vec![], // Simplified - in production, query ToadStool
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_required_towers() {
        // 80GB model, 24GB per tower -> need 4 towers (80 * 1.1 = 88GB / 24GB = 3.67 -> 4)
        assert_eq!(calculate_required_towers(80, 24), 4);

        // 45GB model, 24GB per tower -> need 3 towers (45 * 1.1 = 49.5GB / 24GB = 2.06 -> 3)
        assert_eq!(calculate_required_towers(45, 24), 3);

        // 8GB model, 24GB per tower -> need 2 towers minimum
        assert_eq!(calculate_required_towers(8, 24), 2);
    }

    #[test]
    fn test_estimate_model_vram() {
        assert_eq!(estimate_model_vram("llama2:7b"), 8);
        assert_eq!(estimate_model_vram("llama2:13B"), 16);
        assert_eq!(estimate_model_vram("llama2:70b"), 80);
        assert_eq!(estimate_model_vram("mixtral:8x7b"), 45);
        assert_eq!(estimate_model_vram("mixtral:8x7B"), 45); // Case insensitive
        assert_eq!(estimate_model_vram("unknown"), 10);
    }
}
