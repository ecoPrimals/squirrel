// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Provider selection and optimization algorithms for AI routing.
//!
//! This module implements various strategies for selecting the best AI provider
//! for a given task, including scoring algorithms and routing optimizations.

mod scorer;
mod selector;
mod utils;

pub use scorer::ProviderScorer;
pub use selector::ProviderSelector;
pub use utils::OptimizationUtils;

#[cfg(test)]
mod tests;
