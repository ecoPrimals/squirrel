//! Main executable for the Squirrel CLI.

use squirrel_core::Core;
use std::env;
use std::process;
use std::time::{Duration, Instant};
use std::collections::HashMap;

// Add log crate for consistent logging
use log::{debug, info, warn, error};

/// Helper struct to track lock timing
struct LockTimer {
    operation: String,
    start_time: Instant,
    warn_threshold: Duration,
}

impl LockTimer {
    fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            start_time: Instant::now(),
            warn_threshold: Duration::from_millis(100), // Warn if lock held for more than 100ms
        }
    }
    
    fn end(self) -> Duration {
        let duration = self.start_time.elapsed();
        debug!("Lock operation '{}' completed in {:?}", self.operation, duration);
        
        if duration > self.warn_threshold {
            warn!("Lock operation '{}' took {:?} - potential contention", self.operation, duration);
        }
        
        duration
    }
}

#[tokio::main]
async fn main() {
    // Initialize simple logger if not already initialized
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap_or_else(|_| {
            println!("Logger already initialized or failed to initialize");
        });

    info!("Squirrel CLI starting");
    println!("Squirrel CLI");
    
    // Run the application
    println!("Squirrel Core v{}", squirrel_core::build_info::version());
    
    // Create a core instance
    let _core = Core::new();
    info!("Core instance created");

    println!("Initializing command registry...");
    // Create a new command registry with built-in commands
    let registry = match squirrel_commands::create_command_registry() {
        Ok(registry) => {
            info!("Command registry initialized successfully");
            registry
        },
        Err(e) => {
            error!("Error initializing command registry: {}", e);
            eprintln!("Error initializing command registry: {}", e);
            process::exit(1);
        }
    };

    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    debug!("Command line arguments: {:?}", args);

    // If no command is provided, show usage
    if args.len() < 2 {
        println!("Usage: {} <command> [args...]", args[0]);
        println!("\nAvailable commands:");
        
        // ---- OPTIMIZATION: Batch operations that require locks ----
        // Get all command information in a single lock operation instead of 
        // locking for each command individually
        let commands_with_help = {
            debug!("Acquiring registry lock to get all commands and help");
            let timer = LockTimer::new("list_commands_and_help");
            
            // Lock the registry mutex once
            let registry_guard = match registry.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    error!("Error locking command registry: {}", e);
                    eprintln!("Error locking command registry: {}", e);
                    process::exit(1);
                }
            };
            
            // First get the list of commands
            let commands = match registry_guard.list_commands() {
                Ok(cmds) => cmds,
                Err(e) => {
                    error!("Error listing commands: {}", e);
                    eprintln!("Error listing commands: {}", e);
                    process::exit(1);
                }
            };
            
            // Then collect help for each command while still holding the lock
            let mut help_map = HashMap::new();
            for cmd in &commands {
                match registry_guard.get_help(cmd) {
                    Ok(help) => {
                        help_map.insert(cmd.clone(), help);
                    },
                    Err(e) => {
                        warn!("Error getting help for command {}: {}", cmd, e);
                        help_map.insert(cmd.clone(), format!("{}: <error retrieving help>", cmd));
                    }
                }
            }
            
            // Record timing metrics
            let duration = timer.end();
            info!("Retrieved help for {} commands in {:?}", commands.len(), duration);
            
            // Return both the commands list and help map
            (commands, help_map)
        }; // Registry is unlocked here
        
        // Now display the help without holding any locks
        let (commands, help_map) = commands_with_help;
        for cmd in commands {
            println!("  {}", help_map.get(&cmd).unwrap_or(&format!("{}: No help available", cmd)));
        }
        
        info!("Help display completed, exiting");
        process::exit(1);
    }

    // Execute the command
    let command_name = &args[1];
    let command_args = &args[2..].to_vec();

    info!("Executing command: {} with args: {:?}", command_name, command_args);
    println!("Executing command: {} with args: {:?}", command_name, command_args);
    
    // IMPORTANT: Get the command first, then release the lock before executing
    let result = {
        debug!("Acquiring registry lock to execute command");
        let timer = LockTimer::new(format!("execute_{}", command_name).as_str());
        
        println!("Locking registry to execute command...");
        // Lock the registry mutex before executing
        let registry_guard = match registry.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Error locking command registry: {}", e);
                eprintln!("Error locking command registry: {}", e);
                process::exit(1);
            }
        };

        let result = registry_guard.execute(command_name, command_args);
        timer.end();
        result
    }; // Registry is unlocked here

    match result {
        Ok(output) => {
            info!("Command '{}' executed successfully", command_name);
            println!("{}", output);
            process::exit(0);
        },
        Err(e) => {
            error!("Error executing command '{}': {}", command_name, e);
            eprintln!("Error executing command: {}", e);
            process::exit(1);
        }
    }
} 