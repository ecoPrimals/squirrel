//! Squirrel Primal Provider Module
//!
//! This module provides the concrete implementation of the PrimalProvider trait
//! for the Squirrel AI primal, enabling it to participate in dynamic primal evolution
//! and integrate with the Songbird service mesh.

pub mod core;
pub mod ai_inference;
pub mod session_integration;
pub mod context_analysis;
pub mod ecosystem_integration;
pub mod health_monitoring;

// Re-export the main provider type
pub use core::SquirrelPrimalProvider;

// Re-export commonly used types
pub use ai_inference::{AIInferenceRequest, AIProviderSelection};
pub use session_integration::SessionOperations;
pub use context_analysis::ContextAnalysis;
pub use ecosystem_integration::EcosystemIntegration;
pub use health_monitoring::HealthReporting; 