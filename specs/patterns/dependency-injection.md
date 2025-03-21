---
description: Standard Dependency Injection pattern for the Squirrel codebase
version: 1.0.0
last_updated: 2024-03-21
status: active
---

# Dependency Injection Pattern

## Context

The Dependency Injection (DI) pattern is used throughout the Squirrel codebase to manage component dependencies. This pattern improves testability, maintainability, and flexibility by making dependencies explicit rather than using global state or singletons.

This pattern should be used when:
- A component depends on other components or services
- You need to support multiple implementations of a dependency
- You want to improve testability with mocks
- You want to make component dependencies explicit
- You need to support different configurations in different environments

## Implementation

### Interface-Based Approach

```rust
// 1. Define trait for the dependency
pub trait MetricCollector: Send + Sync + 'static {
    async fn record_metric(&self, name: &str, value: f64) -> Result<(), MetricError>;
    async fn get_metric(&self, name: &str) -> Result<f64, MetricError>;
}

// 2. Create concrete implementations
pub struct PrometheusCollector {
    client: Arc<PrometheusClient>,
    config: MetricConfig,
}

impl MetricCollector for PrometheusCollector {
    async fn record_metric(&self, name: &str, value: f64) -> Result<(), MetricError> {
        // Implementation
    }
    
    async fn get_metric(&self, name: &str) -> Result<f64, MetricError> {
        // Implementation
    }
}

// 3. Component that uses the dependency
pub struct MyService {
    collector: Arc<dyn MetricCollector>,
}

impl MyService {
    // 4. Constructor with explicit dependencies
    pub fn new(collector: Arc<dyn MetricCollector>) -> Self {
        Self { collector }
    }
    
    pub async fn process_data(&self, data: &Data) -> Result<(), ServiceError> {
        // Use the dependency
        self.collector.record_metric("process_count", 1.0).await?;
        // Process data
        Ok(())
    }
}
```

### Factory Pattern

For complex dependency graphs:

```rust
// Factory for creating services with their dependencies
pub struct ServiceFactory {
    metric_collector: Arc<dyn MetricCollector>,
    alert_manager: Arc<dyn AlertManager>,
    config_provider: Arc<dyn ConfigProvider>,
}

impl ServiceFactory {
    pub fn new(
        metric_collector: Arc<dyn MetricCollector>,
        alert_manager: Arc<dyn AlertManager>,
        config_provider: Arc<dyn ConfigProvider>,
    ) -> Self {
        Self {
            metric_collector,
            alert_manager,
            config_provider,
        }
    }
    
    pub fn create_data_service(&self) -> DataService {
        DataService::new(
            self.metric_collector.clone(),
            self.config_provider.clone(),
        )
    }
    
    pub fn create_processing_service(&self) -> ProcessingService {
        ProcessingService::new(
            self.metric_collector.clone(),
            self.alert_manager.clone(),
        )
    }
}
```

### Container Approach

For more complex scenarios:

```rust
pub struct Container {
    services: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn register<T: Send + Sync + 'static>(&self, service: T) {
        let mut services = self.services.write().unwrap();
        services.insert(TypeId::of::<T>(), Box::new(service));
    }
    
    pub fn resolve<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let services = self.services.read().unwrap();
        services.get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .map(|service| Arc::new(service.clone()))
    }
}
```

## Benefits

- **Improved Testability**: Dependencies can be easily mocked for unit testing
- **Flexibility**: Different implementations can be easily swapped
- **Explicit Dependencies**: Component dependencies are clear from the constructor
- **Separation of Concerns**: Each component focuses on its core responsibilities
- **Lifecycle Management**: Dependencies can be properly initialized and cleaned up
- **Runtime Configuration**: Components can be configured differently based on runtime needs

## Tradeoffs

- **Increased Complexity**: More boilerplate code compared to singletons or static state
- **Potential Performance Impact**: Dynamic dispatch can have a small performance cost
- **Learning Curve**: New developers need to understand the DI approach
- **Constructor Overhead**: Can lead to large constructors with many parameters
- **Dependency Management**: Need to manage dependency lifecycles carefully

## When to Use

- When building components that depend on external services
- When unit testing is a priority
- When multiple implementations of a dependency might be needed
- When dependencies may change based on configuration
- When converting from global state to more testable architecture
- For all new code in the Squirrel codebase

## When to Avoid

- For very simple utilities with no external dependencies
- When performance is absolutely critical (though impact is usually minimal)
- For quick prototyping (though consider refactoring later)

## Related Patterns

- [Factory Pattern](./factory-pattern.md)
- [Adapter Pattern](./adapter-pattern.md)
- [Builder Pattern](./builder-pattern.md)

## Examples in Codebase

- `crates/monitoring/src/metric_collector.rs`: MetricCollector implementation
- `crates/mcp/src/protocol_handler.rs`: Protocol handler with injected dependencies
- `crates/app/src/service_factory.rs`: ServiceFactory implementation

## Testing Approach

DI makes testing straightforward by allowing dependencies to be mocked:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    
    mock! {
        MetricCollector {}
        impl MetricCollector for MockMetricCollector {
            async fn record_metric(&self, name: &str, value: f64) -> Result<(), MetricError>;
            async fn get_metric(&self, name: &str) -> Result<f64, MetricError>;
        }
    }
    
    #[tokio::test]
    async fn test_process_data_records_metric() {
        // Arrange
        let mut mock_collector = MockMetricCollector::new();
        mock_collector
            .expect_record_metric()
            .with(eq("process_count"), eq(1.0))
            .times(1)
            .returning(|_, _| Ok(()));
            
        let service = MyService::new(Arc::new(mock_collector));
        let data = Data::new();
        
        // Act
        let result = service.process_data(&data).await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

## Security Considerations

- Ensure dependencies with sensitive data are properly protected
- Consider using capability-based security for dependencies that require special access
- Validate dependencies at construction time to prevent injection of malicious implementations

## Performance Characteristics

- Time complexity: O(1) for dependency resolution
- Space complexity: O(n) where n is the number of dependencies
- Memory usage: Low (mostly Arc references)
- CPU usage: Low (minimal overhead from dynamic dispatch)

## Migration Guide

When migrating from global state or singletons:

1. Identify all dependencies and create interfaces for them
2. Implement concrete versions of each dependency
3. Update components to accept dependencies in constructors
4. Replace direct singleton access with dependency calls
5. Update tests to use mocks instead of global state

## Version History

- 1.0.0 (2024-03-21): Initial version based on existing implementation 