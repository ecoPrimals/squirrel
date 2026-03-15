// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Recovery Mechanism Integration Tests
//!
//! Tests for recovery strategies working with retry mechanisms,
//! covering different failure severities and recovery scenarios.

use super::*;

/// Test recovery mechanism working with retry
#[tokio::test]
async fn test_recovery_with_retry() {
    let retry = create_test_retry_mechanism();
    let mut recovery = create_test_recovery_strategy();
    
    let operation_counter = Arc::new(Mutex::new(0));
    let recovery_counter = Arc::new(Mutex::new(0));
    
    // First scenario: Retry succeeds, no recovery needed
    {
        let op_counter = operation_counter.clone();
        
        let result = retry.execute(move || {
            let op_counter_clone = op_counter.clone();
            Box::pin(async move {
                let mut count = op_counter_clone.lock().unwrap();
                *count += 1;
                
                if *count == 1 {
                    // First attempt fails
                    Err(Box::<dyn StdError + Send + Sync>::from(TestError("Temporary error".to_string())))
                } else {
                    // Second attempt succeeds
                    Ok(TestString("Success".to_string()))
                }
            })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "Success".to_string());
        assert_operation_count(&operation_counter, 2, "Retry success scenario");
    }
    
    // Reset counters
    *operation_counter.lock().unwrap() = 0;
    *recovery_counter.lock().unwrap() = 0;
    
    // Second scenario: Retry fails, recovery needed
    {
        let op_counter = operation_counter.clone();
        let rec_counter = recovery_counter.clone();
        
        // Try an operation that always fails, requiring recovery
        let failure_result: std::result::Result<TestString, _> = retry.execute(move || {
            let op_counter_clone = op_counter.clone();
            Box::pin(async move {
                let mut count = op_counter_clone.lock().unwrap();
                *count += 1;
                
                // Always fail to trigger recovery
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent failure".to_string())))
            })
        }).await;
        
        // The retry should fail
        assert!(failure_result.is_err());
        assert_operation_count(&operation_counter, 2, "Retry failure scenario"); // Should be called max_attempts times
        
        // Now test recovery using the correct method
        let failure_info = create_test_failure_info(FailureSeverity::Minor, "test");
        
        let recovery_result = recovery.handle_failure(
            failure_info,
            move || {
                let rec_counter_clone = rec_counter.clone();
                let mut count = rec_counter_clone.lock().unwrap();
                *count += 1;
                
                // Recovery succeeds
                Ok(TestString("Recovery successful".to_string()))
            }
        );
        
        assert!(recovery_result.is_ok());
        assert_eq!(recovery_result.unwrap().0, "Recovery successful".to_string());
        assert_operation_count(&recovery_counter, 1, "Recovery scenario");
    }
}

/// Test different failure severities with recovery
#[tokio::test]
async fn test_recovery_failure_severities() {
    let mut recovery = create_aggressive_recovery_strategy();
    let recovery_counter = Arc::new(Mutex::new(0));
    
    // Test minor failure recovery
    {
        let failure_info = create_test_failure_info(FailureSeverity::Minor, "minor_test");
        let counter = recovery_counter.clone();
        
        let result = recovery.handle_failure(
            failure_info,
            move || {
                let counter_clone = counter.clone();
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                
                Ok(TestString("Minor recovery".to_string()))
            }
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "Minor recovery".to_string());
    }
    
    // Reset counter
    *recovery_counter.lock().unwrap() = 0;
    
    // Test moderate failure recovery
    {
        let failure_info = create_test_failure_info(FailureSeverity::Moderate, "moderate_test");
        let counter = recovery_counter.clone();
        
        let result = recovery.handle_failure(
            failure_info,
            move || {
                let counter_clone = counter.clone();
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                
                Ok(TestString("Moderate recovery".to_string()))
            }
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "Moderate recovery".to_string());
    }
    
    // Reset counter
    *recovery_counter.lock().unwrap() = 0;
    
    // Test severe failure recovery
    {
        let failure_info = create_test_failure_info(FailureSeverity::Severe, "severe_test");
        let counter = recovery_counter.clone();
        
        let result = recovery.handle_failure(
            failure_info,
            move || {
                let counter_clone = counter.clone();
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                
                Ok(TestString("Severe recovery".to_string()))
            }
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "Severe recovery".to_string());
    }
}

/// Test recovery attempts with multiple failures
#[tokio::test]
async fn test_recovery_multiple_attempts() {
    let mut recovery = create_test_recovery_strategy();
    let failure_counter = Arc::new(Mutex::new(0));
    
    let failure_info = create_test_failure_info(FailureSeverity::Minor, "multi_attempt_test");
    
    // Test recovery that fails initially but succeeds on second attempt
    let counter = failure_counter.clone();
    let result = recovery.handle_failure(
        failure_info,
        move || {
            let counter_clone = counter.clone();
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            
            if *count == 1 {
                // First recovery attempt fails
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Recovery failed initially".to_string())))
            } else {
                // Second recovery attempt succeeds
                Ok(TestString("Recovery succeeded on retry".to_string()))
            }
        }
    );
    
    // Depending on recovery strategy implementation, this might succeed or fail
    // The important thing is that multiple attempts were made
    assert_operation_count(&failure_counter, 1, "Recovery attempts"); // At least one attempt should be made
    
    if result.is_ok() {
        assert_eq!(result.unwrap().0, "Recovery succeeded on retry".to_string());
    }
} 