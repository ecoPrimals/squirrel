// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::missing_docs_in_private_items)]
//! # Squirrel Interfaces
//!
//! This crate contains shared interfaces that are used by multiple Squirrel components.
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]
//! The primary purpose is to break circular dependencies between crates.

pub mod context;
pub mod plugins;
pub mod tracing;

/// Error types and utilities used across the codebase
pub mod error;
