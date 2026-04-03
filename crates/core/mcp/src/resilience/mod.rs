// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Resilience framework for the MCP protocol
//!
//! Provides retry policy, backoff strategy, and error types for resilient
//! request handling.

pub mod resilience_error;
pub mod retry;
mod retry_policy;

pub use resilience_error::{ResilienceError, Result};
pub use retry::{BackoffStrategy, RetryConfig, RetryError, RetryMechanism, RetryMetrics};
pub use retry_policy::{RetryEnvParams, RetryPolicy, StandardRetryPolicy, retry_env_params};
