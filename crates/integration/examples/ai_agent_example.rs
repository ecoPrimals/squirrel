use std::error::Error;
use squirrel_integration::{AIAgentAdapter, AIAgentConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting AI Agent Integration Example");
    
    // Create configuration for the agent
    let config = AIAgentConfig::new("openai", "demo-api-key")
        .with_model("gpt-4")
        .with_timeout(30000);
    
    // Create and initialize adapter
    let mut adapter = AIAgentAdapter::new(config);
    adapter.initialize().await.map_err(|e| Box::<dyn Error>::from(e))?;
    
    println!("AI Agent Adapter initialized");
    
    // Get adapter status
    let status = adapter.get_status().await;
    println!("Adapter status: {:?}", status);
    
    // Generate content
    let prompt = "What are the key components of a resilient system?";
    println!("Generating content for prompt: {}", prompt);
    
    let response = adapter.generate_content(prompt).await
        .map_err(|e| Box::<dyn Error>::from(e))?;
    
    println!("Generated content:\n{}", response);
    
    println!("AI Agent Integration Example completed successfully");
    Ok(())
} 