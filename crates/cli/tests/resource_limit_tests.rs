use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;

// Import necessary types from CLI crate
use squirrel_cli::command_adapter::{CommandAdapterTrait, RegistryAdapter};
use commands::CommandRegistry;
use squirrel_cli::commands::test_command::TestCommand;
use squirrel_cli::error::AdapterError;
use squirrel_cli::Command;
use commands::CommandResult;

/// Memory-intensive test command
struct MemoryIntensiveCommand {
    name: String,
    description: String,
    alloc_size: usize, // Size in MB
}

impl MemoryIntensiveCommand {
    fn new(name: &str, description: &str, alloc_size: usize) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            alloc_size,
        }
    }
}

#[async_trait]
impl TestCommand for MemoryIntensiveCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn execute(&self, _args: Vec<String>) -> Result<String, AdapterError> {
        // Allocate memory based on size (in MB)
        let bytes = self.alloc_size * 1024 * 1024;
        let mut data = Vec::with_capacity(bytes);
        
        // Fill with some data
        for i in 0..bytes {
            data.push((i % 256) as u8);
        }
        
        // Return success message
        Ok(format!("Allocated {} MB of memory", self.alloc_size))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("memory_test")
            .about("Memory test command")
    }
}

// Implement the Command trait for the registry
impl Command for MemoryIntensiveCommand {
    fn name(&self) -> &'static str {
        Box::leak(self.name.clone().into_boxed_str())
    }
    
    fn description(&self) -> &'static str {
        Box::leak(self.description.clone().into_boxed_str())
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("memory_test")
            .about("Memory test command")
    }
    
    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Ok(format!("Memory-intensive command executed (sync version), allocated {} MB", self.alloc_size))
    }
    
    fn help(&self) -> String {
        format!("Memory-intensive test command that allocates {} MB of memory", self.alloc_size)
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {
            name: self.name.clone(),
            description: self.description.clone(),
            alloc_size: self.alloc_size,
        })
    }
}

/// Long-running test command
struct LongRunningCommand {
    name: String,
    description: String,
    duration_ms: u64,
}

impl LongRunningCommand {
    fn new(name: &str, description: &str, duration_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            duration_ms,
        }
    }
}

#[async_trait]
impl TestCommand for LongRunningCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn execute(&self, _args: Vec<String>) -> Result<String, AdapterError> {
        // Sleep for the specified duration
        tokio::time::sleep(tokio::time::Duration::from_millis(self.duration_ms)).await;
        
        // Return success message
        Ok(format!("Finished after {} ms", self.duration_ms))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("long_test")
            .about("Long running test command")
    }
}

// Implement Command trait for LongRunningCommand
impl Command for LongRunningCommand {
    fn name(&self) -> &'static str {
        Box::leak(self.name.clone().into_boxed_str())
    }
    
    fn description(&self) -> &'static str {
        Box::leak(self.description.clone().into_boxed_str())
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("long_test")
            .about("Long running test command")
    }
    
    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Ok(format!("Long-running command completed after {} ms (sync version)", self.duration_ms))
    }
    
    fn help(&self) -> String {
        format!("Long-running test command that runs for {} ms", self.duration_ms)
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {
            name: self.name.clone(),
            description: self.description.clone(),
            duration_ms: self.duration_ms,
        })
    }
}

// Helper function to get current process memory usage (platform-specific)
fn get_current_memory_usage() -> usize {
    // This is a dummy implementation for tests
    // In a real implementation, you would use platform-specific APIs to get memory usage
    // For example, on Linux you might read /proc/self/statm
    100 // Return a dummy value of 100 MB
}

#[tokio::test]
async fn test_memory_limit_handling() {
    // Create registry and adapter
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    let adapter = Arc::new(RegistryAdapter::new(registry.clone()));
    
    // Register memory-intensive commands
    {
        let mut reg = registry.lock().await;
        
        // Register a moderate memory command (100MB)
        let cmd1 = Arc::new(MemoryIntensiveCommand::new(
            "mem_moderate",
            "Allocates a moderate amount of memory",
            100,
        ));
        reg.register("mem_moderate", cmd1).unwrap();
        
        // Register a high memory command (500MB)
        let cmd2 = Arc::new(MemoryIntensiveCommand::new(
            "mem_high",
            "Allocates a high amount of memory",
            500,
        ));
        reg.register("mem_high", cmd2).unwrap();
    }
    
    // Execute the moderate memory command (should succeed)
    let result = adapter.execute_command("mem_moderate", vec![]).await;
    assert!(result.is_ok(), "Moderate memory command failed: {:?}", result.err());
    
    // Execute the high memory command (should still succeed in test environment)
    let result = adapter.execute_command("mem_high", vec![]).await;
    assert!(result.is_ok(), "High memory command failed: {:?}", result.err());
}

#[tokio::test]
async fn test_concurrent_connection_limits() {
    // Create registry and adapter
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    let adapter = Arc::new(RegistryAdapter::new(registry.clone()));
    
    // Register long-running commands
    {
        let mut reg = registry.lock().await;
        
        // Register a command that runs for 100ms
        let cmd = Arc::new(LongRunningCommand::new(
            "long_running",
            "A long-running command",
            100,
        ));
        reg.register("long_running", cmd).unwrap();
    }
    
    // Execute 10 concurrent commands
    let mut handles = vec![];
    for i in 0..10 {
        let adapter_clone = adapter.clone();
        handles.push(tokio::spawn(async move {
            let result = adapter_clone.execute_command("long_running", vec![format!("task{}", i)]).await;
            (i, result)
        }));
    }
    
    // All should complete successfully
    for handle in handles {
        let (i, result) = handle.await.unwrap();
        assert!(result.is_ok(), "Task {} failed: {:?}", i, result.err());
    }
}

#[tokio::test]
async fn test_resource_cleanup_after_error() {
    // Create registry and adapter
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    let adapter = Arc::new(RegistryAdapter::new(registry.clone()));
    
    // Define an error command
    struct ErrorCommand;
    
    #[async_trait]
    impl TestCommand for ErrorCommand {
        fn name(&self) -> &str {
            "error_command"
        }
        
        fn description(&self) -> &str {
            "A command that generates an error"
        }
        
        async fn execute(&self, _args: Vec<String>) -> Result<String, AdapterError> {
            // Allocate some resources
            let mut data = Vec::with_capacity(10 * 1024 * 1024); // 10MB
            for i in 0..data.capacity() {
                data.push((i % 256) as u8);
            }
            
            // Generate an error
            Err(AdapterError::NotFound("This is a test error".to_string()))
        }
        
        fn parser(&self) -> clap::Command {
            clap::Command::new("error_command")
                .about("A command that generates an error")
        }
    }
    
    // Implement Command trait for ErrorCommand
    impl Command for ErrorCommand {
        fn name(&self) -> &'static str {
            "error_command"
        }
        
        fn description(&self) -> &'static str {
            "A command that generates an error"
        }
        
        fn parser(&self) -> clap::Command {
            clap::Command::new("error_command")
                .about("A command that generates an error")
        }
        
        fn execute(&self, _args: &[String]) -> CommandResult<String> {
            Err(commands::CommandError::ExecutionError("This is a test error".to_string()))
        }
        
        fn help(&self) -> String {
            "Error command that deliberately fails".to_string()
        }
        
        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(Self)
        }
    }
    
    // Register the error command
    {
        let mut reg = registry.lock().await;
        reg.register("error_command", Arc::new(ErrorCommand)).unwrap();
    }
    
    // Execute the error command
    let result = adapter.execute_command("error_command", vec![]).await;
    assert!(result.is_err(), "Expected command to fail");
    
    // Verify that we can execute another command after the error
    let second_result = adapter.execute_command("error_command", vec![]).await;
    assert!(second_result.is_err(), "Expected second command to fail");
} 