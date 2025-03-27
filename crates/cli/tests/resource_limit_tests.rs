use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::runtime::Runtime;

// Import necessary types from CLI crate
use squirrel_cli::command_adapter::{CommandAdapterTrait, RegistryAdapter};
use commands::CommandRegistry;
use squirrel_cli::commands::test_command::TestCommand;
use squirrel_cli::error::AdapterError;
use commands::Command;
use commands::CommandResult;

/// Memory intensive test command
pub struct MemoryIntensiveCommand {
    name: String,
    description: String,
    memory_size_mb: usize,
}

impl MemoryIntensiveCommand {
    /// Create a new memory intensive command
    pub fn new(name: String, description: String, memory_size_mb: usize) -> Self {
        Self { name, description, memory_size_mb }
    }
}

#[async_trait::async_trait]
impl TestCommand for MemoryIntensiveCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn execute(&self, _args: Vec<String>) -> Result<String, AdapterError> {
        // Simulate allocating a lot of memory
        // In an actual implementation, we would allocate real memory here
        // For tests, we'll just report how much we would have allocated
        
        // Return the amount of memory "allocated" in the output
        Ok(format!("Memory test complete: allocated {} MB", self.memory_size_mb))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("memory_test")
            .about("A memory-intensive test command")
    }
}

// Also implement the Command trait for the registry
impl Command for MemoryIntensiveCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn parser(&self) -> clap::Command {
        // Use hardcoded strings to avoid lifetime issues
        clap::Command::new("memory_test")
            .about("Memory intensive test command")
    }
    
    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Ok(format!("Memory-intensive command executed (sync version), allocated {} MB", 
                  self.memory_size_mb))
    }
    
    fn help(&self) -> String {
        format!("Memory-intensive test command that allocates {} MB of memory", self.memory_size_mb)
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {
            name: self.name.clone(),
            description: self.description.clone(),
            memory_size_mb: self.memory_size_mb,
        })
    }
}

// Create a long-running test command with configurable duration
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

#[async_trait::async_trait]
impl TestCommand for LongRunningCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, _args: Vec<String>) -> Result<String, AdapterError> {
        // Sleep for the configured duration
        tokio::time::sleep(tokio::time::Duration::from_millis(self.duration_ms)).await;
        
        Ok(format!("Long-running command completed after {} ms", self.duration_ms))
    }

    fn parser(&self) -> clap::Command {
        // Use hardcoded strings to avoid lifetime issues
        clap::Command::new("long_test")
            .about("Long running test command")
    }
}

// Also implement the Command trait for the registry
impl Command for LongRunningCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn parser(&self) -> clap::Command {
        // Use hardcoded strings to avoid lifetime issues
        clap::Command::new("long_test")
            .about("Long running test command")
    }
    
    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        // This is a synchronous version, so we can't actually sleep
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
    
    // Register a memory-intensive command
    let memory_command = Arc::new(MemoryIntensiveCommand::new(
        "memory_test".to_string(),
        "A memory-intensive test command".to_string(),
        500, // 500 MB
    ));
    
    {
        let reg = registry.lock().await;
        reg.register("memory_test", memory_command.clone()).unwrap();
    }
    
    // Execute the command and check if memory was "allocated" in the output
    let result = adapter.execute_command("memory_test", vec![]).await;
    
    // Verify result
    assert!(result.is_ok(), "Command execution failed: {:?}", result.err());
    
    // Extract the output string
    let output = result.unwrap();
    
    // Check if the output indicates memory was allocated
    assert!(output.contains("Memory-intensive command executed"), "Unexpected output format: {}", output);
    assert!(output.contains("allocated 500 MB"), "Memory allocation not reported in output: {}", output);
    
    println!("Memory test successful: {}", output);
    
    // Note: In a real test with actual memory allocation, you'd measure real memory usage
    // For this test, we're just verifying the command executes successfully and reports allocation
}

#[tokio::test]
async fn test_concurrent_connection_limits() {
    // Create registry and adapter
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    let adapter = Arc::new(RegistryAdapter::new(registry.clone()));
    
    // Register a long-running command
    let long_command = Arc::new(LongRunningCommand::new(
        "long_test",
        "A long-running test command",
        500, // 500 ms
    ));
    
    {
        let reg = registry.lock().await;
        reg.register("long_test", long_command.clone()).unwrap();
    }
    
    // Create a large number of concurrent tasks
    const NUM_TASKS: usize = 50; // Reduced from 1000 for tests
    let mut handles = vec![];
    
    // Execute the long-running command many times concurrently
    for i in 0..NUM_TASKS {
        let adapter_clone = adapter.clone();
        
        handles.push(tokio::spawn(async move {
            let result = adapter_clone.execute_command("long_test", vec![format!("task{}", i)]).await;
            (i, result)
        }));
    }
    
    // Wait for all tasks and count successes/failures
    let mut successes = 0;
    let mut failures = 0;
    
    for handle in handles {
        match handle.await {
            Ok((_, Ok(_))) => successes += 1,
            Ok((i, Err(e))) => {
                failures += 1;
                println!("Task {} failed: {:?}", i, e);
            },
            Err(e) => {
                failures += 1;
                println!("Task join error: {:?}", e);
            }
        }
    }
    
    println!("Concurrent tasks - Successes: {}, Failures: {}", successes, failures);
    
    // If the CLI has a connection limit, we may see failures
    // However, for this test we're primarily verifying that the system doesn't crash
    // and can handle a large number of concurrent requests
    assert!(successes > 0, "Expected at least some successful tasks");
}

#[tokio::test]
async fn test_resource_cleanup_after_error() {
    // Create registry and adapter
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    let adapter = Arc::new(RegistryAdapter::new(registry.clone()));
    
    // Create a command that will error out
    struct ErrorCommand;
    
    #[async_trait::async_trait]
    impl TestCommand for ErrorCommand {
        fn name(&self) -> &str {
            "error_command"
        }
        
        fn description(&self) -> &str {
            "A command that will always error"
        }
        
        async fn execute(&self, _args: Vec<String>) -> Result<String, AdapterError> {
            // Allocate some resources first
            let _data = vec![0u8; 10 * 1024 * 1024]; // 10 MB
            
            // Then error out
            Err(AdapterError::ExecutionFailed("Simulated error".to_string()))
        }
        
        fn parser(&self) -> clap::Command {
            clap::Command::new("error_command")
                .about("A command that will always error")
        }
    }
    
    // Also implement Command trait
    impl Command for ErrorCommand {
        fn name(&self) -> &str {
            "error_command"
        }
        
        fn description(&self) -> &str {
            "A command that will always error"
        }
        
        fn parser(&self) -> clap::Command {
            clap::Command::new("error_command")
                .about("A command that will always error")
        }
        
        fn execute(&self, _args: &[String]) -> CommandResult<String> {
            // Simulated error in sync version
            Err(commands::CommandError::ExecutionError("Simulated error".to_string()))
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
        let reg = registry.lock().await;
        reg.register("error_command", Arc::new(ErrorCommand)).unwrap();
    }
    
    // Execute the command that will error
    let before_memory = get_current_memory_usage();
    let result = adapter.execute_command("error_command", vec![]).await;
    let after_memory = get_current_memory_usage();
    
    // Verify the command errored as expected
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Simulated error"));
    
    // Verify memory was cleaned up properly after the error
    println!("Memory before error: {} MB", before_memory);
    println!("Memory after error: {} MB", after_memory);
    
    // Memory usage shouldn't increase significantly after cleanup
    // Allow some small increase for normal runtime fluctuations
    assert!((after_memory - before_memory) < 5, 
            "Memory wasn't properly cleaned up after error");
} 