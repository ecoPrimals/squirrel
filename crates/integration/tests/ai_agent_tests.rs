//! Integration tests for the AI Agent functionality
//!
//! These tests validate the behavior of the AI Agent, including
//! circuit breaker, rate limiting, and caching capabilities.

// Import the integration crate
extern crate squirrel_integration;

// Import the AI agent module
mod ai_agent;

// Re-export the tests
pub use ai_agent::*; 