//! Integration tests for the AI Agent
//!
//! This module contains various tests for the AI Agent implementation,
//! including circuit breaker, rate limiting, caching, and performance tests.

// Import all types we need from the integration library
pub use crate::ai_agent::{
    AIAgentConfig,
    CircuitBreakerConfig,
    CircuitBreakerState,
    ResourceLimits,
    GenerationOptions,
    AgentRequest,
    OperationType,
};

// Import the mock adapter module
pub mod mock_adapter;

// Import test modules
pub mod circuit_breaker_tests;
pub mod cache_tests;

// Performance tests module (existing)
pub mod performance_tests;

// Re-export any common utilities for tests
pub use mock_adapter::MockAIAgentAdapter; 