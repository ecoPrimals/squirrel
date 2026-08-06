// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Model registry for AI model capabilities
//!
//! Re-exported from [`crate::common::capability::registry`] for backward compatibility.
//! The canonical implementation lives in the capability registry module.

pub use crate::common::capability::registry::{
    CostConfig, ModelCapabilities, ModelRegistry, PerformanceConfig, ResourceConfig,
};
