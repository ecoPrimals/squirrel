// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Squirrel Primal Provider Module
//!
//! This module provides the concrete implementation of the `PrimalProvider` trait
//! for the Squirrel AI primal, enabling it to participate in dynamic primal evolution
//! and integrate with the Songbird service mesh.

pub mod ai_inference;
pub mod context_analysis;
pub mod core;
pub mod ecosystem_integration;
pub mod health_monitoring;
pub mod session_integration;

#[cfg(test)]
mod ai_inference_tests;
#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod core_tests;

// Re-export the main provider type
pub use core::SquirrelPrimalProvider;

// Re-export commonly used types
pub use ai_inference::{AIInferenceRequest, AIProviderSelection};
pub use context_analysis::ContextAnalysis;
pub use ecosystem_integration::EcosystemIntegration;
pub use health_monitoring::HealthReporting;
pub use session_integration::SessionOperations;
