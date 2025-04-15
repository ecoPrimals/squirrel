use squirrel_mcp::resilience::examples::run_retry_example;

#[tokio::main]
async fn main() {
    println!("Running retry examples...");
    
    match run_retry_example().await {
        Ok(_) => println!("Retry examples completed successfully!"),
        Err(e) => println!("Error running retry examples: {}", e),
    }
} 