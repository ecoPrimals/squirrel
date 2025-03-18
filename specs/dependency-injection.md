# Dependency Injection in the Monitoring System

## Overview

This guide explains how to use dependency injection (DI) in the monitoring system. The system has been refactored to use DI instead of global singletons, which improves testability, maintainability, and flexibility.

## Components Supporting DI

The following components have been converted to use dependency injection:

1. Protocol Metrics Collector
2. Performance Collector
3. Health Checker
4. Resource Metrics Collector
5. Alert Manager
6. Network Monitor

## Using Components with DI

### Direct Usage with Dependencies

When creating a new component directly with dependencies:

```rust
// Create dependencies
let config = MetricConfig::default();
let storage = Arc::new(RedisMetricStorage::new());

// Create component with dependencies
let collector = MetricCollector::with_dependencies(
    config,
    storage,
);

// Use the component
collector.record_metric("request_count", 42.0).await?;
```

### Using Factory Pattern

For more complex component creation:

```rust
// Create factory with configuration
let factory = MetricCollectorFactory::new(config);

// Create component with dependencies
let collector = factory.create_collector_with_dependencies(
    storage,
);

// Use the component
collector.record_metric("request_count", 42.0).await?;
```

### Using Adapters

For backward compatibility or when exact dependencies are unknown:

```rust
// Create adapter
let collector = create_collector_adapter();

// Use the adapter (will initialize dependencies on demand)
collector.record_metric("request_count", 42.0).await?;
```

## Creating New Components with DI

When creating new components that need monitoring capabilities:

```rust
pub struct MyService {
    metric_collector: Arc<MetricCollectorAdapter>,
    alert_manager: Arc<AlertManagerAdapter>,
}

impl MyService {
    pub fn new() -> Self {
        Self {
            metric_collector: create_collector_adapter(),
            alert_manager: create_manager_adapter(),
        }
    }

    pub fn with_dependencies(
        metric_collector: Arc<MetricCollectorAdapter>,
        alert_manager: Arc<AlertManagerAdapter>,
    ) -> Self {
        Self {
            metric_collector,
            alert_manager,
        }
    }
}
```

## Testing with DI

The DI pattern makes testing much easier:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_service_records_metrics() {
        // Create mock collector
        let mut mock_collector = MockMetricCollector::new();
        mock_collector
            .expect_record_metric()
            .with(eq("test_metric"), eq(42.0))
            .times(1)
            .returning(|_, _| Ok(()));

        // Create service with mock
        let service = MyService::with_dependencies(
            Arc::new(mock_collector),
            create_manager_adapter(),
        );

        // Test the service
        service.do_something().await?;
    }
}
```

## Best Practices

1. **Always Use DI for New Code**
   - Prefer `with_dependencies` constructors
   - Make dependencies explicit
   - Use appropriate interfaces

2. **Adapter Pattern for Legacy Code**
   - Use adapters when converting existing code
   - Maintain backward compatibility
   - Plan for eventual removal of singletons

3. **Testing**
   - Use mock implementations for testing
   - Test with different dependency configurations
   - Include integration tests with real dependencies

4. **Error Handling**
   - Handle initialization errors properly
   - Provide clear error messages
   - Consider fallback behavior

5. **Configuration**
   - Pass configuration explicitly
   - Use builder pattern for complex configs
   - Document configuration options

## Component-Specific Guidelines

### Metric Collector

```rust
// Create with direct dependencies
let collector = MetricCollector::with_dependencies(
    config,
    storage,
    alert_manager,
);

// Or use adapter
let collector = create_collector_adapter();
```

### Alert Manager

```rust
// Create with direct dependencies
let manager = AlertManager::with_dependencies(
    config,
    notification_manager,
);

// Or use adapter
let manager = create_manager_adapter();
```

### Health Checker

```rust
// Create with direct dependencies
let checker = HealthChecker::with_dependencies(
    config,
    storage,
);

// Or use adapter
let checker = create_checker_adapter();
```

### Network Monitor

```rust
// Create with direct dependencies
let monitor = NetworkMonitor::with_dependencies(
    config,
);

// Or use adapter
let monitor = create_monitor_adapter();
```

## Migration Guide

When migrating existing code to use DI:

1. **Identify Dependencies**
   - List all component dependencies
   - Determine configuration needs
   - Plan migration strategy

2. **Create Adapter**
   - Implement adapter pattern
   - Maintain backward compatibility
   - Add proper error handling

3. **Update Factory**
   - Add dependency injection support
   - Update creation methods
   - Document changes

4. **Update Tests**
   - Convert to use mocks
   - Add integration tests
   - Verify backward compatibility

5. **Update Consumers**
   - Switch to adapter pattern
   - Add explicit dependencies
   - Update documentation

## Troubleshooting

Common issues and solutions:

1. **Circular Dependencies**
   - Use interfaces to break cycles
   - Consider restructuring components
   - Use event-based communication

2. **Initialization Order**
   - Use builder pattern
   - Implement proper error handling
   - Consider lazy initialization

3. **Testing Issues**
   - Use appropriate mocks
   - Test different configurations
   - Include integration tests

## Performance Considerations

The DI pattern has minimal performance impact:

- Arc cloning is cheap
- Adapters add negligible overhead
- Initialization is typically one-time cost

## Future Improvements

Planned enhancements to the DI system:

1. Remove remaining singletons
2. Add more test utilities
3. Improve error handling
4. Add performance monitoring
5. Enhance documentation

## References

- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [Testing Best Practices](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Dependency Injection in Rust](https://rust-unofficial.github.io/patterns/patterns/creational/dependency_injection.html) 