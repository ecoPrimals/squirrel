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
