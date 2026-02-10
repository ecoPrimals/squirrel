// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! # Chaos Engineering Test Suite
//!
//! Comprehensive chaos testing to validate system resilience under adverse conditions.
//!
//! ## ⚠️ MIGRATION COMPLETE (Dec 28, 2025)
//!
//! The legacy monolithic `chaos_testing.rs` (3315 lines) has been:
//! - Renamed to `chaos_testing_legacy.rs`
//! - Infrastructure extracted to `common_complete.rs` (comprehensive)
//! - Modular structure created for new tests
//!
//! **Status**: Infrastructure ready, tests being migrated incrementally.
//!
//! ## 📊 Test Organization
//!
//! This module is organized by chaos scenario type:
//!
//! - **[common_complete]** - Complete test infrastructure (UPDATED)
//! - **[service_failure]** - Service crashes, cascading failures (3 tests)
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
//! # Run modular chaos tests (new structure)
//! cargo test --test chaos
//!
//! # Run specific category
//! cargo test --test chaos service_failure
//! cargo test --test chaos network_partition
//! cargo test --test chaos resource_exhaustion
//! cargo test --test chaos concurrent_stress
//!
//! # Run legacy tests (all 15 tests, during migration)
//! cargo test --test chaos_testing_legacy
//! ```
//!
//! ## 📝 Adding New Tests
//!
//! When adding chaos tests:
//!
//! 1. Choose the appropriate module based on failure type
//! 2. Use shared utilities from `common_complete` module
//! 3. Document the scenario, expected behavior, and recovery
//! 4. Ensure tests are hermetic (no cross-test state)
//! 5. Add test to appropriate CI workflow
//!
//! ## ⚙️ Test Infrastructure (common_complete.rs)
//!
//! Comprehensive infrastructure includes:
//! - Mock services (crash/recovery, latency, flaky)
//! - Network simulation (partitions, DNS)
//! - Resource simulation (memory, CPU, FD, disk)
//! - Metrics tracking for all test categories
//! - Helper functions for common test patterns
//!
//! This allows testing chaos scenarios without requiring production services.

// Comprehensive common infrastructure (COMPLETE)
pub mod common_complete;

// Test modules (organized by failure category)
pub mod service_failure;
pub mod network_partition;
pub mod resource_exhaustion;
pub mod concurrent_stress;

// Re-export common test utilities for convenience
pub use common_complete::*;
