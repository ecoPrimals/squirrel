// Plugin Repository Server
//
// This is a simple HTTP server that acts as a plugin repository for demonstration
// purposes. It serves repository information, plugin lists, and plugin downloads.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    extract::{Path as AxumPath, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

use squirrel_plugins::plugins::dynamic::{PluginMetadata, PluginDependency};
use squirrel_plugins::plugins::marketplace::{RepositoryInfo, PluginPackageInfo};

/// Command-line arguments
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to plugin directory
    #[clap(short, long, default_value = "./repo-plugins")]
    plugin_dir: PathBuf,
    
    /// Server host
    #[clap(short, long, default_value = "127.0.0.1")]
    host: String,
    
    /// Server port
    #[clap(short, long, default_value = "3000")]
    port: u16,
    
    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
}

/// Repository state
struct RepositoryState {
    /// Repository information
    info: RepositoryInfo,
    
    /// Plugins
    plugins: HashMap<Uuid, PluginPackageInfo>,
    
    /// Plugin directory
    plugin_dir: PathBuf,
    
    /// Base URL for downloads
    base_url: String,
}

impl RepositoryState {
    /// Create a new repository state
    fn new(plugin_dir: PathBuf, host: &str, port: u16) -> Self {
        let base_url = format!("http://{}:{}", host, port);
        
        // Create repository info
        let info = RepositoryInfo {
            name: "Example Plugin Repository".to_string(),
            url: base_url.clone(),
            description: "Example plugin repository for demonstration purposes".to_string(),
            maintainer: "Squirrel Team".to_string(),
            api_version: "1.0.0".to_string(),
            plugin_count: 0,
            enabled: true,
            priority: 10,
        };
        
        // Create example plugins
        let mut plugins = HashMap::new();
        
        // Command example plugin
        let command_plugin_id = Uuid::new_v4();
        let command_plugin = PluginPackageInfo {
            metadata: PluginMetadata {
                id: command_plugin_id,
                name: "Command Example".to_string(),
                version: "1.0.0".to_string(),
                api_version: "1.0.0".to_string(),
                description: "Example command plugin for Squirrel Plugin System".to_string(),
                author: "Squirrel Team".to_string(),
                dependencies: Vec::new(),
            },
            download_url: format!("{}/download/{}", base_url, command_plugin_id),
            checksum: "abcdef1234567890".to_string(),
            signature: None,
            platforms: vec!["windows".to_string(), "linux".to_string(), "macos".to_string()],
            updated_at: Some("2024-04-20".to_string()),
            download_count: 10,
            rating: Some(4.5),
            repository_id: Some("example".to_string()),
            license: Some("MIT".to_string()),
            size: Some(1024),
        };
        
        // Tool example plugin
        let tool_plugin_id = Uuid::new_v4();
        let tool_plugin = PluginPackageInfo {
            metadata: PluginMetadata {
                id: tool_plugin_id,
                name: "Tool Example".to_string(),
                version: "1.0.0".to_string(),
                api_version: "1.0.0".to_string(),
                description: "Example tool plugin for Squirrel Plugin System".to_string(),
                author: "Squirrel Team".to_string(),
                dependencies: Vec::new(),
            },
            download_url: format!("{}/download/{}", base_url, tool_plugin_id),
            checksum: "0987654321fedcba".to_string(),
            signature: None,
            platforms: vec!["windows".to_string(), "linux".to_string(), "macos".to_string()],
            updated_at: Some("2024-04-21".to_string()),
            download_count: 5,
            rating: Some(4.0),
            repository_id: Some("example".to_string()),
            license: Some("Apache-2.0".to_string()),
            size: Some(2048),
        };
        
        // Dynamic example plugin with dependencies
        let dynamic_plugin_id = Uuid::new_v4();
        let dynamic_plugin = PluginPackageInfo {
            metadata: PluginMetadata {
                id: dynamic_plugin_id,
                name: "Dynamic Example".to_string(),
                version: "1.0.0".to_string(),
                api_version: "1.0.0".to_string(),
                description: "Example dynamic plugin for Squirrel Plugin System".to_string(),
                author: "Squirrel Team".to_string(),
                dependencies: vec![
                    PluginDependency {
                        id: Some(command_plugin_id),
                        name: "Command Example".to_string(),
                        version: "^1.0.0".to_string(),
                    },
                ],
            },
            download_url: format!("{}/download/{}", base_url, dynamic_plugin_id),
            checksum: "1122334455667788".to_string(),
            signature: None,
            platforms: vec!["windows".to_string(), "linux".to_string()],
            updated_at: Some("2024-04-22".to_string()),
            download_count: 3,
            rating: Some(4.8),
            repository_id: Some("example".to_string()),
            license: Some("MIT".to_string()),
            size: Some(3072),
        };
        
        // Add plugins to HashMap
        plugins.insert(command_plugin_id, command_plugin);
        plugins.insert(tool_plugin_id, tool_plugin);
        plugins.insert(dynamic_plugin_id, dynamic_plugin);
        
        Self {
            info: info,
            plugins,
            plugin_dir,
            base_url,
        }
    }
    
    /// Get repository info
    fn get_info(&self) -> RepositoryInfo {
        let mut info = self.info.clone();
        info.plugin_count = self.plugins.len() as u32;
        info
    }
    
    /// Get all plugins
    fn get_plugins(&self) -> Vec<PluginPackageInfo> {
        self.plugins.values().cloned().collect()
    }
    
    /// Get a plugin by ID
    fn get_plugin(&self, id: Uuid) -> Option<PluginPackageInfo> {
        self.plugins.get(&id).cloned()
    }
    
    /// Search plugins
    fn search_plugins(&self, query: &str) -> Vec<PluginPackageInfo> {
        let query = query.to_lowercase();
        
        self.plugins.values()
            .filter(|p| {
                p.metadata.name.to_lowercase().contains(&query) ||
                p.metadata.description.to_lowercase().contains(&query) ||
                p.metadata.author.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }
}

/// Search query parameters
#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

/// Application state
struct AppState {
    repository: Arc<RwLock<RepositoryState>>,
}

/// Handler for repository info
async fn get_repository_info(
    state: axum::extract::State<Arc<AppState>>,
) -> Json<RepositoryInfo> {
    let repository = state.repository.read().await;
    Json(repository.get_info())
}

/// Handler for plugin list
async fn list_plugins(
    state: axum::extract::State<Arc<AppState>>,
) -> Json<Vec<PluginPackageInfo>> {
    let repository = state.repository.read().await;
    Json(repository.get_plugins())
}

/// Handler for plugin info
async fn get_plugin_info(
    state: axum::extract::State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<PluginPackageInfo>, StatusCode> {
    // Parse plugin ID
    let plugin_id = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Get plugin info
    let repository = state.repository.read().await;
    let plugin = repository.get_plugin(plugin_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(plugin))
}

/// Handler for plugin search
async fn search_plugins(
    state: axum::extract::State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Json<Vec<PluginPackageInfo>> {
    let repository = state.repository.read().await;
    Json(repository.search_plugins(&params.q))
}

/// Handler for downloading a plugin
async fn download_plugin(
    state: axum::extract::State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<(StatusCode, String), StatusCode> {
    // Parse plugin ID
    let plugin_id = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Get plugin info
    let repository = state.repository.read().await;
    let plugin = repository.get_plugin(plugin_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Return a mock plugin file
    Ok((StatusCode::OK, format!("Mock plugin data for {}", plugin.metadata.name)))
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
    
    // Create plugin directory if it doesn't exist
    if !cli.plugin_dir.exists() {
        tokio::fs::create_dir_all(&cli.plugin_dir)
            .await
            .expect("Failed to create plugin directory");
    }
    
    // Create repository state
    let repository = RepositoryState::new(cli.plugin_dir.clone(), &cli.host, cli.port);
    
    // Print plugin information for debugging
    info!("Repository: {}", repository.info.name);
    info!("URL: {}", repository.info.url);
    info!("Plugins:");
    for plugin in repository.plugins.values() {
        info!("  {} ({})", plugin.metadata.name, plugin.metadata.id);
        info!("    Download URL: {}", plugin.download_url);
    }
    
    // Create app state
    let app_state = Arc::new(AppState {
        repository: Arc::new(RwLock::new(repository)),
    });
    
    // Create router
    let app = Router::new()
        .route("/info.json", get(get_repository_info))
        .route("/plugins.json", get(list_plugins))
        .route("/plugins/:id.json", get(get_plugin_info))
        .route("/search", get(search_plugins))
        .route("/download/:id", get(download_plugin))
        .nest_service("/static", ServeDir::new(&cli.plugin_dir))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    
    // Start server
    let addr = format!("{}:{}", cli.host, cli.port);
    info!("Starting server on {}", addr);
    
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
} 