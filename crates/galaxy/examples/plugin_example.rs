use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use anyhow::Result;

use galaxy::{
    create_adapter_with_config,
    create_plugin_manager,
    create_tool_plugin,
    create_workflow_plugin,
    create_dataset_plugin,
    GalaxyConfig,
    GalaxyPlugin,
    GalaxyToolPlugin,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create the Galaxy adapter
    let config = GalaxyConfig::default();
    let adapter = Arc::new(create_adapter_with_config(config)?);
    
    // Create a plugin manager
    let mut plugin_manager = create_plugin_manager(Arc::clone(&adapter));
    
    // Create and register various plugin types
    
    // Tool plugin
    let tool_plugin = create_tool_plugin(
        "BioinformaticsTools",
        "1.0.0",
        "Provides access to common bioinformatics tools"
    );
    plugin_manager.register_plugin(Arc::new(tool_plugin)).await?;
    
    // Workflow plugin
    let workflow_plugin = create_workflow_plugin(
        "GenomicsWorkflows",
        "1.0.0",
        "Provides access to genomics analysis workflows"
    )
    .with_workflow(json!({
        "id": "genome-assembly",
        "name": "Genome Assembly Workflow",
        "description": "Assembles a genome from raw sequencing data"
    }));
    plugin_manager.register_plugin(Arc::new(workflow_plugin)).await?;
    
    // Dataset plugin
    let dataset_plugin = create_dataset_plugin(
        "ReferenceDatasets",
        "1.0.0",
        "Provides access to reference genomes and datasets"
    );
    plugin_manager.register_plugin(Arc::new(dataset_plugin)).await?;
    
    // Query for plugins by capability
    println!("Finding plugins with 'galaxy-tool' capability...");
    let tool_plugins = plugin_manager.get_plugins_by_capability("galaxy-tool");
    println!("Found {} tool plugins", tool_plugins.len());
    
    for plugin in tool_plugins {
        println!("Tool plugin: {} v{}", plugin.name(), plugin.version());
    }
    
    // Get all plugins
    println!("\nAll registered plugins:");
    let all_plugins = plugin_manager.get_all_plugins();
    for plugin in all_plugins {
        println!("{} v{} - {}", plugin.name(), plugin.version(), plugin.description());
    }
    
    // Using a tool plugin
    if let Some(tool_plugin) = plugin_manager.get_plugin("BioinformaticsTools") {
        // Try to dynamically cast to a tool plugin
        if let Some(tool_plugin) = tool_plugin.as_any().downcast_ref::<dyn GalaxyToolPlugin>() {
            println!("\nExecuting a tool through the plugin...");
            let params = HashMap::new();
            let job_id = tool_plugin.execute_tool("fastqc", params).await?;
            println!("Tool execution started with job ID: {}", job_id);
            
            println!("Checking job status...");
            let status = tool_plugin.get_job_status(&job_id).await?;
            println!("Job status: {}", status);
            
            println!("Getting job results...");
            let results = tool_plugin.get_job_results(&job_id).await?;
            println!("Job results: {}", results);
        }
    }
    
    // Shutdown all plugins
    println!("\nShutting down plugins...");
    plugin_manager.shutdown().await?;
    println!("All plugins shutdown successfully");
    
    Ok(())
} 