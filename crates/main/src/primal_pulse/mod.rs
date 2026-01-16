//! PrimalPulse - AI-Powered Ecosystem Intelligence
//!
//! This module provides AI-powered tools for analyzing, auditing, and optimizing
//! ecoPrimals development using Squirrel's multi-provider routing.

pub(crate) mod handlers;
pub mod neural_graph;
mod schemas;
mod tools;

#[cfg(test)]
mod tests;

pub use tools::register_primal_pulse_tools;
