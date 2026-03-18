// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin Marketplace Integration
//!
//! This module provides marketplace functionality for discovering, browsing, and installing
//! plugins from remote repositories. It includes search capabilities, plugin verification,
//! and installation management.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::DefaultPluginManager;
use crate::web::{HttpMethod, HttpStatus, WebEndpoint, WebRequest, WebResponse};

/// Plugin marketplace client for interacting with remote plugin repositories
#[derive(Clone)]
pub struct PluginMarketplaceClient {
    /// Plugin manager instance (reserved for marketplace plugin management)
    #[expect(
        dead_code,
        reason = "Phase 2 placeholder — marketplace plugin management"
    )]
    manager: Arc<DefaultPluginManager>,
    /// Configured repositories
    repositories: Arc<RwLock<Vec<PluginRepository>>>,
    /// HTTP client for making requests
    #[cfg(feature = "marketplace")]
    #[expect(dead_code, reason = "used when feature marketplace is enabled")]
    http_client: reqwest::Client,
    /// Cache for marketplace data
    cache: Arc<RwLock<MarketplaceCache>>,
}

/// Plugin repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRepository {
    /// Repository ID
    pub id: Uuid,
    /// Repository name
    pub name: String,
    /// Repository URL
    pub url: String,
    /// Repository type (official, community, private)
    pub repo_type: String,
    /// Whether the repository is enabled
    pub enabled: bool,
    /// Authentication configuration
    pub auth: Option<RepositoryAuth>,
    /// Repository metadata
    pub metadata: HashMap<String, String>,
}

/// Repository authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAuth {
    /// Authentication type (token, basic, oauth)
    pub auth_type: String,
    /// Authentication credentials
    pub credentials: HashMap<String, String>,
}

/// Plugin marketplace entry with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    /// Plugin ID
    pub id: Uuid,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin category
    pub category: String,
    /// Plugin tags
    pub tags: Vec<String>,
    /// Plugin capabilities
    pub capabilities: Vec<String>,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    /// Download URL
    pub download_url: String,
    /// Documentation URL
    pub documentation_url: Option<String>,
    /// Repository URL
    pub repository_url: Option<String>,
    /// Plugin rating
    pub rating: Option<f64>,
    /// Number of downloads
    pub downloads: u64,
    /// Whether the plugin is verified
    pub verified: bool,
    /// Plugin size in bytes
    pub size: u64,
    /// Publication date
    pub published_at: chrono::DateTime<chrono::Utc>,
    /// Last update date
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Minimum system requirements
    pub requirements: SystemRequirements,
    /// Plugin screenshots
    pub screenshots: Vec<String>,
    /// Plugin changelog
    pub changelog: Option<String>,
}

/// System requirements for plugin installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    /// Minimum version required
    pub min_version: String,
    /// Required features
    pub features: Vec<String>,
    /// Memory requirements in MB
    pub memory_mb: Option<u64>,
    /// Disk space requirements in MB
    pub disk_space_mb: Option<u64>,
}

/// Marketplace search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSearchCriteria {
    /// Search query
    pub query: Option<String>,
    /// Filter by category
    pub category: Option<String>,
    /// Filter by author
    pub author: Option<String>,
    /// Filter by capabilities
    pub capabilities: Option<Vec<String>>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Minimum rating
    pub min_rating: Option<f64>,
    /// Only verified plugins
    pub verified_only: Option<bool>,
    /// Sort order (rating, downloads, date, name)
    pub sort_by: Option<String>,
    /// Sort direction (asc, desc)
    pub sort_order: Option<String>,
    /// Page number for pagination
    pub page: Option<usize>,
    /// Number of results per page
    pub per_page: Option<usize>,
}

/// Search results from marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSearchResults {
    /// Found plugins
    pub plugins: Vec<MarketplacePlugin>,
    /// Total number of results
    pub total: usize,
    /// Current page
    pub page: usize,
    /// Results per page
    pub per_page: usize,
    /// Total pages
    pub total_pages: usize,
    /// Search criteria used
    pub criteria: MarketplaceSearchCriteria,
}

/// Marketplace cache for performance
#[derive(Debug, Default)]
pub struct MarketplaceCache {
    /// Cached search results
    pub search_results: HashMap<String, (MarketplaceSearchResults, chrono::DateTime<chrono::Utc>)>,
    /// Cached plugin details
    pub plugin_details: HashMap<Uuid, (MarketplacePlugin, chrono::DateTime<chrono::Utc>)>,
    /// Cached categories
    pub categories: Option<(Vec<String>, chrono::DateTime<chrono::Utc>)>,
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
}

/// Plugin installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationStatus {
    /// Installation ID
    pub id: Uuid,
    /// Plugin being installed
    pub plugin_id: Uuid,
    /// Current status
    pub status: InstallationStatusType,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Current step description
    pub current_step: String,
    /// Installation log
    pub logs: Vec<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Installation started at
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Installation completed at
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Installation status types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationStatusType {
    /// Installation queued
    Queued,
    /// Downloading plugin
    Downloading,
    /// Verifying plugin
    Verifying,
    /// Installing dependencies
    InstallingDependencies,
    /// Installing plugin
    Installing,
    /// Configuring plugin
    Configuring,
    /// Installation completed
    Completed,
    /// Installation failed
    Failed,
    /// Installation cancelled
    Cancelled,
}

impl PluginMarketplaceClient {
    /// Create a new marketplace client
    pub fn new(manager: Arc<DefaultPluginManager>) -> Self {
        let cache = MarketplaceCache {
            ttl_seconds: 300, // 5 minutes cache
            ..Default::default()
        };

        Self {
            manager,
            repositories: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "marketplace")]
            http_client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    /// Get marketplace endpoints
    pub fn get_endpoints(&self) -> Vec<WebEndpoint> {
        vec![
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/search".to_string(),
                HttpMethod::Post,
                "Advanced plugin search with filters".to_string(),
            )
            .make_public()
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/featured".to_string(),
                HttpMethod::Get,
                "Get featured plugins".to_string(),
            )
            .make_public()
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/trending".to_string(),
                HttpMethod::Get,
                "Get trending plugins".to_string(),
            )
            .make_public()
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/repositories".to_string(),
                HttpMethod::Get,
                "List configured repositories".to_string(),
            )
            .with_permission("marketplace.repositories.read")
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/repositories".to_string(),
                HttpMethod::Post,
                "Add a new repository".to_string(),
            )
            .with_permission("marketplace.repositories.write")
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/repositories/:id".to_string(),
                HttpMethod::Delete,
                "Remove a repository".to_string(),
            )
            .with_permission("marketplace.repositories.write")
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/install/:id".to_string(),
                HttpMethod::Post,
                "Install plugin from marketplace".to_string(),
            )
            .with_permission("plugin.install")
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/installations".to_string(),
                HttpMethod::Get,
                "Get installation status list".to_string(),
            )
            .with_permission("plugin.install")
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/installations/:id".to_string(),
                HttpMethod::Get,
                "Get installation status".to_string(),
            )
            .with_permission("plugin.install")
            .with_tag("marketplace"),
            WebEndpoint::new(
                Uuid::new_v4(),
                "/api/marketplace/installations/:id/cancel".to_string(),
                HttpMethod::Post,
                "Cancel installation".to_string(),
            )
            .with_permission("plugin.install")
            .with_tag("marketplace"),
        ]
    }

    /// Handle marketplace request
    pub async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        match (request.method, request.path.as_str()) {
            (HttpMethod::Post, "/api/marketplace/search") => {
                let criteria: MarketplaceSearchCriteria =
                    serde_json::from_value(request.body.unwrap_or_default())?;
                self.search_plugins(criteria).await
            }
            (HttpMethod::Get, "/api/marketplace/featured") => self.get_featured_plugins().await,
            (HttpMethod::Get, "/api/marketplace/trending") => self.get_trending_plugins().await,
            (HttpMethod::Get, "/api/marketplace/repositories") => self.list_repositories().await,
            (HttpMethod::Post, "/api/marketplace/repositories") => {
                let repo: PluginRepository =
                    serde_json::from_value(request.body.unwrap_or_default())?;
                self.add_repository(repo).await
            }
            (HttpMethod::Delete, path) if path.starts_with("/api/marketplace/repositories/") => {
                let repo_id = self.extract_uuid_from_path(path)?;
                self.remove_repository(repo_id).await
            }
            (HttpMethod::Post, path) if path.starts_with("/api/marketplace/install/") => {
                let plugin_id = self.extract_uuid_from_path(path)?;
                self.install_plugin(plugin_id).await
            }
            (HttpMethod::Get, "/api/marketplace/installations") => self.get_installations().await,
            (HttpMethod::Get, path)
                if path.starts_with("/api/marketplace/installations/")
                    && !path.ends_with("/cancel") =>
            {
                let installation_id = self.extract_uuid_from_path(path)?;
                self.get_installation_status(installation_id).await
            }
            (HttpMethod::Post, path) if path.ends_with("/cancel") => {
                let installation_id = self.extract_uuid_from_path(path)?;
                self.cancel_installation(installation_id).await
            }
            _ => Ok(WebResponse {
                status: HttpStatus::NotFound,
                headers: HashMap::new(),
                body: Some(serde_json::json!({
                    "error": "Not Found",
                    "message": format!("No marketplace endpoint found for {} {}", request.method, request.path)
                })),
            }),
        }
    }

    /// Search plugins in marketplace
    async fn search_plugins(&self, criteria: MarketplaceSearchCriteria) -> Result<WebResponse> {
        // Check cache first
        let cache_key = self.generate_cache_key(&criteria);
        {
            let cache = self.cache.read().await;
            if let Some((results, cached_at)) = cache.search_results.get(&cache_key)
                && (chrono::Utc::now() - *cached_at).num_seconds() < cache.ttl_seconds as i64
            {
                return Ok(WebResponse {
                    status: HttpStatus::Ok,
                    headers: HashMap::new(),
                    body: Some(serde_json::to_value(results)?),
                });
            }
        }

        // Perform search across all enabled repositories
        let mut all_plugins = Vec::new();
        let repositories = self.repositories.read().await;

        for repo in repositories.iter().filter(|r| r.enabled) {
            match self.search_repository(repo, &criteria).await {
                Ok(mut plugins) => all_plugins.append(&mut plugins),
                Err(e) => {
                    tracing::warn!("Failed to search repository {}: {}", repo.name, e);
                }
            }
        }

        // Apply sorting and pagination
        let results = self.process_search_results(all_plugins, &criteria)?;

        // Cache results
        {
            let mut cache = self.cache.write().await;
            cache
                .search_results
                .insert(cache_key, (results.clone(), chrono::Utc::now()));
        }

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(results)?),
        })
    }

    /// Get featured plugins
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn get_featured_plugins(&self) -> Result<WebResponse> {
        let featured_plugins = self.get_sample_plugins("featured");

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugins": featured_plugins,
                "total": featured_plugins.len()
            })),
        })
    }

    /// Get trending plugins
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn get_trending_plugins(&self) -> Result<WebResponse> {
        let trending_plugins = self.get_sample_plugins("trending");

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "plugins": trending_plugins,
                "total": trending_plugins.len()
            })),
        })
    }

    /// List configured repositories
    async fn list_repositories(&self) -> Result<WebResponse> {
        let repositories = self.repositories.read().await;

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "repositories": repositories.clone(),
                "total": repositories.len()
            })),
        })
    }

    /// Add a new repository
    async fn add_repository(&self, mut repo: PluginRepository) -> Result<WebResponse> {
        repo.id = Uuid::new_v4();

        let mut repositories = self.repositories.write().await;
        repositories.push(repo.clone());

        Ok(WebResponse {
            status: HttpStatus::Created,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(repo)?),
        })
    }

    /// Remove a repository
    async fn remove_repository(&self, repo_id: Uuid) -> Result<WebResponse> {
        let mut repositories = self.repositories.write().await;
        repositories.retain(|r| r.id != repo_id);

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "message": "Repository removed successfully"
            })),
        })
    }

    /// Install plugin from marketplace
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn install_plugin(&self, plugin_id: Uuid) -> Result<WebResponse> {
        let installation_id = Uuid::new_v4();

        // Create installation status
        let installation_status = InstallationStatus {
            id: installation_id,
            plugin_id,
            status: InstallationStatusType::Queued,
            progress: 0,
            current_step: "Queued for installation".to_string(),
            logs: vec!["Installation queued".to_string()],
            error: None,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        // In real implementation, this would start an async installation process
        // For now, return the installation status
        Ok(WebResponse {
            status: HttpStatus::Accepted,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(installation_status)?),
        })
    }

    /// Get installation status list
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn get_installations(&self) -> Result<WebResponse> {
        // In real implementation, this would return actual installation statuses
        let installations = vec![InstallationStatus {
            id: Uuid::new_v4(),
            plugin_id: Uuid::new_v4(),
            status: InstallationStatusType::Completed,
            progress: 100,
            current_step: "Installation completed".to_string(),
            logs: vec!["Installation completed successfully".to_string()],
            error: None,
            started_at: chrono::Utc::now() - chrono::Duration::minutes(5),
            completed_at: Some(chrono::Utc::now() - chrono::Duration::minutes(2)),
        }];

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "installations": installations,
                "total": installations.len()
            })),
        })
    }

    /// Get installation status
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn get_installation_status(&self, installation_id: Uuid) -> Result<WebResponse> {
        // In real implementation, this would fetch the actual installation status
        let installation_status = InstallationStatus {
            id: installation_id,
            plugin_id: Uuid::new_v4(),
            status: InstallationStatusType::Installing,
            progress: 75,
            current_step: "Installing plugin files".to_string(),
            logs: vec![
                "Download completed".to_string(),
                "Verification successful".to_string(),
                "Installing plugin files...".to_string(),
            ],
            error: None,
            started_at: chrono::Utc::now() - chrono::Duration::minutes(2),
            completed_at: None,
        };

        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::to_value(installation_status)?),
        })
    }

    /// Cancel installation
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn cancel_installation(&self, installation_id: Uuid) -> Result<WebResponse> {
        // In real implementation, this would cancel the actual installation
        Ok(WebResponse {
            status: HttpStatus::Ok,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "installation_id": installation_id,
                "status": "cancelled",
                "message": "Installation cancelled successfully"
            })),
        })
    }

    /// Helper function to search a specific repository
    #[expect(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    async fn search_repository(
        &self,
        _repo: &PluginRepository,
        _criteria: &MarketplaceSearchCriteria,
    ) -> Result<Vec<MarketplacePlugin>> {
        // In real implementation, this would make HTTP requests to the repository API
        // For now, return sample data
        Ok(self.get_sample_plugins("search"))
    }

    /// Helper function to process search results
    fn process_search_results(
        &self,
        mut plugins: Vec<MarketplacePlugin>,
        criteria: &MarketplaceSearchCriteria,
    ) -> Result<MarketplaceSearchResults> {
        // Apply sorting
        match criteria.sort_by.as_deref() {
            Some("rating") => plugins.sort_by(|a, b| {
                let a_rating = a.rating.unwrap_or(0.0);
                let b_rating = b.rating.unwrap_or(0.0);
                if criteria.sort_order.as_deref() == Some("desc") {
                    b_rating
                        .partial_cmp(&a_rating)
                        .unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    a_rating
                        .partial_cmp(&b_rating)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
            Some("downloads") => plugins.sort_by(|a, b| {
                if criteria.sort_order.as_deref() == Some("desc") {
                    b.downloads.cmp(&a.downloads)
                } else {
                    a.downloads.cmp(&b.downloads)
                }
            }),
            Some("name") => plugins.sort_by(|a, b| {
                if criteria.sort_order.as_deref() == Some("desc") {
                    b.name.cmp(&a.name)
                } else {
                    a.name.cmp(&b.name)
                }
            }),
            _ => plugins.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)), // Default: newest first
        }

        // Apply pagination
        let page = criteria.page.unwrap_or(1);
        let per_page = criteria.per_page.unwrap_or(20);
        let total = plugins.len();
        let total_pages = total.div_ceil(per_page);

        let start = (page - 1) * per_page;
        let end = std::cmp::min(start + per_page, total);
        let page_plugins = plugins.into_iter().skip(start).take(end - start).collect();

        Ok(MarketplaceSearchResults {
            plugins: page_plugins,
            total,
            page,
            per_page,
            total_pages,
            criteria: criteria.clone(),
        })
    }

    /// Generate cache key for search criteria
    fn generate_cache_key(&self, criteria: &MarketplaceSearchCriteria) -> String {
        format!("{criteria:?}")
    }

    /// Extract UUID from URL path
    fn extract_uuid_from_path(&self, path: &str) -> Result<Uuid> {
        let parts: Vec<&str> = path.split('/').collect();
        if let Some(id_str) = parts.last() {
            if *id_str == "cancel" && parts.len() > 1 {
                // Handle /cancel endpoints
                Uuid::parse_str(parts[parts.len() - 2])
                    .map_err(|e| anyhow::anyhow!("Invalid UUID: {e}"))
            } else {
                Uuid::parse_str(id_str).map_err(|e| anyhow::anyhow!("Invalid UUID: {e}"))
            }
        } else {
            Err(anyhow::anyhow!("No UUID found in path"))
        }
    }

    /// Get sample plugins for demonstration
    fn get_sample_plugins(&self, plugin_type: &str) -> Vec<MarketplacePlugin> {
        vec![MarketplacePlugin {
            id: Uuid::new_v4(),
            name: format!("Sample {plugin_type} Plugin"),
            version: "1.0.0".to_string(),
            description: format!("A sample {plugin_type} plugin for demonstration"),
            author: "Plugin Author".to_string(),
            category: "utility".to_string(),
            tags: vec!["sample".to_string(), plugin_type.to_string()],
            capabilities: vec!["web".to_string(), "command".to_string()],
            dependencies: vec!["tokio".to_string(), "serde".to_string()],
            download_url: "https://example.com/plugin.zip".to_string(),
            documentation_url: Some("https://example.com/docs".to_string()),
            repository_url: Some("https://github.com/example/plugin".to_string()),
            rating: Some(4.5),
            downloads: 1000,
            verified: true,
            size: 1024 * 1024, // 1MB
            published_at: chrono::Utc::now() - chrono::Duration::days(30),
            updated_at: chrono::Utc::now() - chrono::Duration::days(7),
            requirements: SystemRequirements {
                min_version: "1.0.0".to_string(),
                features: vec!["web".to_string()],
                memory_mb: Some(64),
                disk_space_mb: Some(10),
            },
            screenshots: vec![
                "https://example.com/screenshot1.png".to_string(),
                "https://example.com/screenshot2.png".to_string(),
            ],
            changelog: Some("Initial release".to_string()),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DefaultPluginManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_marketplace_client_creation() {
        let manager = Arc::new(DefaultPluginManager::new());
        let client = PluginMarketplaceClient::new(manager);

        let endpoints = client.get_endpoints();
        assert!(!endpoints.is_empty());
        assert!(
            endpoints
                .iter()
                .any(|ep| ep.path == "/api/marketplace/search")
        );
    }

    #[tokio::test]
    async fn test_repository_management() {
        let manager = Arc::new(DefaultPluginManager::new());
        let client = PluginMarketplaceClient::new(manager);

        let repo = PluginRepository {
            id: Uuid::new_v4(),
            name: "Test Repository".to_string(),
            url: "https://example.com/plugins".to_string(),
            repo_type: "community".to_string(),
            enabled: true,
            auth: None,
            metadata: HashMap::new(),
        };

        let response = client.add_repository(repo).await.unwrap();
        assert_eq!(response.status, HttpStatus::Created);
    }

    #[tokio::test]
    async fn test_plugin_search() {
        let manager = Arc::new(DefaultPluginManager::new());
        let client = PluginMarketplaceClient::new(manager);

        let criteria = MarketplaceSearchCriteria {
            query: Some("test".to_string()),
            category: None,
            author: None,
            capabilities: None,
            tags: None,
            min_rating: None,
            verified_only: None,
            sort_by: Some("rating".to_string()),
            sort_order: Some("desc".to_string()),
            page: Some(1),
            per_page: Some(10),
        };

        let response = client.search_plugins(criteria).await.unwrap();
        assert_eq!(response.status, HttpStatus::Ok);
    }
}
