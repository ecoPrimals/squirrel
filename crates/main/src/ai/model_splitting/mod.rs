//! Model Splitting Module - MOVED TO TOADSTOOL & SONGBIRD
//!
//! ⚠️ **THIS MODULE HAS BEEN RELOCATED** ⚠️
//!
//! ## Why This Was Moved
//!
//! This functionality violated Squirrel's architectural boundaries:
//! - Squirrel should focus on AI orchestration (user intent, routing)
//! - Squirrel should NOT manage GPU layers, VRAM splits, or hardware topology
//!
//! ## Where It Went
//!
//! **ToadStool** (`../toadstool/crates/model-loading/`):
//! - Layer distribution algorithms
//! - VRAM calculation
//! - Model loading on GPUs
//! - GPU execution management
//!
//! **Songbird** (`../songbird/crates/coordination/`):
//! - Cross-tower coordination
//! - Tower assignment
//! - Tensor routing between towers
//!
//! ## The Right Architecture
//!
//! ```text
//! User → Squirrel: "Load llama-70b"
//!   ↓
//! Squirrel → Songbird: "Coordinate model load"
//!   ↓
//! Songbird → ToadStool(s): "Load layers X-Y"
//!   ↓
//! ToadStool(s) → Execute on GPUs
//! ```
//!
//! ## Migration Guide
//!
//! See: `docs/architecture/MODEL_SPLITTING_MOVED_TO_TOADSTOOL.md`
//!
//! ## Stub Types (For Backward Compatibility)
//!
//! These are minimal stubs to avoid breaking compilation.
//! They will be removed in a future version.

use serde::{Deserialize, Serialize};

/// STUB: Model split configuration
/// MOVED TO: ToadStool (`toadstool/crates/model-loading/src/split_config.rs`)
#[deprecated(
    since = "0.2.0",
    note = "Moved to ToadStool. Use ToadStool's model loading API instead."
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSplitConfig {
    pub model_id: String,
    pub provider: String,
    pub total_layers: u32,
    pub vram_per_layer: f32,
}

/// STUB: Tower assignment
/// MOVED TO: Songbird (`songbird/crates/coordination/src/tower_assignment.rs`)
#[deprecated(
    since = "0.2.0",
    note = "Moved to Songbird. Use Songbird's coordination API instead."
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerAssignment {
    pub tower_id: String,
    pub start_layer: u32,
    pub end_layer: u32,
    pub vram_allocated_gb: f32,
}

/// STUB: Split status
/// MOVED TO: Songbird coordination
#[deprecated(since = "0.2.0", note = "Moved to Songbird coordination API")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SplitStatus {
    Pending,
    Loading,
    Ready,
    Failed,
}

/// STUB: Split session
/// MOVED TO: Songbird coordination
#[deprecated(since = "0.2.0", note = "Moved to Songbird coordination API")]
#[derive(Debug, Clone)]
pub struct SplitSession {
    pub session_id: String,
    pub status: SplitStatus,
}

/// STUB: Model split state
/// MOVED TO: ToadStool model loading
#[deprecated(since = "0.2.0", note = "Moved to ToadStool model loading")]
#[derive(Debug, Clone)]
pub struct ModelSplitState {
    pub config: ModelSplitConfig,
    pub my_tower_id: String,
}

impl ModelSplitState {
    #[deprecated(since = "0.2.0", note = "Use ToadStool's API")]
    pub fn new(config: ModelSplitConfig, my_tower_id: &str) -> Self {
        Self {
            config,
            my_tower_id: my_tower_id.to_string(),
        }
    }
}

/// STUB: Model split coordinator
/// MOVED TO: Songbird coordination
#[deprecated(since = "0.2.0", note = "Use Songbird's coordination API")]
#[derive(Debug)]
pub struct ModelSplitCoordinator {
    pub my_tower_id: String,
}

impl ModelSplitCoordinator {
    #[deprecated(since = "0.2.0", note = "Use Songbird coordination API")]
    pub fn new(tower_id: String) -> Self {
        Self {
            my_tower_id: tower_id,
        }
    }
}

/// STUB: Layer distribution strategy
/// MOVED TO: ToadStool model loading
#[deprecated(since = "0.2.0", note = "Moved to ToadStool model loading")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayerDistributionStrategy {
    Even,
    Weighted,
    PerformanceOptimized,
    EfficiencyOptimized,
}

/// STUB: Tower capability
/// MOVED TO: ToadStool GPU detection
#[deprecated(since = "0.2.0", note = "Use ToadStool's GPU detection API")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerCapability {
    pub tower_id: String,
    pub vram_total_gb: f32,
    pub vram_available_gb: f32,
    pub gpu_model: String,
}

/// STUB: Layer distribution
/// MOVED TO: ToadStool model loading
#[deprecated(since = "0.2.0", note = "Use ToadStool's layer distribution API")]
#[derive(Debug, Clone)]
pub struct LayerDistribution {
    pub total_layers: u32,
    pub tower_assignments: Vec<TowerAssignment>,
    pub strategy: LayerDistributionStrategy,
}

impl LayerDistribution {
    #[deprecated(since = "0.2.0", note = "Use ToadStool API")]
    pub fn even_distribution(
        _total_layers: u32,
        _vram_per_layer: f32,
        _towers: Vec<TowerCapability>,
    ) -> Result<Self, String> {
        Err("MOVED TO TOADSTOOL: Use ToadStool's model loading API".to_string())
    }

    #[deprecated(since = "0.2.0", note = "Use ToadStool API")]
    pub fn performance_optimized(
        _total_layers: u32,
        _vram_per_layer: f32,
        _towers: Vec<TowerCapability>,
    ) -> Result<Self, String> {
        Err("MOVED TO TOADSTOOL: Use ToadStool's model loading API".to_string())
    }
}

/// STUB: Tensor message
/// MOVED TO: Songbird tensor routing
#[deprecated(since = "0.2.0", note = "Use Songbird's tensor routing API")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorMessage {
    pub session_id: String,
    pub from_tower: String,
    pub to_tower: String,
    pub layer_index: u32,
}

/// STUB: Tensor protocol
/// MOVED TO: Songbird tensor routing
#[deprecated(since = "0.2.0", note = "Use Songbird's tensor routing API")]
pub struct TensorProtocol;

/// STUB: Performance prediction
/// MOVED TO: ToadStool performance analysis
#[deprecated(since = "0.2.0", note = "Use ToadStool's performance API")]
pub struct PerformancePrediction;

impl PerformancePrediction {
    #[deprecated(since = "0.2.0", note = "Use ToadStool API")]
    pub fn predict_latency(_towers: &[TowerCapability]) -> Result<f32, String> {
        Err("MOVED TO TOADSTOOL: Use ToadStool's performance prediction API".to_string())
    }
}

// Don't re-export - types are already defined above
// pub use self::{...};
