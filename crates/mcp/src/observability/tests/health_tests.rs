//! Health Tests
//! 
//! This module contains tests for the health checking functionality in the observability framework,
//! including health check registration, execution, and status management.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::observability::health::{HealthChecker, HealthCheck, HealthCheckResult, HealthStatus};
use crate::observability::ObservabilityError;

/// Test health status types
#[test]
fn test_health_status_types() {
    // Test health status ordering and comparison
    assert!(HealthStatus::Healthy < HealthStatus::Degraded);
    assert!(HealthStatus::Degraded < HealthStatus::Unhealthy);
    assert!(HealthStatus::Unknown < HealthStatus::Healthy);
    
    // Test from_str conversion
    assert_eq!(HealthStatus::from_str("healthy").unwrap(), HealthStatus::Healthy);
    assert_eq!(HealthStatus::from_str("degraded").unwrap(), HealthStatus::Degraded);
    assert_eq!(HealthStatus::from_str("unhealthy").unwrap(), HealthStatus::Unhealthy);
    assert_eq!(HealthStatus::from_str("unknown").unwrap(), HealthStatus::Unknown);
    
    // Test invalid conversion
    assert!(HealthStatus::from_str("invalid").is_err());
    
    // Test to_string representation
    assert_eq!(HealthStatus::Healthy.to_string(), "HEALTHY");
    assert_eq!(HealthStatus::Degraded.to_string(), "DEGRADED");
    assert_eq!(HealthStatus::Unhealthy.to_string(), "UNHEALTHY");
    assert_eq!(HealthStatus::Unknown.to_string(), "UNKNOWN");
}

/// Test health check result creation
#[test]
fn test_health_check_result_creation() {
    // Create basic result
    let result = HealthCheckResult::new(HealthStatus::Healthy, "Service is responding normally");
    assert_eq!(result.status(), HealthStatus::Healthy);
    assert_eq!(result.message(), "Service is responding normally");
    assert!(result.details().is_empty());
    
    // Add details
    let result_with_details = result.with_detail("response_time_ms", "42")
                                    .with_detail("success_rate", "99.9");
    
    assert_eq!(result_with_details.details().len(), 2);
    assert_eq!(result_with_details.details().get("response_time_ms").unwrap(), "42");
    assert_eq!(result_with_details.details().get("success_rate").unwrap(), "99.9");
}

/// Test health check creation
#[test]
fn test_health_check_creation() {
    // Create a simple health check
    let check = HealthCheck::new(
        "memory_check",
        "system",
        "Checks available system memory",
        Box::new(|| HealthCheckResult::healthy("Memory usage nominal"))
    );
    
    // Verify properties
    assert_eq!(check.id(), "memory_check");
    assert_eq!(check.component(), "system");
    assert_eq!(check.description(), "Checks available system memory");
    
    // Execute the check
    let result = check.execute().unwrap();
    assert_eq!(result.status(), HealthStatus::Healthy);
    assert_eq!(result.message(), "Memory usage nominal");
}

/// Test health check registration and retrieval
#[test]
fn test_health_check_registration() {
    let health_checker = HealthChecker::new();
    
    // Create health checks
    let memory_check = HealthCheck::new(
        "memory_check",
        "system",
        "Checks available system memory",
        Box::new(|| HealthCheckResult::healthy("Memory usage nominal"))
    );
    
    let cpu_check = HealthCheck::new(
        "cpu_check",
        "system",
        "Checks CPU utilization",
        Box::new(|| HealthCheckResult::degraded("CPU usage high"))
    );
    
    // Register checks
    let memory_ref = health_checker.register_check(memory_check).unwrap();
    let cpu_ref = health_checker.register_check(cpu_check).unwrap();
    
    // Verify registration
    assert_eq!(memory_ref.id(), "memory_check");
    assert_eq!(cpu_ref.id(), "cpu_check");
    
    // Retrieve checks
    let retrieved_memory = health_checker.get_check("memory_check").unwrap();
    let retrieved_cpu = health_checker.get_check("cpu_check").unwrap();
    
    assert_eq!(retrieved_memory.id(), "memory_check");
    assert_eq!(retrieved_cpu.id(), "cpu_check");
    
    // Execute retrieved checks
    let memory_result = retrieved_memory.execute().unwrap();
    let cpu_result = retrieved_cpu.execute().unwrap();
    
    assert_eq!(memory_result.status(), HealthStatus::Healthy);
    assert_eq!(cpu_result.status(), HealthStatus::Degraded);
    
    // Test non-existent check
    let not_found = health_checker.get_check("nonexistent");
    assert!(not_found.is_err());
}

/// Test health check execution
#[test]
fn test_health_check_execution() {
    let health_checker = HealthChecker::new();
    
    // Create a dynamic health check that returns different results each time
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);
    
    let dynamic_check = HealthCheck::new(
        "dynamic_check",
        "test",
        "Changes status each execution",
        Box::new(move || {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            
            match *count % 3 {
                0 => HealthCheckResult::healthy("Status: Good"),
                1 => HealthCheckResult::degraded("Status: Degraded"),
                _ => HealthCheckResult::unhealthy("Status: Bad"),
            }
        })
    );
    
    // Register check
    let check_ref = health_checker.register_check(dynamic_check).unwrap();
    
    // Execute multiple times
    for i in 1..=6 {
        let result = check_ref.execute().unwrap();
        
        match i % 3 {
            1 => {
                assert_eq!(result.status(), HealthStatus::Degraded);
                assert_eq!(result.message(), "Status: Degraded");
            },
            2 => {
                assert_eq!(result.status(), HealthStatus::Unhealthy);
                assert_eq!(result.message(), "Status: Bad");
            },
            0 => {
                assert_eq!(result.status(), HealthStatus::Healthy);
                assert_eq!(result.message(), "Status: Good");
            },
            _ => unreachable!(),
        }
    }
}

/// Test all checks execution
#[test]
fn test_execute_all_checks() {
    let health_checker = HealthChecker::new();
    
    // Register multiple checks
    health_checker.register_check(HealthCheck::new(
        "check1",
        "component1",
        "First check",
        Box::new(|| HealthCheckResult::healthy("Check 1 OK"))
    )).unwrap();
    
    health_checker.register_check(HealthCheck::new(
        "check2",
        "component1",
        "Second check",
        Box::new(|| HealthCheckResult::degraded("Check 2 slow"))
    )).unwrap();
    
    health_checker.register_check(HealthCheck::new(
        "check3",
        "component2",
        "Third check",
        Box::new(|| HealthCheckResult::unhealthy("Check 3 failed"))
    )).unwrap();
    
    // Execute all checks
    let results = health_checker.execute_all().unwrap();
    
    // Verify results
    assert_eq!(results.len(), 3);
    
    // Results should be in a HashMap with check IDs as keys
    assert!(results.contains_key("check1"));
    assert!(results.contains_key("check2"));
    assert!(results.contains_key("check3"));
    
    assert_eq!(results.get("check1").unwrap().status(), HealthStatus::Healthy);
    assert_eq!(results.get("check2").unwrap().status(), HealthStatus::Degraded);
    assert_eq!(results.get("check3").unwrap().status(), HealthStatus::Unhealthy);
}

/// Test component-specific checks
#[test]
fn test_component_checks() {
    let health_checker = HealthChecker::new();
    
    // Register checks for different components
    health_checker.register_check(HealthCheck::new(
        "db_connection",
        "database",
        "Database connection check",
        Box::new(|| HealthCheckResult::healthy("Database connected"))
    )).unwrap();
    
    health_checker.register_check(HealthCheck::new(
        "db_latency",
        "database",
        "Database latency check",
        Box::new(|| HealthCheckResult::degraded("Database latency high"))
    )).unwrap();
    
    health_checker.register_check(HealthCheck::new(
        "api_status",
        "api",
        "API status check",
        Box::new(|| HealthCheckResult::healthy("API responsive"))
    )).unwrap();
    
    health_checker.register_check(HealthCheck::new(
        "cache_status",
        "cache",
        "Cache status check",
        Box::new(|| HealthCheckResult::unhealthy("Cache offline"))
    )).unwrap();
    
    // Execute checks for specific component
    let db_results = health_checker.execute_component_checks("database").unwrap();
    
    // Verify component results
    assert_eq!(db_results.len(), 2);
    assert!(db_results.contains_key("db_connection"));
    assert!(db_results.contains_key("db_latency"));
    assert!(!db_results.contains_key("api_status"));
    
    // Test component health status
    let db_status = health_checker.get_component_status("database").unwrap();
    let api_status = health_checker.get_component_status("api").unwrap();
    let cache_status = health_checker.get_component_status("cache").unwrap();
    
    // Component status should be the worst status among its checks
    assert_eq!(db_status, HealthStatus::Degraded);
    assert_eq!(api_status, HealthStatus::Healthy);
    assert_eq!(cache_status, HealthStatus::Unhealthy);
    
    // Test non-existent component
    let not_found = health_checker.get_component_status("nonexistent");
    assert_eq!(not_found.unwrap(), HealthStatus::Unknown);
}

/// Test overall system health
#[test]
fn test_overall_system_health() {
    let health_checker = HealthChecker::new();
    
    // No checks registered yet, overall status should be Unknown
    assert_eq!(health_checker.overall_status().unwrap(), HealthStatus::Unknown);
    
    // Register checks with different statuses
    health_checker.register_check(HealthCheck::new(
        "check1",
        "component1",
        "First check",
        Box::new(|| HealthCheckResult::healthy("Good"))
    )).unwrap();
    
    // Overall still Unknown until checks are executed
    assert_eq!(health_checker.overall_status().unwrap(), HealthStatus::Unknown);
    
    // Execute all checks
    health_checker.execute_all().unwrap();
    
    // Now overall should be Healthy
    assert_eq!(health_checker.overall_status().unwrap(), HealthStatus::Healthy);
    
    // Add a degraded check
    health_checker.register_check(HealthCheck::new(
        "check2",
        "component2",
        "Second check",
        Box::new(|| HealthCheckResult::degraded("Degraded"))
    )).unwrap();
    
    // Execute all checks
    health_checker.execute_all().unwrap();
    
    // Overall should now be Degraded
    assert_eq!(health_checker.overall_status().unwrap(), HealthStatus::Degraded);
    
    // Add an unhealthy check
    health_checker.register_check(HealthCheck::new(
        "check3",
        "component3",
        "Third check",
        Box::new(|| HealthCheckResult::unhealthy("Failed"))
    )).unwrap();
    
    // Execute all checks
    health_checker.execute_all().unwrap();
    
    // Overall should now be Unhealthy
    assert_eq!(health_checker.overall_status().unwrap(), HealthStatus::Unhealthy);
}

/// Test health status transitions and time tracking
#[test]
fn test_health_status_tracking() {
    let health_checker = HealthChecker::new();
    
    // Create a check with changing status
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);
    
    let dynamic_check = HealthCheck::new(
        "tracking_check",
        "test",
        "Check with status transitions",
        Box::new(move || {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            
            match *count {
                1 => HealthCheckResult::healthy("Initial: Healthy"),
                2 => HealthCheckResult::degraded("Change: Degraded"),
                3 => HealthCheckResult::unhealthy("Change: Unhealthy"),
                4 => HealthCheckResult::degraded("Change: Back to Degraded"),
                _ => HealthCheckResult::healthy("Change: Back to Healthy"),
            }
        })
    );
    
    // Register check
    let check_ref = health_checker.register_check(dynamic_check).unwrap();
    
    // First execution (Healthy)
    let result1 = check_ref.execute().unwrap();
    assert_eq!(result1.status(), HealthStatus::Healthy);
    
    // Track initial timestamp
    let timestamp1 = result1.timestamp();
    
    // Short pause
    thread::sleep(Duration::from_millis(10));
    
    // Second execution (Degraded) - status transition
    let result2 = check_ref.execute().unwrap();
    assert_eq!(result2.status(), HealthStatus::Degraded);
    
    // Timestamp should be updated
    let timestamp2 = result2.timestamp();
    assert!(timestamp2 > timestamp1);
    
    // Get status history
    let history = health_checker.get_status_history("tracking_check").unwrap();
    
    // Should have two entries
    assert_eq!(history.len(), 2);
    
    // First entry should be Healthy
    assert_eq!(history[0].status, HealthStatus::Healthy);
    assert_eq!(history[0].timestamp, timestamp1);
    
    // Second entry should be Degraded
    assert_eq!(history[1].status, HealthStatus::Degraded);
    assert_eq!(history[1].timestamp, timestamp2);
}

/// Test health check timeout handling
#[test]
fn test_health_check_timeout() {
    let health_checker = HealthChecker::new();
    
    // Create a health check that sleeps (simulating a slow check)
    let slow_check = HealthCheck::new(
        "slow_check",
        "system",
        "A deliberately slow health check",
        Box::new(|| {
            // Sleep for 100ms
            thread::sleep(Duration::from_millis(100));
            HealthCheckResult::healthy("Completed after delay")
        })
    );
    
    // Register with a timeout
    let check_ref = health_checker.register_check_with_timeout(
        slow_check,
        Duration::from_millis(50) // 50ms timeout (less than the sleep)
    ).unwrap();
    
    // Execute the check - should timeout
    let result = check_ref.execute();
    
    // Verify timeout error
    assert!(result.is_err());
    match result {
        Err(ObservabilityError::HealthError(msg)) => {
            assert!(msg.contains("timed out"));
        },
        _ => panic!("Expected health timeout error"),
    }
}

/// Test health check dependencies
#[test]
fn test_health_check_dependencies() {
    let health_checker = HealthChecker::new();
    
    // Create a primary check
    let primary_check = HealthCheck::new(
        "database",
        "data",
        "Database health",
        Box::new(|| HealthCheckResult::healthy("Database is healthy"))
    );
    
    // Register primary check
    let primary_ref = health_checker.register_check(primary_check).unwrap();
    
    // Create a dependent check
    let dependent_check = HealthCheck::new_with_dependencies(
        "cache",
        "data",
        "Cache health",
        Box::new(|| HealthCheckResult::healthy("Cache is healthy")),
        vec!["database".to_string()]
    );
    
    // Register dependent check
    let dependent_ref = health_checker.register_check(dependent_check).unwrap();
    
    // Execute dependent check - should execute primary first
    let result = dependent_ref.execute().unwrap();
    assert_eq!(result.status(), HealthStatus::Healthy);
    
    // Change primary check to unhealthy
    let unhealthy_primary = HealthCheck::new(
        "database",
        "data",
        "Database health",
        Box::new(|| HealthCheckResult::unhealthy("Database is down"))
    );
    
    // Update the check
    health_checker.update_check(unhealthy_primary).unwrap();
    
    // Execute dependent check again - should be unhealthy due to dependency
    let result = dependent_ref.execute().unwrap();
    assert_eq!(result.status(), HealthStatus::Unhealthy);
    assert!(result.message().contains("dependency"));
}

/// Test health check error handling
#[test]
fn test_health_check_error_handling() {
    let health_checker = HealthChecker::new();
    
    // Create a check that panics
    let panicking_check = HealthCheck::new(
        "panicking_check",
        "test",
        "A check that panics",
        Box::new(|| {
            panic!("This check deliberately panics");
        })
    );
    
    // Register check
    let check_ref = health_checker.register_check(panicking_check).unwrap();
    
    // Execute check - should handle panic and return an error
    let result = check_ref.execute();
    assert!(result.is_err());
    
    // Create a check that returns an error
    let error_check = HealthCheck::new(
        "error_check",
        "test",
        "A check that returns an error",
        Box::new(|| {
            let result: Result<_, &str> = Err("Deliberate error");
            result.map_err(|e| ObservabilityError::HealthError(e.to_string()))?;
            Ok(HealthCheckResult::healthy("Unreachable"))
        })
    );
    
    // Register check
    let check_ref = health_checker.register_check(error_check).unwrap();
    
    // Execute check - should return the error
    let result = check_ref.execute();
    assert!(result.is_err());
}

/// Test concurrent health check execution
#[tokio::test]
async fn test_concurrent_health_checks() {
    let health_checker = Arc::new(HealthChecker::new());
    
    // Register multiple checks
    for i in 0..5 {
        let check = HealthCheck::new(
            &format!("check{}", i),
            "test",
            &format!("Test check {}", i),
            Box::new(move || {
                // Simulate work with varying durations
                thread::sleep(Duration::from_millis(10 * (i as u64 + 1)));
                HealthCheckResult::healthy(&format!("Check {} is healthy", i))
            })
        );
        
        health_checker.register_check(check).unwrap();
    }
    
    // Execute all checks concurrently
    let mut handles = vec![];
    
    for i in 0..5 {
        let checker_clone = Arc::clone(&health_checker);
        let handle = tokio::spawn(async move {
            let check_ref = checker_clone.get_check(&format!("check{}", i)).unwrap();
            check_ref.execute().unwrap()
        });
        
        handles.push(handle);
    }
    
    // Wait for all checks to complete
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    
    // Verify all checks completed successfully
    assert_eq!(results.len(), 5);
    
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.status(), HealthStatus::Healthy);
        assert_eq!(result.message(), format!("Check {} is healthy", i));
    }
}

/// Test health check serialization
#[test]
fn test_health_check_serialization() {
    let health_checker = HealthChecker::new();
    
    // Register some checks
    health_checker.register_check(HealthCheck::new(
        "api_check",
        "api",
        "API health check",
        Box::new(|| HealthCheckResult::healthy("API is healthy"))
    )).unwrap();
    
    health_checker.register_check(HealthCheck::new(
        "db_check",
        "database",
        "Database health check",
        Box::new(|| HealthCheckResult::degraded("Database is slow"))
    )).unwrap();
    
    // Execute all checks
    health_checker.execute_all().unwrap();
    
    // Get JSON representation
    let json_report = health_checker.get_json_report().unwrap();
    
    // Basic validation of JSON structure
    assert!(json_report.contains("api_check"));
    assert!(json_report.contains("db_check"));
    assert!(json_report.contains("HEALTHY"));
    assert!(json_report.contains("DEGRADED"));
    
    // Get health report structure
    let report = health_checker.get_health_report().unwrap();
    
    // Verify report structure
    assert_eq!(report.overall_status, HealthStatus::Degraded);
    assert_eq!(report.components.len(), 2);
    assert!(report.components.contains_key("api"));
    assert!(report.components.contains_key("database"));
    
    // Verify component status
    assert_eq!(report.components.get("api").unwrap().status, HealthStatus::Healthy);
    assert_eq!(report.components.get("database").unwrap().status, HealthStatus::Degraded);
}

/// Test health check removal
#[test]
fn test_health_check_removal() {
    let health_checker = HealthChecker::new();
    
    // Register a check
    health_checker.register_check(HealthCheck::new(
        "removable_check",
        "test",
        "A check to be removed",
        Box::new(|| HealthCheckResult::healthy("Still here"))
    )).unwrap();
    
    // Verify check exists
    assert!(health_checker.get_check("removable_check").is_ok());
    
    // Remove the check
    let removed = health_checker.remove_check("removable_check").unwrap();
    assert_eq!(removed.id(), "removable_check");
    
    // Verify check is gone
    assert!(health_checker.get_check("removable_check").is_err());
    
    // Test removing non-existent check
    assert!(health_checker.remove_check("nonexistent").is_err());
}

#[test]
fn test_health_status_display() {
    assert_eq!(format!("{}", HealthStatus::Healthy), "HEALTHY");
    assert_eq!(format!("{}", HealthStatus::Degraded), "DEGRADED");
    assert_eq!(format!("{}", HealthStatus::Unhealthy), "UNHEALTHY");
    assert_eq!(format!("{}", HealthStatus::Unknown), "UNKNOWN");
}

#[test]
fn test_health_checker_initialization() {
    let health_checker = HealthChecker::new();
    assert!(health_checker.initialize().is_ok());
}

#[test]
fn test_register_check() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().unwrap();
    
    // Register a basic check
    let check_result = health_checker.register_check_fn(
        "memory_usage",
        "system",
        "Checks if memory usage is within acceptable limits",
        || HealthCheckResult::healthy("Memory usage is normal")
    );
    
    assert!(check_result.is_ok());
    
    // Execute the check
    let result = health_checker.execute_check("memory_usage");
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.status(), HealthStatus::Healthy);
    assert_eq!(result.message(), "Memory usage is normal");
}

#[test]
fn test_component_checks() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().unwrap();
    
    // Register multiple checks for the same component
    health_checker.register_check_fn(
        "disk_space",
        "storage",
        "Checks available disk space",
        || HealthCheckResult::healthy("Disk space is sufficient")
    ).unwrap();
    
    health_checker.register_check_fn(
        "disk_io",
        "storage",
        "Checks disk I/O performance",
        || HealthCheckResult::degraded("Disk I/O is slower than expected")
            .with_detail("read_latency_ms", "250")
            .with_detail("write_latency_ms", "300")
    ).unwrap();
    
    // Check all storage checks
    let results = health_checker.check_component("storage").unwrap();
    assert_eq!(results.len(), 2);
    
    // Get overall component status (should be degraded because one check is degraded)
    let status = health_checker.component_status("storage").unwrap();
    assert_eq!(status, HealthStatus::Degraded);
}

#[test]
fn test_get_component_checks() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().unwrap();
    
    // Register checks for different components
    health_checker.register_check_fn(
        "api_health",
        "api",
        "Check API endpoints",
        || HealthCheckResult::healthy("API is responsive")
    ).unwrap();
    
    health_checker.register_check_fn(
        "database_connection",
        "database",
        "Check database connection",
        || HealthCheckResult::healthy("Database connection is active")
    ).unwrap();
    
    health_checker.register_check_fn(
        "database_performance",
        "database",
        "Check database query performance",
        || HealthCheckResult::healthy("Database queries are performing well")
    ).unwrap();
    
    // Get all checks for the database component
    let db_checks = health_checker.get_component_checks("database").unwrap();
    assert_eq!(db_checks.len(), 2);
    
    // Verify check descriptions
    for check in db_checks {
        assert_eq!(check.component(), "database");
        assert!(check.name() == "database_connection" || check.name() == "database_performance");
    }
    
    // Get all component statuses
    let all_statuses = health_checker.all_component_status().unwrap();
    assert_eq!(all_statuses.len(), 2); // api and database
    assert_eq!(all_statuses.get("api").unwrap(), &HealthStatus::Healthy);
    assert_eq!(all_statuses.get("database").unwrap(), &HealthStatus::Healthy);
}

#[test]
fn test_health_status_change_notifications() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().unwrap();
    
    // Subscribe to health status changes
    let mut subscriber = health_checker.subscribe();
    
    // Initial status is unknown
    let counter_state = Arc::new(Mutex::new(0));
    let counter_clone = counter_state.clone();
    
    // Register a check that will change status
    health_checker.register_check_fn(
        "service_check",
        "critical_service",
        "Check critical service health",
        move || {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            
            match *count {
                1 => HealthCheckResult::unknown("Initial check"),
                2 => HealthCheckResult::healthy("Service is healthy"),
                3 => HealthCheckResult::degraded("Service is degraded"),
                _ => HealthCheckResult::unhealthy("Service is unhealthy"),
            }
        }
    ).unwrap();
    
    // Run the check multiple times, causing status changes
    for _ in 0..4 {
        health_checker.check_component("critical_service").unwrap();
        
        // Try to receive status change notification
        let notification = subscriber.try_recv();
        assert!(notification.is_ok());
        
        let status_change = notification.unwrap();
        assert_eq!(status_change.component, "critical_service");
    }
}

#[test]
fn test_check_all_components() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().unwrap();
    
    // Register checks for multiple components
    health_checker.register_check_fn(
        "api_check",
        "api",
        "API health check",
        || HealthCheckResult::healthy("API is healthy")
    ).unwrap();
    
    health_checker.register_check_fn(
        "db_check",
        "database",
        "Database health check",
        || HealthCheckResult::degraded("Database is slow")
    ).unwrap();
    
    health_checker.register_check_fn(
        "cache_check",
        "cache",
        "Cache health check",
        || HealthCheckResult::unhealthy("Cache is down")
    ).unwrap();
    
    // Check all components
    let all_results = health_checker.check_all().unwrap();
    
    // Verify results
    assert_eq!(all_results.len(), 3); // three components
    assert_eq!(all_results.get("api").unwrap().len(), 1);
    assert_eq!(all_results.get("database").unwrap().len(), 1);
    assert_eq!(all_results.get("cache").unwrap().len(), 1);
    
    // Verify overall status
    let all_statuses = health_checker.all_component_status().unwrap();
    assert_eq!(all_statuses.get("api").unwrap(), &HealthStatus::Healthy);
    assert_eq!(all_statuses.get("database").unwrap(), &HealthStatus::Degraded);
    assert_eq!(all_statuses.get("cache").unwrap(), &HealthStatus::Unhealthy);
}

#[test]
fn test_check_caching() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().unwrap();
    
    // Create a counter to track executions
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    
    // Register a check that increments the counter
    health_checker.register_check_fn(
        "counting_check",
        "counter",
        "A check that counts executions",
        move || {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            HealthCheckResult::healthy(format!("Check executed {} times", *count))
        }
    ).unwrap();
    
    // Execute the check
    health_checker.execute_check("counting_check").unwrap();
    
    // Get last result directly from the check
    let check = health_checker.get_check("counting_check").unwrap().unwrap();
    let last_result = check.last_result().unwrap();
    
    // Verify last result exists and is correct
    assert!(last_result.is_some());
    let result = last_result.unwrap();
    assert_eq!(result.status(), HealthStatus::Healthy);
    assert_eq!(result.message(), "Check executed 1 times");
    
    // Execute again and verify counter increased
    health_checker.execute_check("counting_check").unwrap();
    let count = counter.lock().unwrap();
    assert_eq!(*count, 2);
}

#[test]
fn test_concurrent_health_checks() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().unwrap();
    
    // Register a check that sleeps briefly
    health_checker.register_check_fn(
        "slow_check",
        "concurrent_test",
        "A deliberately slow check",
        || {
            thread::sleep(Duration::from_millis(50));
            HealthCheckResult::healthy("Slow check completed")
        }
    ).unwrap();
    
    // Execute the check from multiple threads
    let num_threads = 5;
    let mut handles = vec![];
    
    let checker = Arc::new(health_checker);
    
    for _ in 0..num_threads {
        let checker_clone = checker.clone();
        let handle = thread::spawn(move || {
            checker_clone.execute_check("slow_check").unwrap()
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete and collect results
    let results: Vec<_> = handles.into_iter()
                               .map(|handle| handle.join().unwrap())
                               .collect();
    
    // Verify all threads got a result
    assert_eq!(results.len(), num_threads);
    
    // All results should indicate healthy status
    for result in results {
        assert_eq!(result.status(), HealthStatus::Healthy);
        assert_eq!(result.message(), "Slow check completed");
    }
}

#[test]
fn test_error_handling() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().unwrap();
    
    // Try to execute a non-existent check
    let result = health_checker.execute_check("non_existent_check");
    assert!(result.is_err());
    
    // Try to get component status for non-existent component
    let result = health_checker.component_status("non_existent_component");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), HealthStatus::Unknown); // Non-existent components have Unknown status
}

#[test]
fn test_check_descriptions() {
    let health_checker = HealthChecker::new();
    health_checker.initialize().unwrap();
    
    health_checker.register_check_fn(
        "descriptive_check",
        "documentation",
        "This check has a detailed description of its purpose and function",
        || HealthCheckResult::healthy("Documentation is complete")
    ).unwrap();
    
    let check = health_checker.get_check("descriptive_check").unwrap().unwrap();
    assert_eq!(check.name(), "descriptive_check");
    assert_eq!(check.component(), "documentation");
    assert_eq!(check.description(), "This check has a detailed description of its purpose and function");
} 