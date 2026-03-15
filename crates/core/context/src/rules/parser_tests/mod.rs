// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the Rules Parser
//!
//! These tests cover the core functionality of parsing MDC/YAML formatted rules.
//!
//! # Organization
//!
//! - **`basic_parsing`** - Core parsing functionality, frontmatter, required fields
//! - **`validation_tests`** - Validation rules and constraint checking
//! - **`advanced_features`** - Metadata, multiple items, custom configuration

#[cfg(test)]
mod basic_parsing;

#[cfg(test)]
mod validation_tests;

#[cfg(test)]
mod advanced_features;

#[cfg(test)]
mod edge_cases;
