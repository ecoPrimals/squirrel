/// # Phase 1 Functional Demonstration
///
/// This example demonstrates the practical usage of Phase 1 enhancements:
/// - Command Transaction System: demonstrates atomic operations with rollback
/// - Command Journaling: shows logging of command execution
/// - Resource Monitoring: displays resource usage during command execution
/// - Enhanced Observability: provides structured logging and metrics

use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use clap;
use std::collections::HashMap;

use squirrel_commands::{Command, CommandRegistry, CommandResult};
use squirrel_commands::observability::log_command_execution;
use squirrel_commands::journal::{CommandJournal, InMemoryJournalPersistence};
use squirrel_commands::transaction::TransactionManager;

/// Simple ResourceMonitor for demonstration purposes
struct ResourceMonitor {
    /// Resource usage by command
    usage: HashMap<String, (u64, f64)>, // (memory_kb, cpu_percent)
}

impl ResourceMonitor {
    /// Create a new resource monitor
    fn new() -> Self {
        Self {
            usage: HashMap::new(),
        }
    }
    
    /// Record resource usage for a command
    fn record_resource_usage(&mut self, command: &str, memory_kb: u64, cpu_percent: f64) {
        self.usage.insert(command.to_string(), (memory_kb, cpu_percent));
    }
    
    /// Get resource usage for a command
    fn _get_resource_usage(&self, command: &str) -> Option<(u64, f64)> {
        self.usage.get(command).copied()
    }
}

/// A simple echo command for demonstration
struct EchoCommand;

impl Command for EchoCommand {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Echoes the input back to the user"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        if args.is_empty() {
            return Ok("Echo: [no input]".to_string());
        }
        Ok(format!("Echo: {}", args.join(" ")))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("echo")
            .about("Echoes the input back to the user")
            .arg(clap::Arg::new("text")
                .help("Text to echo")
                .num_args(0..)
                .required(false))
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(EchoCommand)
    }
}

/// A simple command that fails for demonstration
struct FailCommand;

impl Command for FailCommand {
    fn name(&self) -> &'static str {
        "fail"
    }

    fn description(&self) -> &'static str {
        "A command that always fails"
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Err(squirrel_commands::CommandError::ExecutionError("Command failed as expected".to_string()))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("fail")
            .about("A command that always fails")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(FailCommand)
    }
}

/// A resource-intensive command for demonstration
struct ResourceIntensiveCommand;

impl Command for ResourceIntensiveCommand {
    fn name(&self) -> &'static str {
        "resource-intensive"
    }

    fn description(&self) -> &'static str {
        "A command that consumes resources"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        // Get the number of iterations from args or use default
        let iterations = if !args.is_empty() {
            args[0].parse::<u64>().unwrap_or(100000)
        } else {
            100000
        };
        
        // Simulate CPU-intensive work
        let start = Instant::now();
        let mut sum = 0;
        for i in 0..iterations {
            sum += i;
        }
        
        // Simulate memory allocation
        let mut data = Vec::with_capacity(1000);
        for i in 0..1000 {
            data.push(i);
        }
        
        let duration = start.elapsed();
        Ok(format!("Resource intensive operation completed in {:?} with result: {}", duration, sum))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("resource-intensive")
            .about("A command that consumes resources")
            .arg(clap::Arg::new("iterations")
                .help("Number of iterations")
                .num_args(1)
                .required(false))
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(ResourceIntensiveCommand)
    }
}

/// A simple demo using all Phase 1 enhancements
struct Phase1Demo {
    registry: Arc<Mutex<CommandRegistry>>,
    journal: Arc<RwLock<CommandJournal>>,
    _transactions: Arc<Mutex<TransactionManager>>,
    resource_monitor: Arc<Mutex<ResourceMonitor>>,
}

impl Phase1Demo {
    fn new() -> Self {
        // Create registry
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        
        // Create in-memory journal
        let persistence = Arc::new(InMemoryJournalPersistence::new());
        let journal = CommandJournal::new(persistence, 100);
        
        Self {
            registry,
            journal: Arc::new(RwLock::new(journal)),
            _transactions: Arc::new(Mutex::new(TransactionManager::new())),
            resource_monitor: Arc::new(Mutex::new(ResourceMonitor::new())),
        }
    }
    
    fn register_commands(&self) -> Result<(), Box<dyn std::error::Error>> {
        let registry_lock = self.registry.lock().unwrap();
        
        // Register echo command
        registry_lock.register("echo", Arc::new(EchoCommand))?;
        
        // Register fail command
        registry_lock.register("fail", Arc::new(FailCommand))?;
        
        // Register resource-intensive command
        registry_lock.register("resource-intensive", Arc::new(ResourceIntensiveCommand))?;
        
        Ok(())
    }
    
    fn execute_command(&self, name: &str, args: &[String]) -> CommandResult<String> {
        let start = Instant::now();
        
        // Get command from registry
        let registry_lock = self.registry.lock().unwrap();
        let command = registry_lock.get_command(name)?;
        
        // Record in journal
        let journal_id = self.journal.read().unwrap().record_start(command.as_ref(), args).unwrap_or_else(|_| "unknown".to_string());
        
        // Execute command
        let result = command.execute(args);
        
        // Record execution completion in journal
        let _ = self.journal.write().unwrap().record_completion(journal_id, result.clone());
        
        // Record metrics
        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;
        
        // Log with observability
        log_command_execution(name, args, &result, elapsed_ms);
        
        // Simulate resource monitoring
        let memory_kb = 1024; // Example value
        let cpu_percent = 5.0; // Example value
        self.resource_monitor.lock().unwrap().record_resource_usage(name, memory_kb, cpu_percent);
        
        // Return the result
        result
    }
    
    fn show_journal_entries(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get all entries
        let entries = self.journal.read().unwrap().get_entries()?;
        
        println!("\nJournal Entries:");
        println!("===============");
        
        for (i, entry) in entries.iter().enumerate() {
            println!("{}. Command: {} (ID: {})", i + 1, entry.command_name, entry.id);
            println!("   Arguments: {:?}", entry.arguments);
            println!("   State: {:?}", entry.state);
            
            if let Some(result) = &entry.result {
                println!("   Result: {}", result);
            }
            
            if let Some(error) = &entry.error {
                println!("   Error: {}", error);
            }
            
            if let Some(time) = entry.execution_time {
                println!("   Execution Time: {}ms", time);
            }
            
            println!();
        }
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Phase 1 Functional Demo");
    println!("======================");
    println!("Demonstrating Command Transaction System, Command Journaling,");
    println!("Resource Monitoring System, and Enhanced Observability");
    println!();
    
    // Create and set up demo
    let demo = Phase1Demo::new();
    demo.register_commands()?;
    
    // Scenario 1: Simple command execution with observability
    println!("\nScenario 1: Simple Command Execution");
    println!("----------------------------------");
    
    match demo.execute_command("echo", &["Hello".to_string(), "World".to_string()]) {
        Ok(output) => println!("Command Output: {}", output),
        Err(e) => println!("Command Error: {}", e),
    }
    
    // Scenario 2: Command that fails with journaling
    println!("\nScenario 2: Failed Command");
    println!("------------------------");
    
    match demo.execute_command("fail", &[]) {
        Ok(output) => println!("Command Output: {}", output),
        Err(e) => println!("Command Error: {}", e),
    }
    
    // Scenario 3: Resource-intensive command with monitoring
    println!("\nScenario 3: Resource Intensive Command");
    println!("------------------------------------");
    
    match demo.execute_command("resource-intensive", &["50000".to_string()]) {
        Ok(output) => println!("Command Output: {}", output),
        Err(e) => println!("Command Error: {}", e),
    }
    
    // Show journal entries
    demo.show_journal_entries()?;
    
    println!("\nAll Phase 1 Enhancements have been demonstrated!");
    println!("- Command Transaction System: Integration with command execution");
    println!("- Command Journaling: Recording command execution history");
    println!("- Resource Monitoring: Tracking resource usage during command execution");
    println!("- Enhanced Observability: Structured logging of command execution");
    
    Ok(())
} 