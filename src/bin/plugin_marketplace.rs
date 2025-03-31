// Plugin Marketplace CLI
//
// This is a command-line utility for discovering, downloading, and managing
// plugins from remote repositories. It demonstrates the use of the plugin
// marketplace API.

use clap::{Parser, Subcommand};
use serde_json;
use std::env;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use tokio;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;
use squirrel_mcp::plugins::marketplace::{MarketplaceManager, RepositoryConfig};
use squirrel_mcp::error::Result;

// Import directly from squirrel_plugins
use squirrel_plugins::RepositoryManager;
use squirrel_plugins::HttpRepositoryProvider;
use squirrel_plugins::PluginPackageInfo;
use squirrel_plugins::create_repository_manager;

/// Command-line arguments
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to download directory
    #[clap(short, long, default_value = "./plugins")]
    download_dir: PathBuf,
    
    /// Plugin API version
    #[clap(short, long, default_value = "1.0.0")]
    api_version: String,
    
    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
    
    /// Subcommand
    #[clap(subcommand)]
    command: Commands,
}

/// Subcommands
#[derive(Subcommand)]
enum Commands {
    /// Add repository
    AddRepo {
        /// Repository ID
        #[clap(short, long)]
        id: String,
        
        /// Repository URL
        #[clap(short, long)]
        url: String,
    },
    
    /// Remove repository
    RemoveRepo {
        /// Repository ID
        #[clap(short, long)]
        id: String,
    },
    
    /// List repositories
    ListRepos,
    
    /// List plugins
    ListPlugins,
    
    /// Search plugins
    Search {
        /// Search query
        #[clap(required = true)]
        query: String,
    },
    
    /// Download plugin
    Download {
        /// Repository ID
        #[clap(short, long)]
        repo: String,
        
        /// Plugin ID
        #[clap(short, long)]
        id: String,
    },
    
    /// Get plugin details
    Info {
        /// Repository ID
        #[clap(short, long)]
        repo: String,
        
        /// Plugin ID
        #[clap(short, long)]
        id: String,
    },
}

/// CLI application
struct App {
    /// Repository manager
    manager: Arc<RepositoryManager>,
}

impl App {
    /// Create a new CLI application
    async fn new(download_dir: PathBuf, api_version: &str) -> Result<Self, String> {
        // Create repository manager
        let manager = create_repository_manager(api_version, download_dir)
            .map_err(|e| format!("Failed to create repository manager: {}", e))?;
        
        Ok(Self {
            manager,
        })
    }
    
    /// Run the CLI application
    async fn run(&self, command: Commands) -> Result<(), String> {
        match command {
            Commands::AddRepo { id, url } => {
                self.add_repository(&id, &url).await?;
            }
            
            Commands::RemoveRepo { id } => {
                self.remove_repository(&id).await?;
            }
            
            Commands::ListRepos => {
                self.list_repositories().await?;
            }
            
            Commands::ListPlugins => {
                self.list_plugins().await?;
            }
            
            Commands::Search { query } => {
                self.search_plugins(&query).await?;
            }
            
            Commands::Download { repo, id } => {
                self.download_plugin(&repo, &id).await?;
            }
            
            Commands::Info { repo, id } => {
                self.get_plugin_info(&repo, &id).await?;
            }
        }
        
        Ok(())
    }
    
    /// Add a repository
    async fn add_repository(&self, id: &str, url: &str) -> Result<(), String> {
        info!("Adding repository {} at {}", id, url);
        
        let provider = Arc::new(HttpRepositoryProvider::new(url)
            .map_err(|e| format!("Failed to create repository provider: {}", e))?);
        
        self.manager.add_repository(id, provider)
            .await
            .map_err(|e| format!("Failed to add repository: {}", e))?;
        
        println!("Repository {} added successfully", id);
        Ok(())
    }
    
    /// Remove a repository
    async fn remove_repository(&self, id: &str) -> Result<(), String> {
        info!("Removing repository {}", id);
        
        self.manager.remove_repository(id)
            .await
            .map_err(|e| format!("Failed to remove repository: {}", e))?;
        
        println!("Repository {} removed successfully", id);
        Ok(())
    }
    
    /// List repositories
    async fn list_repositories(&self) -> Result<(), String> {
        info!("Listing repositories");
        
        let repositories = self.manager.get_repositories().await;
        
        if repositories.is_empty() {
            println!("No repositories configured");
            return Ok(());
        }
        
        println!("Repositories:");
        for (id, info) in repositories {
            println!("  {} - {}", id, info.name);
            println!("    URL: {}", info.url);
            println!("    Description: {}", info.description);
            println!("    API Version: {}", info.api_version);
            println!("    Plugin Count: {}", info.plugin_count);
            println!("    Priority: {}", info.priority);
            println!("    Enabled: {}", info.enabled);
            println!();
        }
        
        Ok(())
    }
    
    /// List plugins from all repositories
    async fn list_plugins(&self) -> Result<(), String> {
        info!("Listing plugins");
        
        let plugins = self.manager.list_plugins().await;
        
        if plugins.is_empty() {
            println!("No plugins found");
            return Ok(());
        }
        
        for (repo_id, repo_plugins) in plugins {
            println!("Repository: {}", repo_id);
            
            if repo_plugins.is_empty() {
                println!("  No plugins found");
                continue;
            }
            
            println!("  Plugins:");
            for plugin in repo_plugins {
                self.print_plugin_summary(&plugin);
            }
            
            println!();
        }
        
        Ok(())
    }
    
    /// Search plugins
    async fn search_plugins(&self, query: &str) -> Result<(), String> {
        info!("Searching plugins for: {}", query);
        
        let results = self.manager.search_plugins(query).await;
        
        if results.is_empty() {
            println!("No plugins found for query: {}", query);
            return Ok(());
        }
        
        for (repo_id, repo_plugins) in results {
            println!("Repository: {}", repo_id);
            
            if repo_plugins.is_empty() {
                println!("  No plugins found");
                continue;
            }
            
            println!("  Plugins:");
            for plugin in repo_plugins {
                self.print_plugin_summary(&plugin);
            }
            
            println!();
        }
        
        Ok(())
    }
    
    /// Download a plugin
    async fn download_plugin(&self, repo_id: &str, plugin_id_str: &str) -> Result<(), String> {
        info!("Downloading plugin {} from repository {}", plugin_id_str, repo_id);
        
        let plugin_id = Uuid::parse_str(plugin_id_str)
            .map_err(|e| format!("Invalid plugin ID: {}", e))?;
        
        let plugin_path = self.manager.download_plugin(repo_id, &plugin_id)
            .await
            .map_err(|e| format!("Failed to download plugin: {}", e))?;
        
        println!("Plugin downloaded successfully to: {}", plugin_path.display());
        Ok(())
    }
    
    /// Get plugin information
    async fn get_plugin_info(&self, repo_id: &str, plugin_id_str: &str) -> Result<(), String> {
        info!("Getting plugin info for {} from repository {}", plugin_id_str, repo_id);
        
        let plugin_id = Uuid::parse_str(plugin_id_str)
            .map_err(|e| format!("Invalid plugin ID: {}", e))?;
        
        // Get repositories
        let repositories = self.manager.get_repositories().await;
        
        // Find the repository
        let repo_info = repositories.iter()
            .find(|(id, _)| *id == repo_id)
            .map(|(_, info)| info)
            .ok_or_else(|| format!("Repository not found: {}", repo_id))?;
        
        // Print repository info
        println!("Repository: {} - {}", repo_id, repo_info.name);
        println!("  URL: {}", repo_info.url);
        
        // Find repository provider from the manager
        let repositories = self.manager.list_plugins().await;
        
        // Find the repository
        let repo_plugins = repositories.iter()
            .find(|(id, _)| *id == repo_id)
            .map(|(_, plugins)| plugins)
            .ok_or_else(|| format!("Repository not found: {}", repo_id))?;
        
        // Find the plugin
        let plugin = repo_plugins.iter()
            .find(|p| p.metadata.id == plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
        
        // Print plugin info
        self.print_plugin_details(plugin);
        
        Ok(())
    }
    
    /// Print plugin summary
    fn print_plugin_summary(&self, plugin: &PluginPackageInfo) {
        println!("  - {} ({})", plugin.metadata.name, plugin.metadata.id);
        println!("    Version: {}", plugin.metadata.version);
        println!("    Description: {}", plugin.metadata.description);
        println!("    Author: {}", plugin.metadata.author);
    }
    
    /// Print plugin details
    fn print_plugin_details(&self, plugin: &PluginPackageInfo) {
        println!("Plugin: {} ({})", plugin.metadata.name, plugin.metadata.id);
        println!("  Version: {}", plugin.metadata.version);
        println!("  API Version: {}", plugin.metadata.api_version);
        println!("  Description: {}", plugin.metadata.description);
        println!("  Author: {}", plugin.metadata.author);
        
        if !plugin.metadata.dependencies.is_empty() {
            println!("  Dependencies:");
            for dep in &plugin.metadata.dependencies {
                println!("    - {} ({})", dep.name, dep.version);
            }
        }
        
        println!("  Download URL: {}", plugin.download_url);
        println!("  Checksum: {}", plugin.checksum);
        
        if let Some(sig) = &plugin.signature {
            println!("  Signature: {}", sig);
        }
        
        if !plugin.platforms.is_empty() {
            println!("  Platforms: {}", plugin.platforms.join(", "));
        }
        
        if let Some(updated) = &plugin.updated_at {
            println!("  Updated: {}", updated);
        }
        
        println!("  Download Count: {}", plugin.download_count);
        
        if let Some(rating) = plugin.rating {
            println!("  Rating: {:.1} / 5.0", rating);
        }
        
        if let Some(license) = &plugin.license {
            println!("  License: {}", license);
        }
        
        if let Some(size) = plugin.size {
            println!("  Size: {} bytes", size);
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set default subscriber");
    
    // Parse command-line arguments
    let cli = Cli::parse();
    
    // Set log level
    if cli.verbose {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish();
        
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set default subscriber");
    }
    
    // Create and run app
    let app = match App::new(cli.download_dir, &cli.api_version).await {
        Ok(app) => app,
        Err(e) => {
            error!("Failed to create app: {}", e);
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    
    // Run command
    if let Err(e) = app.run(cli.command).await {
        error!("Command failed: {}", e);
        eprintln!("Error: {}", e);
        process::exit(1);
    }
} 