use squirrel_galaxy::{create_adapter_with_config, GalaxyConfig, Error};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create a config for the Galaxy instance
    let config = GalaxyConfig::new("https://usegalaxy.org/api")
        .with_api_key("YOUR_API_KEY"); // Replace with your actual API key
    
    // Create an adapter with the specified configuration
    let adapter = create_adapter_with_config(config).await?;
    
    // First, create a history to store our results
    println!("Creating a new history...");
    let history = adapter.create_history().await?;
    println!("Created history: {} (ID: {})\n", history.metadata.name, history.metadata.id);
    
    // For this example, we'll use a simple text manipulation tool: 'Cut' tool
    let tool_id = "Cut1";
    
    // Get details of the tool to understand its parameters
    println!("Getting details for tool: {}", tool_id);
    let tool = adapter.get_tool(tool_id).await?;
    println!("Tool name: {}", tool.metadata.name);
    if let Some(desc) = &tool.metadata.description {
        println!("Description: {}", desc);
    }
    println!("Input parameters:");
    for param in &tool.inputs {
        println!("  {} ({}): {}", 
            param.name, 
            param.type_name,
            if param.required { "Required" } else { "Optional" }
        );
    }
    println!();
    
    // For this example, we'll create a simple input dataset with some text
    println!("Uploading test dataset...");
    let content = "Column1\tColumn2\tColumn3\nValue1\tValue2\tValue3\nValue4\tValue5\tValue6";
    let dataset = adapter.upload_dataset(
        "test_data.txt", 
        content.as_bytes().to_vec(),
        "tabular",
        Some(&history.metadata.id)
    ).await?;
    println!("Uploaded dataset: {} (ID: {})\n", dataset.metadata.name, dataset.metadata.id);
    
    // Now let's prepare parameters for the cut tool
    // We want to cut out column 2
    let mut parameters = HashMap::new();
    parameters.insert("input".to_string(), squirrel_galaxy::models::ParameterValue::String(dataset.metadata.id.clone()));
    parameters.insert("columnList".to_string(), squirrel_galaxy::models::ParameterValue::String("2".to_string()));
    parameters.insert("delimiter".to_string(), squirrel_galaxy::models::ParameterValue::String("T".to_string()));
    
    // Execute the tool
    println!("Executing tool: {}", tool_id);
    let job_id = adapter.execute_tool(tool_id, &parameters).await?;
    println!("Tool execution started. Job ID: {}\n", job_id);
    
    // Poll for job completion
    println!("Waiting for job to complete...");
    loop {
        let status = adapter.get_job_status(&job_id).await?;
        println!("Job status: {:?}", status);
        
        if status.is_terminal() {
            if status.is_successful() {
                println!("Job completed successfully!\n");
                break;
            } else {
                return Err(Error::ToolExecution(format!("Job failed with status: {:?}", status)));
            }
        }
        
        // Wait a bit before checking again
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    
    // Retrieve the results
    println!("Retrieving job results...");
    let outputs = adapter.get_job_results(&job_id).await?;
    println!("Job produced {} outputs:", outputs.len());
    
    for (i, output) in outputs.iter().enumerate() {
        println!("{}. Output name: {}, Dataset ID: {}", i + 1, output.name, output.id);
        
        // Download the output content
        let content = adapter.download_dataset(&output.id).await?;
        println!("   Content ({} bytes):", content.len());
        println!("   {}", String::from_utf8_lossy(&content[..content.len().min(200)]));
        if content.len() > 200 {
            println!("   ... (truncated)");
        }
        println!();
    }
    
    println!("Example completed successfully!");
    Ok(())
} 