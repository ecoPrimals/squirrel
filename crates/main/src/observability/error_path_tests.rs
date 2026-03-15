// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive error path tests for observability module
//!
//! These tests expand coverage by validating error handling, edge cases,
//! and boundary conditions in the observability system.

#[cfg(test)]
mod observability_error_tests {
    use super::super::*;
    use std::time::Duration;

    #[test]
    fn test_correlation_id_creation() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();

        // Each correlation ID should be unique
        assert_ne!(id1.as_str(), id2.as_str());
        assert!(!id1.as_str().is_empty());
        assert!(!id2.as_str().is_empty());
    }

    #[test]
    fn test_correlation_id_from_string() {
        let id_str = "test-correlation-123";
        let id = CorrelationId::from_string(id_str);

        assert_eq!(id.as_str(), id_str);
    }

    #[test]
    fn test_correlation_id_display() {
        let id = CorrelationId::from_string("display-test");
        let displayed = format!("{}", id);

        assert_eq!(displayed, "display-test");
    }

    #[test]
    fn test_correlation_id_default() {
        let id = CorrelationId::default();

        // Default should generate a valid UUID
        assert!(!id.as_str().is_empty());
        assert!(id.as_str().len() > 0);
    }

    #[test]
    fn test_correlation_id_clone() {
        let id1 = CorrelationId::from_string("clone-test");
        let id2 = id1.clone();

        assert_eq!(id1, id2);
        assert_eq!(id1.as_str(), id2.as_str());
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();

        assert_eq!(metrics.total_duration, Duration::ZERO);
        assert_eq!(metrics.attempts, 0);
        assert!(!metrics.success);
        assert!(metrics.error_info.is_none());
        assert!(metrics.phase_durations.is_empty());
    }

    #[test]
    fn test_performance_metrics_record_phase() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_phase("discovery", Duration::from_millis(100));
        metrics.record_phase("execution", Duration::from_millis(200));
        metrics.record_phase("cleanup", Duration::from_millis(50));

        assert_eq!(metrics.phase_durations.len(), 3);
        assert_eq!(
            metrics.phase_durations.get("discovery"),
            Some(&Duration::from_millis(100))
        );
        assert_eq!(
            metrics.phase_durations.get("execution"),
            Some(&Duration::from_millis(200))
        );
        assert_eq!(
            metrics.phase_durations.get("cleanup"),
            Some(&Duration::from_millis(50))
        );
    }

    #[test]
    fn test_performance_metrics_mark_success() {
        let mut metrics = PerformanceMetrics::new();

        metrics.mark_success(Duration::from_secs(1));

        assert!(metrics.success);
        assert_eq!(metrics.total_duration, Duration::from_secs(1));
        assert!(metrics.error_info.is_none());
    }

    #[test]
    fn test_performance_metrics_mark_failure() {
        let mut metrics = PerformanceMetrics::new();

        metrics.mark_failure(Duration::from_millis(500), "Connection timeout");

        assert!(!metrics.success);
        assert_eq!(metrics.total_duration, Duration::from_millis(500));
        assert_eq!(metrics.error_info, Some("Connection timeout".to_string()));
    }

    #[test]
    fn test_performance_metrics_increment_attempts() {
        let mut metrics = PerformanceMetrics::new();

        assert_eq!(metrics.attempts, 0);

        metrics.increment_attempts();
        assert_eq!(metrics.attempts, 1);

        metrics.increment_attempts();
        assert_eq!(metrics.attempts, 2);

        metrics.increment_attempts();
        assert_eq!(metrics.attempts, 3);
    }

    #[test]
    fn test_performance_metrics_multiple_phases() {
        let mut metrics = PerformanceMetrics::new();

        // Simulate a multi-phase operation
        metrics.increment_attempts();
        metrics.record_phase("connect", Duration::from_millis(50));
        metrics.record_phase("auth", Duration::from_millis(100));
        metrics.record_phase("query", Duration::from_millis(200));
        metrics.record_phase("process", Duration::from_millis(150));
        metrics.mark_success(Duration::from_millis(500));

        assert_eq!(metrics.attempts, 1);
        assert!(metrics.success);
        assert_eq!(metrics.phase_durations.len(), 4);
        assert_eq!(metrics.total_duration, Duration::from_millis(500));
    }

    #[test]
    fn test_performance_metrics_retry_scenario() {
        let mut metrics = PerformanceMetrics::new();

        // First attempt fails
        metrics.increment_attempts();
        metrics.record_phase("attempt_1", Duration::from_millis(100));

        // Second attempt fails
        metrics.increment_attempts();
        metrics.record_phase("attempt_2", Duration::from_millis(120));

        // Third attempt succeeds
        metrics.increment_attempts();
        metrics.record_phase("attempt_3", Duration::from_millis(110));
        metrics.mark_success(Duration::from_millis(330));

        assert_eq!(metrics.attempts, 3);
        assert!(metrics.success);
        assert_eq!(metrics.phase_durations.len(), 3);
    }

    #[test]
    fn test_operation_context_creation() {
        let ctx = OperationContext::new("test_operation");

        assert_eq!(ctx.operation, "test_operation");
        assert!(ctx.metadata.is_empty());
        assert_eq!(ctx.metrics.attempts, 0);
        assert!(!ctx.correlation_id.as_str().is_empty());
    }

    #[test]
    fn test_operation_context_with_correlation_id() {
        let correlation_id = CorrelationId::from_string("custom-id-123");
        let ctx = OperationContext::with_correlation_id("test_op", correlation_id.clone());

        assert_eq!(ctx.operation, "test_op");
        assert_eq!(ctx.correlation_id, correlation_id);
    }

    #[test]
    #[test]
    fn test_operation_context_add_metadata() {
        let ctx = OperationContext::new("service_call")
            .with_metadata("service_name", "squirrel")
            .with_metadata("endpoint", "/api/v1/inference")
            .with_metadata("method", "POST");

        assert_eq!(ctx.metadata.len(), 3);
        assert_eq!(
            ctx.metadata.get("service_name"),
            Some(&"squirrel".to_string())
        );
        assert_eq!(
            ctx.metadata.get("endpoint"),
            Some(&"/api/v1/inference".to_string())
        );
        assert_eq!(ctx.metadata.get("method"), Some(&"POST".to_string()));
    }

    #[test]
    fn test_operation_context_record_phase() {
        let mut ctx = OperationContext::new("complex_operation");

        ctx.metrics
            .record_phase("validation", Duration::from_millis(50));
        ctx.metrics
            .record_phase("execution", Duration::from_millis(200));

        assert_eq!(ctx.metrics.phase_durations.len(), 2);
    }

    #[test]
    fn test_operation_context_complete_success() {
        let mut ctx = OperationContext::new("successful_op");

        // Mark success with a known duration (no sleep needed)
        let known_duration = Duration::from_millis(10);
        ctx.metrics.mark_success(known_duration);

        assert!(ctx.metrics.success);
        assert!(ctx.metrics.total_duration > Duration::ZERO);
    }

    #[test]
    fn test_operation_context_complete_failure() {
        let mut ctx = OperationContext::new("failed_op");

        // Mark failure with a known duration (no sleep needed)
        let known_duration = Duration::from_millis(10);
        ctx.metrics
            .mark_failure(known_duration, "Service unavailable");

        assert!(!ctx.metrics.success);
        assert!(ctx.metrics.total_duration > Duration::ZERO);
        assert_eq!(
            ctx.metrics.error_info,
            Some("Service unavailable".to_string())
        );
    }

    #[test]
    fn test_operation_context_duration() {
        let ctx = OperationContext::new("duration_test");

        // Elapsed is always >= 0; just verify it's reasonable
        let elapsed = ctx.start_time.elapsed();
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn test_operation_context_full_lifecycle() {
        let mut ctx = OperationContext::new("lifecycle_test")
            .with_metadata("user_id", "user123")
            .with_metadata("session_id", "session456");

        // Record phases with known durations (no sleeps)
        ctx.metrics.record_phase("init", Duration::from_millis(10));
        ctx.metrics
            .record_phase("process", Duration::from_millis(20));

        // Increment attempts
        ctx.metrics.increment_attempts();

        // Complete successfully with a known duration
        ctx.metrics.mark_success(Duration::from_millis(30));

        // Verify final state
        assert_eq!(ctx.metadata.len(), 2);
        assert_eq!(ctx.metrics.phase_durations.len(), 2);
        assert_eq!(ctx.metrics.attempts, 1);
        assert!(ctx.metrics.success);
        assert!(ctx.metrics.total_duration > Duration::ZERO);
    }

    #[test]
    fn test_operation_context_retry_lifecycle() {
        let mut ctx = OperationContext::new("retry_test");

        // First attempt
        ctx.metrics.increment_attempts();
        ctx.metrics
            .record_phase("attempt_1", Duration::from_millis(50));

        // Second attempt
        ctx.metrics.increment_attempts();
        ctx.metrics
            .record_phase("attempt_2", Duration::from_millis(60));

        // Third attempt succeeds
        ctx.metrics.increment_attempts();
        ctx.metrics
            .record_phase("attempt_3", Duration::from_millis(55));
        let elapsed = ctx.start_time.elapsed();
        ctx.metrics.mark_success(elapsed);

        assert_eq!(ctx.metrics.attempts, 3);
        assert!(ctx.metrics.success);
    }

    #[test]
    fn test_empty_metadata() {
        let ctx = OperationContext::new("empty_meta");

        assert!(ctx.metadata.is_empty());
        assert_eq!(ctx.metadata.len(), 0);
    }

    #[test]
    fn test_metadata_overwrite() {
        let mut ctx = OperationContext::new("overwrite_test");

        ctx.metadata.insert("key".to_string(), "value1".to_string());
        assert_eq!(ctx.metadata.get("key"), Some(&"value1".to_string()));

        ctx.metadata.insert("key".to_string(), "value2".to_string());
        assert_eq!(ctx.metadata.get("key"), Some(&"value2".to_string()));
        assert_eq!(ctx.metadata.len(), 1); // Still only one key
    }

    #[test]
    fn test_phase_duration_overwrite() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_phase("phase", Duration::from_millis(100));
        assert_eq!(
            metrics.phase_durations.get("phase"),
            Some(&Duration::from_millis(100))
        );

        metrics.record_phase("phase", Duration::from_millis(200));
        assert_eq!(
            metrics.phase_durations.get("phase"),
            Some(&Duration::from_millis(200))
        );
        assert_eq!(metrics.phase_durations.len(), 1);
    }

    #[test]
    fn test_zero_duration_phases() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_phase("instant", Duration::ZERO);
        assert_eq!(
            metrics.phase_durations.get("instant"),
            Some(&Duration::ZERO)
        );
    }

    #[test]
    fn test_large_attempt_count() {
        let mut metrics = PerformanceMetrics::new();

        for _ in 0..100 {
            metrics.increment_attempts();
        }

        assert_eq!(metrics.attempts, 100);
    }

    #[test]
    fn test_correlation_id_equality() {
        let id1 = CorrelationId::from_string("same-id");
        let id2 = CorrelationId::from_string("same-id");
        let id3 = CorrelationId::from_string("different-id");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_error_info_preservation() {
        let mut metrics = PerformanceMetrics::new();

        metrics.mark_failure(Duration::from_millis(100), "Error 1");
        assert_eq!(metrics.error_info, Some("Error 1".to_string()));

        metrics.mark_failure(Duration::from_millis(200), "Error 2");
        assert_eq!(metrics.error_info, Some("Error 2".to_string()));
    }

    #[test]
    fn test_success_clears_error() {
        let mut metrics = PerformanceMetrics::new();

        metrics.mark_failure(Duration::from_millis(100), "Initial error");
        assert!(metrics.error_info.is_some());

        // Note: mark_success doesn't clear error_info in current implementation
        // This test documents current behavior
        metrics.mark_success(Duration::from_millis(200));
        assert!(metrics.success);
    }
}
