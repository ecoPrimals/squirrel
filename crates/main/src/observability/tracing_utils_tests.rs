//! Comprehensive tests for tracing utilities
//!
//! Tests universal tracing coordinator, endpoint discovery, and distributed tracing capabilities.

#[cfg(test)]
mod tests {
    use super::super::tracing_utils::*;
    use std::collections::HashMap;

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        
        assert!(config.enable_distributed_tracing);
        assert_eq!(config.max_span_duration.as_secs(), 300);
        assert_eq!(config.trace_sampling_ratio, 1.0);
        assert!(config.custom_attributes.is_empty());
    }

    #[test]
    fn test_tracing_config_custom() {
        let mut custom_attrs = HashMap::new();
        custom_attrs.insert("env".to_string(), "production".to_string());
        
        let config = TracingConfig {
            enable_distributed_tracing: false,
            max_span_duration: std::time::Duration::from_secs(600),
            custom_attributes: custom_attrs.clone(),
            trace_sampling_ratio: 0.5,
        };

        assert!(!config.enable_distributed_tracing);
        assert_eq!(config.max_span_duration.as_secs(), 600);
        assert_eq!(config.trace_sampling_ratio, 0.5);
        assert_eq!(config.custom_attributes.len(), 1);
        assert_eq!(config.custom_attributes.get("env").unwrap(), "production");
    }

    #[test]
    fn test_traced_operation_creation() {
        let operation = TracedOperation {
            operation_id: uuid::Uuid::new_v4(),
            operation_name: "test_op".to_string(),
            start_time: chrono::Utc::now(),
            attributes: HashMap::new(),
            source_primal: Some("squirrel".to_string()),
            related_operations: Vec::new(),
        };

        assert_eq!(operation.operation_name, "test_op");
        assert_eq!(operation.source_primal.unwrap(), "squirrel");
        assert!(operation.attributes.is_empty());
        assert!(operation.related_operations.is_empty());
    }

    #[test]
    fn test_traced_operation_with_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert("user_id".to_string(), "12345".to_string());
        attrs.insert("request_type".to_string(), "query".to_string());

        let operation = TracedOperation {
            operation_id: uuid::Uuid::new_v4(),
            operation_name: "api_call".to_string(),
            start_time: chrono::Utc::now(),
            attributes: attrs.clone(),
            source_primal: Some("squirrel".to_string()),
            related_operations: Vec::new(),
        };

        assert_eq!(operation.attributes.len(), 2);
        assert_eq!(operation.attributes.get("user_id").unwrap(), "12345");
        assert_eq!(operation.attributes.get("request_type").unwrap(), "query");
    }

    #[test]
    fn test_tracing_endpoint_creation() {
        let endpoint = TracingEndpoint {
            primal_type: "test_primal".to_string(),
            endpoint: "http://localhost:8080/tracing".to_string(),
            capabilities: vec![
                TracingCapability::SpanCollection,
                TracingCapability::EventStreaming,
            ],
            discovered_at: chrono::Utc::now(),
        };

        assert_eq!(endpoint.primal_type, "test_primal");
        assert_eq!(endpoint.endpoint, "http://localhost:8080/tracing");
        assert_eq!(endpoint.capabilities.len(), 2);
    }

    #[test]
    fn test_tracing_capability_variants() {
        let _span_collection = TracingCapability::SpanCollection;
        let _event_streaming = TracingCapability::EventStreaming;
        let _metrics_integration = TracingCapability::MetricsIntegration;
        let _custom_attributes = TracingCapability::CustomAttributes;
        let _distributed_tracing = TracingCapability::DistributedTracing;
    }

    #[tokio::test]
    async fn test_coordinator_creation() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let active_ops = coordinator.get_active_operations().await;
        assert!(active_ops.is_empty());
    }

    #[tokio::test]
    async fn test_coordinator_with_custom_config() {
        let config = TracingConfig {
            enable_distributed_tracing: false,
            max_span_duration: std::time::Duration::from_secs(60),
            custom_attributes: HashMap::new(),
            trace_sampling_ratio: 0.1,
        };
        
        let _coordinator = UniversalTracingCoordinator::new(config.clone());
        
        // Verify coordinator created successfully
        assert_eq!(config.trace_sampling_ratio, 0.1);
        assert!(!config.enable_distributed_tracing);
    }

    #[tokio::test]
    async fn test_discover_tracing_endpoints() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let result = coordinator.discover_tracing_endpoints().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_start_traced_operation() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let mut attributes = HashMap::new();
        attributes.insert("test_attr".to_string(), "test_value".to_string());

        let operation_id = coordinator
            .start_traced_operation("test_operation".to_string(), attributes)
            .await
            .unwrap();

        // Verify operation is tracked
        let active_ops = coordinator.get_active_operations().await;
        assert_eq!(active_ops.len(), 1);
        assert_eq!(active_ops[0].operation_id, operation_id);
        assert_eq!(active_ops[0].operation_name, "test_operation");
    }

    #[tokio::test]
    async fn test_start_multiple_traced_operations() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let id1 = coordinator
            .start_traced_operation("op1".to_string(), HashMap::new())
            .await
            .unwrap();
        let id2 = coordinator
            .start_traced_operation("op2".to_string(), HashMap::new())
            .await
            .unwrap();
        let id3 = coordinator
            .start_traced_operation("op3".to_string(), HashMap::new())
            .await
            .unwrap();

        let active_ops = coordinator.get_active_operations().await;
        assert_eq!(active_ops.len(), 3);

        // Verify all IDs are unique
        let ids: Vec<uuid::Uuid> = active_ops.iter().map(|op| op.operation_id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
        assert!(ids.contains(&id3));
    }

    #[tokio::test]
    async fn test_finish_traced_operation() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let operation_id = coordinator
            .start_traced_operation("test_op".to_string(), HashMap::new())
            .await
            .unwrap();

        // Wait a bit to ensure duration is measurable
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let result = coordinator.finish_traced_operation(operation_id).await;
        assert!(result.is_ok());

        let finished_op = result.unwrap();
        assert_eq!(finished_op.operation_name, "test_op");
        assert!(finished_op.attributes.contains_key("duration_ms"));

        // Verify operation is no longer active
        let active_ops = coordinator.get_active_operations().await;
        assert!(active_ops.is_empty());
    }

    #[tokio::test]
    async fn test_finish_nonexistent_operation() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let fake_id = uuid::Uuid::new_v4();
        let result = coordinator.finish_traced_operation(fake_id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_operation_attributes_preserved() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let mut attrs = HashMap::new();
        attrs.insert("user_id".to_string(), "user123".to_string());
        attrs.insert("action".to_string(), "login".to_string());

        let operation_id = coordinator
            .start_traced_operation("user_action".to_string(), attrs.clone())
            .await
            .unwrap();

        let active_ops = coordinator.get_active_operations().await;
        let operation = &active_ops[0];

        assert_eq!(operation.attributes.get("user_id").unwrap(), "user123");
        assert_eq!(operation.attributes.get("action").unwrap(), "login");
    }

    #[tokio::test]
    async fn test_concurrent_traced_operations() {
        let config = TracingConfig::default();
        let coordinator = std::sync::Arc::new(UniversalTracingCoordinator::new(config));

        let mut handles = vec![];

        for i in 0..20 {
            let coord = coordinator.clone();
            let handle = tokio::spawn(async move {
                let op_name = format!("operation_{}", i);
                coord
                    .start_traced_operation(op_name, HashMap::new())
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let active_ops = coordinator.get_active_operations().await;
        assert_eq!(active_ops.len(), 20);
    }

    #[tokio::test]
    async fn test_operation_lifecycle() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        // Start operation
        let operation_id = coordinator
            .start_traced_operation("lifecycle_test".to_string(), HashMap::new())
            .await
            .unwrap();

        // Verify it's active
        assert_eq!(coordinator.get_active_operations().await.len(), 1);

        // Finish operation
        let finished = coordinator.finish_traced_operation(operation_id).await.unwrap();

        // Verify it's no longer active
        assert_eq!(coordinator.get_active_operations().await.len(), 0);

        // Verify duration was recorded
        assert!(finished.attributes.contains_key("duration_ms"));
    }

    #[tokio::test]
    async fn test_operation_duration_accuracy() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let operation_id = coordinator
            .start_traced_operation("duration_test".to_string(), HashMap::new())
            .await
            .unwrap();

        // Wait for a known duration
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let finished = coordinator.finish_traced_operation(operation_id).await.unwrap();

        let duration_str = finished.attributes.get("duration_ms").unwrap();
        let duration_ms: i64 = duration_str.parse().unwrap();

        // Should be at least 100ms, but give some margin for execution time
        assert!(duration_ms >= 100);
        assert!(duration_ms < 200); // Should not be way off
    }

    #[tokio::test]
    async fn test_get_active_operations_empty() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let active_ops = coordinator.get_active_operations().await;
        assert!(active_ops.is_empty());
    }

    #[tokio::test]
    async fn test_operation_with_source_primal() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let _operation_id = coordinator
            .start_traced_operation("test_op".to_string(), HashMap::new())
            .await
            .unwrap();

        let active_ops = coordinator.get_active_operations().await;
        let operation = &active_ops[0];

        assert_eq!(operation.source_primal.as_ref().unwrap(), "squirrel");
    }

    #[tokio::test]
    async fn test_tracing_sampling_ratio() {
        let config = TracingConfig {
            enable_distributed_tracing: true,
            max_span_duration: std::time::Duration::from_secs(300),
            custom_attributes: HashMap::new(),
            trace_sampling_ratio: 0.5,
        };

        assert_eq!(config.trace_sampling_ratio, 0.5);
        let _coordinator = UniversalTracingCoordinator::new(config);
    }

    #[tokio::test]
    async fn test_distributed_tracing_enabled() {
        let config = TracingConfig::default();
        assert!(config.enable_distributed_tracing);

        let _coordinator = UniversalTracingCoordinator::new(config);
    }

    #[tokio::test]
    async fn test_distributed_tracing_disabled() {
        let config = TracingConfig {
            enable_distributed_tracing: false,
            max_span_duration: std::time::Duration::from_secs(300),
            custom_attributes: HashMap::new(),
            trace_sampling_ratio: 1.0,
        };

        assert!(!config.enable_distributed_tracing);
        let _coordinator = UniversalTracingCoordinator::new(config);
    }

    #[tokio::test]
    async fn test_max_span_duration_configuration() {
        let config = TracingConfig {
            enable_distributed_tracing: true,
            max_span_duration: std::time::Duration::from_secs(120),
            custom_attributes: HashMap::new(),
            trace_sampling_ratio: 1.0,
        };

        assert_eq!(config.max_span_duration.as_secs(), 120);
        let _coordinator = UniversalTracingCoordinator::new(config);
    }

    #[tokio::test]
    async fn test_custom_attributes_in_config() {
        let mut custom_attrs = HashMap::new();
        custom_attrs.insert("region".to_string(), "us-west-2".to_string());
        custom_attrs.insert("env".to_string(), "staging".to_string());

        let config = TracingConfig {
            enable_distributed_tracing: true,
            max_span_duration: std::time::Duration::from_secs(300),
            custom_attributes: custom_attrs.clone(),
            trace_sampling_ratio: 1.0,
        };

        assert_eq!(config.custom_attributes.len(), 2);
        assert_eq!(config.custom_attributes.get("region").unwrap(), "us-west-2");
        let _coordinator = UniversalTracingCoordinator::new(config);
    }

    #[tokio::test]
    async fn test_operation_with_empty_attributes() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let _operation_id = coordinator
            .start_traced_operation("empty_attrs".to_string(), HashMap::new())
            .await
            .unwrap();

        let active_ops = coordinator.get_active_operations().await;
        let operation = &active_ops[0];

        assert!(operation.attributes.is_empty());
    }

    #[tokio::test]
    async fn test_operation_with_many_attributes() {
        let config = TracingConfig::default();
        let coordinator = UniversalTracingCoordinator::new(config);

        let mut attrs = HashMap::new();
        for i in 0..50 {
            attrs.insert(format!("attr_{}", i), format!("value_{}", i));
        }

        let _operation_id = coordinator
            .start_traced_operation("many_attrs".to_string(), attrs)
            .await
            .unwrap();

        let active_ops = coordinator.get_active_operations().await;
        let operation = &active_ops[0];

        assert_eq!(operation.attributes.len(), 50);
    }

    #[tokio::test]
    async fn test_concurrent_finish_operations() {
        let config = TracingConfig::default();
        let coordinator = std::sync::Arc::new(UniversalTracingCoordinator::new(config));

        // Start multiple operations
        let mut ids = vec![];
        for i in 0..10 {
            let id = coordinator
                .start_traced_operation(format!("op_{}", i), HashMap::new())
                .await
                .unwrap();
            ids.push(id);
        }

        // Finish them concurrently
        let mut handles = vec![];
        for id in ids {
            let coord = coordinator.clone();
            let handle = tokio::spawn(async move {
                coord.finish_traced_operation(id).await.unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // All should be finished
        let active_ops = coordinator.get_active_operations().await;
        assert!(active_ops.is_empty());
    }
}

