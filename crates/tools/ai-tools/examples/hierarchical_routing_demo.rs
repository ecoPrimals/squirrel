//! Hierarchical AI Router Demo
//!
//! This demo shows how to use the hierarchical routing capabilities
//! of the AI tools system to route requests across multiple AI providers
//! and models in a sophisticated network topology.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::time::sleep;
use tracing::{debug, info, warn};
use uuid::Uuid;

use squirrel_ai_tools::{
    common::{
        capability::{AITask, SecurityLevel, SecurityRequirements, TaskType},
        clients::mock::MockAIClient,
        ChatMessage, ChatRequest, MessageRole, ModelParameters,
    },
    router::{AIRouter, RequestContext, RouterConfig, RoutingHint, RoutingStrategy},
    Result,
};

use serde_json::json;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Demo temporarily disabled - API updates needed");
    Ok(())
}

/// Example configuration for production deployment
pub fn create_production_config() -> serde_json::Value {
    json!({
        "providers": {
            "openrouter": {
                "type": "openrouter",
                "enabled": true,
                "priority": 70,
                "settings": {
                    "api_key": "${OPENROUTER_API_KEY}",
                    "app_name": "Squirrel Production",
                    "site_url": "https://squirrel.example.com"
                },
                "rate_limits": {
                    "requests_per_minute": 60,
                    "requests_per_hour": 1000
                }
            },
            "llamacpp_local": {
                "type": "llamacpp",
                "enabled": true,
                "priority": 90,
                "settings": {
                    "endpoint": "http://localhost:8080",
                    "model_name": "llama-2-70b-chat",
                    "context_size": 8192
                }
            },
            "llamacpp_gpu_cluster": {
                "type": "llamacpp",
                "enabled": true,
                "priority": 95,
                "settings": {
                    "endpoint": "http://gpu-cluster.internal:8080",
                    "model_name": "llama-2-70b-chat",
                    "context_size": 16384
                }
            }
        },
        "routing": {
            "default_strategy": "BestFit",
            "allow_remote_routing": true,
            "timeout_ms": 30000,
            "max_attempts": 3
        },
        "squirrel_nodes": [
            {
                "id": "gpu-cluster-1",
                "name": "GPU Cluster West",
                "endpoint": "https://gpu-west.squirrel.example.com",
                "priority": 95,
                "region": "us-west-2",
                "tags": ["gpu", "high-performance"]
            },
            {
                "id": "edge-node-1",
                "name": "Edge Node East",
                "endpoint": "https://edge-east.squirrel.example.com",
                "priority": 70,
                "region": "us-east-1",
                "tags": ["edge", "low-latency"]
            }
        ]
    })
}
