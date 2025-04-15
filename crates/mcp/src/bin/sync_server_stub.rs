use clap::Parser;
use std::net::SocketAddr;
use tokio::signal;
use tokio::time::Duration;
use tracing::{info, error, debug, Level};
use tracing_subscriber::FmtSubscriber;

/// Define command-line options
#[derive(Parser, Debug)]
#[clap(name = "sync_server_stub", about = "MCP Sync Server Stub")]
struct Opt {
    /// Socket address to listen on
    #[clap(short, long, default_value = "[::1]:50051")]
    address: String,

    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,
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

    info!("Starting MCP Sync Server Stub on {}", addr);
    debug!("Server options: {:?}", opt);

    // This is a stub implementation that doesn't actually serve anything
    info!("!!! STUB IMPLEMENTATION - NOT A REAL SERVER !!!");
    info!("This is a placeholder for the real sync server");

    // Wait for Ctrl+C to exit
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received shutdown signal, stopping server gracefully...");
        },
        Err(err) => {
            error!("Failed to listen for shutdown signal: {}", err);
        }
    }
    
    info!("MCP Sync Server Stub shut down successfully");
    Ok(())
} 