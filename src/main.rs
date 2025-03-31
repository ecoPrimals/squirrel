//! Main entry point for the Squirrel application

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, error};
use squirrel_interfaces::plugins::PluginRegistry;
use squirrel_mcp::plugins::PluginManager;
use dashboard_core::config::DashboardConfig;
use std::sync::Arc;

// Conditional imports for TUI
#[cfg(feature = "tui")]
use dashboard_core::service::{DashboardService, DefaultDashboardService};
#[cfg(feature = "tui")]
use ui_terminal;
#[cfg(feature = "tui")]
use std::sync::Mutex;

mod plugins;
mod adapter;
// mod lib; // Removed as src/lib/ directory was deleted during refactoring

/// Squirrel: Your versatile data management and monitoring tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // Add other global options here if needed
}

#[derive(Parser, Debug)]
enum Commands {
    /// Launch the Terminal User Interface (TUI)
    Tui,
    // Add other subcommands like 'run', 'config', etc. here
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();

    // Handle commands
    match cli.command {
        Some(Commands::Tui) => {
            #[cfg(feature = "tui")]
            {
                info!("Launching TUI...");
                // --- Placeholder for actual TUI launch ---
                // TODO: Replace this with the actual call to ui_terminal::run_ui
                // For now, let's simulate using the DefaultDashboardService like in the example
                let (dashboard_service, _) = DefaultDashboardService::new(DashboardConfig::default());
                let dashboard_service = Arc::new(Mutex::new(dashboard_service));
                let data_update_interval_ms = 1000; // Example: update data every 1 second
                let ui_refresh_rate_ms = 50;       // Example: refresh UI every 50ms

                // The actual call would look something like this:
                // if let Err(e) = ui_terminal::run_ui(dashboard_service, data_update_interval_ms, ui_refresh_rate_ms).await {
                //     error!("TUI error: {}", e);
                // }
                println!("TUI placeholder: Would launch TUI here using DefaultDashboardService.");
                println!("Run the example directly for now: cargo run --example basic_run --package ui-terminal")
                // --- End Placeholder ---
            }
            #[cfg(not(feature = "tui"))]
            {
                error!("TUI feature is not enabled in this build.");
            }
        }
        None => {
            // Default behavior if no subcommand is given
            info!("Starting Squirrel application (default mode)");
            
            // Initialize the plugin system
            let plugin_manager = plugins::create_plugin_manager();
            let plugin_dirs = vec![PathBuf::from("./plugins")];
            
            match plugin_manager.initialize(&plugin_dirs).await {
                Ok(plugin_ids) => {
                    info!("Plugin system initialized with {} plugins", plugin_ids.len());
                    
                    // Log loaded plugins
                    for (i, plugin) in plugin_manager.list_plugins().await.iter().enumerate() {
                        let metadata = plugin.metadata();
                        info!(
                            "Plugin {}: {} ({}), capabilities: {:?}",
                            i + 1,
                            metadata.name,
                            metadata.version,
                            metadata.capabilities
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to initialize plugin system: {}", e);
                    // Continue execution even if plugin system fails
                }
            }
            
            // Example: access a plugin by capability
            let registry = plugin_manager.registry();
            if let Some(cmd_plugin) = registry.get_plugin_by_capability("command_execution").await {
                info!(
                    "Found command execution plugin: {}",
                    cmd_plugin.metadata().name
                );
            }
            
            // Shut down the plugin system before exiting
            if let Err(e) = plugin_manager.shutdown().await {
                error!("Error shutting down plugin system: {}", e);
            }
            
            info!("Squirrel application default mode finished.");
            info!("Use 'squirrel --help' to see available commands (like 'tui').");
        }
        // Handle other commands here
    }
    
    Ok(())
}