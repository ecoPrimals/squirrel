use squirrel_galaxy::{create_adapter_with_config, GalaxyConfig, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create a config for the Galaxy instance
    let config = GalaxyConfig::new("https://usegalaxy.org/api")
        .with_api_key("YOUR_API_KEY"); // Replace with your actual API key
    
    // Create an adapter with the specified configuration
    let adapter = create_adapter_with_config(config).await?;
    
    // List all tools
    println!("Listing Galaxy tools...");
    let tools = adapter.list_tools().await?;
    
    // Output tool information
    println!("Found {} tools:", tools.len());
    for (i, tool) in tools.iter().take(10).enumerate() {
        println!("{}. {} ({})", i + 1, tool.metadata.name, tool.id);
        if let Some(desc) = &tool.metadata.description {
            println!("   Description: {}", desc);
        }
        println!();
    }
    
    if tools.len() > 10 {
        println!("... and {} more tools", tools.len() - 10);
    }
    
    Ok(())
} 