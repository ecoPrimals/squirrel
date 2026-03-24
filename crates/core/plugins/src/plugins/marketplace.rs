// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin marketplace functionality
//!
//! This module contains types and functions for plugin discovery, download,
//! and management from remote repositories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::dynamic::PluginMetadata;

/// Information about a plugin repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Name of the repository
    pub name: String,

    /// URL of the repository
    pub url: String,

    /// Description of the repository
    pub description: String,

    /// Maintainer of the repository
    pub maintainer: String,

    /// API version of the repository
    pub api_version: String,

    /// Number of plugins in the repository
    pub plugin_count: u32,

    /// Whether the repository is enabled
    pub enabled: bool,

    /// Priority of the repository (higher = more important)
    pub priority: u32,
}

/// Information about a plugin package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPackageInfo {
    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// URL to download the plugin
    pub download_url: String,

    /// Checksum for verifying the plugin package
    pub checksum: String,

    /// Optional cryptographic signature
    pub signature: Option<String>,

    /// Supported platforms
    pub platforms: Vec<String>,

    /// When the plugin was last updated
    pub updated_at: Option<String>,

    /// Number of times the plugin has been downloaded
    pub download_count: u32,

    /// User rating
    pub rating: Option<f32>,

    /// ID of the repository containing this plugin
    pub repository_id: Option<String>,

    /// License information
    pub license: Option<String>,

    /// Size in bytes
    pub size: Option<u64>,
}

/// A provider for a plugin repository
#[async_trait::async_trait]
pub trait RepositoryProvider: Send + Sync {
    /// Get repository information
    async fn get_info(&self) -> Result<RepositoryInfo>;

    /// List all plugins
    async fn list_plugins(&self) -> Result<Vec<PluginPackageInfo>>;

    /// Get a specific plugin by ID
    async fn get_plugin(&self, id: &Uuid) -> Result<Option<PluginPackageInfo>>;

    /// Search for plugins by query string
    async fn search_plugins(&self, query: &str) -> Result<Vec<PluginPackageInfo>>;

    /// Download a plugin
    async fn download_plugin(&self, id: &Uuid, path: &Path) -> Result<()>;
}

/// HTTP-based repository provider
pub struct HttpRepositoryProvider {
    /// Base URL of the repository
    base_url: String,

    /// HTTP client
    #[cfg(feature = "marketplace")]
    client: reqwest::Client,
}

/// Repository manager for handling multiple plugin repositories
pub struct RepositoryManager {
    /// API version this manager supports
    api_version: String,

    /// Directory for downloading plugins
    download_dir: PathBuf,

    /// Map of repository providers
    repositories: RwLock<HashMap<String, Arc<dyn RepositoryProvider>>>,

    /// Map of repository info
    info_cache: RwLock<HashMap<String, RepositoryInfo>>,
}

#[cfg(feature = "marketplace")]
impl HttpRepositoryProvider {
    /// Create a new HTTP repository provider
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        })
    }

    /// Get the base URL of the repository
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(feature = "marketplace")]
#[async_trait::async_trait]
impl RepositoryProvider for HttpRepositoryProvider {
    async fn get_info(&self) -> Result<RepositoryInfo> {
        let url = format!("{}/info", self.base_url);
        let info = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<RepositoryInfo>()
            .await?;
        Ok(info)
    }

    async fn list_plugins(&self) -> Result<Vec<PluginPackageInfo>> {
        let url = format!("{}/plugins", self.base_url);
        let plugins = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Vec<PluginPackageInfo>>()
            .await?;
        Ok(plugins)
    }

    async fn get_plugin(&self, id: &Uuid) -> Result<Option<PluginPackageInfo>> {
        let url = format!("{}/plugin/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let plugin = response.json::<PluginPackageInfo>().await?;
            Ok(Some(plugin))
        } else {
            Ok(None)
        }
    }

    async fn search_plugins(&self, query: &str) -> Result<Vec<PluginPackageInfo>> {
        let url = format!("{}/search?q={}", self.base_url, urlencoding::encode(query));
        let plugins = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Vec<PluginPackageInfo>>()
            .await?;
        Ok(plugins)
    }

    async fn download_plugin(&self, id: &Uuid, path: &Path) -> Result<()> {
        let url = format!("{}/download/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            tokio::fs::write(path, bytes).await?;
            Ok(())
        } else {
            anyhow::bail!("Failed to download plugin: {}", response.status())
        }
    }
}

impl RepositoryManager {
    /// Create a new repository manager
    pub fn new(api_version: impl Into<String>, download_dir: PathBuf) -> Self {
        Self {
            api_version: api_version.into(),
            download_dir,
            repositories: RwLock::new(HashMap::new()),
            info_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get the API version supported by this manager
    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    /// Add a repository
    pub async fn add_repository(
        &self,
        id: &str,
        provider: Arc<dyn RepositoryProvider>,
    ) -> Result<()> {
        // Get repository info
        let info = provider.get_info().await?;

        // Add to repositories
        let mut repositories = self.repositories.write().await;
        repositories.insert(id.to_string(), provider);

        // Add to info cache
        let mut info_cache = self.info_cache.write().await;
        info_cache.insert(id.to_string(), info);

        Ok(())
    }

    /// Remove a repository
    pub async fn remove_repository(&self, id: &str) -> Result<()> {
        // Remove from repositories
        let mut repositories = self.repositories.write().await;
        repositories.remove(id);

        // Remove from info cache
        let mut info_cache = self.info_cache.write().await;
        info_cache.remove(id);

        Ok(())
    }

    /// Get repository information
    pub async fn get_repositories(&self) -> HashMap<String, RepositoryInfo> {
        let info_cache = self.info_cache.read().await;
        info_cache.clone()
    }

    /// List all plugins from all repositories
    pub async fn list_plugins(&self) -> HashMap<String, Vec<PluginPackageInfo>> {
        let mut result = HashMap::new();
        let repositories = self.repositories.read().await;

        for (id, provider) in repositories.iter() {
            if let Ok(plugins) = provider.list_plugins().await {
                result.insert(id.clone(), plugins);
            }
        }

        result
    }

    /// Search for plugins by query string
    pub async fn search_plugins(&self, query: &str) -> HashMap<String, Vec<PluginPackageInfo>> {
        let mut result = HashMap::new();
        let repositories = self.repositories.read().await;

        for (id, provider) in repositories.iter() {
            if let Ok(plugins) = provider.search_plugins(query).await {
                if !plugins.is_empty() {
                    result.insert(id.clone(), plugins);
                }
            }
        }

        result
    }

    /// Get a specific plugin by ID from a specific repository
    pub async fn get_plugin(
        &self,
        repo_id: &str,
        plugin_id: &Uuid,
    ) -> Result<Option<PluginPackageInfo>> {
        let repositories = self.repositories.read().await;

        if let Some(provider) = repositories.get(repo_id) {
            provider.get_plugin(plugin_id).await
        } else {
            anyhow::bail!("Repository not found: {}", repo_id)
        }
    }

    /// Download a plugin
    pub async fn download_plugin(&self, repo_id: &str, plugin_id: &Uuid) -> Result<PathBuf> {
        let repositories = self.repositories.read().await;

        if let Some(provider) = repositories.get(repo_id) {
            // Create download directory if it doesn't exist
            tokio::fs::create_dir_all(&self.download_dir).await?;

            // Generate output path
            let output_path = self.download_dir.join(format!("{plugin_id}.zip"));

            // Download the plugin
            provider.download_plugin(plugin_id, &output_path).await?;

            Ok(output_path)
        } else {
            anyhow::bail!("Repository not found: {}", repo_id)
        }
    }
}

/// Create a new repository manager with default settings
#[cfg(feature = "marketplace")]
pub fn create_repository_manager(
    api_version: &str,
    download_dir: PathBuf,
) -> Result<Arc<RepositoryManager>> {
    Ok(Arc::new(RepositoryManager::new(api_version, download_dir)))
}
