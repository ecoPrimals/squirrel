//! Tests for Display trait implementations
//!
//! This module contains tests for various Display trait implementations
//! in the squirrel-integration library.

extern crate squirrel_integration;

use squirrel_integration::ai_agent::CircuitBreakerState;

#[test]
fn test_circuit_breaker_state_display() {
    assert_eq!(CircuitBreakerState::Open.to_string(), "Open");
    assert_eq!(CircuitBreakerState::HalfOpen.to_string(), "HalfOpen");
    assert_eq!(CircuitBreakerState::Closed.to_string(), "Closed");
} 