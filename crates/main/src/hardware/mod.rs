// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Hardware Detection and Management
//!
//! Provides hardware detection for this instance only.
//! Maintains primal self-knowledge - no knowledge of other instances.

#[cfg(feature = "gpu-detection")]
pub mod gpu;

#[cfg(feature = "gpu-detection")]
pub use gpu::{detect_local_gpus, GpuInfo, LocalGpuCapabilities};
