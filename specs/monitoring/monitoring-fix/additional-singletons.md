---
version: 1.0.0
last_updated: 2025-03-16
status: draft
priority: high
---

# Additional Singleton Patterns to Refactor

## Overview

While working on the monitoring service refactoring, we've identified several other components in the codebase that use similar global singleton patterns. These should be refactored following the same dependency injection approach to improve testability, flexibility, and resource management.

## Identified Singletons

The following global singleton patterns have been identified in the codebase:

### Alert Manager
```rust
// crates/core/src/monitoring/alerts/mod.rs
static ALERT_MANAGER: tokio::sync::OnceCell<Arc<DefaultAlertManager>> = tokio::sync::OnceCell::const_new();

pub async fn initialize(config: Option<AlertConfig>) -> Result<()> {
    // Implementation...
}

pub fn get_manager() -> Option<Arc<DefaultAlertManager>> {
    ALERT_MANAGER.get().cloned()
}
```

### Dashboard Manager
```rust
// crates/core/src/monitoring/dashboard/mod.rs
static INSTANCE: OnceCell<Arc<DashboardManager>> = OnceCell::const_new();
static DASHBOARD_MANAGER: OnceCell<Arc<DashboardManager>> = OnceCell::const_new();
```

### Metrics Collectors
Several metrics collectors use the singleton pattern:

1. **Tool Metrics Collector**
```rust
// crates/core/src/monitoring/metrics/tool.rs
static TOOL_COLLECTOR: tokio::sync::OnceCell<Arc<ToolMetricsCollector>> = tokio::sync::OnceCell::const_new();
```

2. **Protocol Metrics Collector**
```rust
// crates/core/src/monitoring/metrics/protocol.rs
static PROTOCOL_COLLECTOR: tokio::sync::OnceCell<Arc<ProtocolMetricsCollector>> = tokio::sync::OnceCell::const_new();
```

3. **Performance Collector**
```rust
// crates/core/src/monitoring/metrics/performance.rs
static PERFORMANCE_COLLECTOR: tokio::sync::OnceCell<Arc<PerformanceCollector>> = tokio::sync::OnceCell::const_new();
```

4. **Resource Metrics Collector**
```rust
// crates/core/src/monitoring/metrics/resource.rs
static RESOURCE_COLLECTOR: tokio::sync::OnceCell<Arc<ResourceMetricsCollector>> = tokio::sync::OnceCell::const_new();
```

### Notification Manager
```rust
// crates/core/src/monitoring/alerts/notify.rs
static NOTIFICATION_MANAGER: tokio::sync::OnceCell<Arc<NotificationManager>> = tokio::sync::OnceCell::const_new();
```

### Network Monitoring
```rust
// crates/core/src/monitoring/network/mod.rs
lazy_static::lazy_static! {
    static ref SYSTEM: Arc<RwLock<s>> = Arc::new(RwLock::new(System::new_all()));
}
```

### Metric Exporter
```rust
// crates/core/src/monitoring/metrics/export.rs
static EXPORTER: tokio::sync::OnceCell<Arc<RwLock<Arc<dyn MetricExporter + Send + Sync>>>> = tokio::sync::OnceCell::const_new();
```

## Refactoring Priority

We should prioritize refactoring these components based on:

1. **Test Impact**: Components most affected by testing issues
2. **Dependency Chain**: Components that others depend on
3. **Complexity**: Simpler components first to establish patterns

Suggested refactoring order:

1. MonitoringService (already planned)
2. AlertManager (closely tied to monitoring)
3. Metrics Collectors (used by tests)
4. Network Monitoring (has test dependencies)
5. Dashboard Manager
6. Notification Manager
7. Metric Exporter

## Implementation Strategy

For each component, we'll follow the same refactoring pattern detailed in the main refactoring plan:

1. Create a Factory class that manages creation
2. Update component to accept dependencies
3. Modify consumers to use dependency injection
4. Update tests to use isolated instances

For example, the AlertManager refactoring would look like:

```rust
// Before
static ALERT_MANAGER: tokio::sync::OnceCell<Arc<DefaultAlertManager>> = tokio::sync::OnceCell::const_new();

pub fn get_manager() -> Option<Arc<DefaultAlertManager>> {
    ALERT_MANAGER.get().cloned()
}

// After
pub struct AlertManagerFactory {
    default_config: AlertConfig,
}

impl AlertManagerFactory {
    pub fn new(config: AlertConfig) -> Self {
        Self {
            default_config: config
        }
    }
    
    pub fn create_manager(&self) -> Arc<dyn AlertManager + Send + Sync> {
        Arc::new(DefaultAlertManager::new(self.default_config.clone()))
    }
}
```

## Integration with Main Refactoring

This broader refactoring effort should be coordinated with the monitoring service refactoring:

1. Establish common patterns in the MonitoringService refactoring
2. Apply those patterns consistently to other components
3. Update all affected tests to follow the same approach
4. Ensure backwards compatibility during the transition

## Timeline Impact

Adding these additional components to the refactoring plan will impact the original timeline:

| Phase | Original Estimate | New Estimate |
|-------|------------------|-------------|
| Preparation & Analysis | 3 hours | 4 hours |
| Core Refactoring | 4 hours | 6 hours |
| Component Migration | 6 hours | 12 hours |
| Validation & Cleanup | 3 hours | 5 hours |
| **Total** | **16 hours** | **27 hours** |

## Risk Assessment

Additional risks with the expanded scope:

1. **Interdependencies**: Components might have complex relationships
2. **Test Coverage**: Some components might lack adequate tests
3. **Backward Compatibility**: More components means more compatibility concerns
4. **Project Scope**: Expanded scope might delay other priorities

## Conclusion

While refactoring the MonitoringService is a good start, addressing all identified singleton patterns will provide greater benefits for testing and maintenance. We recommend following the established DI pattern from the initial refactoring and gradually applying it to all components.

This more comprehensive approach will take longer but will result in a more consistent and maintainable codebase without the test issues caused by global singletons. 