//! AI capability endpoints
//!
//! Universal AI capability routing with vendor-agnostic provider selection.
//!
//! ## Features
//!
//! - **Universal endpoints**: Single interface for all AI types
//! - **Intelligent routing**: Automatic provider selection based on requirements
//! - **Vendor agnostic**: Zero hardcoding, environment-based discovery
//! - **Modern async**: Fully async/await, naturally concurrent
//! - **Error handling**: Retries, fallback, graceful degradation
//!
//! ## Endpoints
//!
//! - `POST /ai/generate-image` - Generate images
//! - `POST /ai/generate-text` - Generate text
//! - `POST /ai/execute` - Universal endpoint for any action
//! - `GET /api/v1/capabilities` - Query available capabilities
//!
//! ## Example
//!
//! ```bash
//! # Generate an image
//! curl -X POST http://localhost:9090/ai/generate-image \
//!   -H "Content-Type: application/json" \
//!   -d '{
//!     "prompt": "A futuristic AI network",
//!     "size": "512x512",
//!     "n": 1
//!   }'
//! ```

use std::sync::Arc;
use warp::Filter;

pub mod action_registry;
mod adapters;
mod constraint_router;
mod constraints;
mod endpoints;
mod models;
mod provider_registration;
pub mod router;
mod selector;
// service_mesh_integration deleted - HTTP-based AI integration deprecated
pub mod types;

pub use action_registry::ActionRegistry;
pub use router::AiRouter;
// ServiceMeshAiIntegration removed - use capability discovery instead

// Re-export handlers
pub use endpoints::{
    handle_execute_ai, handle_generate_image, handle_generate_text, handle_query_capabilities,
};
pub use models::{handle_compatible_models, handle_model_load};
pub use provider_registration::{
    handle_deregister_provider, handle_list_actions, handle_list_providers,
    handle_register_provider,
};

/// Create AI routes for warp server (Phase 1-4)
pub fn ai_routes(
    router: Arc<AiRouter>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let generate_image = warp::path!("ai" / "generate-image")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_router(router.clone()))
        .and_then(handle_generate_image);

    let generate_text = warp::path!("ai" / "generate-text")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_router(router.clone()))
        .and_then(handle_generate_text);

    let execute = warp::path!("ai" / "execute")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_router(router.clone()))
        .and_then(handle_execute_ai);

    let capabilities = warp::path!("api" / "v1" / "capabilities")
        .and(warp::get())
        .and(with_router(router.clone()))
        .and_then(handle_query_capabilities);

    generate_image
        .or(generate_text)
        .or(execute)
        .or(capabilities)
}

/// Create provider registration routes (Phase 6)
pub fn provider_routes(
    registry: Arc<ActionRegistry>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let register_provider = warp::path!("api" / "v1" / "providers" / "register")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_registry(registry.clone()))
        .and_then(handle_register_provider);

    let list_providers = warp::path!("api" / "v1" / "providers")
        .and(warp::get())
        .and(with_registry(registry.clone()))
        .and_then(handle_list_providers);

    let deregister_provider = warp::path!("api" / "v1" / "providers" / String)
        .and(warp::delete())
        .and(with_registry(registry.clone()))
        .and_then(handle_deregister_provider);

    let list_actions = warp::path!("api" / "v1" / "actions")
        .and(warp::get())
        .and(with_registry(registry.clone()))
        .and_then(handle_list_actions);

    register_provider
        .or(list_providers)
        .or(deregister_provider)
        .or(list_actions)
}

/// Warp filter to inject router
fn with_router(
    router: Arc<AiRouter>,
) -> impl Filter<Extract = (Arc<AiRouter>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || router.clone())
}

/// Warp filter to inject action registry
fn with_registry(
    registry: Arc<ActionRegistry>,
) -> impl Filter<Extract = (Arc<ActionRegistry>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || registry.clone())
}
