use std::sync::Arc;
use std::time::Duration;
use std::net::SocketAddr;
use std::process::{Child, Command, Stdio};

use reqwest;
use serde_json::json;
use tokio::time::sleep;
use uuid;

use squirrel_web::{config::Config, create_app};
// We might need helpers to start/manage the MCP gRPC server process if it's separate
// use some_mcp_server_runner; 

// Helper function to find the workspace root
fn find_workspace_root() -> std::path::PathBuf {
    let mut current_dir = std::env::current_dir().expect("Cannot get current directory");
    loop {
        if current_dir.join("Cargo.toml").exists() {
            let content = std::fs::read_to_string(current_dir.join("Cargo.toml")).unwrap_or_default();
            if content.contains("[workspace]") {
                return current_dir;
            }
        }
        if !current_dir.pop() {
            panic!("Could not find workspace root");
        }
    }
}

// Helper to spawn the MCP gRPC server (task_server) in the background
async fn spawn_mcp_server() -> Result<(Child, SocketAddr), std::io::Error> {
    // Find a free port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    drop(listener); // Drop the listener so the port can be used by the server

    let addr_string = addr.to_string();
    println!("Attempting to start MCP server on: {}", addr_string);

    // Build the command
    let workspace_root = find_workspace_root();
    let mut command = Command::new("cargo");
    command
        .current_dir(workspace_root) // Ensure cargo runs from the workspace root
        .arg("run")
        .arg("-p")
        .arg("squirrel-mcp")
        .arg("--bin")
        .arg("task_server")
        .arg("--") // Pass arguments to the binary, not cargo
        .arg("--address")
        .arg(&addr_string)
        .stdout(Stdio::piped()) // Capture stdout/stderr for debugging if needed
        .stderr(Stdio::piped());

    // Spawn the process
    let child = command.spawn()
        .map_err(|e| {
            eprintln!("Failed to spawn MCP server: {}. Command: {:?}", e, command);
            e
        })?;

    // TODO: Add a check to ensure the server started successfully (e.g., wait for a log message or try connecting)
    sleep(Duration::from_secs(2)).await; // Simple delay for now
    println!("MCP server process presumably started (PID: {})", child.id());

    Ok((child, addr))
}

// Create our own mock pool function since create_mock_pool doesn't exist
async fn create_mock_db_pool() -> squirrel_web::db::SqlitePool {
    // This is a placeholder. Create an in-memory SQLite database for testing
    squirrel_web::db::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database")
}

// Helper to spawn the web server in the background
async fn spawn_app(mcp_host: String, mcp_port: u16) -> String {
    // Create a test-specific configuration pointing to the test MCP server
    let test_mcp_config = squirrel_web::mcp::McpClientConfig {
        host: mcp_host,
        port: mcp_port,
        ..Default::default() // Use defaults for timeout, retries etc.
    };

    let config = squirrel_web::config::Config {
        mcp: test_mcp_config,
        ..Default::default() // Use defaults for api_base_url, request_timeout etc.
    };
    
    // Use our own mock pool instead of create_mock_pool
    let db_pool = create_mock_db_pool().await; 
    let app = create_app(db_pool, config).await;
    
    // Add the skip auth middleware to bypass authentication for tests
    let app = app.layer(axum::middleware::from_fn(skip_auth_for_tests));

    // Use a random available port for the web server itself
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let addr = listener.local_addr().unwrap();
    let server_url = format!("http://{}", addr);

    // Updated to use the newer axum API
    tokio::spawn(async move {
        // We need to convert the tokio listener to std listener
        let std_listener = listener
            .into_std()
            .expect("Failed to convert tokio::net::TcpListener to std::net::TcpListener");
            
        // Use Server::from_tcp instead of axum::serve
        axum::Server::from_tcp(std_listener)
            .expect("Failed to create server from TCP listener")
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    // Give the server a moment to start
    sleep(Duration::from_millis(100)).await;

    server_url
}

// Middleware function to skip authentication for tests
async fn skip_auth_for_tests<B>(
    mut request: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> axum::response::Response {
    // Add test auth claims to the request extensions
    use squirrel_web::auth::extractor::AuthClaims;
    use uuid::Uuid;
    
    // Create test auth claims with a fixed UUID for consistency across requests
    let claims = AuthClaims {
        sub: "test-user-id".to_string(), // Using a fixed ID for all tests
        iat: chrono::Utc::now().timestamp(),
        exp: chrono::Utc::now().timestamp() + 3600, // 1 hour from now
        roles: vec!["admin".to_string()],
    };
    
    // Insert the claims into the request extensions
    request.extensions_mut().insert(claims);
    
    // Continue to the next middleware or handler
    next.run(request).await
}

// Helper to get a test auth header for requests
fn get_test_auth_header() -> (&'static str, String) {
    // For tests, we're using a simple bearer token since the middleware will handle authentication
    ("Authorization", "Bearer test-token".to_string())
}

#[tokio::test]
async fn test_execute_mcp_command_via_web_api() {
    // Start the MCP gRPC server and get its host/port
    let (mut mcp_server_process, mcp_addr) = spawn_mcp_server()
        .await
        .expect("Failed to start MCP server");
    let mcp_host = mcp_addr.ip().to_string();
    let mcp_port = mcp_addr.port();

    // Start the web server, configured to connect to the test MCP server
    let server_url = spawn_app(mcp_host, mcp_port).await;
    let client = reqwest::Client::new();

    // Define the command payload for creating a task
    let task_name = format!("Integration Test Task - {}", uuid::Uuid::new_v4());
    let command_payload = json!({
        "command": task_name, // Use task name as the command
        "parameters": { // Parameters map to CreateTaskRequest.metadata (encoded as bytes)
            "source": "web_integration_test",
            "test_run_id": uuid::Uuid::new_v4().to_string()
        }
    });

    // Send request to the web server's command execution endpoint
    // TODO: Update endpoint path if needed (check api/router.rs)
    let response = client
        .post(format!("{}/api/commands", server_url)) // Placeholder path
        .header(get_test_auth_header().0, get_test_auth_header().1)
        .json(&command_payload)
        .send()
        .await
        .expect("Failed to send request");

    // Assertions
    assert!(response.status().is_success(), "Request failed: {:?}", response.text().await);

    let response_body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse response JSON");
        
    // Verify the API response structure
    assert_eq!(response_body["success"], json!(true));
    assert!(response_body["data"].is_object());
    assert!(response_body["error"].is_null());
    
    // Verify the CreateCommandResponse fields
    let data = &response_body["data"];
    assert!(data["id"].is_string() && !data["id"].as_str().unwrap().is_empty());
    assert_eq!(data["command"], command_payload["command"]); // Command name should match task name
    
    // Add debugging to see what status we got
    println!("Command status received: {:?}", data["status"]);
    
    // Case-insensitive check for status being queued or running
    assert!(
        data["status"].as_str().map_or(false, |s| s.to_lowercase() == "queued" || s.to_lowercase() == "running"),
        "Initial status should be queued or running, got: {:?}", 
        data["status"]
    );
    assert!(data["status_url"].is_string());

    // TODO: Optionally, query the status endpoint using the command_id
    // let command_id = data["id"].as_str().unwrap();

    // Shutdown the MCP gRPC server
    println!("Stopping MCP server process...");
    mcp_server_process.kill().expect("Failed to kill MCP server process");
    mcp_server_process.wait().expect("Failed to wait for MCP server process");
    println!("MCP server process stopped.");
} 