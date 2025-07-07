---
version: 1.0.0
last_updated: 2025-03-16
status: draft
priority: high
---

# Parallel Development Strategy

## Overview

This document outlines how teams can continue implementing new features while simultaneously refactoring existing singleton-based code to use dependency injection patterns. This approach allows for continuous product development while incrementally improving code quality and test stability.

## Team Organization

### Recommended Team Structure

- **Feature Teams**: Primarily develop new functionality using proper DI patterns
- **Refactoring Team**: Focus on converting critical existing components
- **Hybrid Approach**: Developers can allocate time (e.g., 70% features, 30% refactoring)

### Coordination Requirements

1. **Weekly Cross-Team Sync** (15-30 minutes)
   - Share progress on refactoring efforts
   - Identify dependencies needed for upcoming features
   - Adjust priorities based on feature requirements

2. **Refactoring Backlog**
   - Maintain a prioritized list of components to refactor
   - Update based on feature team requirements
   - Track progress and dependencies

## Development Workflow

### For New Feature Development

1. **Planning Phase**
   - Identify which existing components the feature depends on
   - Check if these components have been refactored
   - If not, decide whether to:
     a. Temporarily use adapters to bridge old code
     b. Request priority refactoring for critical dependencies

2. **Implementation Phase**
   - Use proper dependency injection for all new code
   - For unrefactored dependencies, implement adapter pattern (see below)
   - Write tests that follow the DI pattern, even when using adapters

3. **Testing Phase**
   - Test features in isolation using test fixtures and mocks
   - For integration tests with legacy code, use adapter patterns
   - Document test dependencies and patterns

### For Refactoring Existing Code

1. **Prioritization Phase**
   - Focus on components with the highest test impact
   - Prioritize components needed by upcoming features
   - Start with less complex components to establish patterns

2. **Implementation Phase**
   - Follow the Factory/DI patterns from Rule 1011
   - Create backward compatibility adapters when needed
   - Update tests to use the new pattern

3. **Validation Phase**
   - Ensure tests run successfully with new implementation
   - Verify feature teams can use the refactored component
   - Document the migration for other teams

## Integration Patterns

### Adapter Pattern for Legacy Dependencies

When building new features that depend on unrefactored components, use the adapter pattern:

```rust
// Adapter for legacy singleton service
pub struct AlertManagerAdapter {
    inner: Option<Arc<DefaultAlertManager>>, 
}

impl AlertManagerAdapter {
    pub fn new() -> Self {
        Self {
            inner: alerts::get_manager() // Gets from singleton
        }
    }
}

impl AlertManager for AlertManagerAdapter {
    // Implement the AlertManager trait by delegating to inner
    fn send_alert(&self, message: &str) -> Result<()> {
        if let Some(manager) = &self.inner {
            manager.send_alert(message)
        } else {
            Err(Error::ServiceNotInitialized)
        }
    }
}

// New code uses the adapter but follows DI principles
pub struct NotificationService {
    alert_manager: Arc<dyn AlertManager>,
}

impl NotificationService {
    pub fn new(alert_manager: Arc<dyn AlertManager>) -> Self {
        Self { alert_manager }
    }
    
    // Methods use the injected dependency
}

// Factory function that bridges old and new patterns
pub fn create_notification_service() -> NotificationService {
    // This will be simplified once AlertManager is refactored
    let adapter = Arc::new(AlertManagerAdapter::new());
    NotificationService::new(adapter)
}
```

### Legacy-to-Modern Bridge Pattern

When refactoring a component used by existing code:

```rust
// Modern implementation
pub struct AlertManagerFactory {
    config: AlertConfig,
}

impl AlertManagerFactory {
    pub fn new(config: AlertConfig) -> Self {
        Self { config }
    }
    
    pub fn create_manager(&self) -> Arc<DefaultAlertManager> {
        Arc::new(DefaultAlertManager::new(self.config.clone()))
    }
}

// Legacy compatibility layer
// This maintains the singleton but uses the factory internally
pub mod compat {
    use super::*;
    
    static MANAGER: OnceCell<Arc<DefaultAlertManager>> = OnceCell::const_new();
    
    pub fn initialize(config: AlertConfig) -> Result<()> {
        let factory = AlertManagerFactory::new(config);
        let manager = factory.create_manager();
        
        MANAGER.set(manager)
            .map_err(|_| Error::AlreadyInitialized)?;
        
        Ok(())
    }
    
    pub fn get_manager() -> Option<Arc<DefaultAlertManager>> {
        MANAGER.get().cloned()
    }
}
```

## Testing Strategy

### For New Features

1. **Use Dependency Injection from the Start**
   ```rust
   #[test]
   fn test_notification_service() {
       let mock_alert_manager = Arc::new(MockAlertManager::new());
       let service = NotificationService::new(mock_alert_manager.clone());
       
       service.notify("Test message");
       
       assert!(mock_alert_manager.was_called_with("Test message"));
   }
   ```

2. **Mock Dependencies**
   - Create mock implementations of core interfaces
   - Use mockall or similar framework
   - Inject mocks rather than relying on global state

### For Legacy Components During Transition

1. **Use Test-Specific Factories**
   ```rust
   #[test]
   fn test_legacy_integration() {
       // Setup test-specific instance
       let factory = AlertManagerFactory::new(test_config());
       let manager = factory.create_manager();
       
       // Test with the specific instance
       let result = with_alert_manager(manager, |m| {
           m.send_alert("test")
       });
       
       assert!(result.is_ok());
   }
   
   // Helper function that takes a specific instance
   fn with_alert_manager<F, R>(manager: Arc<DefaultAlertManager>, f: F) -> R
   where
       F: FnOnce(&DefaultAlertManager) -> R
   {
       f(&manager)
   }
   ```

## Coordinating Refactoring Priorities

### Critical Path Components

Focus refactoring efforts on:

1. Components with highest test impact (most flaky tests)
2. Components needed by multiple new features
3. Core infrastructure used across the codebase
4. Components with complex resource management

### Request Process

1. Feature teams request refactoring of specific components
2. Refactoring team evaluates complexity and dependencies
3. Teams agree on timeline and migration approach
4. Feature teams can proceed with adapter pattern if needed

## Tracking Progress

Track refactoring progress with a shared dashboard:

| Component | Status | Dependencies | Test Impact | Assigned To | Target Date |
|-----------|--------|--------------|------------|-------------|-------------|
| AlertManager | In Progress | MonitoringService | High | Team A | Week 3 |
| MetricCollector | Planned | None | Medium | Team B | Week 5 |
| NetworkMonitor | Backlog | MonitoringService | Medium | Unassigned | TBD |

## Conclusion

This parallel development strategy allows teams to continue building new features while systematically improving the codebase. By using adapter patterns and clear coordination, we can gradually migrate to a more testable and maintainable architecture without freezing feature development. 