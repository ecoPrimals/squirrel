// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![expect(
    clippy::missing_errors_doc,
    clippy::significant_drop_tightening,
    clippy::option_if_let_else,
    clippy::use_self,
    clippy::missing_const_for_fn,
    reason = "Progressive lint tightening for rule-system crate"
)]

//! Rule System for Squirrel
//!
//! This crate provides a rule system for the Squirrel Context Management System,
//! allowing for the definition, parsing, evaluation, and application of rules to context data.
//!
//! # Overview
//!
//! The rule system consists of several key components:
//!
//! - **Rule Directory Structure**: Organization structure for rule files
//! - **Rule Models**: Data models representing rules, conditions, and actions
//! - **Rule Parser**: Parser for rule files in YAML/MDC format
//! - **Rule Repository**: Storage and indexing system for rules
//! - **Rule Manager**: Management interface for rules with dependency resolution
//! - **Rule Evaluator**: Engine for evaluating rules against context data
//! - **Rule Actions**: System for executing actions based on rule evaluations

pub mod actions;
pub mod directory;
pub mod error;
pub mod evaluator;
#[cfg(test)]
mod evaluator_tests;
pub mod manager;
#[cfg(test)]
mod manager_tests;
pub mod models;
pub mod parser;
pub mod repository;
#[cfg(test)]
mod repository_tests;
pub mod utils;

pub use actions::*;
pub use directory::*;
pub use error::*;
pub use evaluator::*;
pub use manager::*;
pub use models::*;
pub use parser::*;
pub use repository::*;

/// Tests for the rule system
#[cfg(test)]
pub mod tests;
