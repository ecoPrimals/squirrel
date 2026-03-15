// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Unified Retry Configuration
//!
//! This module provides the canonical retry configuration used throughout Squirrel.
//! It consolidates 6 previously scattered RetryConfig definitions into a single,
//! comprehensive configuration with all features.
//!
//! # Migration
//!
//! Old code:
//! ```ignore
//! use crate::RetryConfig;  // From various locations
//! ```
//!
//! New code:
//! ```ignore
//! use squirrel_config::unified::retry::RetryConfig;
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Retry strategy for operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// Fixed delay between retries
    Fixed,

    /// Exponential backoff (delay doubles each time)
    Exponential,

    /// Linear increase in delay
    Linear,

    /// Exponential with jitter to avoid thundering herd
    ExponentialWithJitter,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::Exponential
    }
}

/// Canonical retry configuration for Squirrel
///
/// This configuration consolidates retry settings from multiple domains:
/// - MCP resilience
/// - MCP workflows
/// - Security operations
/// - Ecosystem registry
/// - SDK infrastructure
/// - Ecosystem API
///
/// # Examples
///
/// ```rust,ignore
/// use squirrel_mcp_config::unified::retry::{RetryConfig, RetryStrategy};
/// use std::time::Duration;
///
/// // Default configuration (3 attempts, 100ms initial delay)
/// let config = RetryConfig::default();
///
/// // Custom configuration
/// let config = RetryConfig::builder()
///     .max_attempts(5)
///     .initial_delay(Duration::from_millis(200))
///     .max_delay(Duration::from_secs(60))
///     .strategy(RetryStrategy::ExponentialWithJitter)
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (including the first attempt)
    ///
    /// Default: 3
    pub max_attempts: u32,

    /// Initial delay before the first retry
    ///
    /// Default: 100ms
    #[serde(with = "duration_serde")]
    pub initial_delay: Duration,

    /// Maximum delay between retries (prevents exponential from growing too large)
    ///
    /// Default: Some(30 seconds)
    #[serde(with = "option_duration_serde")]
    pub max_delay: Option<Duration>,

    /// Backoff multiplier for exponential and linear strategies
    ///
    /// For exponential: delay = initial_delay * multiplier^attempt
    /// For linear: delay = initial_delay * multiplier * attempt
    ///
    /// Default: 2.0
    pub backoff_multiplier: f64,

    /// Retry strategy to use
    ///
    /// Default: Exponential
    pub strategy: RetryStrategy,

    /// Whether to retry on timeout errors
    ///
    /// Default: true
    pub retry_on_timeout: bool,

    /// Whether to retry on connection errors
    ///
    /// Default: true
    pub retry_on_connection_error: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Some(Duration::from_secs(30)),
            backoff_multiplier: 2.0,
            strategy: RetryStrategy::Exponential,
            retry_on_timeout: true,
            retry_on_connection_error: true,
        }
    }
}

impl RetryConfig {
    /// Create a new RetryConfig builder
    pub fn builder() -> RetryConfigBuilder {
        RetryConfigBuilder::default()
    }

    /// Calculate the delay for a given attempt number
    ///
    /// # Arguments
    ///
    /// * `attempt` - The attempt number (0-indexed, so 0 is the first retry)
    ///
    /// # Returns
    ///
    /// The duration to wait before the next attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay = match self.strategy {
            RetryStrategy::Fixed => self.initial_delay,

            RetryStrategy::Exponential => {
                let multiplier = self.backoff_multiplier.powi(attempt as i32);
                self.initial_delay.mul_f64(multiplier)
            }

            RetryStrategy::Linear => {
                let multiplier = self.backoff_multiplier * (attempt as f64 + 1.0);
                self.initial_delay.mul_f64(multiplier)
            }

            RetryStrategy::ExponentialWithJitter => {
                let multiplier = self.backoff_multiplier.powi(attempt as i32);
                let base_delay = self.initial_delay.mul_f64(multiplier);

                // Add up to 25% jitter
                use std::collections::hash_map::RandomState;
                use std::hash::BuildHasher;

                let jitter =
                    (RandomState::new().hash_one(std::time::SystemTime::now()) % 25) as f64 / 100.0;

                base_delay.mul_f64(1.0 + jitter)
            }
        };

        // Apply max_delay cap if set
        match self.max_delay {
            Some(max) if delay > max => max,
            _ => delay,
        }
    }

    /// Check if we should retry based on the attempt count
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

/// Builder for RetryConfig
#[derive(Debug, Default)]
pub struct RetryConfigBuilder {
    max_attempts: Option<u32>,
    initial_delay: Option<Duration>,
    max_delay: Option<Option<Duration>>,
    backoff_multiplier: Option<f64>,
    strategy: Option<RetryStrategy>,
    retry_on_timeout: Option<bool>,
    retry_on_connection_error: Option<bool>,
}

impl RetryConfigBuilder {
    /// Set the maximum number of retry attempts
    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = Some(max_attempts);
        self
    }

    /// Set the initial delay
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = Some(delay);
        self
    }

    /// Set the maximum delay
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = Some(Some(delay));
        self
    }

    /// Set no maximum delay (allow unlimited backoff)
    pub fn no_max_delay(mut self) -> Self {
        self.max_delay = Some(None);
        self
    }

    /// Set the backoff multiplier
    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = Some(multiplier);
        self
    }

    /// Set the retry strategy
    pub fn strategy(mut self, strategy: RetryStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Set whether to retry on timeout
    pub fn retry_on_timeout(mut self, retry: bool) -> Self {
        self.retry_on_timeout = Some(retry);
        self
    }

    /// Set whether to retry on connection errors
    pub fn retry_on_connection_error(mut self, retry: bool) -> Self {
        self.retry_on_connection_error = Some(retry);
        self
    }

    /// Build the RetryConfig
    pub fn build(self) -> RetryConfig {
        let default = RetryConfig::default();

        RetryConfig {
            max_attempts: self.max_attempts.unwrap_or(default.max_attempts),
            initial_delay: self.initial_delay.unwrap_or(default.initial_delay),
            max_delay: self.max_delay.unwrap_or(default.max_delay),
            backoff_multiplier: self
                .backoff_multiplier
                .unwrap_or(default.backoff_multiplier),
            strategy: self.strategy.unwrap_or(default.strategy),
            retry_on_timeout: self.retry_on_timeout.unwrap_or(default.retry_on_timeout),
            retry_on_connection_error: self
                .retry_on_connection_error
                .unwrap_or(default.retry_on_connection_error),
        }
    }
}

// Preset configurations for common use cases
impl RetryConfig {
    /// Quick retry configuration (2 attempts, 50ms delay)
    ///
    /// Use for operations that should fail fast
    pub fn quick() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(50),
            max_delay: Some(Duration::from_millis(500)),
            backoff_multiplier: 2.0,
            strategy: RetryStrategy::Fixed,
            retry_on_timeout: true,
            retry_on_connection_error: true,
        }
    }

    /// Standard retry configuration (3 attempts, 100ms delay)
    ///
    /// This is the same as default(), provided for explicitness
    pub fn standard() -> Self {
        Self::default()
    }

    /// Aggressive retry configuration (5 attempts, 200ms delay)
    ///
    /// Use for critical operations that should be retried more
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(200),
            max_delay: Some(Duration::from_secs(60)),
            backoff_multiplier: 2.0,
            strategy: RetryStrategy::ExponentialWithJitter,
            retry_on_timeout: true,
            retry_on_connection_error: true,
        }
    }

    /// Persistent retry configuration (10 attempts, 500ms delay)
    ///
    /// Use for background operations that can retry many times
    pub fn persistent() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_millis(500),
            max_delay: Some(Duration::from_secs(120)),
            backoff_multiplier: 1.5,
            strategy: RetryStrategy::Linear,
            retry_on_timeout: true,
            retry_on_connection_error: true,
        }
    }
}

// Serde helpers for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => serializer.serialize_some(&(d.as_millis() as u64)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = Option::<u64>::deserialize(deserializer)?;
        Ok(millis.map(Duration::from_millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Some(Duration::from_secs(30)));
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_builder() {
        let config = RetryConfig::builder()
            .max_attempts(5)
            .initial_delay(Duration::from_millis(200))
            .build();

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(200));
    }

    #[test]
    fn test_exponential_delay() {
        let config = RetryConfig {
            strategy: RetryStrategy::Exponential,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_delay: None,
            ..Default::default()
        };

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(400));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig {
            strategy: RetryStrategy::Exponential,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_delay: Some(Duration::from_millis(250)),
            ..Default::default()
        };

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(250)); // Capped
        assert_eq!(config.calculate_delay(3), Duration::from_millis(250)); // Capped
    }

    #[test]
    fn test_should_retry() {
        let config = RetryConfig {
            max_attempts: 3,
            ..Default::default()
        };

        assert!(config.should_retry(0));
        assert!(config.should_retry(1));
        assert!(config.should_retry(2));
        assert!(!config.should_retry(3));
    }

    #[test]
    fn test_presets() {
        let quick = RetryConfig::quick();
        assert_eq!(quick.max_attempts, 2);

        let aggressive = RetryConfig::aggressive();
        assert_eq!(aggressive.max_attempts, 5);

        let persistent = RetryConfig::persistent();
        assert_eq!(persistent.max_attempts, 10);
    }
}
