use squirrel_mcp::sync::state::{StateChange, StateOperation};
use squirrel_mcp::generated::mcp_sync::sync_service_client::SyncServiceClient;
use squirrel_mcp::generated::mcp_sync::{ContextChange, SyncRequest};
use structopt::StructOpt;
use std::time::Duration;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error};
use tokio::time::sleep;

#[derive(StructOpt, Debug)]
#[structopt(name = "sync-test-cli")]
struct Opt {
    /// Server URL to connect to
    #[structopt(short, long, default_value = "http://[::1]:50051")]
    server: String,

    /// Client ID to use (will be generated if not provided)
    #[structopt(short, long)]
    client_id: Option<String>,

    /// Operation to perform (create, update, delete, get)
    #[structopt(short, long, default_value = "create")]
    operation: String,
    
    /// Context ID to operate on (required for update/delete/get)
    #[structopt(short, long)]
    context_id: Option<String>,
    
    /// Context name (for create/update)
    #[structopt(short, long, default_value = "Test Context")]
    name: String,
    
    /// Number of test operations to perform
    #[structopt(short, long, default_value = "1")]
    count: usize,
    
    /// Wait seconds between operations
    #[structopt(short, long, default_value = "1")]
    wait: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let opt = Opt::from_args();
    
    // Generate a client ID if not provided
    let client_id = opt.client_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    println!("Sync Test CLI");
    println!("-------------");
    println!("Server: {}", opt.server);
    println!("Client ID: {}", client_id);
    println!("Operation: {}", opt.operation);
    println!("Context ID: {}", opt.context_id.as_deref().unwrap_or("Auto-generated"));
    println!("Context Name: {}", opt.name);
    println!("Operation Count: {}", opt.count);
    println!("");
    
    // Connect to the server
    info!("Connecting to server at {}", opt.server);
    let channel = tonic::transport::Channel::from_shared(opt.server.clone())?
        .timeout(Duration::from_secs(5))
        .connect()
        .await?;
    
    let mut client = SyncServiceClient::new(channel);
    info!("Connected to server");
    
    // Determine operation type
    let operation_type = match opt.operation.to_lowercase().as_str() {
        "create" => 1, // Create
        "update" => 2, // Update
        "delete" => 3, // Delete
        _ => {
            error!("Unknown operation: {}", opt.operation);
            return Err("Unknown operation".into());
        }
    };
    
    // Perform the requested operations
    for i in 0..opt.count {
        // Generate or use the provided context ID
        let context_id = match (operation_type, &opt.context_id) {
            (1, None) => Uuid::new_v4().to_string(), // Create - generate ID
            (_, Some(id)) => id.clone(),           // Use provided ID
            (_, None) => {
                error!("Context ID is required for update/delete operations");
                return Err("Context ID required".into());
            }
        };
        
        println!("\nOperation {}/{}", i + 1, opt.count);
        println!("Context ID: {}", context_id);
        
        // Create test data
        let test_data = serde_json::json!({
            "source": "sync_test_cli",
            "timestamp": Utc::now().to_string(),
            "iteration": i + 1,
            "data": {
                "test": true,
                "value": format!("test-{}", i)
            }
        });
        
        // Serialize test data to bytes
        let data_bytes = serde_json::to_vec(&test_data)?;
        
        // Create a context change
        let context_change = ContextChange {
            operation_type,
            context_id: context_id.clone(),
            name: format!("{} {}", opt.name, i + 1),
            parent_id: String::new(),
            created_at_unix_secs: Utc::now().timestamp(),
            updated_at_unix_secs: Utc::now().timestamp(),
            data: data_bytes,
            metadata: Vec::new(),
        };
        
        // Create the sync request
        let request = SyncRequest {
            client_id: client_id.clone(),
            last_known_version: 0,
            local_changes: vec![context_change],
        };
        
        // Send the request
        info!("Sending sync request for context {}", context_id);
        match client.sync(request).await {
            Ok(response) => {
                let response = response.into_inner();
                println!("Sync response:");
                println!("  Success: {}", response.success);
                println!("  Server version: {}", response.current_server_version);
                println!("  Remote changes: {}", response.remote_changes.len());
                
                if !response.remote_changes.is_empty() {
                    println!("Remote changes received:");
                    for (idx, change) in response.remote_changes.iter().enumerate() {
                        println!("  Change {}:", idx + 1);
                        println!("    ID: {}", change.context_id);
                        println!("    Operation: {}", change.operation_type);
                        println!("    Name: {}", change.name);
                        
                        // Try to decode data if present
                        if !change.data.is_empty() {
                            match serde_json::from_slice::<serde_json::Value>(&change.data) {
                                Ok(data) => println!("    Data: {}", data),
                                Err(_) => println!("    Data: {} bytes (not JSON)", change.data.len()),
                            }
                        }
                    }
                }
            },
            Err(err) => {
                error!("Sync request failed: {}", err);
                println!("Error: {}", err);
            }
        }
        
        // Wait between operations
        if i < opt.count - 1 && opt.wait > 0 {
            println!("Waiting {} seconds...", opt.wait);
            sleep(Duration::from_secs(opt.wait)).await;
        }
    }
    
    println!("\nTest completed");
    Ok(())
} 