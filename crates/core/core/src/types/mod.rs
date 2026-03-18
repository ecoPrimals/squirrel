// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Type definitions for Squirrel core.

mod core;
#[cfg(feature = "mesh")]
mod mesh;

pub use core::*;
#[cfg(feature = "mesh")]
pub use mesh::*;
