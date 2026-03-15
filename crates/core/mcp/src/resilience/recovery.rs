// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Recovery strategy for handling failures in the MCP resilience framework
//! 
//! This module provides functionality to recover from failures in a structured way.

use std::fmt;
use std::time::{Duration, Instant};
use std::error::Error as StdError;


/// The severity level of a failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureSeverity {
    /// Minor failure that can be handled locally
    Minor,
    /// Moderate failure that requires coordinated recovery
    Moderate,
    /// Severe failure that requires system-wide intervention
    Severe,
    /// Critical failure that might require manual intervention
    Critical,
}

impl fmt::Display for FailureSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Minor => write!(f, "Minor"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Severe => write!(f, "Severe"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Information about a failure that needs recovery
#[derive(Debug, Clone)]
pub struct FailureInfo {
    /// A descriptive error message
    pub message: String,
    /// The severity of the failure
    pub severity: FailureSeverity,
    /// The context where the failure occurred
    pub context: String,
    /// The number of attempts already made to recover
    pub recovery_attempts: u32,
}

/// Configuration for the recovery strategy
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum number of recovery attempts for minor failures
    pub max_minor_attempts: u32,
    /// Maximum number of recovery attempts for moderate failures
    pub max_moderate_attempts: u32,
    /// Maximum number of recovery attempts for severe failures
    pub max_severe_attempts: u32,
    /// Whether to attempt recovery for critical failures
    pub recover_critical: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_minor_attempts: 5,
            max_moderate_attempts: 3,
            max_severe_attempts: 1,
            recover_critical: false,
        }
    }
}

/// Error type for recovery operations
#[derive(Debug)]
pub enum RecoveryError {
    /// Maximum number of recovery attempts exceeded
    MaxAttemptsExceeded {
        severity: FailureSeverity,
        attempts: u32,
        max_attempts: u32,
    },
    
    /// Recovery is not attempted for critical failures
    CriticalFailureNoRecovery,
    
    /// Failure during recovery action
    RecoveryActionFailed {
        message: String,
        source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    },
    
    /// Timeout during recovery
    Timeout {
        duration: Duration,
    },
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaxAttemptsExceeded { severity, attempts, max_attempts } => {
                write!(f, "Maximum recovery attempts ({max_attempts}) exceeded for {severity} failure: {attempts} attempts made")
            },
            Self::CriticalFailureNoRecovery => {
                write!(f, "Recovery not attempted for critical failure")
            },
            Self::RecoveryActionFailed { message, .. } => {
                write!(f, "Recovery action failed: {message}")
            },
            Self::Timeout { duration } => {
                write!(f, "Recovery timed out after {duration:?}")
            },
        }
    }
}

impl StdError for RecoveryError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::RecoveryActionFailed { source, .. } => {
                source.as_ref().map(|s| s.as_ref() as &(dyn StdError + 'static))
            },
            _ => None,
        }
    }
}

/// Metrics for tracking recovery operations
#[derive(Debug, Default, Clone)]
pub struct RecoveryMetrics {
    /// Count of successful recoveries
    pub successful_recoveries: u32,
    
    /// Count of failed recoveries
    pub failed_recoveries: u32,
    
    /// Count of recoveries by severity level
    pub recoveries_by_severity: [u32; 4], // Minor, Moderate, Severe, Critical
    
    /// Timestamp of the last recovery attempt
    pub last_recovery_time: Option<Instant>,
}

impl RecoveryMetrics {
    /// Reset all metrics to their default values
    pub fn reset(&mut self) {
        self.successful_recoveries = 0;
        self.failed_recoveries = 0;
        self.recoveries_by_severity = [0; 4];
        self.last_recovery_time = None;
    }
}

/// Recovery strategy for handling failures
#[derive(Debug, Clone)]
pub struct RecoveryStrategy {
    config: RecoveryConfig,
    metrics: RecoveryMetrics,
}

impl RecoveryStrategy {
    /// Create a new recovery strategy with the specified configuration
    #[must_use] pub fn new(config: RecoveryConfig) -> Self {
        Self {
            config,
            metrics: RecoveryMetrics::default(),
        }
    }
    
    /// Create a new recovery strategy with default configuration
    #[must_use] pub fn default() -> Self {
        Self::new(RecoveryConfig::default())
    }
    
    /// Handle a failure by attempting recovery
    ///
    /// Executes the provided recovery action based on the failure information and
    /// recovery configuration. The recovery action is only executed if the maximum
    /// number of attempts for the given severity level has not been exceeded.
    ///
    /// # Arguments
    ///
    /// * `failure` - Information about the failure
    /// * `recovery_action` - The action to take to recover from the failure
    ///
    /// # Returns
    ///
    /// The result of the recovery action if successful
    ///
    /// # Errors
    ///
    /// Returns a `RecoveryError` if:
    /// * The maximum number of recovery attempts for the given severity has been exceeded
    /// * Recovery for critical failures is disabled in the configuration
    /// * The recovery action itself fails
    pub fn handle_failure<F, R>(&mut self, failure: FailureInfo, recovery_action: F) -> std::result::Result<R, RecoveryError>
    where
        F: FnOnce() -> std::result::Result<R, Box<dyn StdError + Send + Sync + 'static>>,
    {
        self.metrics.last_recovery_time = Some(Instant::now());
        
        // Increment the recovery count for this severity
        let severity_index = match failure.severity {
            FailureSeverity::Minor => 0,
            FailureSeverity::Moderate => 1,
            FailureSeverity::Severe => 2,
            FailureSeverity::Critical => 3,
        };
        self.metrics.recoveries_by_severity[severity_index] += 1;
        
        // Check if we should attempt recovery based on severity and attempt count
        match failure.severity {
            FailureSeverity::Minor if failure.recovery_attempts >= self.config.max_minor_attempts => {
                self.metrics.failed_recoveries += 1;
                return Err(RecoveryError::MaxAttemptsExceeded { 
                    severity: failure.severity,
                    attempts: failure.recovery_attempts,
                    max_attempts: self.config.max_minor_attempts,
                });
            },
            FailureSeverity::Moderate if failure.recovery_attempts >= self.config.max_moderate_attempts => {
                self.metrics.failed_recoveries += 1;
                return Err(RecoveryError::MaxAttemptsExceeded { 
                    severity: failure.severity,
                    attempts: failure.recovery_attempts,
                    max_attempts: self.config.max_moderate_attempts,
                });
            },
            FailureSeverity::Severe if failure.recovery_attempts >= self.config.max_severe_attempts => {
                self.metrics.failed_recoveries += 1;
                return Err(RecoveryError::MaxAttemptsExceeded { 
                    severity: failure.severity,
                    attempts: failure.recovery_attempts,
                    max_attempts: self.config.max_severe_attempts,
                });
            },
            FailureSeverity::Critical if !self.config.recover_critical => {
                self.metrics.failed_recoveries += 1;
                return Err(RecoveryError::CriticalFailureNoRecovery);
            },
            _ => {
                // Attempt recovery
                match recovery_action() {
                    Ok(result) => {
                        self.metrics.successful_recoveries += 1;
                        Ok(result)
                    },
                    Err(e) => {
                        self.metrics.failed_recoveries += 1;
                        Err(RecoveryError::RecoveryActionFailed {
                            message: format!("Recovery failed for {} failure: {}",
                                failure.severity, e),
                            source: Some(e),
                        })
                    }
                }
            }
        }
    }
    
    /// Get the current recovery metrics
    #[must_use] pub const fn get_metrics(&self) -> &RecoveryMetrics {
        &self.metrics
    }
    
    /// Reset the recovery metrics
    pub fn reset_metrics(&mut self) {
        self.metrics.reset();
    }
    
    /// Get the maximum number of recovery attempts for a severity level
    #[must_use] pub const fn max_attempts_for_severity(&self, severity: FailureSeverity) -> u32 {
        match severity {
            FailureSeverity::Minor => self.config.max_minor_attempts,
            FailureSeverity::Moderate => self.config.max_moderate_attempts,
            FailureSeverity::Severe => self.config.max_severe_attempts,
            FailureSeverity::Critical => if self.config.recover_critical { 1 } else { 0 },
        }
    }
    
    /// Handle a failure with a timeout
    ///
    /// Similar to `handle_failure` but with an added timeout constraint. If the recovery
    /// takes longer than the specified timeout, a timeout error is returned.
    ///
    /// # Arguments
    ///
    /// * `failure` - Information about the failure
    /// * `timeout` - Maximum duration allowed for recovery
    /// * `recovery_action` - The action to take to recover from the failure
    ///
    /// # Returns
    ///
    /// The result of the recovery action if successful and completed within timeout
    ///
    /// # Errors
    ///
    /// Returns a `RecoveryError` if:
    /// * The recovery operation exceeds the specified timeout
    /// * The maximum number of recovery attempts for the given severity has been exceeded
    /// * Recovery for critical failures is disabled in the configuration
    /// * The recovery action itself fails
    pub fn handle_failure_with_timeout<F, R>(&mut self, failure: FailureInfo, timeout: Duration, recovery_action: F) -> std::result::Result<R, RecoveryError>
    where
        F: FnOnce() -> std::result::Result<R, Box<dyn StdError + Send + Sync + 'static>>,
    {
        let start_time = Instant::now();
        let result = self.handle_failure(failure, recovery_action);
        
        if start_time.elapsed() > timeout {
            Err(RecoveryError::Timeout { duration: timeout })
        } else {
            result
        }
    }

    /// Execute an operation with recovery capability
    ///
    /// Executes the provided operation and returns its result. If implemented, this would
    /// catch failures and apply recovery strategies automatically.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute
    ///
    /// # Returns
    ///
    /// The result of the operation if successful
    ///
    /// # Errors
    ///
    /// Returns a `RecoveryError` if:
    /// * The operation fails and recovery is not possible
    /// * The recovery process itself fails
    /// * The current implementation will always return an error as it is not yet implemented
    pub fn execute<F, R>(&self, operation: F) -> std::result::Result<R, RecoveryError>
    where
        F: FnOnce() -> std::result::Result<R, Box<dyn StdError + Send + Sync + 'static>>,
        R: Send + 'static,
    {
        // Execute the operation directly
        // This is a simple pass-through implementation
        // In a more sophisticated implementation, this could include:
        // - Automatic retry logic
        // - Circuit breaker patterns
        // - Monitoring and metrics collection
        operation().map_err(|error| RecoveryError::RecoveryActionFailed {
            message: error.to_string(),
            source: Some(error),
        })
    }

    /// Attempt to recover from a failure asynchronously
    ///
    /// Executes the provided recovery action based on the failure information and
    /// recovery configuration. The recovery action is only executed if the maximum
    /// number of attempts for the given severity level has not been exceeded.
    ///
    /// # Arguments
    ///
    /// * `failure` - Information about the failure
    /// * `recovery_action` - The action to take to recover from the failure
    ///
    /// # Returns
    ///
    /// The result of the recovery action if successful
    ///
    /// # Errors
    ///
    /// Returns a `RecoveryError` if:
    /// * The maximum number of recovery attempts for the given severity has been exceeded
    /// * Recovery for critical failures is disabled in the configuration
    /// * The recovery action itself fails
    pub async fn recover<F, R>(&mut self, failure: FailureInfo, recovery_action: F) -> std::result::Result<R, RecoveryError>
    where
        F: FnOnce() -> std::result::Result<R, Box<dyn StdError + Send + Sync>>,
    {
        self.metrics.last_recovery_time = Some(Instant::now());
        
        // Increment the recovery count for this severity
        let severity_index = match failure.severity {
            FailureSeverity::Minor => 0,
            FailureSeverity::Moderate => 1,
            FailureSeverity::Severe => 2,
            FailureSeverity::Critical => 3,
        };
        self.metrics.recoveries_by_severity[severity_index] += 1;
        
        // Check if we've exceeded the maximum number of recovery attempts
        let max_attempts = self.max_attempts_for_severity(failure.severity);
        if failure.recovery_attempts >= max_attempts {
            self.metrics.failed_recoveries += 1;
            return Err(RecoveryError::MaxAttemptsExceeded {
                severity: failure.severity,
                attempts: failure.recovery_attempts,
                max_attempts,
            });
        }
        
        // Don't attempt recovery for critical failures unless configured to do so
        if failure.severity == FailureSeverity::Critical && !self.config.recover_critical {
            self.metrics.failed_recoveries += 1;
            return Err(RecoveryError::CriticalFailureNoRecovery);
        }
        
        // Execute the recovery action
        match recovery_action() {
            Ok(result) => {
                self.metrics.successful_recoveries += 1;
                Ok(result)
            },
            Err(error) => {
                self.metrics.failed_recoveries += 1;
                Err(RecoveryError::RecoveryActionFailed {
                    message: error.to_string(),
                    source: Some(error),
                })
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    // A test error type
    #[derive(Debug)]
    struct TestError(String);
    
    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "TestError: {}", self.0)
        }
    }
    
    impl StdError for TestError {}
    
    #[test]
    fn test_recovery_minor_success() {
        let mut recovery = RecoveryStrategy::default();
        
        let failure = FailureInfo {
            message: "Test minor failure".to_string(),
            severity: FailureSeverity::Minor,
            context: "test_context".to_string(),
            recovery_attempts: 0,
        };
        
        // Successful recovery action
        let result = recovery.handle_failure(failure, || {
            // Simulate successful recovery
            Ok::<_, Box<dyn StdError + Send + Sync + 'static>>(42)
        });
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(recovery.metrics.successful_recoveries, 1);
        assert_eq!(recovery.metrics.failed_recoveries, 0);
        assert_eq!(recovery.metrics.recoveries_by_severity[0], 1); // Minor
    }
    
    #[test]
    fn test_recovery_max_attempts_exceeded() {
        let mut recovery = RecoveryStrategy::new(RecoveryConfig {
            max_minor_attempts: 2,
            ..RecoveryConfig::default()
        });
        
        let failure = FailureInfo {
            message: "Test minor failure".to_string(),
            severity: FailureSeverity::Minor,
            context: "test_context".to_string(),
            recovery_attempts: 2, // Already at max
        };
        
        let result = recovery.handle_failure(failure, || {
            // This shouldn't be called
            Ok::<_, Box<dyn StdError + Send + Sync + 'static>>(42)
        });
        
        assert!(matches!(result, Err(RecoveryError::MaxAttemptsExceeded { .. })));
        assert_eq!(recovery.metrics.successful_recoveries, 0);
        assert_eq!(recovery.metrics.failed_recoveries, 1);
    }
    
    #[test]
    fn test_recovery_critical_failure() {
        let mut recovery = RecoveryStrategy::new(RecoveryConfig {
            recover_critical: false,
            ..RecoveryConfig::default()
        });
        
        let failure = FailureInfo {
            message: "Test critical failure".to_string(),
            severity: FailureSeverity::Critical,
            context: "test_context".to_string(),
            recovery_attempts: 0,
        };
        
        let result = recovery.handle_failure(failure, || {
            // This shouldn't be called
            Ok::<_, Box<dyn StdError + Send + Sync + 'static>>(42)
        });
        
        assert!(matches!(result, Err(RecoveryError::CriticalFailureNoRecovery)));
        assert_eq!(recovery.metrics.successful_recoveries, 0);
        assert_eq!(recovery.metrics.failed_recoveries, 1);
        assert_eq!(recovery.metrics.recoveries_by_severity[3], 1); // Critical
    }
    
    #[test]
    fn test_recovery_action_failed() {
        let mut recovery = RecoveryStrategy::default();
        
        let failure = FailureInfo {
            message: "Test failure".to_string(),
            severity: FailureSeverity::Moderate,
            context: "test_context".to_string(),
            recovery_attempts: 0,
        };
        
        let result: std::result::Result<i32, RecoveryError> = recovery.handle_failure(failure, || {
            // Simulate recovery action failure
            Err(Box::new(TestError("Recovery action failed".to_string())))
        });
        
        assert!(matches!(result, Err(RecoveryError::RecoveryActionFailed { .. })));
        assert_eq!(recovery.metrics.successful_recoveries, 0);
        assert_eq!(recovery.metrics.failed_recoveries, 1);
        assert_eq!(recovery.metrics.recoveries_by_severity[1], 1); // Moderate
    }
    
    #[test]
    fn test_recovery_metrics_reset() {
        let mut recovery = RecoveryStrategy::default();
        
        let failure = FailureInfo {
            message: "Test failure".to_string(),
            severity: FailureSeverity::Minor,
            context: "test_context".to_string(),
            recovery_attempts: 0,
        };
        
        // One successful recovery
        let _: std::result::Result<i32, RecoveryError> = recovery.handle_failure(failure.clone(), || {
            Ok::<_, Box<dyn StdError + Send + Sync + 'static>>(42)
        });
        
        // One failed recovery
        let _: std::result::Result<i32, RecoveryError> = recovery.handle_failure(failure.clone(), || {
            Err(Box::new(TestError("Recovery action failed".to_string())))
        });
        
        assert_eq!(recovery.metrics.successful_recoveries, 1);
        assert_eq!(recovery.metrics.failed_recoveries, 1);
        
        // Reset metrics
        recovery.reset_metrics();
        
        assert_eq!(recovery.metrics.successful_recoveries, 0);
        assert_eq!(recovery.metrics.failed_recoveries, 0);
        assert_eq!(recovery.metrics.recoveries_by_severity, [0, 0, 0, 0]);
        assert!(recovery.metrics.last_recovery_time.is_none());
    }
} 