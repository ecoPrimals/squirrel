use clap::Parser;
use squirrel_mcp::generated::mcp_sync::sync_service_server::SyncServiceServer;
use squirrel_mcp::sync::server::{SyncService, MCPSyncServer};
use squirrel_mcp::sync::state::StateSyncManager;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::time::Duration;
use tonic::transport::Server;
use tracing::{info, error, debug, Level};
use tracing_subscriber::FmtSubscriber;

// Define command-line options
#[derive(Parser, Debug)]
#[clap(name = "sync_server", about = "MCP Sync Server")]
struct Opt {
    /// Socket address to listen on
    #[clap(short, long, default_value = "[::1]:50051")]
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

/// Wrapper for the SyncService to handle incoming gRPC requests
#[derive(Debug)]
struct GrpcSyncServer {
    /// The actual implementation of the SyncService
    inner: MCPSyncServer,
}

#[tonic::async_trait]
impl squirrel_mcp::generated::mcp_sync::sync_service_server::SyncService for GrpcSyncServer {
    async fn sync(
        &self,
        request: tonic::Request<squirrel_mcp::generated::mcp_sync::SyncRequest>,
    ) -> Result<tonic::Response<squirrel_mcp::generated::mcp_sync::SyncResponse>, tonic::Status> {
        // Delegate to our implementation
        self.inner.sync(request).await
    }
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

    info!("Starting MCP Sync Server on {}", addr);
    debug!("Server options: {:?}", opt);

    // Create StateSyncManager
    let state_sync_manager = Arc::new(StateSyncManager::new());
    
    // Create and configure MCPSyncServer
    let sync_server = MCPSyncServer::new(state_sync_manager);
    
    // Create gRPC service handler
    let grpc_service = GrpcSyncServer {
        inner: sync_server,
    };
    
    // Build the service
    let mut server = Server::builder()
        .timeout(Duration::from_secs(opt.request_timeout))
        .concurrency_limit_per_connection(opt.max_connections);
    
    // Build and serve the service with graceful shutdown
    let serve_future = server
        .add_service(SyncServiceServer::new(grpc_service))
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
    
    info!("MCP Sync Server is ready to accept connections");
    
    // Start the server
    if let Err(e) = serve_future.await {
        error!("Server error: {}", e);
        return Err(e.into());
    }
    
    info!("MCP Sync Server shut down successfully");
    Ok(())
} 