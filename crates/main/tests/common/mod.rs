//! Common test utilities
//!
//! ## Modern Concurrent Testing
//!
//! This module provides utilities for truly robust, concurrent tests.
//! **Test issues ARE production issues** - no sleeps, no race conditions.

pub mod async_test_utils;
pub mod concurrent_helpers;
pub mod provider_helpers;
pub mod test_utils;

// Re-export modern async utilities (PREFERRED)
pub use async_test_utils::{
    assert_eventually, assert_eventually_async, create_notification, retry_until_success,
    retry_until_success_async, wait_for, wait_for_all, wait_for_any, wait_for_async, with_timeout,
    Notifier, TimeoutError, Waiter,
};

// Re-export commonly used utilities
// NOTE: provider_helpers functions are temporarily disabled due to type resolution issues
// pub use provider_helpers::{
//     create_test_provider, create_test_provider_with_config, create_test_provider_with_id,
// };
pub use test_utils::{
    create_minimal_ecosystem_manager, create_test_ecosystem_manager,
    create_test_ecosystem_manager_with_config, quick_test_duration, retry_with_backoff,
    run_concurrent, test_duration, with_test_timeout,
};
