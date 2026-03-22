// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Core traits for ecosystem integration
//!
//! This module contains the standardized traits that all primals in the
//! ecoPrimals ecosystem must implement for seamless integration.

mod ai;
mod config;
mod discovery;
mod mesh;
mod primal;

#[cfg(test)]
mod tests;

pub use ai::*;
pub use config::*;
pub use discovery::*;
pub use mesh::*;
pub use primal::*;
