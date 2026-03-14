// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! AI Subsystem
//!
//! Central AI capabilities including routing and providers.
//! Model splitting has been moved to ToadStool and Songbird.

pub mod model_splitting;

// Re-export deprecated stubs for backward compatibility
// Backward compatibility: AI module uses deprecated types during migration
#[allow(deprecated)]
pub use model_splitting::{
    LayerDistribution, LayerDistributionStrategy, ModelSplitConfig, ModelSplitCoordinator,
    ModelSplitState, PerformancePrediction, SplitSession, SplitStatus, TensorMessage,
    TensorProtocol, TowerAssignment, TowerCapability,
};
