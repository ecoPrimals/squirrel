// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Common test utilities
//!
//! ## Modern Concurrent Testing
//!
//! This module provides utilities for truly robust, concurrent tests.
//! **Test issues ARE production issues** - no sleeps, no race conditions.

pub mod async_test_utils;
pub mod concurrent_helpers;
pub mod mock_providers;
pub mod provider_factory;
pub mod provider_helpers;
pub mod test_utils;

// Re-export modern async utilities (PREFERRED)
pub use async_test_utils::{
    create_notification, retry_until_success, retry_until_success_async, wait_for, wait_for_all,
    wait_for_any, wait_for_async, with_timeout, Notifier, TimeoutError, Waiter,
};

// Note: assert_eventually and assert_eventually_async are macros defined in async_test_utils.rs
// They are exported via #[macro_export] and available at the crate root

// Re-export modern provider factory (NEW - for test modernization)
pub use provider_factory::{
    create_test_provider, create_test_provider_with_config, ProviderFactory,
};

// Re-export commonly used utilities
pub use test_utils::{
    quick_test_duration, retry_with_backoff, run_concurrent, test_duration, with_test_timeout,
};
