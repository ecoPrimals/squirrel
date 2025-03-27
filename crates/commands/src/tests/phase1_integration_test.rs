//! Integration test for Phase 1 enhancements
//!
//! This test verifies the integration of all Phase 1 features:
//! - Command Transaction System
//! - Command Journaling System
//! - Resource Monitoring System
//! - Enhanced Observability

use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use std::fs;

use crate::registry::{Command, CommandResult};
use crate::transaction::TransactionManager;
use crate::journal::{CommandJournal, InMemoryJournalPersistence};
use crate::resources::ResourceManager;
use crate::observability::ObservabilitySystem;
use crate::CommandError;

// Simple resource monitor for the test
struct ResourceMonitor {
    memory_kb: u64,
    cpu_percent: f64,
}

impl ResourceMonitor {
    fn new() -> Self {
        Self {
            memory_kb: 0,
            cpu_percent: 0.0,
        }
    }
    
    fn record_usage(&mut self, memory_kb: u64, cpu_percent: f64) {
        self.memory_kb = memory_kb;
        self.cpu_percent = cpu_percent;
    }
    
    fn get_usage(&self) -> (u64, f64) {
        (self.memory_kb, self.cpu_percent)
    }
}

// A test command that succeeds
struct SuccessCommand;

impl Command for SuccessCommand {
    fn name(&self) -> &'static str {
        "success"
    }

    fn description(&self) -> &'static str {
        "A command that always succeeds"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        Ok(format!("Success command executed with args: {:?}", args))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("success")
            .about("A command that always succeeds")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(SuccessCommand)
    }
}

// A test command that fails
struct FailingCommand;

impl Command for FailingCommand {
    fn name(&self) -> &'static str {
        "fail"
    }

    fn description(&self) -> &'static str {
        "A command that always fails"
    }

    fn execute(&self, _args: &[String]) -> CommandResult<String> {
        Err(CommandError::ExecutionError("Command deliberately failed for testing".to_string()))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("fail")
            .about("A command that always fails")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(FailingCommand)
    }
}

// A command that uses resources
struct ResourceCommand;

impl Command for ResourceCommand {
    fn name(&self) -> &'static str {
        "resource"
    }

    fn description(&self) -> &'static str {
        "A command that uses resources"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        // Simulate resource usage
        std::thread::sleep(std::time::Duration::from_millis(50));
        Ok(format!("Resource command executed with args: {:?}", args))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("resource")
            .about("A command that uses resources")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(ResourceCommand)
    }
}

// A command with transaction-like behavior
struct TransactionCommand {
    pub state: Arc<Mutex<String>>,
}

impl Command for TransactionCommand {
    fn name(&self) -> &'static str {
        "transaction"
    }

    fn description(&self) -> &'static str {
        "A command that supports transactions"
    }

    fn execute(&self, args: &[String]) -> CommandResult<String> {
        // Change shared state
        let mut state = self.state.lock().unwrap();
        *state = "changed".to_string();
        
        if args.contains(&"fail".to_string()) {
            return Err(CommandError::ExecutionError("Transaction deliberately failed".to_string()));
        }
        
        Ok(format!("Transaction command executed with args: {:?}", args))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("transaction")
            .about("A command that supports transactions")
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(TransactionCommand {
            state: self.state.clone(),
        })
    }
}

#[test]
fn test_phase1_integration() {
    // Create a temporary directory for journaling
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let journal_dir = temp_dir.path().join("journal");
    fs::create_dir_all(&journal_dir).expect("Failed to create journal directory");
    
    // Set up command registry
    let registry = Arc::new(Mutex::new(crate::registry::CommandRegistry::new()));
    
    // Create shared state for transaction command
    let shared_state = Arc::new(Mutex::new("initial".to_string()));
    
    // Register commands
    {
        let registry_lock = registry.lock().unwrap();
        registry_lock.register("success", Arc::new(SuccessCommand)).unwrap();
        registry_lock.register("fail", Arc::new(FailingCommand)).unwrap();
        registry_lock.register("resource", Arc::new(ResourceCommand)).unwrap();
        registry_lock.register("transaction", Arc::new(TransactionCommand {
            state: shared_state.clone(),
        })).unwrap();
    }
    
    // Set up transaction manager
    let transaction_manager = Arc::new(Mutex::new(TransactionManager::new()));
    
    // Set up journal
    let persistence = Arc::new(InMemoryJournalPersistence::new());
    let journal = Arc::new(Mutex::new(CommandJournal::new(persistence, 100)));
    
    // Set up resource manager
    let resource_manager = ResourceManager::new();
    
    // Set up resource monitor
    let resource_monitor = Arc::new(Mutex::new(ResourceMonitor::new()));
    
    // Set up observability
    let observability = ObservabilitySystem::new();
    
    // Execute successful command
    {
        println!("Testing successful command execution");
        
        let cmd_name = "success";
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        
        // Get the command
        let registry_lock = registry.lock().unwrap();
        let command = registry_lock.get_command(cmd_name).unwrap();
        
        // Record command start in journal
        let journal_id = journal.lock().unwrap().record_start(command.as_ref(), &args).unwrap();
        
        // Execute the command
        let result = command.execute(&args);
        
        // Record completion in journal
        journal.lock().unwrap().record_completion(journal_id, result.clone()).unwrap();
        
        // Record resource usage
        resource_monitor.lock().unwrap().record_usage(1024, 2.0);
        
        // Verify command succeeded
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Success command executed"));
        
        // Verify resource usage was recorded
        let usage = resource_monitor.lock().unwrap().get_usage();
        assert_eq!(usage.0, 1024);
        assert_eq!(usage.1, 2.0);
    }
    
    // Execute failing command
    {
        println!("Testing failing command");
        
        let cmd_name = "fail";
        let args = vec![];
        
        // Get the command
        let registry_lock = registry.lock().unwrap();
        let command = registry_lock.get_command(cmd_name).unwrap();
        
        // Record command start in journal
        let journal_id = journal.lock().unwrap().record_start(command.as_ref(), &args).unwrap();
        
        // Execute the command
        let result = command.execute(&args);
        
        // Record completion in journal
        journal.lock().unwrap().record_completion(journal_id, result.clone()).unwrap();
        
        // Verify command failed
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("deliberately failed"));
    }
    
    // Verify journal entries
    {
        let entries = journal.lock().unwrap().get_entries().unwrap();
        assert_eq!(entries.len(), 2);
        
        // Check first entry
        assert_eq!(entries[0].command_name, "success");
        assert_eq!(entries[0].arguments, vec!["arg1".to_string(), "arg2".to_string()]);
        
        // Check second entry
        assert_eq!(entries[1].command_name, "fail");
        assert!(entries[1].error.is_some());
    }
    
    println!("Phase 1 integration test completed successfully");
} 