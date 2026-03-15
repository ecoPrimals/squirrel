// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Resilience framework for the MCP protocol
//!
//! This module provides mechanisms for enhancing fault tolerance and reliability
//! in MCP systems. It includes circuit breakers, retry mechanisms, recovery strategies,
//! state synchronization, and health monitoring.
//!
//! The resilience framework is designed to:
//! - Prevent cascading failures using circuit breakers
//! - Handle transient errors with retry mechanisms
//! - Recover from failures using configurable strategies
//! - Synchronize state between primary and backup systems
//! - Monitor system health and trigger automatic recovery
//! - Isolate failures using bulkhead pattern
//! - Protect services from overload using rate limiting
//!
//! The main components include:
//! - `CircuitBreaker`: Prevents repeated failures by temporarily disabling operations
//! - `RetryMechanism`: Automatically retries failed operations with configurable backoff
//! - `RecoveryStrategy`: Implements recovery procedures for different types of failures
//! - `StateSynchronizer`: Manages state synchronization between distributed components
//! - `HealthMonitor`: Tracks component health and triggers recovery when needed
//! - `Bulkhead`: Isolates failures by limiting concurrent operations
//! - `RateLimiter`: Protects services from overload by limiting operations per time period

pub mod circuit_breaker;
pub mod retry;
pub mod recovery;
/// State synchronization mechanisms for resilient distributed systems
pub mod state_sync;
/// Health monitoring capabilities for system components
pub mod health;
/// Bulkhead isolation pattern for limiting concurrent calls
pub mod bulkhead;
/// Rate limiting pattern for controlling access rates
pub mod rate_limiter;
/// Error types and handling for resilience operations
pub mod resilience_error;
/// Usage examples for the resilience framework
pub mod examples;
/// Resilience operation executors
mod operations;
/// Resilience component builder
mod builder;
/// Retry policy trait and implementations
mod retry_policy;

#[cfg(test)]
pub mod tests;

// Re-export error types
pub use resilience_error::{ResilienceError, Result};

// Re-export from submodules
pub use circuit_breaker::{
    BreakerState,
    BreakerMetrics,
    CircuitBreakerState,
    new_circuit_breaker,
};
pub use recovery::FailureSeverity;
pub use retry::{
    RetryMechanism,
    RetryConfig,
    RetryMetrics,
    RetryError,
    BackoffStrategy,
};
pub use examples::{
    run_circuit_breaker_example,
    run_retry_example,
};

// Re-export operations
pub use operations::{
    with_bulkhead,
    with_rate_limiting,
    with_resilience,
    with_recovery,
    with_health_monitoring,
    with_complete_resilience,
    with_state_sync,
    with_comprehensive_resilience,
    execute_with_recovery,
    execute_with_resilience_components,
};

// Re-export builder and retry policy
pub use builder::ResilienceBuilder;
pub use retry_policy::{RetryPolicy, StandardRetryPolicy};
