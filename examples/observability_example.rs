/// Example demonstrating the observability features of the command system
///
/// This example creates a command registry with observability hooks and executes
/// several commands to demonstrate the metrics and tracing features.

use std::sync::{Arc, Mutex};
use squirrel_commands::{Command, CommandRegistry, CommandResult, CommandError};
use squirrel_commands::observability::{ObservabilitySystem, log_command_execution, record_resource_usage};
use std::error::Error;

// Simple test command for demonstration
struct TestCommand;

impl Command for TestCommand {
    fn name(&self) -> &'static str {
        "test"
    }

    fn description(&self) -> &'static str {
        "A test command for observability demonstration"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        println!("Executing test command with args: {:?}", args);
        
        // Sleep to simulate work
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok("Test command executed successfully".to_string())
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("test")
            .about("A test command for observability demonstration")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(TestCommand)
    }
}

// A command that will fail
struct FailingCommand;

impl Command for FailingCommand {
    fn name(&self) -> &'static str {
        "fail"
    }

    fn description(&self) -> &'static str {
        "A command that always fails"
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Err(CommandError::ExecutionError("This command deliberately fails for testing".to_string()))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("fail")
            .about("A command that always fails")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(FailingCommand)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging (using simple stdout logging)
    println!("Command Observability Example");
    println!("============================");
    
    // Create a command registry
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    
    // Register test commands
    {
        let registry_lock = registry.lock().unwrap();
        registry_lock.register("test", Arc::new(TestCommand))?;
        registry_lock.register("fail", Arc::new(FailingCommand))?;
    }
    
    // Create observability system
    let _observability = ObservabilitySystem::new();
    
    // Execute the test command multiple times
    {
        println!("\nExecuting test command...");
        let registry_lock = registry.lock().unwrap();
        
        // Execute successful command multiple times
        for i in 1..=5 {
            println!("\nExecution #{}", i);
            let command = registry_lock.get_command("test")?;
            let args = vec!["arg1".to_string(), "arg2".to_string()];
            
            // Record start time
            let start = std::time::Instant::now();
            
            // Execute the command
            let result = command.execute(&args);
            
            // Calculate execution time
            let elapsed = start.elapsed();
            let elapsed_ms = elapsed.as_millis() as u64;
            
            // Log with observability
            log_command_execution("test", &args, &result, elapsed_ms);
            
            // Record resource usage
            record_resource_usage("test", 1024, 2.5);
            
            match &result {
                Ok(output) => println!("Output: {}", output),
                Err(e) => println!("Error: {}", e),
            }
            
            // Sleep briefly between executions
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        
        // Execute failing command
        println!("\nExecuting failing command...");
        let command = registry_lock.get_command("fail")?;
        let args = vec!["arg1".to_string()];
        
        // Record start time
        let start = std::time::Instant::now();
        
        // Execute the command
        let result = command.execute(&args);
        
        // Calculate execution time
        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        
        // Log with observability
        log_command_execution("fail", &args, &result, elapsed_ms);
        
        match &result {
            Ok(output) => println!("Output: {}", output),
            Err(e) => println!("Error: {}", e),
        }
    }
    
    println!("\nObservability Example Completed");
    Ok(())
} 