//! AI Subsystem
//!
//! Central AI capabilities including routing and providers.
//! Model splitting has been moved to ToadStool and Songbird.

pub mod model_splitting;

// Re-export deprecated stubs for backward compatibility
#[allow(deprecated)]
pub use model_splitting::{
    LayerDistribution, LayerDistributionStrategy, ModelSplitConfig, ModelSplitCoordinator,
    ModelSplitState, PerformancePrediction, SplitSession, SplitStatus, TensorMessage,
    TensorProtocol, TowerAssignment, TowerCapability,
};
