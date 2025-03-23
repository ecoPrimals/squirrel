use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use squirrel_monitoring::dashboard::manager::{Manager, Component};
use tokio::signal::ctrl_c;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

const SERVER_ADDR: &str = "[::1]:8765";

// Create a MockManager that implements the Manager trait
#[derive(Debug)]
struct MockManager {
    components: Vec<Component>,
}

#[async_trait]
impl Manager for MockManager {
    async fn get_components(&self) -> Vec<Component> {
        self.components.clone()
    }
    
    async fn get_component_data(&self, _id: &str) -> Option<Value> {
        Some(serde_json::json!({
            "value": 42.0,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }
    
    async fn get_health_status(&self) -> Value {
        serde_json::json!({
            "status": "healthy",
            "components": []
        })
    }
}

impl MockManager {
    fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting dashboard example");
    
    // Create a MockManager that implements the Manager trait
    let manager = Arc::new(MockManager::new());
    
    // Start the dashboard server
    let addr: SocketAddr = SERVER_ADDR.parse()?;
    info!("Starting dashboard server on {}", addr);
    
    // Start the server in a separate task
    let server_manager = Arc::clone(&manager);
    let _server_handle = tokio::spawn(async move {
        if let Err(e) = squirrel_monitoring::dashboard::server::start_server(addr, server_manager).await {
            eprintln!("Server error: {}", e);
        }
    });

    // Wait for Ctrl+C signal
    info!("Dashboard running. Press Ctrl+C to exit");
    ctrl_c().await?;
    info!("Shutting down");

    Ok(())
} 