# Singleton to Dependency Injection Conversion

## Context
The monitoring system currently relies heavily on global singletons for various components. This design creates tight coupling, makes testing difficult, and can lead to hidden dependencies. We are undertaking a systematic conversion to dependency injection to improve maintainability, testability, and system clarity.

## Current Status
Last Updated: 2024-03-18

### Components Overview

#### High Priority (100% Conversion Required)

1. **Protocol Metrics Collector**
   - Status: Complete
   - Priority: High
   - Dependencies: Resource Metrics Collector
   - Current: Successfully transitioned from singleton pattern to dependency injection via constructor parameters.
   - Target: Dependency Injection via constructor parameters
   - Files: `monitoring/metrics/protocol.rs`
   - Progress:
     - [x] Implemented adapter pattern for backward compatibility
     - [x] Added DI constructor with resource collector dependency
     - [x] Updated factory methods to support DI pattern
     - [x] Updated module-level functions for DI support
     - [x] Added comprehensive integration tests
     - [x] Updated main consumer (get_protocol_metrics)
     - [x] Updated DefaultMetricCollector integration
     - [x] Verified all external consumers updated
     - [x] Documentation updated

   Technical Details:
   - Added `resource_collector` field to `ProtocolMetricsCollector`
   - Implemented `create_collector_with_dependencies` in factory
   - Added adapter pattern via `ProtocolMetricsCollectorAdapter`
   - Integration tests cover:
     - DI constructor usage
     - Adapter pattern functionality
     - Error handling
     - Backward compatibility
     - Resource collector integration
   - DefaultMetricCollector now uses adapter pattern for protocol metrics
   - All singleton access points now use adapter pattern

   Migration Guide for New Consumers:
   1. Use `ProtocolMetricsCollector::with_dependencies()` for direct DI
   2. Use `create_collector_adapter()` for adapter pattern
   3. Prefer DI over adapter pattern for new code
   4. Handle potential errors from metrics collection

   Next Steps:
   1. Monitor adapter pattern usage and gather feedback
   2. Plan for eventual removal of singleton pattern
   3. Consider similar patterns for other collectors
   4. Keep documentation updated for new consumers

2. **Performance Collector**
   - Status: Complete
   - Priority: High
   - Dependencies: None
   - Current: Dependency Injection via constructor parameters
   - Files: `monitoring/metrics/performance.rs`
   - Progress:
     - [x] Implemented adapter pattern in `metrics/performance/adapter.rs`
     - [x] Added DI constructor `with_config` to `PerformanceCollector`
     - [x] Updated factory methods to support DI pattern
     - [x] Added configuration injection support
     - [x] Added comprehensive metrics collection
     - [x] Updated helper functions to use adapter pattern
     - [x] Added integration tests for DI pattern
     - [x] Documented migration path for consumers

   Technical Details:
   - Added `PerformanceCollectorAdapter` to facilitate DI transition
   - Implemented `with_config` constructor accepting configuration
   - Factory methods now support both adapter and direct DI patterns:
     - `create_collector()` for direct DI
     - `create_collector_adapter()` for adapter pattern
   - Maintained backward compatibility through adapter pattern
   - Added proper error handling and configuration management
   - Improved metrics collection with histograms and labels
   - Updated test suite to verify adapter pattern functionality

   Migration Guide for New Consumers:
   1. Use `PerformanceCollectorFactory::create_collector()` for direct DI
   2. Or use `create_collector_adapter()` for adapter pattern
   3. Pass configuration through constructor when needed
   4. Use helper functions for common operations

   For existing consumers:
   1. Replace direct collector usage with adapter
   2. Use helper functions like `time_operation` and `record_operation`
   3. Gradually migrate to direct DI pattern where appropriate

   Next Steps:
   1. Monitor adapter pattern usage
   2. Plan for eventual removal of singleton pattern
   3. Consider applying similar patterns to other collectors
   4. Keep documentation updated for new consumers

3. **Health Checker**
   - Status: Complete
   - Priority: High
   - Dependencies: None
   - Current: Singleton via `ensure_factory().get_global_checker()`
   - Files: `monitoring/health/mod.rs`
   - Progress:
     - [x] Implemented adapter pattern in `health/adapter.rs`
     - [x] Added DI constructor `with_dependencies` to `DefaultHealthChecker`
     - [x] Updated factory methods to support DI pattern
     - [x] Added configuration injection support
     - [x] Updated MonitoringService to use HealthCheckerAdapter
     - [x] Updated tests to use DI pattern
     - [x] Added integration tests for DI pattern
     - [x] Documented migration path for consumers

   Technical Details:
   - Added `HealthCheckerAdapter` to facilitate DI transition
   - Implemented `with_dependencies` constructor accepting optional config
   - Factory methods now support both adapter and direct DI patterns:
     - `create_checker_with_dependencies()` for direct DI
     - `create_checker_adapter()` for adapter pattern
   - Maintained backward compatibility through adapter pattern
   - Added proper error handling and configuration management
   - MonitoringService now uses `HealthCheckerAdapter` instead of `DefaultHealthChecker`
   - Updated test suite to verify adapter pattern functionality

   Migration Guide for New Consumers:
   1. Use `HealthCheckerFactory::create_checker_with_dependencies()` for direct DI
   2. Or use `create_checker_adapter()` for adapter pattern
   3. Pass configuration through constructor when needed
   4. Avoid using global factory methods

   For existing consumers:
   1. Replace `DefaultHealthChecker` with `HealthCheckerAdapter`
   2. Use `create_checker_adapter()` for backward compatibility
   3. Gradually migrate to direct DI pattern where appropriate

   Next Steps:
   1. Monitor adapter pattern usage
   2. Plan for eventual removal of singleton pattern
   3. Consider applying similar patterns to other components
   4. Update documentation for new consumers

4. **Resource Metrics Collector**
   - Status: Complete
   - Priority: High
   - Dependencies: Performance Collector
   - Current: Dependency Injection via constructor parameters
   - Files: `monitoring/metrics/resource.rs`

   ### Progress
   - [x] Implemented adapter pattern in `metrics/resource/adapter.rs`
   - [x] Added DI constructor `with_dependencies` to `ResourceMetricsCollector`
   - [x] Updated factory methods to support DI pattern
   - [x] Added configuration injection support
   - [x] Added comprehensive metrics collection
   - [x] Updated helper functions to use adapter pattern
   - [x] Added integration tests for DI pattern
   - [x] Documented migration path for consumers

   ### Technical Details
   - Added `ResourceMetricsCollectorAdapter` to facilitate DI transition
   - Implemented `with_dependencies` constructor accepting configuration and performance collector
   - Factory methods now support both adapter and direct DI patterns:
     - `create_collector()` for direct DI
     - `create_collector_adapter()` for adapter pattern
   - Maintained backward compatibility through adapter pattern
   - Added proper error handling and configuration management
   - Improved metrics collection with system resource tracking
   - Updated test suite to verify adapter pattern functionality

   ### Migration Guide
   For new consumers:
   1. Use `ResourceMetricsCollectorFactory::create_collector()` for direct DI
   2. Or use `create_collector_adapter()` for adapter pattern
   3. Pass configuration through constructor when needed
   4. Use helper functions for common operations

   For existing consumers:
   1. Replace direct collector usage with adapter
   2. Use helper functions like `get_team_metrics` and `register_team`
   3. Gradually migrate to direct DI pattern where appropriate

   ### Next Steps
   1. Monitor adapter pattern usage
   2. Plan for eventual removal of singleton pattern
   3. Consider applying similar patterns to other collectors
   4. Keep documentation updated for new consumers

5. **Alert Manager**
   - Status: Complete
   - Priority: High
   - Dependencies: Notification Manager
   - Current: Dependency Injection via constructor parameters
   - Files: `monitoring/alerts/mod.rs`

   ### Progress
   - [x] Implemented adapter pattern in `alerts/adapter.rs`
   - [x] Added DI constructor `with_dependencies` to `DefaultAlertManager`
   - [x] Updated factory methods to support DI pattern
   - [x] Added notification manager integration
   - [x] Added comprehensive alert handling
   - [x] Updated helper functions to use adapter pattern
   - [x] Added integration tests for DI pattern
   - [x] Documented migration path for consumers

   ### Technical Details
   - Added `AlertManagerAdapter` to facilitate DI transition
   - Implemented `with_dependencies` constructor accepting configuration and notification manager
   - Factory methods now support both adapter and direct DI patterns:
     - `create_manager()` for direct DI
     - `create_manager_adapter()` for adapter pattern
   - Maintained backward compatibility through adapter pattern
   - Added proper error handling and configuration management
   - Improved alert handling with notification integration
   - Updated test suite to verify adapter pattern functionality

   ### Migration Guide
   For new consumers:
   1. Use `AlertManagerFactory::create_manager()` for direct DI
   2. Or use `create_manager_adapter()` for adapter pattern
   3. Pass configuration through constructor when needed
   4. Use helper functions for common operations

   For existing consumers:
   1. Replace direct manager usage with adapter
   2. Use helper functions like `send_alert` and `handle_alert`
   3. Gradually migrate to direct DI pattern where appropriate

   ### Next Steps
   1. Monitor adapter pattern usage
   2. Plan for eventual removal of singleton pattern
   3. Consider applying similar patterns to other components
   4. Keep documentation updated for new consumers

6. **Network Monitor**
   - Status: Complete
   - Priority: High
   - Dependencies: None
   - Current: Dependency Injection via constructor parameters
   - Files: `monitoring/network/mod.rs`

   ### Progress
   - [x] Implemented adapter pattern in `network/adapter.rs`
   - [x] Added DI constructor `with_dependencies` to `NetworkMonitor`
   - [x] Updated factory methods to support DI pattern
   - [x] Added configuration injection support
   - [x] Added comprehensive network statistics collection
   - [x] Updated helper functions to use adapter pattern
   - [x] Added integration tests for DI pattern
   - [x] Documented migration path for consumers

   ### Technical Details
   - Added `NetworkMonitorAdapter` to facilitate DI transition
   - Implemented `with_dependencies` constructor accepting configuration
   - Factory methods now support both adapter and direct DI patterns:
     - `create_monitor_with_dependencies()` for direct DI
     - `create_monitor_adapter()` for adapter pattern
   - Maintained backward compatibility through adapter pattern
   - Added proper error handling and configuration management
   - Improved network statistics collection with detailed metrics
   - Updated test suite to verify adapter pattern functionality

   ### Migration Guide
   For new consumers:
   1. Use `NetworkMonitorFactory::create_monitor_with_dependencies()` for direct DI
   2. Or use `create_monitor_adapter()` for adapter pattern
   3. Pass configuration through constructor when needed
   4. Use helper functions for common operations

   For existing consumers:
   1. Replace direct monitor usage with adapter
   2. Use helper functions like `get_stats` and `get_interface_stats`
   3. Gradually migrate to direct DI pattern where appropriate

   ### Next Steps
   1. Monitor adapter pattern usage
   2. Plan for eventual removal of singleton pattern
   3. Consider applying similar patterns to other components
   4. Keep documentation updated for new consumers

7. **Dashboard Manager**
   - Status: Complete
   - Priority: Medium
   - Dependencies: None
   - Current: Dependency Injection via constructor parameters and adapter pattern
   - Files: `monitoring/dashboard/mod.rs`
   
   ### Progress
   - [x] Implemented adapter pattern in `dashboard/adapter.rs`
   - [x] Added DI constructor to `DashboardManager`
   - [x] Updated factory methods to support DI pattern
   - [x] Added adapter creation methods
   - [x] Updated module-level functions for adapter support
   - [x] Added integration with existing `Manager` class
   - [x] Ensured all tests pass
   - [x] Maintained backwards compatibility
   
   ### Technical Details
   - Added `DashboardManagerAdapter` for backward compatibility
   - Updated `DashboardManagerFactory` to support both DI and adapter patterns
   - Added methods for creating managers with dependencies
   - Added helper function `create_adapter()` for easy adapter creation
   - Ensured compatibility with existing code
   - All tests passing without modifications
   
   ### Migration Guide
   For new consumers:
   1. Use `DashboardManagerFactory::create_manager_with_dependencies()` for direct DI
   2. Or use `create_adapter()` for adapter pattern
   3. Pass configuration through constructor when needed
   4. Use the adapter interface for standard operations
   
   For existing consumers:
   1. Replace direct manager usage with adapter
   2. Gradually migrate to direct DI pattern where appropriate
   
   ### Next Steps
   1. Mark global functions as deprecated
   2. Monitor adapter pattern usage
   3. Plan for eventual removal of singleton pattern
   4. Update any dependent components

8. **Notification Manager**
   - Status: Complete
   - Priority: Medium  
   - Dependencies: None
   - Current: Dependency Injection via constructor parameters and adapter pattern
   - Files: `monitoring/alerts/notify.rs`
   
   ### Progress
   - [x] Implemented adapter pattern in `alerts/adapter.rs`
   - [x] Added DI constructor to `NotificationManager`
   - [x] Updated factory methods to support DI pattern
   - [x] Added adapter creation methods
   - [x] Implemented explicit dependencies
   - [x] Updated error handling
   - [x] Ensured all tests pass
   - [x] Maintained backwards compatibility
   
   ### Technical Details
   - Added `NotificationManagerAdapter` for backward compatibility
   - Updated `NotificationManagerFactory` to support both DI and adapter patterns
   - Added methods for creating managers with dependencies
   - Added `create_adapter()` function for easy adapter creation
   - Improved error handling for adapter pattern
   - Ensured compatibility with dependent components (AlertManager)
   
   ### Migration Guide
   For new consumers:
   1. Use `NotificationManagerFactory::create_manager_with_dependencies()` for direct DI
   2. Or use `create_adapter()` for adapter pattern
   3. Pass configuration through constructor when needed
   4. Use the adapter interface for standard notification operations
   
   For existing consumers:
   1. Replace direct manager usage with adapter
   2. Handle potential errors appropriately
   3. Gradually migrate to direct DI pattern where appropriate
   
   ### Next Steps
   1. Mark global functions as deprecated
   2. Monitor adapter pattern usage
   3. Plan for eventual removal of singleton pattern
   4. Review integration with AlertManager

## Completed Systems for Conversion

### Monitoring Service Components

1. **MonitoringServiceFactory**
   - Status: Complete
   - Priority: High
   - Current Implementation: Dependency Injection via constructor parameters and adapter pattern
   - Files: `crates/core/src/monitoring/mod.rs`
   - Dependencies: Multiple component factories
   - Progress:
     - [x] Implemented adapter pattern for MonitoringServiceFactory
     - [x] Added DI constructor with component factory dependencies
     - [x] Removed global static instance
     - [x] Updated factory methods to support DI pattern
     - [x] Added comprehensive error handling
     - [x] Updated initialization flow
     - [x] Added integration tests
     - [x] Updated documentation

   Technical Details:
   - Introduced `MonitoringServiceFactoryAdapter` for backward compatibility
   - Added `with_dependencies()` constructor accepting component factories:
     - HealthCheckerFactory
     - MetricCollectorFactory
     - AlertManagerFactory
     - NetworkMonitorFactory
   - Removed global `MONITORING_FACTORY` static
   - Factory methods now use injected dependencies when available
   - Improved error handling and initialization flow
   - Updated test suite to verify adapter pattern functionality

   Migration Guide for New Consumers:
   1. Use `MonitoringServiceFactory::with_dependencies()` for direct DI
   2. Or use `create_factory_adapter()` for adapter pattern
   3. Pass configuration through constructor when needed
   4. Use helper functions for common operations

   For existing consumers:
   1. Replace direct factory usage with adapter
   2. Use helper functions like `create_service_with_config()`
   3. Gradually migrate to direct DI pattern where appropriate

   Next Steps:
   1. Monitor adapter pattern usage and gather feedback
   2. Plan for eventual removal of singleton pattern
   3. Update integration tests to cover new DI patterns
   4. Review and update related documentation

2. **System Information Manager**
   - Status: Complete
   - Priority: High
   - Current: Uses `lazy_static` for system info
   - Files: `crates/core/src/monitoring/network/mod.rs`
   - Dependencies: None

### Tool Components

1. **Tool Metrics Collector**
   - Status: Complete
   - Priority: Medium
   - Current: Uses `OnceCell` for global instance
   - Files: `crates/core/src/monitoring/metrics/tool.rs`
   - Dependencies: Performance Collector

2. **Metric Exporter**
   - Status: Complete
   - Priority: Medium
   - Current: Uses `OnceCell` for global exporter
   - Files: `crates/core/src/monitoring/metrics/export.rs`
   - Dependencies: None

## Migration Strategy - Implementing Deprecation

To encourage migration to the DI pattern, we'll mark global access functions as deprecated:

```rust
/// Get the notification manager instance
#[deprecated(
  since = "0.2.0",
  note = "Use dependency injection via NotificationManagerFactory::create_adapter() instead"
)]
pub fn get_manager() -> Option<Arc<NotificationManager>> {
    NOTIFICATION_MANAGER.get().cloned()
}
```

This will:
1. Provide a clear warning during compilation
2. Document the preferred alternative
3. Allow for a gradual transition
4. Guide new code to use the DI pattern

We should implement deprecation warnings for:
- Global `get_*` functions
- Singleton initialization methods
- Direct global state access points
- Factory methods that rely on global state

Responsible deprecation will ensure existing code continues to work while guiding towards better practices.

## Implementation Plan for Deprecation

1. **Phase 1: Add Deprecation Annotations**
   - Add `#[deprecated]` attributes to all global access functions
   - Include migration guidance in deprecation messages
   - Update documentation to highlight preferred patterns
   - Timeline: Next sprint

2. **Phase 2: Monitor Usage and Gather Feedback**
   - Track which components still use deprecated functions
   - Gather feedback from teams on migration challenges
   - Provide additional support for complex migrations
   - Timeline: 1-2 weeks after Phase 1

3. **Phase 3: Migration Support**
   - Create migration guides for common patterns
   - Provide helper utilities for transition
   - Support teams with specific migration needs
   - Timeline: Ongoing

4. **Phase 4: Eventual Removal Plan**
   - Develop timeline for future removal of singleton patterns
   - Create roadmap for complete DI adoption
   - Timeline: To be determined based on migration progress

## Implementation Strategy

For newly converted components, we have followed this pattern:

1. Create an adapter:
```rust
pub struct ComponentAdapter {
    inner: Option<Arc<Component>>,
}

impl ComponentAdapter {
    pub fn new() -> Self {
        Self { inner: None }
    }

    pub fn with_component(component: Arc<Component>) -> Self {
        Self {
            inner: Some(component),
        }
    }
}
```

2. Implement DI constructor:
```rust
impl Component {
    pub fn with_dependencies(
        config: ComponentConfig,
        dependencies: Arc<Dependencies>,
    ) -> Self {
        Self {
            config,
            dependencies,
        }
    }
}
```

3. Create factory methods:
```rust
pub struct ComponentFactory {
    config: ComponentConfig,
}

impl ComponentFactory {
    pub fn create_with_dependencies(
        &self,
        dependencies: Arc<Dependencies>,
    ) -> Arc<Component> {
        Arc::new(Component::with_dependencies(
            self.config.clone(),
            dependencies,
        ))
    }

    pub fn create_adapter(&self) -> Arc<ComponentAdapter> {
        create_component_adapter()
    }
}
```

4. Deprecate global access functions:
```rust
#[deprecated(
  since = "0.2.0",
  note = "Use dependency injection via ComponentFactory::create_adapter() instead"
)]
pub fn get_component() -> Option<Arc<Component>> {
    COMPONENT.get().cloned()
}
```

## Risks and Mitigations

1. **Backward Compatibility**
   - Maintain adapter pattern support
   - Keep existing global access methods during transition
   - Add deprecation warnings for global access

2. **Performance Impact**
   - Monitor metrics during conversion
   - Implement lazy initialization where needed
   - Profile critical paths

3. **Testing Coverage**
   - Add new tests before conversion
   - Verify all paths with integration tests
   - Monitor test coverage metrics

4. **Dependencies**
   - Document all component dependencies
   - Create dependency graphs
   - Plan conversion order carefully

## Progress Tracking

### Completed
- ✅ All component conversions:
  - Protocol Metrics Collector
  - Performance Collector
  - Health Checker
  - Resource Metrics Collector
  - Alert Manager
  - Network Monitor
  - Dashboard Manager
  - Notification Manager
  - MonitoringServiceFactory
  - System Information Manager
  - Tool Metrics Collector
  - Metric Exporter

### In Progress
- 🔄 Implementation of deprecation annotations

### Next Steps
- ⏳ Create comprehensive migration guides
- ⏳ Develop strategy for eventual removal
- ⏳ Consider application to other system components

## Conclusion

The Singleton to DI Conversion project has successfully completed all planned component conversions. All high and medium priority components now support the dependency injection pattern while maintaining backward compatibility through adapters.

The next phase will focus on helping consumers migrate to the new patterns by implementing appropriate deprecation warnings and providing clear migration guides.