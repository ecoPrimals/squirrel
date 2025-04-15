use clap::Parser;
use squirrel_mcp::generated::mcp_task::task_service_server::TaskServiceServer;
use squirrel_mcp::task::{TaskManager, server::TaskServiceImpl};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::time::Duration;
use tonic::transport::Server;
use tracing::{info, error, debug, Level};
use tracing_subscriber::FmtSubscriber;

// Import the CLI crate
#[cfg(all(feature = "command-registry", not(test)))]
use squirrel_cli;

// Conditionally import the command registry
#[cfg(all(feature = "command-registry", not(test)))]
use squirrel_commands::{Command, CommandRegistry};
#[cfg(all(feature = "command-registry", not(test)))]
use std::sync::Mutex;

// Define command-line options
#[derive(Parser, Debug)]
#[clap(name = "task_server", about = "MCP Task Management Server")]
struct Opt {
    /// Socket address to listen on
    #[clap(short, long, default_value = "[::1]:50052")]
    address: String,

    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,

    /// Maximum concurrent connections
    #[clap(long, default_value = "100")]
    max_connections: usize,

    /// Request timeout in seconds
    #[clap(long, default_value = "30")]
    request_timeout: u64,
    
    /// Enable test mode (uses mock command registry)
    #[clap(long)]
    test_mode: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let opt = Opt::parse();

    // Initialize tracing
    let level = if opt.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    // Parse socket address
    let addr: SocketAddr = match opt.address.parse() {
        Ok(addr) => addr,
        Err(e) => {
            error!("Failed to parse address '{}': {}", opt.address, e);
            return Err(e.into());
        }
    };

    info!("Starting MCP Task Server on {}", addr);
    debug!("Server options: {:?}", opt);

    // Create task manager
    let task_manager = Arc::new(TaskManager::new());
    
    // Create service handler based on mode
    let task_service = if opt.test_mode {
        info!("Using mock command registry (test mode)");
        TaskServiceImpl::new(task_manager).with_mock_command_registry()
    } else {
        // Use command registry if feature is enabled
        #[cfg(feature = "command-registry")]
        {
            // Create and initialize command registry
            let command_registry = create_command_registry()?;
            // Create service handler with command registry
            TaskServiceImpl::new(task_manager)
                .with_command_registry(command_registry.clone())
        }
        
        #[cfg(not(feature = "command-registry"))]
        {
            TaskServiceImpl::new(task_manager)
        }
    };
    
    // Build the service
    let mut server = Server::builder()
        .timeout(Duration::from_secs(opt.request_timeout))
        .concurrency_limit_per_connection(opt.max_connections);
    
    // Build and serve the service with graceful shutdown
    let serve_future = server
        .add_service(TaskServiceServer::new(task_service))
        .serve_with_shutdown(addr, async {
            // Listen for Ctrl+C
            match signal::ctrl_c().await {
                Ok(()) => {
                    info!("Received shutdown signal, stopping server gracefully...");
                },
                Err(err) => {
                    error!("Failed to listen for shutdown signal: {}", err);
                }
            }
        });
    
    info!("MCP Task Server is ready to accept connections");
    
    // Start the server
    if let Err(e) = serve_future.await {
        error!("Server error: {}", e);
        return Err(e.into());
    }
    
    info!("MCP Task Server shut down successfully");
    Ok(())
}

// Helper function to create and populate a command registry
#[cfg(all(feature = "command-registry", not(test)))]
fn create_command_registry() -> Result<Arc<Mutex<CommandRegistry>>, Box<dyn std::error::Error>> {
    let registry = Arc::new(Mutex::new(CommandRegistry::new()));
    
    // Register built-in commands
    {
        let mut reg = registry.lock().unwrap();
        
        // Register at least one test command for integration tests
        reg.register(Box::new(TestCommand::new()))?;
        
        // Register basic commands directly without depending on CLI
        // This is a lighter approach than using the CLI crate directly
        reg.register(Box::new(EchoCommand::new()))?;
        
        // Add more built-in commands as needed
        // Each command is self-contained without external dependencies
    }
    
    info!("Command registry initialized with essential commands");
    Ok(registry)
}

// Test command implementation for integration tests
#[cfg(all(feature = "command-registry", not(test)))]
struct TestCommand;

#[cfg(all(feature = "command-registry", not(test)))]
impl TestCommand {
    fn new() -> Self {
        Self {}
    }
}

#[cfg(all(feature = "command-registry", not(test)))]
impl Command for TestCommand {
    fn name(&self) -> &str {
        "test_command"
    }
    
    fn description(&self) -> &str {
        "Test command for integration tests"
    }
    
    fn execute(&self, args: &[String]) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!("Test command executed with args: {:?}", args))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("test_command")
            .about("Test command for integration tests")
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(TestCommand)
    }
}

// New command implementation
#[cfg(all(feature = "command-registry", not(test)))]
struct EchoCommand;

#[cfg(all(feature = "command-registry", not(test)))]
impl EchoCommand {
    fn new() -> Self {
        Self {}
    }
}

#[cfg(all(feature = "command-registry", not(test)))]
impl Command for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }
    
    fn description(&self) -> &str {
        "Echo back the input"
    }
    
    fn execute(&self, args: &[String]) -> Result<String, Box<dyn std::error::Error>> {
        Ok(args.join(" "))
    }
    
    fn parser(&self) -> clap::Command {
        clap::Command::new("echo")
            .about("Echo back the input")
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(EchoCommand)
    }
} 