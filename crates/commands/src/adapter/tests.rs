#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_adapter_initialization() {
        // Create a new adapter
        let mut adapter = CommandRegistryAdapter::new();
        
        // Adapter should not be initialized
        assert!(!adapter.is_initialized());
        
        // Initialize the adapter
        let result = adapter.initialize();
        assert!(result.is_ok());
        
        // Now it should be initialized
        assert!(adapter.is_initialized());
        
        // Attempting to initialize again should fail
        let result = adapter.initialize();
        assert!(result.is_err());
    }

    #[test]
    fn test_adapter_operations_when_not_initialized() {
        // Create a new adapter without initializing it
        let adapter = CommandRegistryAdapter::new();
        
        // Operations should fail with NotInitialized error
        let result = adapter.get_registry();
        assert!(result.is_err());
        
        let result = adapter.add_validation_rule(Box::new(MockValidationRule {}));
        assert!(result.is_err());
        
        let result = adapter.add_lifecycle_handler(Box::new(MockLifecycleHandler {}));
        assert!(result.is_err());
        
        let result = adapter.register_command(Box::new(MockCommand {}));
        assert!(result.is_err());
        
        let result = adapter.get_command("test");
        assert!(result.is_err());
        
        let result = adapter.list_commands();
        assert!(result.is_err());
        
        let result = adapter.execute_command("test", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_functions() {
        // Test create_registry_adapter
        let adapter = create_registry_adapter();
        assert!(!adapter.is_initialized());
        
        // Test create_initialized_registry_adapter
        let adapter_result = create_initialized_registry_adapter();
        assert!(adapter_result.is_ok());
        let adapter = adapter_result.unwrap();
        assert!(adapter.is_initialized());
        
        // Test create_registry_adapter_with_registry
        let registry = Arc::new(RwLock::new(CommandRegistry::new()));
        let adapter = create_registry_adapter_with_registry(registry.clone());
        assert!(adapter.is_initialized());
        
        // The registry should be accessible
        let retrieved_registry = adapter.get_registry();
        assert!(retrieved_registry.is_ok());
    }

    // Mock implementations for testing
    struct MockValidationRule;
    impl ValidationRule for MockValidationRule {
        fn validate(&self, _command: &dyn Command) -> Result<()> {
            Ok(())
        }
    }

    struct MockLifecycleHandler;
    impl LifecycleHandler for MockLifecycleHandler {
        fn before_execution(&self, _command: &dyn Command) -> Result<()> {
            Ok(())
        }
        
        fn after_execution(&self, _command: &dyn Command, _result: Result<()>) -> Result<()> {
            Ok(())
        }
    }

    struct MockCommand;
    impl Command for MockCommand {
        fn execute(&self, _args: Vec<String>) -> Result<()> {
            Ok(())
        }
        
        fn name(&self) -> &str {
            "mock_command"
        }
        
        fn description(&self) -> &str {
            "A mock command for testing"
        }
    }
} 