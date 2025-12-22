//! Hardware Detection and Management
//!
//! Provides hardware detection for this instance only.
//! Maintains primal self-knowledge - no knowledge of other instances.

pub mod gpu;

pub use gpu::{detect_local_gpus, GpuInfo, LocalGpuCapabilities};
