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

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_raw_string_hashes)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

pub mod models;
pub mod directory;
pub mod parser;
// TODO: Implement these modules in subsequent phases
// pub mod repository;
// pub mod manager;
// pub mod evaluator;
// pub mod actions;
pub mod error;
pub mod utils;

pub use models::*;
pub use directory::*;
pub use parser::*;
// pub use repository::*;
// pub use manager::*;
// pub use evaluator::*;
// pub use actions::*;
pub use error::*;

/// Tests for the rule system
#[cfg(test)]
pub mod tests; 