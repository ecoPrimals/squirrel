// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;
use std::time::Duration;
use std::error::Error as StdError;
use std::fmt;
use std::sync::{Mutex};
use std::thread;

use crate::resilience::recovery::{RecoveryStrategy, RecoveryConfig, FailureSeverity, FailureInfo, RecoveryError};

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
fn test_recovery_strategy_defaults() {
    let recovery = RecoveryStrategy::default();
    
    // Verify default limits
    assert_eq!(recovery.max_attempts_for_severity(FailureSeverity::Minor), 5);
    assert_eq!(recovery.max_attempts_for_severity(FailureSeverity::Moderate), 3);
    assert_eq!(recovery.max_attempts_for_severity(FailureSeverity::Severe), 1);
    assert_eq!(recovery.max_attempts_for_severity(FailureSeverity::Critical), 0); // Default is not to recover critical failures
}

#[test]
fn test_recovery_strategy_custom_config() {
    let config = RecoveryConfig {
        max_minor_attempts: 10,
        max_moderate_attempts: 5,
        max_severe_attempts: 2,
        recover_critical: true,
    };
    
    let recovery = RecoveryStrategy::new(config);
    
    // Verify custom limits
    assert_eq!(recovery.max_attempts_for_severity(FailureSeverity::Minor), 10);
    assert_eq!(recovery.max_attempts_for_severity(FailureSeverity::Moderate), 5);
    assert_eq!(recovery.max_attempts_for_severity(FailureSeverity::Severe), 2);
    assert_eq!(recovery.max_attempts_for_severity(FailureSeverity::Critical), 1); // Now we recover critical failures
}

#[test]
fn test_recovery_strategy_minor_success() {
    let mut recovery = RecoveryStrategy::default();
    
    let failure = FailureInfo {
        message: "Minor database connection issue".to_string(),
        severity: FailureSeverity::Minor,
        context: "database.connection".to_string(),
        recovery_attempts: 0,
    };
    
    // Mock a successful recovery action
    let result: std::result::Result<i32, RecoveryError> = recovery.handle_failure(failure, || {
        // Simulate successful recovery
        Ok::<i32, Box<dyn StdError + Send + Sync>>(42)
    });
    
    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed"), 42);
    
    // Check metrics
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 1);
    assert_eq!(metrics.failed_recoveries, 0);
    assert_eq!(metrics.recoveries_by_severity[0], 1); // Minor severity
}

#[test]
fn test_recovery_strategy_moderate_success() {
    let mut recovery = RecoveryStrategy::default();
    
    let failure = FailureInfo {
        message: "Moderate API timeout".to_string(),
        severity: FailureSeverity::Moderate,
        context: "api.request".to_string(),
        recovery_attempts: 0,
    };
    
    // Mock a successful recovery action
    let result: std::result::Result<String, RecoveryError> = recovery.handle_failure(failure, || {
        // Simulate successful recovery
        Ok::<String, Box<dyn StdError + Send + Sync>>("API recovered".to_string())
    });
    
    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed"), "API recovered".to_string());
    
    // Check metrics
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 1);
    assert_eq!(metrics.failed_recoveries, 0);
    assert_eq!(metrics.recoveries_by_severity[1], 1); // Moderate severity
}

#[test]
fn test_recovery_strategy_severe_success() {
    let mut recovery = RecoveryStrategy::default();
    
    let failure = FailureInfo {
        message: "Severe disk error".to_string(),
        severity: FailureSeverity::Severe,
        context: "storage.disk".to_string(),
        recovery_attempts: 0,
    };
    
    // Mock a successful recovery action
    let result: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure, || {
        // Simulate successful recovery
        Ok::<_, Box<dyn StdError + Send + Sync>>(())
    });
    
    assert!(result.is_ok());
    
    // Check metrics
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 1);
    assert_eq!(metrics.failed_recoveries, 0);
    assert_eq!(metrics.recoveries_by_severity[2], 1); // Severe severity
}

#[test]
fn test_recovery_strategy_critical_not_attempted() {
    let mut recovery = RecoveryStrategy::default(); // Default doesn't recover critical
    
    let failure = FailureInfo {
        message: "Critical system failure".to_string(),
        severity: FailureSeverity::Critical,
        context: "system.kernel".to_string(),
        recovery_attempts: 0,
    };
    
    // Recovery should not be attempted
    let result: std::result::Result<String, RecoveryError> = recovery.handle_failure(failure, || {
        // This shouldn't be called
        unreachable!("Recovery action should not be called for critical failures by default");
    });
    
    assert!(matches!(result, Err(RecoveryError::CriticalFailureNoRecovery)));
    
    // Check metrics
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 0);
    assert_eq!(metrics.failed_recoveries, 1);
    assert_eq!(metrics.recoveries_by_severity[3], 1); // Critical severity
}

#[test]
fn test_recovery_strategy_critical_attempted() {
    let mut recovery = RecoveryStrategy::new(RecoveryConfig {
        recover_critical: true,
        ..RecoveryConfig::default()
    });
    
    let failure = FailureInfo {
        message: "Critical system failure".to_string(),
        severity: FailureSeverity::Critical,
        context: "system.kernel".to_string(),
        recovery_attempts: 0,
    };
    
    // Recovery should be attempted now
    let result: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure, || {
        // Simulate successful recovery
        Ok::<(), Box<dyn StdError + Send + Sync>>(())
    });
    
    assert!(result.is_ok());
    
    // Check metrics
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 1);
    assert_eq!(metrics.failed_recoveries, 0);
    assert_eq!(metrics.recoveries_by_severity[3], 1); // Critical severity
}

#[test]
fn test_recovery_strategy_max_minor_attempts() {
    let mut recovery = RecoveryStrategy::new(RecoveryConfig {
        max_minor_attempts: 3,
        ..RecoveryConfig::default()
    });
    
    // Try with 2 previous attempts (still under max)
    let failure1 = FailureInfo {
        message: "Minor failure".to_string(),
        severity: FailureSeverity::Minor,
        context: "test".to_string(),
        recovery_attempts: 2,
    };
    
    let result1: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure1, || {
        Ok::<(), Box<dyn StdError + Send + Sync>>(())
    });
    
    assert!(result1.is_ok());
    
    // Try with 3 previous attempts (at max)
    let failure2 = FailureInfo {
        message: "Minor failure".to_string(),
        severity: FailureSeverity::Minor,
        context: "test".to_string(),
        recovery_attempts: 3,
    };
    
    let result2: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure2, || {
        unreachable!("This should not be called");
    });
    
    assert!(matches!(result2, Err(RecoveryError::MaxAttemptsExceeded { .. })));
}

#[test]
fn test_recovery_strategy_max_moderate_attempts() {
    let mut recovery = RecoveryStrategy::new(RecoveryConfig {
        max_moderate_attempts: 2,
        ..RecoveryConfig::default()
    });
    
    // Try with 1 previous attempt (still under max)
    let failure1 = FailureInfo {
        message: "Moderate failure".to_string(),
        severity: FailureSeverity::Moderate,
        context: "test".to_string(),
        recovery_attempts: 1,
    };
    
    let result1: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure1, || {
        Ok::<(), Box<dyn StdError + Send + Sync>>(())
    });
    
    assert!(result1.is_ok());
    
    // Try with 2 previous attempts (at max)
    let failure2 = FailureInfo {
        message: "Moderate failure".to_string(),
        severity: FailureSeverity::Moderate,
        context: "test".to_string(),
        recovery_attempts: 2,
    };
    
    let result2: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure2, || {
        unreachable!("This should not be called");
    });
    
    assert!(matches!(result2, Err(RecoveryError::MaxAttemptsExceeded { .. })));
}

#[test]
fn test_recovery_strategy_failure() {
    let mut recovery = RecoveryStrategy::default();
    
    let failure = FailureInfo {
        message: "Minor failure".to_string(),
        severity: FailureSeverity::Minor,
        context: "test".to_string(),
        recovery_attempts: 0,
    };
    
    // Mock a failed recovery action
    let result: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure, || {
        Err(Box::new(TestError("Recovery action failed".to_string())))
    });
    
    assert!(matches!(result, Err(RecoveryError::RecoveryActionFailed { .. })));
    
    // Check metrics
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 0);
    assert_eq!(metrics.failed_recoveries, 1);
}

#[test]
fn test_recovery_strategy_with_timeout_success() {
    let mut recovery = RecoveryStrategy::default();
    
    let failure = FailureInfo {
        message: "Minor failure".to_string(),
        severity: FailureSeverity::Minor,
        context: "test".to_string(),
        recovery_attempts: 0,
    };
    
    // Recovery action completes within timeout
    let result: std::result::Result<(), RecoveryError> = recovery.handle_failure_with_timeout(
        failure,
        Duration::from_millis(1000),
        || {
            // Fast operation
            Ok::<(), Box<dyn StdError + Send + Sync>>(())
        }
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_recovery_strategy_with_timeout_exceeded() {
    let mut recovery = RecoveryStrategy::default();
    
    let failure = FailureInfo {
        message: "Minor failure".to_string(),
        severity: FailureSeverity::Minor,
        context: "test".to_string(),
        recovery_attempts: 0,
    };
    
    // Recovery action exceeds timeout
    let result: std::result::Result<(), RecoveryError> = recovery.handle_failure_with_timeout(
        failure,
        Duration::from_millis(50),
        || {
            // Slow operation
            thread::sleep(Duration::from_millis(100));
            Ok::<(), Box<dyn StdError + Send + Sync>>(())
        }
    );
    
    assert!(matches!(result, Err(RecoveryError::Timeout { .. })));
}

#[test]
fn test_recovery_strategy_metrics() {
    let mut recovery = RecoveryStrategy::default();
    
    // Create a failure
    let failure = FailureInfo {
        message: "Minor failure".to_string(),
        severity: FailureSeverity::Minor,
        context: "test".to_string(),
        recovery_attempts: 0,
    };
    
    // Simulate successful recovery action
    let result: std::result::Result<i32, RecoveryError> = recovery.handle_failure(failure.clone(), || {
        // Simulate recovery action failure
        Err(Box::new(TestError("Recovery action failed".to_string())))
    });
    
    assert!(matches!(result, Err(RecoveryError::RecoveryActionFailed { .. })));
    
    // Create two more failures with successful recovery
    for _ in 0..2 {
        let _: std::result::Result<i32, RecoveryError> = recovery.handle_failure(failure.clone(), || {
            Ok::<i32, Box<dyn StdError + Send + Sync>>(42)
        });
    }
    
    // Create another failure with failed recovery
    let _: std::result::Result<i32, RecoveryError> = recovery.handle_failure(failure.clone(), || {
        Err(Box::new(TestError("Recovery action failed".to_string())))
    });
    
    // Check metrics
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 2);
    assert_eq!(metrics.failed_recoveries, 2);
    
    // Reset metrics
    recovery.reset_metrics();
    
    // Metrics should be reset to zero
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 0);
    assert_eq!(metrics.failed_recoveries, 0);
}

#[test]
fn test_recovery_strategy_metrics_accumulation() {
    let mut recovery = RecoveryStrategy::default();
    let counter = Arc::new(Mutex::new(0));
    
    // Create several failures with different severities
    for severity in [FailureSeverity::Minor, FailureSeverity::Moderate, FailureSeverity::Severe] {
        let failure = FailureInfo {
            message: format!("{:?} failure", severity),
            severity,
            context: "test".to_string(),
            recovery_attempts: 0,
        };
        
        // Mock a successful recovery action
        let counter_clone = counter.clone();
        let _: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure.clone(), move || {
            let mut count = counter_clone.lock().expect("should succeed");
            *count += 1;
            Ok::<(), Box<dyn StdError + Send + Sync>>(())
        });
    }
    
    // And one failure that fails recovery
    let failure = FailureInfo {
        message: "Failed recovery".to_string(),
        severity: FailureSeverity::Minor,
        context: "test".to_string(),
        recovery_attempts: 0,
    };
    
    let _: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure.clone(), || {
        Err(Box::new(TestError("Recovery failed".to_string())))
    });
    
    // Check metrics
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 3);
    assert_eq!(metrics.failed_recoveries, 1);
    assert_eq!(metrics.recoveries_by_severity[0], 2); // Minor (1 success, 1 failure)
    assert_eq!(metrics.recoveries_by_severity[1], 1); // Moderate (1 success)
    assert_eq!(metrics.recoveries_by_severity[2], 1); // Severe (1 success)
    
    // Verify the counter was incremented 3 times
    assert_eq!(*counter.lock().expect("should succeed"), 3);
    
    // Now reset metrics
    recovery.reset_metrics();
    
    // Metrics should be reset to zero
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 0);
    assert_eq!(metrics.failed_recoveries, 0);
    assert_eq!(metrics.recoveries_by_severity, [0, 0, 0, 0]);
}

#[test]
fn test_recovery_strategy_max_severe_attempts() {
    let mut recovery = RecoveryStrategy::new(RecoveryConfig {
        max_severe_attempts: 1,
        ..RecoveryConfig::default()
    });
    
    // Try with no previous attempts (under max)
    let failure1 = FailureInfo {
        message: "Severe failure".to_string(),
        severity: FailureSeverity::Severe,
        context: "test".to_string(),
        recovery_attempts: 0,
    };
    
    let result1: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure1, || {
        Ok::<(), Box<dyn StdError + Send + Sync>>(())
    });
    
    assert!(result1.is_ok());
    
    // Try with 1 previous attempt (at max)
    let failure2 = FailureInfo {
        message: "Severe failure".to_string(),
        severity: FailureSeverity::Severe,
        context: "test".to_string(),
        recovery_attempts: 1,
    };
    
    let result2: std::result::Result<(), RecoveryError> = recovery.handle_failure(failure2, || {
        unreachable!("This should not be called");
    });
    
    assert!(matches!(result2, Err(RecoveryError::MaxAttemptsExceeded { .. })));
}

#[test]
fn test_recovery_real_world_scenario() {
    // Simulating a real-world recovery strategy for a database connection
    let mut recovery = RecoveryStrategy::new(RecoveryConfig {
        max_minor_attempts: 3,
        max_moderate_attempts: 2,
        max_severe_attempts: 1,
        recover_critical: false,
    });
    
    // Simulate a database connection that fails on first attempt but succeeds on retry
    let db_connection = Arc::new(Mutex::new(false)); // Initially disconnected
    
    let failure = FailureInfo {
        message: "Database connection lost".to_string(),
        severity: FailureSeverity::Moderate,
        context: "database.connection".to_string(),
        recovery_attempts: 0,
    };
    
    // First recovery attempt
    let db_connection_clone = db_connection.clone();
    let result1: std::result::Result<String, RecoveryError> = recovery.handle_failure(failure.clone(), move || {
        let mut conn = db_connection_clone.lock().expect("should succeed");
        *conn = true; // Set to connected
        Ok::<_, Box<dyn StdError + Send + Sync>>("Connection restored".to_string())
    });
    
    assert!(result1.is_ok());
    assert_eq!(result1.expect("should succeed"), "Connection restored".to_string());
    assert!(*db_connection.lock().expect("should succeed")); // Should be connected now
    
    // Check metrics after successful recovery
    let metrics = recovery.get_metrics();
    assert_eq!(metrics.successful_recoveries, 1);
    assert_eq!(metrics.failed_recoveries, 0);
    assert_eq!(metrics.recoveries_by_severity[1], 1); // Moderate severity
} 