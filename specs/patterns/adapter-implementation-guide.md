# Adapter Implementation Guide for Dependency Injection

This guide demonstrates how to properly implement the adapter pattern for dependency injection in our codebase. We'll use the Alert Manager as an example.

## 1. Basic Adapter Structure

```rust
/// Adapter for the Alert Manager to support dependency injection
#[derive(Debug)]
pub struct AlertManagerAdapter {
    /// The inner manager instance
    inner: Option<Arc<AlertManager>>,
}

impl AlertManagerAdapter {
    /// Creates a new adapter without initializing it
    pub fn new() -> Self {
        Self { inner: None }
    }
    
    /// Creates an adapter with an existing manager
    pub fn with_manager(manager: Arc<AlertManager>) -> Self {
        Self { inner: Some(manager) }
    }
    
    /// Checks if the adapter is initialized
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }
}
```

## 2. Initialization Methods

```rust
impl AlertManagerAdapter {
    /// Initializes the adapter with default configuration
    pub fn initialize(&mut self) -> Result<(), AlertError> {
        if self.is_initialized() {
            return Err(AlertError::AlreadyInitialized);
        }
        
        let config = AlertConfig::default();
        let manager = AlertManager::new(config);
        self.inner = Some(Arc::new(manager));
        Ok(())
    }
    
    /// Initializes the adapter with custom configuration
    pub fn initialize_with_config(&mut self, config: AlertConfig) -> Result<(), AlertError> {
        if self.is_initialized() {
            return Err(AlertError::AlreadyInitialized);
        }
        
        let manager = AlertManager::new(config);
        self.inner = Some(Arc::new(manager));
        Ok(())
    }
}
```

## 3. Operation Methods with Proper Error Handling

```rust
impl AlertManagerAdapter {
    /// Registers an alert rule
    pub fn register_rule(&self, rule: AlertRule) -> Result<Uuid, AlertError> {
        match &self.inner {
            Some(manager) => manager.register_rule(rule),
            None => Err(AlertError::NotInitialized)
        }
    }
    
    /// Processes an alert event
    pub async fn process_event(&self, event: AlertEvent) -> Result<bool, AlertError> {
        match &self.inner {
            Some(manager) => manager.process_event(event).await,
            None => Err(AlertError::NotInitialized)
        }
    }
    
    /// Gets all registered alert rules
    pub fn get_rules(&self) -> Result<Vec<AlertRule>, AlertError> {
        match &self.inner {
            Some(manager) => Ok(manager.get_rules()),
            None => Err(AlertError::NotInitialized)
        }
    }
}
```

## 4. Factory Functions for Easy Creation

```rust
/// Creates and initializes an alert manager adapter with default configuration
pub fn create_initialized_alert_adapter() -> Result<AlertManagerAdapter, AlertError> {
    let mut adapter = AlertManagerAdapter::new();
    adapter.initialize()?;
    Ok(adapter)
}

/// Creates and initializes an alert manager adapter with custom configuration
pub fn create_alert_adapter_with_config(config: AlertConfig) -> Result<AlertManagerAdapter, AlertError> {
    let mut adapter = AlertManagerAdapter::new();
    adapter.initialize_with_config(config)?;
    Ok(adapter)
}
```

## 5. Error Types

```rust
#[derive(Debug, Error)]
pub enum AlertError {
    #[error("Alert Manager not initialized")]
    NotInitialized,
    
    #[error("Alert Manager already initialized")]
    AlreadyInitialized,
    
    #[error("Invalid alert rule: {0}")]
    InvalidRule(String),
    
    #[error("Failed to process alert: {0}")]
    ProcessingFailure(String),
}
```

## 6. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_uninitialized_adapter() {
        let adapter = AlertManagerAdapter::new();
        let rule = AlertRule::new("test", "test condition");
        
        // Should fail when not initialized
        let result = adapter.register_rule(rule);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AlertError::NotInitialized));
        }
    }
    
    #[tokio::test]
    async fn test_initialized_adapter() {
        let mut adapter = AlertManagerAdapter::new();
        adapter.initialize().unwrap();
        
        let rule = AlertRule::new("test", "test condition");
        let result = adapter.register_rule(rule);
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_double_initialization() {
        let mut adapter = AlertManagerAdapter::new();
        adapter.initialize().unwrap();
        
        // Second initialization should fail
        let result = adapter.initialize();
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AlertError::AlreadyInitialized));
        }
    }
    
    #[tokio::test]
    async fn test_factory_functions() {
        // Test default initialization
        let adapter = create_initialized_alert_adapter().unwrap();
        assert!(adapter.is_initialized());
        
        // Test with custom config
        let config = AlertConfig::default();
        let adapter = create_alert_adapter_with_config(config).unwrap();
        assert!(adapter.is_initialized());
    }
}
```

## 7. Example Usage

### Before (using global state/implicit initialization)

```rust
// Old approach with global state
fn process_alert(event: AlertEvent) -> Result<bool, Error> {
    // This would implicitly initialize if not already done
    let manager = ensure_factory().get_global_manager().await?;
    manager.process_event(event).await
}
```

### After (using explicit DI)

```rust
// Approach 1: Explicit initialization
fn process_alert(event: AlertEvent) -> Result<bool, Error> {
    let mut adapter = AlertManagerAdapter::new();
    adapter.initialize()?;
    adapter.process_event(event).await
}

// Approach 2: Using factory function
fn process_alert(event: AlertEvent) -> Result<bool, Error> {
    let adapter = create_initialized_alert_adapter()?;
    adapter.process_event(event).await
}

// Approach 3: Passing an existing adapter
fn process_alert(adapter: &AlertManagerAdapter, event: AlertEvent) -> Result<bool, Error> {
    adapter.process_event(event).await
}
```

## 8. Best Practices

1. **Never initialize implicitly**: Always require explicit initialization.
2. **Return clear errors**: Return descriptive errors when the adapter is not initialized.
3. **Provide factory functions**: Make it easy to create and initialize adapters.
4. **Use appropriate error types**: Define specific error types for your adapter.
5. **Test initialization states**: Verify behavior for both initialized and uninitialized states.
6. **Document usage patterns**: Provide clear examples of how to use your adapter.
7. **Consider thread safety**: Use Arc for shared ownership when needed.
8. **Avoid global state**: Never use static variables or global state in adapters.

## 9. Implementation Checklist

- [ ] Create adapter struct with Option<Arc<T>> for inner state
- [ ] Implement new() constructor and with_* constructors
- [ ] Add is_initialized() method
- [ ] Implement initialize() and initialize_with_config() methods
- [ ] Add operation methods with proper error handling
- [ ] Create factory functions for easy creation
- [ ] Define specific error types
- [ ] Write comprehensive tests
- [ ] Update documentation with examples 