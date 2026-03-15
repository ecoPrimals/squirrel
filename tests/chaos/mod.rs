// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Chaos Testing Suite
//!
//! This module provides chaos engineering tests to validate system resilience
//! under adverse conditions including service failures, network partitions,
//! resource exhaustion, and concurrent stress scenarios.
//!
//! ## Test Categories
//! 1. **Service Failure**: Simulates primal service crashes and recoveries
//! 2. **Network Partition**: Tests behavior during network failures
//! 3. **Resource Exhaustion**: Validates graceful degradation under resource limits
//! 4. **Concurrent Stress**: Tests system behavior under extreme load
//!
//! ## Running Chaos Tests
//! ```bash
//! cargo test --test chaos_testing
//! cargo test --test chaos_testing service_failure
//! cargo test --test chaos_testing network_partition
//! ```

mod concurrent;
mod helpers;
mod network;
mod recovery;
mod service;
mod timing;
