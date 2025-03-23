#[cfg(test)]
mod tests {
    use crate::adapter::helper::*;
    use crate::{Command, CommandResult};
    
    #[test]
    fn test_adapter_creation() {
        let adapter = create_empty_registry_adapter().unwrap();
        
        // Test that the adapter can list commands
        let commands = adapter.list_commands().unwrap();
        assert!(commands.is_empty());
    }
    
    #[test]
    fn test_adapter_command_registration() {
        // Create a test command
        #[derive(Debug, Clone)]
        struct TestCommand;
        
        impl Command for TestCommand {
            fn name(&self) -> &str {
                "test_command"
            }
            
            fn description(&self) -> &str {
                "Test command"
            }
            
            fn execute(&self, _args: &[String]) -> CommandResult<String> {
                Ok("Test command executed".to_string())
            }
            
            fn parser(&self) -> clap::Command {
                clap::Command::new("test_command")
                    .about("Test command")
            }
            
            fn clone_box(&self) -> Box<dyn Command> {
                Box::new(self.clone())
            }
        }
        
        let adapter = create_empty_registry_adapter().unwrap();
        
        // Register the command
        adapter.register_command(Box::new(TestCommand)).unwrap();
        
        // Verify the command was registered
        let commands = adapter.list_commands().unwrap();
        assert!(commands.contains(&"test_command".to_string()));
    }
    
    #[test]
    fn test_initialized_adapter() {
        let adapter = create_initialized_registry_adapter().unwrap();
        
        // Verify built-in commands
        let commands = adapter.list_commands().unwrap();
        assert!(commands.contains(&"help".to_string()));
        assert!(commands.contains(&"version".to_string()));
    }
} 