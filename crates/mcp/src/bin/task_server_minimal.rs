use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::time::Duration;
use tonic::transport::Server;
use tracing::{info, error, debug, Level};
use tracing_subscriber::FmtSubscriber;

use squirrel_mcp::generated::mcp_task::task_service_server::TaskServiceServer;
use squirrel_mcp::task::{TaskManager, server::TaskServiceImpl};

// Define command-line options
#[derive(Parser, Debug)]
#[clap(name = "task_server_minimal", about = "MCP Task Management Server (Minimal Version)")]
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

    info!("Starting MCP Task Server (Minimal) on {}", addr);
    debug!("Server options: {:?}", opt);

    // Create TaskManager
    let task_manager = Arc::new(TaskManager::new());
    
    // Create gRPC service handler
    let task_service = TaskServiceImpl::new(task_manager);
    
    // Build and serve the service with graceful shutdown
    let serve_future = Server::builder()
        .timeout(Duration::from_secs(opt.request_timeout))
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