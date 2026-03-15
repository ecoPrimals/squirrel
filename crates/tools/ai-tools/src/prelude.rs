// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Prelude module for AI tools
//!
//! This module re-exports commonly used types and traits for convenience.

// Common imports for AI tools
pub use crate::common::*;
pub use crate::error::{AIError, Result};

// Capability-based AI client (TRUE PRIMAL!)
pub use crate::capability_ai::AiClient; 