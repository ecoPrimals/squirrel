// Plugin Marketplace Module
//
// This module provides functionality for discovering, downloading,
// and verifying plugins from remote repositories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use url::Url;
use uuid::Uuid;

use crate::plugins::dynamic::{PluginMetadata, VersionCompatibilityChecker};
use crate::plugins::errors::{PluginError, Result};
use crate::plugins::security::PluginSecurityManager;

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Repository name
    pub name: String,
    
    /// Repository URL
    pub url: String,
    
    /// Repository description
    pub description: String,
    
    /// Repository maintainer
    pub maintainer: String,
    
    /// Repository API version
    pub api_version: String,
    
    /// Repository plugin count
    #[serde(default)]
    pub plugin_count: u32,
    
    /// Whether the repository is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Repository priority (lower is higher priority)
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_enabled() -> bool {
    true
}

fn default_priority() -> u32 {
    100
}

/// Plugin package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPackageInfo {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    
    /// Download URL
    pub download_url: String,
    
    /// Checksum (SHA-256)
    pub checksum: String,
    
    /// Signature (optional)
    #[serde(default)]
    pub signature: Option<String>,
    
    /// Available platforms
    #[serde(default)]
    pub platforms: Vec<String>,
    
    /// Last updated timestamp
    #[serde(default)]
    pub updated_at: Option<String>,
    
    /// Download count
    #[serde(default)]
    pub download_count: u32,
    
    /// Rating (0-5)
    #[serde(default)]
    pub rating: Option<f32>,
    
    /// Repository ID
    #[serde(default)]
    pub repository_id: Option<String>,
    
    /// License
    #[serde(default)]
    pub license: Option<String>,
    
    /// Plugin size in bytes
    #[serde(default)]
    pub size: Option<u64>,
}

/// Repository provider interface
#[async_trait]
pub trait RepositoryProvider: Send + Sync {
    /// Get repository information
    async fn get_repository_info(&self) -> Result<RepositoryInfo>;
    
    /// List available plugins
    async fn list_plugins(&self) -> Result<Vec<PluginPackageInfo>>;
    
    /// Get plugin package information
    async fn get_plugin_info(&self, plugin_id: Uuid) -> Result<PluginPackageInfo>;
    
    /// Download plugin package
    async fn download_plugin(&self, plugin_id: Uuid, target_dir: &Path) -> Result<PathBuf>;
    
    /// Search for plugins
    async fn search_plugins(&self, query: &str) -> Result<Vec<PluginPackageInfo>>;
}

/// HTTP repository provider
#[derive(Debug)]
pub struct HttpRepositoryProvider {
    /// Repository URL
    url: Url,
    
    /// HTTP client
    client: Client,
    
    /// Cache
    cache: RwLock<HashMap<String, Vec<u8>>>,
    
    /// Cache expiration in seconds
    cache_expiration: u64,
}

impl HttpRepositoryProvider {
    /// Create a new HTTP repository provider
    pub fn new(url: &str) -> Result<Self> {
        let url = Url::parse(url).map_err(|e| {
            PluginError::DiscoveryError(format!("Invalid repository URL: {}", e))
        })?;
        
        Ok(Self {
            url,
            client: Client::new(),
            cache: RwLock::new(HashMap::new()),
            cache_expiration: 300, // 5 minutes
        })
    }
    
    /// Set cache expiration
    pub fn with_cache_expiration(mut self, seconds: u64) -> Self {
        self.cache_expiration = seconds;
        self
    }
    
    /// Get cached response or fetch from repository
    async fn get_cached_or_fetch(&self, path: &str) -> Result<Vec<u8>> {
        let cache_key = path.to_string();
        
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(data) = cache.get(&cache_key) {
                return Ok(data.clone());
            }
        }
        
        // Fetch from repository
        let url = self.url.join(path).map_err(|e| {
            PluginError::DiscoveryError(format!("Invalid repository path: {}", e))
        })?;
        
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| {
                PluginError::DiscoveryError(format!("Failed to connect to repository: {}", e))
            })?;
        
        let status = response.status();
        if !status.is_success() {
            return Err(PluginError::DiscoveryError(
                format!("Repository returned error: {}", status)
            ));
        }
        
        let data = response.bytes()
            .await
            .map_err(|e| {
                PluginError::DiscoveryError(format!("Failed to read repository response: {}", e))
            })?
            .to_vec();
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, data.clone());
        }
        
        Ok(data)
    }
}

#[async_trait]
impl RepositoryProvider for HttpRepositoryProvider {
    async fn get_repository_info(&self) -> Result<RepositoryInfo> {
        let data = self.get_cached_or_fetch("info.json").await?;
        
        let info: RepositoryInfo = serde_json::from_slice(&data)
            .map_err(|e| {
                PluginError::DiscoveryError(format!("Failed to parse repository info: {}", e))
            })?;
        
        Ok(info)
    }
    
    async fn list_plugins(&self) -> Result<Vec<PluginPackageInfo>> {
        let data = self.get_cached_or_fetch("plugins.json").await?;
        
        let plugins: Vec<PluginPackageInfo> = serde_json::from_slice(&data)
            .map_err(|e| {
                PluginError::DiscoveryError(format!("Failed to parse plugin list: {}", e))
            })?;
        
        Ok(plugins)
    }
    
    async fn get_plugin_info(&self, plugin_id: Uuid) -> Result<PluginPackageInfo> {
        let path = format!("plugins/{}.json", plugin_id);
        let data = self.get_cached_or_fetch(&path).await?;
        
        let info: PluginPackageInfo = serde_json::from_slice(&data)
            .map_err(|e| {
                PluginError::DiscoveryError(format!("Failed to parse plugin info: {}", e))
            })?;
        
        Ok(info)
    }
    
    async fn download_plugin(&self, plugin_id: Uuid, target_dir: &Path) -> Result<PathBuf> {
        // Get plugin info
        let info = self.get_plugin_info(plugin_id).await?;
        
        // Prepare target path
        let target_path = target_dir.join(format!("{}.plugin", plugin_id));
        
        // Create target directory if it doesn't exist
        if !target_dir.exists() {
            tokio::fs::create_dir_all(target_dir)
                .await
                .map_err(|e| {
                    PluginError::IoError(e)
                })?;
        }
        
        // Download plugin
        let url = Url::parse(&info.download_url)
            .map_err(|e| {
                PluginError::DiscoveryError(format!("Invalid download URL: {}", e))
            })?;
        
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| {
                PluginError::DiscoveryError(format!("Failed to download plugin: {}", e))
            })?;
        
        let status = response.status();
        if !status.is_success() {
            return Err(PluginError::DiscoveryError(
                format!("Repository returned error during download: {}", status)
            ));
        }
        
        let data = response.bytes()
            .await
            .map_err(|e| {
                PluginError::DiscoveryError(format!("Failed to read plugin data: {}", e))
            })?
            .to_vec();
        
        // Verify checksum
        let calculated_checksum = sha256::digest(&data);
        if calculated_checksum != info.checksum {
            return Err(PluginError::SecurityError(
                format!("Checksum mismatch: expected {}, got {}", info.checksum, calculated_checksum)
            ));
        }
        
        // Save plugin to file
        tokio::fs::write(&target_path, data)
            .await
            .map_err(|e| {
                PluginError::IoError(e)
            })?;
        
        Ok(target_path)
    }
    
    async fn search_plugins(&self, query: &str) -> Result<Vec<PluginPackageInfo>> {
        let path = format!("search?q={}", urlencoding::encode(query));
        let data = self.get_cached_or_fetch(&path).await?;
        
        let plugins: Vec<PluginPackageInfo> = serde_json::from_slice(&data)
            .map_err(|e| {
                PluginError::DiscoveryError(format!("Failed to parse search results: {}", e))
            })?;
        
        Ok(plugins)
    }
}

/// Repository manager for managing multiple repositories
#[derive(Debug)]
pub struct RepositoryManager {
    /// Repositories
    repositories: RwLock<HashMap<String, Arc<dyn RepositoryProvider>>>,
    
    /// Version compatibility checker
    version_checker: VersionCompatibilityChecker,
    
    /// Plugin download directory
    download_dir: PathBuf,
    
    /// Security manager
    security_manager: Arc<dyn PluginSecurityManager>,
}

impl RepositoryManager {
    /// Create a new repository manager
    pub fn new(
        app_version: &str,
        download_dir: PathBuf,
        security_manager: Arc<dyn PluginSecurityManager>,
    ) -> Result<Self> {
        let version_checker = VersionCompatibilityChecker::new(app_version)?;
        
        Ok(Self {
            repositories: RwLock::new(HashMap::new()),
            version_checker,
            download_dir,
            security_manager,
        })
    }
    
    /// Add a repository
    pub async fn add_repository(
        &self,
        id: &str,
        provider: Arc<dyn RepositoryProvider>,
    ) -> Result<()> {
        // Get repository info and verify API compatibility
        let info = provider.get_repository_info().await?;
        
        // Check API version compatibility
        if !self.version_checker.check_compatibility(&info.api_version, "^1.0.0")? {
            return Err(PluginError::IncompatibleVersion(
                format!("Repository API version {} is not compatible", info.api_version)
            ));
        }
        
        // Add repository
        let mut repositories = self.repositories.write().await;
        repositories.insert(id.to_string(), provider);
        
        info!("Added repository: {}", id);
        Ok(())
    }
    
    /// Remove a repository
    pub async fn remove_repository(&self, id: &str) -> Result<()> {
        let mut repositories = self.repositories.write().await;
        
        if repositories.remove(id).is_some() {
            info!("Removed repository: {}", id);
            Ok(())
        } else {
            Err(PluginError::NotFound(format!("Repository not found: {}", id)))
        }
    }
    
    /// Get all repositories
    pub async fn get_repositories(&self) -> Vec<(String, RepositoryInfo)> {
        let repositories = self.repositories.read().await;
        
        let mut result = Vec::new();
        for (id, provider) in repositories.iter() {
            match provider.get_repository_info().await {
                Ok(info) => {
                    result.push((id.clone(), info));
                },
                Err(e) => {
                    warn!("Failed to get info for repository {}: {}", id, e);
                }
            }
        }
        
        // Sort by priority
        result.sort_by_key(|(_, info)| info.priority);
        
        result
    }
    
    /// List plugins from all repositories
    pub async fn list_plugins(&self) -> Vec<(String, Vec<PluginPackageInfo>)> {
        let repositories = self.repositories.read().await;
        
        let mut result = Vec::new();
        for (id, provider) in repositories.iter() {
            match provider.list_plugins().await {
                Ok(plugins) => {
                    result.push((id.clone(), plugins));
                },
                Err(e) => {
                    warn!("Failed to list plugins from repository {}: {}", id, e);
                }
            }
        }
        
        result
    }
    
    /// Search plugins from all repositories
    pub async fn search_plugins(&self, query: &str) -> Vec<(String, Vec<PluginPackageInfo>)> {
        let repositories = self.repositories.read().await;
        
        let mut result = Vec::new();
        for (id, provider) in repositories.iter() {
            match provider.search_plugins(query).await {
                Ok(plugins) => {
                    if !plugins.is_empty() {
                        result.push((id.clone(), plugins));
                    }
                },
                Err(e) => {
                    warn!("Failed to search plugins in repository {}: {}", id, e);
                }
            }
        }
        
        result
    }
    
    /// Download a plugin from a specific repository
    pub async fn download_plugin(
        &self,
        repository_id: &str,
        plugin_id: Uuid,
    ) -> Result<PathBuf> {
        let repositories = self.repositories.read().await;
        
        let provider = repositories.get(repository_id)
            .ok_or_else(|| {
                PluginError::NotFound(format!("Repository not found: {}", repository_id))
            })?;
        
        // Get plugin info
        let info = provider.get_plugin_info(plugin_id).await?;
        
        // Check if plugin is compatible with the current platform
        let platform = get_current_platform();
        if !info.platforms.is_empty() && !info.platforms.contains(&platform) {
            return Err(PluginError::IncompatibleVersion(
                format!("Plugin is not compatible with platform: {}", platform)
            ));
        }
        
        // Check API version compatibility
        if !self.version_checker.check_compatibility(&info.metadata.api_version, "^1.0.0")? {
            return Err(PluginError::IncompatibleVersion(
                format!("Plugin API version {} is not compatible", info.metadata.api_version)
            ));
        }
        
        // Verify plugin security
        if let Some(signature) = &info.signature {
            // Verify signature
            let verification_result = self.security_manager.verify_signature(
                plugin_id,
                signature,
                &info.checksum.as_bytes().to_vec(),
            ).await;
            
            if let Err(e) = verification_result {
                return Err(PluginError::SecurityError(
                    format!("Plugin signature verification failed: {}", e)
                ));
            }
        }
        
        // Download plugin
        let target_dir = self.download_dir.join(repository_id);
        let plugin_path = provider.download_plugin(plugin_id, &target_dir).await?;
        
        info!("Downloaded plugin {} from repository {}", plugin_id, repository_id);
        Ok(plugin_path)
    }
    
    /// Check for updates to installed plugins
    ///
    /// # Arguments
    ///
    /// * `installed_plugins` - List of installed plugins with their metadata
    ///
    /// # Returns
    ///
    /// List of update information for plugins that have updates available
    pub async fn check_for_updates(&self, installed_plugins: &[PluginMetadata]) -> Vec<UpdateInfo> {
        let mut updates = Vec::new();
        
        // Get current timestamp in ISO 8601 format
        let now = chrono::Utc::now().to_rfc3339();
        
        // Check each repository for updates
        let repositories = self.repositories.read().await;
        for (repo_id, provider) in repositories.iter() {
            match provider.list_plugins().await {
                Ok(available_plugins) => {
                    for installed in installed_plugins {
                        // Find matching plugin in repository
                        if let Some(available) = available_plugins.iter().find(|p| p.metadata.id == installed.id) {
                            // Check if version is newer
                            let result = self.version_checker.compare_versions(
                                &installed.version,
                                &available.metadata.version,
                            );
                            
                            if result.is_ok() && result.unwrap() == std::cmp::Ordering::Less {
                                // This is an update
                                let importance = UpdateImportance::from_version_diff(
                                    &installed.version,
                                    &available.metadata.version,
                                );
                                
                                updates.push(UpdateInfo {
                                    plugin_id: installed.id,
                                    name: installed.name.clone(),
                                    current_version: installed.version.clone(),
                                    available_version: available.metadata.version.clone(),
                                    repository_id: repo_id.clone(),
                                    importance,
                                    description: available.metadata.description.clone().into(),
                                    last_checked: now.clone(),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to check for updates in repository {}: {}", repo_id, e);
                }
            }
        }
        
        updates
    }
    
    /// Schedule regular update checks in the background
    ///
    /// # Arguments
    ///
    /// * `installed_plugins_provider` - Function that returns the current list of installed plugins
    /// * `update_handler` - Function to call when updates are found
    /// * `check_interval` - Interval between checks in seconds
    ///
    /// # Returns
    ///
    /// JoinHandle for the background task
    pub fn schedule_update_checks<F, H>(
        self: &Arc<Self>,
        installed_plugins_provider: F,
        update_handler: H,
        check_interval: u64,
    ) -> tokio::task::JoinHandle<()>
    where
        F: Fn() -> Vec<PluginMetadata> + Send + Sync + 'static,
        H: Fn(Vec<UpdateInfo>) + Send + Sync + 'static,
    {
        let manager = self.clone();
        
        tokio::spawn(async move {
            let interval = std::time::Duration::from_secs(check_interval);
            
            loop {
                // Sleep first to avoid immediate check at startup
                tokio::time::sleep(interval).await;
                
                // Get installed plugins
                let installed_plugins = installed_plugins_provider();
                
                // Check for updates
                let updates = manager.check_for_updates(&installed_plugins).await;
                
                // Handle updates if any
                if !updates.is_empty() {
                    update_handler(updates);
                }
            }
        })
    }
    
    /// Check if an update is available for a specific plugin
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - The ID of the plugin to check
    /// * `current_version` - The current version of the plugin
    ///
    /// # Returns
    ///
    /// Update information if an update is available, None otherwise
    pub async fn check_plugin_update(&self, plugin_id: Uuid, current_version: &str) -> Option<UpdateInfo> {
        let now = chrono::Utc::now().to_rfc3339();
        let repositories = self.repositories.read().await;
        
        for (repo_id, provider) in repositories.iter() {
            match provider.list_plugins().await {
                Ok(available_plugins) => {
                    // Find matching plugin in repository
                    if let Some(available) = available_plugins.iter().find(|p| p.metadata.id == plugin_id) {
                        // Check if version is newer
                        let result = self.version_checker.compare_versions(
                            current_version,
                            &available.metadata.version,
                        );
                        
                        if result.is_ok() && result.unwrap() == std::cmp::Ordering::Less {
                            // This is an update
                            let importance = UpdateImportance::from_version_diff(
                                current_version,
                                &available.metadata.version,
                            );
                            
                            return Some(UpdateInfo {
                                plugin_id,
                                name: available.metadata.name.clone(),
                                current_version: current_version.to_string(),
                                available_version: available.metadata.version.clone(),
                                repository_id: repo_id.clone(),
                                importance,
                                description: available.metadata.description.clone().into(),
                                last_checked: now,
                            });
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
        
        None
    }
    
    /// Enhanced search functionality with filtering and sorting
    ///
    /// # Arguments
    ///
    /// * `query` - Search query
    /// * `category` - Optional category filter
    /// * `tags` - Optional tag filters (all must match)
    /// * `sort_by` - Optional sort field
    /// * `sort_order` - Optional sort order
    ///
    /// # Returns
    ///
    /// List of matching plugin packages grouped by repository
    pub async fn enhanced_search(
        &self,
        query: &str,
        category: Option<&str>,
        tags: Option<&[&str]>,
        sort_by: Option<PluginSortField>,
        sort_order: Option<SortOrder>,
    ) -> Vec<(String, Vec<PluginPackageInfo>)> {
        let mut results = Vec::new();
        
        // Get repositories
        let repositories = self.repositories.read().await;
        
        for (repo_id, provider) in repositories.iter() {
            match provider.search_plugins(query).await {
                Ok(mut plugins) => {
                    // Filter by category if specified
                    if let Some(cat) = category {
                        plugins.retain(|p| {
                            p.metadata.capabilities.iter().any(|c| c == cat)
                        });
                    }
                    
                    // Filter by tags if specified
                    if let Some(tag_filters) = tags {
                        plugins.retain(|p| {
                            tag_filters.iter().all(|tag| {
                                p.metadata.capabilities.iter().any(|c| c == *tag)
                            })
                        });
                    }
                    
                    // Sort results if specified
                    if let Some(field) = sort_by {
                        let order = sort_order.unwrap_or(SortOrder::Ascending);
                        
                        match field {
                            PluginSortField::Name => {
                                if order == SortOrder::Ascending {
                                    plugins.sort_by(|a, b| a.metadata.name.cmp(&b.metadata.name));
                                } else {
                                    plugins.sort_by(|a, b| b.metadata.name.cmp(&a.metadata.name));
                                }
                            }
                            PluginSortField::UpdatedAt => {
                                plugins.sort_by(|a, b| {
                                    match (&a.updated_at, &b.updated_at) {
                                        (Some(a_date), Some(b_date)) => {
                                            if order == SortOrder::Ascending {
                                                a_date.cmp(b_date)
                                            } else {
                                                b_date.cmp(a_date)
                                            }
                                        }
                                        (Some(_), None) => std::cmp::Ordering::Less,
                                        (None, Some(_)) => std::cmp::Ordering::Greater,
                                        (None, None) => std::cmp::Ordering::Equal,
                                    }
                                });
                            }
                            PluginSortField::Rating => {
                                plugins.sort_by(|a, b| {
                                    match (a.rating, b.rating) {
                                        (Some(a_rating), Some(b_rating)) => {
                                            if order == SortOrder::Ascending {
                                                a_rating.partial_cmp(&b_rating).unwrap_or(std::cmp::Ordering::Equal)
                                            } else {
                                                b_rating.partial_cmp(&a_rating).unwrap_or(std::cmp::Ordering::Equal)
                                            }
                                        }
                                        (Some(_), None) => std::cmp::Ordering::Less,
                                        (None, Some(_)) => std::cmp::Ordering::Greater,
                                        (None, None) => std::cmp::Ordering::Equal,
                                    }
                                });
                            }
                            PluginSortField::Downloads => {
                                if order == SortOrder::Ascending {
                                    plugins.sort_by(|a, b| a.download_count.cmp(&b.download_count));
                                } else {
                                    plugins.sort_by(|a, b| b.download_count.cmp(&a.download_count));
                                }
                            }
                        }
                    }
                    
                    if !plugins.is_empty() {
                        results.push((repo_id.clone(), plugins));
                    }
                }
                Err(e) => {
                    warn!("Failed to search plugins in repository {}: {}", repo_id, e);
                }
            }
        }
        
        results
    }
}

/// Get current platform string
fn get_current_platform() -> String {
    #[cfg(target_os = "windows")]
    {
        "windows".to_string()
    }
    
    #[cfg(target_os = "linux")]
    {
        "linux".to_string()
    }
    
    #[cfg(target_os = "macos")]
    {
        "macos".to_string()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        "unknown".to_string()
    }
}

/// Update notification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// Plugin name
    pub name: String,
    
    /// Current version
    pub current_version: String,
    
    /// Available version
    pub available_version: String,
    
    /// Repository ID
    pub repository_id: String,
    
    /// Update importance level
    pub importance: UpdateImportance,
    
    /// Update description
    pub description: Option<String>,
    
    /// Last checked timestamp (ISO 8601)
    pub last_checked: String,
}

/// Update importance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateImportance {
    /// Low importance (minor update)
    Low,
    
    /// Medium importance (feature update)
    Medium,
    
    /// High importance (major update or security fix)
    High,
    
    /// Critical importance (security vulnerability fix)
    Critical,
}

impl UpdateImportance {
    /// Parse from version comparison
    fn from_version_diff(current: &str, available: &str) -> Self {
        use semver::Version;
        
        if let (Ok(current), Ok(available)) = (Version::parse(current), Version::parse(available)) {
            if available.major > current.major {
                return Self::High;
            } else if available.minor > current.minor {
                return Self::Medium;
            } else {
                return Self::Low;
            }
        }
        
        // Default to medium if we can't parse versions
        Self::Medium
    }
}

/// Plugin sort field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginSortField {
    /// Sort by name
    Name,
    
    /// Sort by last updated timestamp
    UpdatedAt,
    
    /// Sort by rating
    Rating,
    
    /// Sort by download count
    Downloads,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending order (A-Z, 0-9)
    Ascending,
    
    /// Descending order (Z-A, 9-0)
    Descending,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::security::PluginSecurityValidator;
    use std::path::PathBuf;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_repository_manager() {
        // Create a mock repository provider
        struct MockRepositoryProvider;
        
        #[async_trait]
        impl RepositoryProvider for MockRepositoryProvider {
            async fn get_repository_info(&self) -> Result<RepositoryInfo> {
                Ok(RepositoryInfo {
                    name: "Mock Repository".to_string(),
                    url: "https://example.com".to_string(),
                    description: "Mock repository for testing".to_string(),
                    maintainer: "Test Maintainer".to_string(),
                    api_version: "1.0.0".to_string(),
                    plugin_count: 2,
                    enabled: true,
                    priority: 10,
                })
            }
            
            async fn list_plugins(&self) -> Result<Vec<PluginPackageInfo>> {
                let plugin1 = PluginPackageInfo {
                    metadata: PluginMetadata {
                        id: Uuid::new_v4(),
                        name: "Test Plugin 1".to_string(),
                        version: "1.0.0".to_string(),
                        api_version: "1.0.0".to_string(),
                        description: "Test plugin 1".to_string(),
                        author: "Test Author".to_string(),
                        dependencies: Vec::new(),
                    },
                    download_url: "https://example.com/plugin1".to_string(),
                    checksum: "abcdef".to_string(),
                    signature: None,
                    platforms: vec!["windows".to_string(), "linux".to_string(), "macos".to_string()],
                    updated_at: Some("2024-04-20".to_string()),
                    download_count: 100,
                    rating: Some(4.5),
                    repository_id: Some("mock".to_string()),
                    license: Some("MIT".to_string()),
                    size: Some(1024),
                };
                
                let plugin2 = PluginPackageInfo {
                    metadata: PluginMetadata {
                        id: Uuid::new_v4(),
                        name: "Test Plugin 2".to_string(),
                        version: "1.0.0".to_string(),
                        api_version: "1.0.0".to_string(),
                        description: "Test plugin 2".to_string(),
                        author: "Test Author".to_string(),
                        dependencies: Vec::new(),
                    },
                    download_url: "https://example.com/plugin2".to_string(),
                    checksum: "123456".to_string(),
                    signature: None,
                    platforms: vec!["windows".to_string(), "linux".to_string()],
                    updated_at: Some("2024-04-19".to_string()),
                    download_count: 50,
                    rating: Some(4.0),
                    repository_id: Some("mock".to_string()),
                    license: Some("Apache-2.0".to_string()),
                    size: Some(2048),
                };
                
                Ok(vec![plugin1, plugin2])
            }
            
            async fn get_plugin_info(&self, _plugin_id: Uuid) -> Result<PluginPackageInfo> {
                Ok(PluginPackageInfo {
                    metadata: PluginMetadata {
                        id: Uuid::new_v4(),
                        name: "Test Plugin".to_string(),
                        version: "1.0.0".to_string(),
                        api_version: "1.0.0".to_string(),
                        description: "Test plugin".to_string(),
                        author: "Test Author".to_string(),
                        dependencies: Vec::new(),
                    },
                    download_url: "https://example.com/plugin".to_string(),
                    checksum: "abcdef".to_string(),
                    signature: None,
                    platforms: vec!["windows".to_string(), "linux".to_string(), "macos".to_string()],
                    updated_at: Some("2024-04-20".to_string()),
                    download_count: 100,
                    rating: Some(4.5),
                    repository_id: Some("mock".to_string()),
                    license: Some("MIT".to_string()),
                    size: Some(1024),
                })
            }
            
            async fn download_plugin(&self, _plugin_id: Uuid, target_dir: &Path) -> Result<PathBuf> {
                // Create a mock file
                let target_path = target_dir.join("mock-plugin.dll");
                
                // Create target directory if it doesn't exist
                if !target_dir.exists() {
                    tokio::fs::create_dir_all(target_dir)
                        .await
                        .map_err(|e| PluginError::IoError(e))?;
                }
                
                // Write mock data
                tokio::fs::write(&target_path, b"mock plugin data")
                    .await
                    .map_err(|e| PluginError::IoError(e))?;
                
                Ok(target_path)
            }
            
            async fn search_plugins(&self, _query: &str) -> Result<Vec<PluginPackageInfo>> {
                // Return a subset of plugins
                let plugin = PluginPackageInfo {
                    metadata: PluginMetadata {
                        id: Uuid::new_v4(),
                        name: "Test Plugin".to_string(),
                        version: "1.0.0".to_string(),
                        api_version: "1.0.0".to_string(),
                        description: "Test plugin".to_string(),
                        author: "Test Author".to_string(),
                        dependencies: Vec::new(),
                    },
                    download_url: "https://example.com/plugin".to_string(),
                    checksum: "abcdef".to_string(),
                    signature: None,
                    platforms: vec!["windows".to_string(), "linux".to_string(), "macos".to_string()],
                    updated_at: Some("2024-04-20".to_string()),
                    download_count: 100,
                    rating: Some(4.5),
                    repository_id: Some("mock".to_string()),
                    license: Some("MIT".to_string()),
                    size: Some(1024),
                };
                
                Ok(vec![plugin])
            }
        }
        
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let download_dir = temp_dir.path().to_path_buf();
        
        let security_manager = Arc::new(PluginSecurityValidator::new());
        
        let manager = RepositoryManager::new(
            "1.0.0",
            download_dir,
            security_manager,
        ).expect("Failed to create repository manager");
        
        // Add repository
        manager.add_repository(
            "mock",
            Arc::new(MockRepositoryProvider),
        ).await.expect("Failed to add repository");
        
        // List repositories
        let repositories = manager.get_repositories().await;
        assert_eq!(repositories.len(), 1);
        assert_eq!(repositories[0].0, "mock");
        assert_eq!(repositories[0].1.name, "Mock Repository");
        
        // List plugins
        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].0, "mock");
        assert_eq!(plugins[0].1.len(), 2);
        
        // Search plugins
        let search_results = manager.search_plugins("test").await;
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].0, "mock");
        assert_eq!(search_results[0].1.len(), 1);
        
        // Download plugin
        let plugin_id = Uuid::new_v4();
        let plugin_path = manager.download_plugin("mock", plugin_id).await
            .expect("Failed to download plugin");
        
        assert!(plugin_path.exists());
        
        // Remove repository
        manager.remove_repository("mock").await
            .expect("Failed to remove repository");
        
        let repositories = manager.get_repositories().await;
        assert_eq!(repositories.len(), 0);
    }
} 