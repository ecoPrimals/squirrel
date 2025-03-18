#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};
    use crate::commands::Command;

    #[test]
    fn test_adapter_initialization() {
        // Create a new adapter
        let mut adapter = CommandHandlerAdapter::new();
        
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
        let adapter = CommandHandlerAdapter::new();
        
        // Operations should fail with NotInitialized error
        let result = adapter.get_handler();
        assert!(result.is_err());
        
        let result = adapter.handle(&MockCommand {});
        assert!(result.is_err());
        
        let result = adapter.register_handler("mock_command", Box::new(MockProcessor {}));
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_functions() {
        // Test create_handler_adapter
        let adapter = create_handler_adapter();
        assert!(!adapter.is_initialized());
        
        // Test create_initialized_handler_adapter
        let adapter_result = create_initialized_handler_adapter();
        assert!(adapter_result.is_ok());
        let adapter = adapter_result.unwrap();
        assert!(adapter.is_initialized());
        
        // Test create_handler_adapter_with_handler
        let processor = Arc::new(RwLock::new(MockProcessor {}));
        let adapter = create_handler_adapter_with_handler(processor);
        assert!(adapter.is_initialized());
        
        // The handler should be accessible
        let handler = adapter.get_handler();
        assert!(handler.is_ok());
    }

    // Mock implementations for testing
    struct MockCommand;
    impl Command for MockCommand {
        fn execute(&self, _args: Vec<String>) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        fn name(&self) -> &str {
            "mock_command"
        }
        
        fn description(&self) -> &str {
            "A mock command for testing"
        }
    }

    struct MockProcessor;
    impl CommandProcessor for MockProcessor {
        fn process(&self, _command: &dyn Command) -> Result<()> {
            Ok(())
        }
    }
} 