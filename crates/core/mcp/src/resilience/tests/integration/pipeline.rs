// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Resilience Pipeline Integration Tests
//!
//! Tests for complete resilience pipeline integration including complex workflows,
//! multi-stage operations, and comprehensive pipeline scenarios.

use super::*;

/// Test the full resilience pipeline with complex workflow
#[tokio::test]
async fn test_full_resilience_pipeline() {
    let mut circuit_breaker = create_lenient_circuit_breaker("test-pipeline");
    let retry = create_exponential_retry_mechanism();
    let mut recovery = create_test_recovery_strategy();
    
    let api_data_cache = Arc::new(Mutex::new(HashMap::new()));
    {
        let mut cache = api_data_cache.lock().unwrap();
        cache.insert("test_data".to_string(), 42);
    }
    
    // Create health monitor
    let health_monitor = create_test_health_monitor();
    
    // Test the complete integration
    let _: Result<i32, ResilienceError> = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "test_component",
        create_test_failure_info(FailureSeverity::Moderate, "test-service"),
        move || {
            // This should succeed
            Result::<i32, Box<dyn StdError + Send + Sync>>::Ok(123)
        },
        move || {
            // This is the fallback
            Result::<i32, Box<dyn StdError + Send + Sync>>::Ok(999)
        }
    ).await;
}

/// Test pipeline with data transformation stages
#[tokio::test]
async fn test_pipeline_data_transformation() {
    let mut circuit_breaker = create_test_circuit_breaker("data-pipeline");
    let retry = create_test_retry_mechanism();
    let mut recovery = create_test_recovery_strategy();
    let health_monitor = create_test_health_monitor();
    
    let processing_state = Arc::new(Mutex::new(HashMap::new()));
    
    // Initialize processing state
    {
        let mut state = processing_state.lock().unwrap();
        state.insert("raw_data".to_string(), "input,data,values".to_string());
        state.insert("processed_data".to_string(), String::new());
    }
    
    // Stage 1: Data ingestion
    let state1 = processing_state.clone();
    let ingestion_result = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "data_ingestion",
        create_test_failure_info(FailureSeverity::Minor, "ingestion"),
        move || {
            let state_clone = state1.clone();
            let state = state_clone.lock().unwrap();
            
            if let Some(raw_data) = state.get("raw_data") {
                Ok(TestString(format!("ingested:{}", raw_data)))
            } else {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("No raw data found".to_string())))
            }
        },
        move || {
            // Fallback: return empty dataset
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("ingested:fallback_data".to_string()))
        }
    ).await;
    
    assert!(ingestion_result.is_ok());
    let ingested_data = ingestion_result.unwrap().0;
    assert!(ingested_data.starts_with("ingested:"));
    
    // Stage 2: Data processing
    let state2 = processing_state.clone();
    let processing_result = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "data_processing",
        create_test_failure_info(FailureSeverity::Minor, "processing"),
        move || {
            // Process the ingested data
            let processed = ingested_data.replace("ingested:", "processed:");
            Ok(TestString(processed))
        },
        move || {
            // Fallback: return cached processed data
            let state_clone = state2.clone();
            let state = state_clone.lock().unwrap();
            
            if let Some(cached) = state.get("processed_data") {
                if !cached.is_empty() {
                    Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString(cached.clone()))
                } else {
                    Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("processed:cached_fallback".to_string()))
                }
            } else {
                Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("processed:default_fallback".to_string()))
            }
        }
    ).await;
    
    assert!(processing_result.is_ok());
    let processed_data = processing_result.unwrap().0;
    assert!(processed_data.starts_with("processed:"));
}

/// Test pipeline with multiple failure scenarios
#[tokio::test]
async fn test_pipeline_multiple_failures() {
    let mut circuit_breaker = create_test_circuit_breaker("multi-failure-pipeline");
    let retry = create_test_retry_mechanism();
    let mut recovery = create_aggressive_recovery_strategy();
    let health_monitor = create_test_health_monitor();
    
    let failure_counter = Arc::new(Mutex::new(0));
    let recovery_counter = Arc::new(Mutex::new(0));
    
    // Stage 1: Always fails
    let fail_counter = failure_counter.clone();
    let rec_counter = recovery_counter.clone();
    
    let stage1_result = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "failing_stage",
        create_test_failure_info(FailureSeverity::Moderate, "stage1"),
        move || {
            let counter_clone = fail_counter.clone();
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            
            // Always fail
            Err(Box::<dyn StdError + Send + Sync>::from(TestError("Stage 1 failure".to_string())))
        },
        move || {
            let counter_clone = rec_counter.clone();
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            
            // Recovery provides fallback
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("stage1_recovery_result".to_string()))
        }
    ).await;
    
    // Should succeed via recovery
    assert!(stage1_result.is_ok());
    assert_eq!(stage1_result.unwrap().0, "stage1_recovery_result".to_string());
    
    // Verify failure and recovery attempts
    assert!(
        *failure_counter.lock().unwrap() >= 2, 
        "Expected at least 2 failure attempts due to retries"
    );
    assert_operation_count(&recovery_counter, 1, "Recovery attempts");
}

/// Test pipeline with conditional processing
#[tokio::test]
async fn test_pipeline_conditional_processing() {
    let mut circuit_breaker = create_test_circuit_breaker("conditional-pipeline");
    let retry = create_test_retry_mechanism();
    let mut recovery = create_test_recovery_strategy();
    let health_monitor = create_test_health_monitor();
    
    let condition_state = Arc::new(Mutex::new(true));
    
    // Conditional stage: behaves differently based on state
    let state = condition_state.clone();
    let result = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "conditional_stage",
        create_test_failure_info(FailureSeverity::Minor, "conditional"),
        move || {
            let state_clone = state.clone();
            let condition = *state_clone.lock().unwrap();
            
            if condition {
                Ok(TestString("condition_met".to_string()))
            } else {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Condition not met".to_string())))
            }
        },
        move || {
            // Recovery: provide alternative result
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("condition_recovery".to_string()))
        }
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "condition_met".to_string());
    
    // Change condition and test again
    *condition_state.lock().unwrap() = false;
    
    let state2 = condition_state.clone();
    let result2 = with_complete_resilience(
        &mut circuit_breaker,
        retry,
        &mut recovery,
        &health_monitor,
        "conditional_stage_2",
        create_test_failure_info(FailureSeverity::Minor, "conditional2"),
        move || {
            let state_clone = state2.clone();
            let condition = *state_clone.lock().unwrap();
            
            if condition {
                Ok(TestString("condition_met_2".to_string()))
            } else {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Condition not met 2".to_string())))
            }
        },
        move || {
            // Recovery: provide alternative result
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("condition_recovery_2".to_string()))
        }
    ).await;
    
    // Should succeed via recovery since condition is false
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().0, "condition_recovery_2".to_string());
}

/// Test pipeline with resource management
#[tokio::test]
async fn test_pipeline_resource_management() {
    let mut circuit_breaker = create_lenient_circuit_breaker("resource-pipeline");
    let retry = create_test_retry_mechanism();
    let mut recovery = create_test_recovery_strategy();
    let health_monitor = create_test_health_monitor();
    
    let resource_pool = Arc::new(Mutex::new(HashMap::new()));
    
    // Initialize resource pool
    {
        let mut pool = resource_pool.lock().unwrap();
        pool.insert("db_connection".to_string(), true);
        pool.insert("cache_connection".to_string(), true);
        pool.insert("queue_connection".to_string(), false); // Unavailable
    }
    
    // Test resource acquisition
    let pool1 = resource_pool.clone();
    let resource_result = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "resource_acquisition",
        create_test_failure_info(FailureSeverity::Minor, "resources"),
        move || {
            let pool_clone = pool1.clone();
            let pool = pool_clone.lock().unwrap();
            
            // Try to acquire a database connection
            if let Some(&available) = pool.get("db_connection") {
                if available {
                    Ok(TestString("resource_acquired:db_connection".to_string()))
                } else {
                    Err(Box::<dyn StdError + Send + Sync>::from(TestError("DB connection unavailable".to_string())))
                }
            } else {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("No DB connection in pool".to_string())))
            }
        },
        move || {
            // Fallback: use cache connection
            let pool_clone = resource_pool.clone();
            let pool = pool_clone.lock().unwrap();
            
            if let Some(&available) = pool.get("cache_connection") {
                if available {
                    Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("resource_acquired:cache_connection".to_string()))
                } else {
                    Err(Box::<dyn StdError + Send + Sync>::from(TestError("No resources available".to_string())))
                }
            } else {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("No cache connection in pool".to_string())))
            }
        }
    ).await;
    
    assert!(resource_result.is_ok());
    let resource_response = resource_result.unwrap().0;
    assert!(resource_response.contains("resource_acquired:"));
}

/// Test simplified multi-stage pipeline
#[tokio::test]
async fn test_simplified_multi_stage_pipeline() {
    let mut circuit_breaker = create_test_circuit_breaker("simplified-integration");
    let retry = create_exponential_retry_mechanism();
    let mut recovery = create_aggressive_recovery_strategy();
    let health_monitor = create_test_health_monitor();
    
    // Test 3-stage pipeline with static data
    let mut pipeline_result = "start".to_string();
    
    // Stage 1: Ingestion
    let stage1_input = pipeline_result.clone();
    let stage1_result = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "stage1",
        create_test_failure_info(FailureSeverity::Minor, "ingestion"),
        move || {
            let output = format!("{}_ingested", stage1_input);
            Ok(TestString(output))
        },
        move || {
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("fallback_ingested".to_string()))
        }
    ).await;
    
    assert!(stage1_result.is_ok());
    pipeline_result = stage1_result.unwrap().0;
    
    // Stage 2: Processing
    let stage2_input = pipeline_result.clone();
    let stage2_result = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "stage2",
        create_test_failure_info(FailureSeverity::Minor, "processing"),
        move || {
            let output = format!("{}_processed", stage2_input);
            Ok(TestString(output))
        },
        move || {
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("fallback_processed".to_string()))
        }
    ).await;
    
    assert!(stage2_result.is_ok());
    pipeline_result = stage2_result.unwrap().0;
    
    // Stage 3: Output
    let stage3_input = pipeline_result.clone();
    let stage3_result = with_complete_resilience(
        &mut circuit_breaker,
        retry,
        &mut recovery,
        &health_monitor,
        "stage3",
        create_test_failure_info(FailureSeverity::Minor, "output"),
        move || {
            let output = format!("{}_output", stage3_input);
            Ok(TestString(output))
        },
        move || {
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("fallback_output".to_string()))
        }
    ).await;
    
    assert!(stage3_result.is_ok());
    let final_result = stage3_result.unwrap().0;
    
    // Verify complete pipeline execution
    assert_eq!(final_result, "start_ingested_processed_output");
} 