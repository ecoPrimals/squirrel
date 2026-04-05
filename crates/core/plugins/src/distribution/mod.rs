// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin distribution module
//!
//! This module provides functionality for plugin distribution.

use std::fmt::Debug;
use std::path::Path;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use squirrel_interfaces::plugins::PluginMetadata;

/// Plugin package metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginPackageMetadata {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    
    /// Package version
    pub package_version: String,
    
    /// Package format
    pub package_format: String,
    
    /// Package dependencies
    pub package_dependencies: Vec<String>,
    
    /// Checksum
    pub checksum: String,
    
    /// Signature
    pub signature: Option<String>,
    
    /// Additional metadata
    pub additional_metadata: serde_json::Value,
}

/// Plugin package
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginPackage {
    /// Package ID
    pub id: Uuid,
    
    /// Package metadata
    pub metadata: PluginPackageMetadata,
    
    /// Package URL
    pub url: Option<String>,
    
    /// Package size
    pub size: u64,
    
    /// Package status
    pub status: PluginPackageStatus,
}

/// Plugin package status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginPackageStatus {
    /// Available
    Available,
    /// Downloaded
    Downloaded,
    /// Installed
    Installed,
    /// Failed
    Failed,
}

/// Plugin repository
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginRepository {
    /// Repository ID
    pub id: Uuid,
    
    /// Repository name
    pub name: String,
    
    /// Repository URL
    pub url: String,
    
    /// Repository authentication
    pub authentication: Option<RepositoryAuthentication>,
    
    /// Repository priority
    pub priority: u32,
    
    /// Repository enabled
    pub enabled: bool,
}

/// Repository authentication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepositoryAuthentication {
    /// Authentication type
    pub auth_type: String,
    
    /// Authentication credentials
    pub credentials: serde_json::Value,
}

/// Plugin distribution trait
#[expect(async_fn_in_trait, reason = "internal trait — all impls are Send + Sync")]
pub trait PluginDistribution: Send + Sync + Debug {
    /// List available plugins
    async fn list_available_plugins(&self) -> Result<Vec<PluginPackage>>;
    
    /// Get plugin package by ID
    async fn get_plugin_package(&self, id: Uuid) -> Result<PluginPackage>;
    
    /// Get plugin package by name
    async fn get_plugin_package_by_name(&self, name: &str) -> Result<Vec<PluginPackage>>;
    
    /// Search for plugin packages
    async fn search_plugin_packages(&self, query: &str) -> Result<Vec<PluginPackage>>;
    
    /// Download plugin package
    async fn download_plugin_package(&self, id: Uuid, destination: &Path) -> Result<PluginPackage>;
    
    /// Install plugin package
    async fn install_plugin_package(&self, id: Uuid) -> Result<Uuid>;
    
    /// Uninstall plugin
    async fn uninstall_plugin(&self, id: Uuid) -> Result<()>;
    
    /// Update plugin
    async fn update_plugin(&self, id: Uuid) -> Result<Uuid>;
    
    /// Add repository
    async fn add_repository(&self, repository: PluginRepository) -> Result<Uuid>;
    
    /// Remove repository
    async fn remove_repository(&self, id: Uuid) -> Result<()>;
    
    /// List repositories
    async fn list_repositories(&self) -> Result<Vec<PluginRepository>>;
    
    /// Enable repository
    async fn enable_repository(&self, id: Uuid) -> Result<()>;
    
    /// Disable repository
    async fn disable_repository(&self, id: Uuid) -> Result<()>;
    
    /// Refresh repositories
    async fn refresh_repositories(&self) -> Result<()>;
    
    /// Create plugin package
    async fn create_plugin_package(&self, plugin_id: Uuid, destination: &Path) -> Result<PluginPackage>;
    
    /// Verify plugin package
    async fn verify_plugin_package(&self, package_path: &Path) -> Result<bool>;
}

/// Standalone plugin distribution — no remote repository backend.
///
/// All listing methods return empty collections (no repositories configured).
/// Mutating operations return a typed error directing callers to configure
/// a repository backend or delegate to a plugin-management primal.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPluginDistribution;

impl DefaultPluginDistribution {
    /// Create a new default (no-backend) plugin distribution handler.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl PluginDistribution for DefaultPluginDistribution {
    async fn list_available_plugins(&self) -> Result<Vec<PluginPackage>> {
        Ok(Vec::new())
    }

    async fn get_plugin_package(&self, id: Uuid) -> Result<PluginPackage> {
        Err(anyhow::anyhow!(
            "No plugin repository configured — cannot fetch package {id}"
        ))
    }

    async fn get_plugin_package_by_name(&self, _name: &str) -> Result<Vec<PluginPackage>> {
        Ok(Vec::new())
    }

    async fn search_plugin_packages(&self, _query: &str) -> Result<Vec<PluginPackage>> {
        Ok(Vec::new())
    }

    async fn download_plugin_package(
        &self,
        id: Uuid,
        _destination: &Path,
    ) -> Result<PluginPackage> {
        Err(anyhow::anyhow!(
            "No plugin repository configured — cannot download package {id}"
        ))
    }

    async fn install_plugin_package(&self, id: Uuid) -> Result<Uuid> {
        Err(anyhow::anyhow!(
            "No plugin repository configured — cannot install package {id}"
        ))
    }

    async fn uninstall_plugin(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn update_plugin(&self, id: Uuid) -> Result<Uuid> {
        Err(anyhow::anyhow!(
            "No plugin repository configured — cannot update plugin {id}"
        ))
    }

    async fn add_repository(&self, repository: PluginRepository) -> Result<Uuid> {
        Err(anyhow::anyhow!(
            "Standalone distribution does not persist repositories — \
             cannot add '{}'. Configure a persistent backend.",
            repository.name,
        ))
    }

    async fn remove_repository(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn list_repositories(&self) -> Result<Vec<PluginRepository>> {
        Ok(Vec::new())
    }

    async fn enable_repository(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn disable_repository(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn refresh_repositories(&self) -> Result<()> {
        Ok(())
    }

    async fn create_plugin_package(
        &self,
        plugin_id: Uuid,
        _destination: &Path,
    ) -> Result<PluginPackage> {
        Err(anyhow::anyhow!(
            "No packaging backend configured — cannot create package for plugin {plugin_id}"
        ))
    }

    async fn verify_plugin_package(&self, _package_path: &Path) -> Result<bool> {
        Ok(true)
    }
}