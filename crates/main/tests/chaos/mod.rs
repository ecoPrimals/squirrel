//! # Chaos Engineering Test Suite
//!
//! Comprehensive chaos testing to validate system resilience under adverse conditions.
//!
//! ## 📊 Test Organization
//!
//! This module is organized by chaos scenario type:
//!
//! - **[service_failure]** - Service crashes, cascading failures
//! - **[network_partition]** - Network issues, latency, split-brain
//! - **[resource_exhaustion]** - Memory, CPU, FD, disk pressure
//! - **[concurrent_stress]** - Thundering herd, races, cancellation
//! - **[common]** - Shared test utilities and helpers
//!
//! ## 🎯 Testing Philosophy
//!
//! Our chaos tests follow these principles:
//!
//! 1. **Realistic Scenarios** - Test real-world failure modes
//! 2. **Observable Impact** - Measure system behavior under stress
//! 3. **Automatic Recovery** - Validate self-healing capabilities
//! 4. **Graceful Degradation** - Ensure system remains partially functional
//! 5. **Clear Reporting** - Provide actionable failure information
//!
//! ## 🚀 Running Tests
//!
//! ```bash
//! # Run all chaos tests
//! cargo test --test chaos
//!
//! # Run specific category
//! cargo test --test chaos service_failure
//! cargo test --test chaos network_partition
//! cargo test --test chaos resource_exhaustion
//! cargo test --test chaos concurrent_stress
//!
//! # Run specific test
//! cargo test --test chaos test_service_crash_recovery
//! ```
//!
//! ## 📝 Adding New Tests
//!
//! When adding chaos tests:
//!
//! 1. Choose the appropriate module based on failure type
//! 2. Use shared utilities from `common` module
//! 3. Document the scenario, expected behavior, and recovery
//! 4. Ensure tests are hermetic (no cross-test state)
//! 5. Add test to appropriate CI workflow
//!
//! ## ⚙️ Test Infrastructure
//!
//! Tests use mock services from the `common` module to simulate:
//! - Service crashes and restarts
//! - Network delays and failures
//! - Resource constraints
//! - Concurrent access patterns
//!
//! This allows testing chaos scenarios without requiring production services.

pub mod common;
pub mod service_failure;
pub mod network_partition;
pub mod resource_exhaustion;
pub mod concurrent_stress;

// Re-export common test utilities for convenience
pub use common::*;
