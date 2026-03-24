// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Resilience framework for the MCP protocol
//!
//! Core retry policy and mechanism are published here. Heavier components
//! (circuit breaker, bulkhead, recovery, etc.) live under `src/resilience/` but
//! are not yet wired into the crate root build.

pub mod resilience_error;
pub mod retry;
mod retry_policy;

pub use resilience_error::{ResilienceError, Result};
pub use retry::{BackoffStrategy, RetryConfig, RetryError, RetryMechanism, RetryMetrics};
pub use retry_policy::{RetryEnvParams, RetryPolicy, StandardRetryPolicy, retry_env_params};
