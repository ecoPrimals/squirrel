// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Rule directory structure and utilities

use glob::glob;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

use crate::error::{RuleSystemError, RuleSystemResult};
use crate::models::RuleSourceMetadata;

/// Configuration for rule directories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDirectoryConfig {
    /// Root directory for rules
    pub root_directory: PathBuf,
    /// Default rule file extension
    pub default_extension: String,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Whether to watch for changes
    pub watch_for_changes: bool,
    /// Recursion depth for finding rules (0 = no recursion, -1 = unlimited)
    pub recursion_depth: i32,
}

impl Default for RuleDirectoryConfig {
    fn default() -> Self {
        Self {
            root_directory: PathBuf::from(".rules"),
            default_extension: "mdc".to_string(),
            include_patterns: vec![
                "**/*.mdc".to_string(),
                "**/*.yaml".to_string(),
                "**/*.yml".to_string(),
            ],
            exclude_patterns: vec!["**/.git/**".to_string(), "**/node_modules/**".to_string()],
            watch_for_changes: true,
            recursion_depth: -1,
        }
    }
}

/// Manager for rule directories
#[derive(Debug)]
pub struct RuleDirectoryManager {
    /// Configuration for rule directories
    config: Arc<RwLock<RuleDirectoryConfig>>,
    /// Sources for rules
    sources: Arc<RwLock<HashMap<String, RuleSourceMetadata>>>,
}

impl RuleDirectoryManager {
    /// Creates a new rule directory manager with the given configuration
    #[must_use]
    pub fn new(config: RuleDirectoryConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            sources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the rule directory manager
    ///
    /// This method creates the rule directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    pub async fn initialize(&self) -> RuleSystemResult<()> {
        let config = self.config.read().await;
        let root_dir = &config.root_directory;

        // Create root directory if it doesn't exist
        if !root_dir.exists() {
            fs::create_dir_all(root_dir).await?;
        }

        // Add default source
        let mut sources = self.sources.write().await;
        sources.insert(
            "default".to_string(),
            RuleSourceMetadata {
                directory: root_dir.clone(),
                pattern: "**/*.mdc".to_string(),
                watch: config.watch_for_changes,
            },
        );

        Ok(())
    }

    /// Add a rule source
    ///
    /// # Errors
    ///
    /// Returns an error if the source already exists or the directory is invalid.
    pub async fn add_source(
        &self,
        id: impl Into<String>,
        metadata: RuleSourceMetadata,
    ) -> RuleSystemResult<()> {
        let id = id.into();
        let mut sources = self.sources.write().await;

        // Check if source already exists
        if sources.contains_key(&id) {
            return Err(RuleSystemError::Other(format!(
                "Source already exists: {id}"
            )));
        }

        // Check if directory exists
        if !metadata.directory.exists() {
            return Err(RuleSystemError::DirectoryNotFound(
                metadata.directory.clone(),
            ));
        }

        // Add the source
        sources.insert(id, metadata);

        Ok(())
    }

    /// Get all rule files from all sources
    ///
    /// # Errors
    ///
    /// Returns an error if any source cannot be read.
    pub async fn get_all_rule_files(&self) -> RuleSystemResult<Vec<PathBuf>> {
        let sources = self.sources.read().await;
        let mut rule_files = Vec::new();

        for source in sources.values() {
            let pattern = format!("{}/{}", source.directory.display(), source.pattern);
            let glob_pattern = if cfg!(windows) {
                pattern.replace('\\', "/")
            } else {
                pattern
            };

            // Use glob to find matching files
            // Note: glob is synchronous, but ok for occasional use
            for entry in glob(&glob_pattern).map_err(|e| RuleSystemError::Other(e.to_string()))? {
                match entry {
                    Ok(path) => rule_files.push(path),
                    Err(e) => return Err(RuleSystemError::Other(e.to_string())),
                }
            }
        }

        Ok(rule_files)
    }

    /// Get all rule files from a specific source
    ///
    /// # Errors
    ///
    /// Returns an error if the source doesn't exist or cannot be read.
    pub async fn get_source_rule_files(&self, source_id: &str) -> RuleSystemResult<Vec<PathBuf>> {
        let sources = self.sources.read().await;

        // Check if source exists
        let source = sources
            .get(source_id)
            .ok_or_else(|| RuleSystemError::Other(format!("Source not found: {source_id}")))?;

        let pattern = format!("{}/{}", source.directory.display(), source.pattern);
        let glob_pattern = if cfg!(windows) {
            pattern.replace('\\', "/")
        } else {
            pattern
        };

        let mut rule_files = Vec::new();

        // Use glob to find matching files
        for entry in glob(&glob_pattern).map_err(|e| RuleSystemError::Other(e.to_string()))? {
            match entry {
                Ok(path) => rule_files.push(path),
                Err(e) => return Err(RuleSystemError::Other(e.to_string())),
            }
        }

        Ok(rule_files)
    }

    /// Create a rule file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created.
    pub async fn create_rule_file(
        &self,
        id: impl AsRef<str>,
        category: Option<impl AsRef<str>>,
        content: impl AsRef<str>,
    ) -> RuleSystemResult<PathBuf> {
        let config = self.config.read().await;
        let root_dir = &config.root_directory;

        // Determine the file path
        let file_path = if let Some(category) = category {
            let category_dir = root_dir.join(category.as_ref());

            // Create category directory if it doesn't exist
            if !category_dir.exists() {
                fs::create_dir_all(&category_dir).await?;
            }

            category_dir.join(format!("{}.{}", id.as_ref(), config.default_extension))
        } else {
            root_dir.join(format!("{}.{}", id.as_ref(), config.default_extension))
        };

        // Write the file
        fs::write(&file_path, content.as_ref()).await?;

        Ok(file_path)
    }

    /// Check if a rule file exists
    ///
    /// # Errors
    ///
    /// Returns an error if the file system cannot be accessed.
    pub async fn rule_file_exists(
        &self,
        id: impl AsRef<str>,
        category: Option<impl AsRef<str>>,
    ) -> RuleSystemResult<bool> {
        let config = self.config.read().await;
        let root_dir = &config.root_directory;

        // Determine the file path
        let file_path = if let Some(category) = category {
            root_dir.join(category.as_ref()).join(format!(
                "{}.{}",
                id.as_ref(),
                config.default_extension
            ))
        } else {
            root_dir.join(format!("{}.{}", id.as_ref(), config.default_extension))
        };

        Ok(file_path.exists())
    }

    /// Delete a rule file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be deleted.
    pub async fn delete_rule_file(
        &self,
        id: impl AsRef<str>,
        category: Option<impl AsRef<str>>,
    ) -> RuleSystemResult<()> {
        let config = self.config.read().await;
        let root_dir = &config.root_directory;

        // Determine the file path
        let file_path = if let Some(category) = category {
            root_dir.join(category.as_ref()).join(format!(
                "{}.{}",
                id.as_ref(),
                config.default_extension
            ))
        } else {
            root_dir.join(format!("{}.{}", id.as_ref(), config.default_extension))
        };

        // Check if file exists
        if !file_path.exists() {
            return Err(RuleSystemError::FileNotFound(file_path));
        }

        // Delete the file
        fs::remove_file(&file_path).await?;

        Ok(())
    }

    /// Get all categories
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read.
    pub async fn get_categories(&self) -> RuleSystemResult<Vec<String>> {
        let config = self.config.read().await;
        let root_dir = &config.root_directory;

        let mut categories = Vec::new();

        let mut entries = fs::read_dir(root_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir()
                && let Some(category) = path.file_name()
                && let Some(category_str) = category.to_str()
            {
                categories.push(category_str.to_string());
            }
        }

        Ok(categories)
    }

    /// Update the configuration
    pub async fn update_config(&self, config: RuleDirectoryConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
    }

    /// Get the configuration
    pub async fn get_config(&self) -> RuleDirectoryConfig {
        let config = self.config.read().await;
        config.clone()
    }
}

impl Default for RuleDirectoryManager {
    fn default() -> Self {
        Self::new(RuleDirectoryConfig::default())
    }
}

/// Factory for creating rule directory managers
#[derive(Debug, Default)]
pub struct RuleDirectoryManagerFactory;

impl RuleDirectoryManagerFactory {
    /// Creates a new rule directory manager with the default configuration
    #[must_use]
    pub fn create_manager() -> RuleDirectoryManager {
        RuleDirectoryManager::default()
    }

    /// Creates a new rule directory manager with the given configuration
    #[must_use]
    pub fn create_manager_with_config(config: RuleDirectoryConfig) -> RuleDirectoryManager {
        RuleDirectoryManager::new(config)
    }

    /// Creates a new rule directory manager with a custom root directory
    pub fn create_manager_with_root_dir(root_dir: impl Into<PathBuf>) -> RuleDirectoryManager {
        let config = RuleDirectoryConfig {
            root_directory: root_dir.into(),
            ..Default::default()
        };
        RuleDirectoryManager::new(config)
    }
}

/// Creates a new rule directory manager with the default configuration
#[must_use]
pub fn create_rule_directory_manager() -> RuleDirectoryManager {
    RuleDirectoryManagerFactory::create_manager()
}

/// Creates a new rule directory manager with the given configuration
#[must_use]
pub fn create_rule_directory_manager_with_config(
    config: RuleDirectoryConfig,
) -> RuleDirectoryManager {
    RuleDirectoryManagerFactory::create_manager_with_config(config)
}

/// Creates a new rule directory manager with a custom root directory
pub fn create_rule_directory_manager_with_root_dir(
    root_dir: impl Into<PathBuf>,
) -> RuleDirectoryManager {
    RuleDirectoryManagerFactory::create_manager_with_root_dir(root_dir)
}

/// File watcher trait for watching rule files
#[async_trait::async_trait]
pub trait FileWatcher: Send + Sync {
    /// Start watching a directory
    async fn watch_directory(
        &self,
        directory: impl AsRef<Path> + Send,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Stop watching a directory
    async fn stop_watching(
        &self,
        directory: impl AsRef<Path> + Send,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get all watched directories
    async fn get_watched_directories(&self) -> Vec<PathBuf>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RuleSourceMetadata;

    #[test]
    fn rule_directory_config_default_and_serde() {
        let c = RuleDirectoryConfig::default();
        assert_eq!(c.default_extension, "mdc");
        assert_eq!(c.recursion_depth, -1);
        let json = serde_json::to_string(&c).unwrap();
        let back: RuleDirectoryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.root_directory, c.root_directory);
    }

    #[tokio::test]
    async fn initialize_creates_root_and_default_source() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = RuleDirectoryConfig {
            root_directory: dir.path().to_path_buf(),
            ..RuleDirectoryConfig::default()
        };
        let mgr = RuleDirectoryManager::new(cfg);
        mgr.initialize().await.unwrap();
        assert!(dir.path().exists());
        let files = mgr.get_all_rule_files().await.unwrap();
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn add_source_errors() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = RuleDirectoryManager::new(RuleDirectoryConfig {
            root_directory: dir.path().to_path_buf(),
            ..Default::default()
        });
        mgr.initialize().await.unwrap();

        let other = tempfile::tempdir().unwrap();
        mgr.add_source(
            "s1",
            RuleSourceMetadata {
                directory: other.path().to_path_buf(),
                pattern: "*.mdc".to_string(),
                watch: false,
            },
        )
        .await
        .unwrap();

        let err = mgr
            .add_source(
                "s1",
                RuleSourceMetadata {
                    directory: other.path().to_path_buf(),
                    pattern: "*.mdc".to_string(),
                    watch: false,
                },
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("already exists"));

        let missing = dir.path().join("nope");
        let err = mgr
            .add_source(
                "bad",
                RuleSourceMetadata {
                    directory: missing.clone(),
                    pattern: "*.mdc".to_string(),
                    watch: false,
                },
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Directory") || err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn create_list_delete_rule_files_and_categories() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = RuleDirectoryManager::new(RuleDirectoryConfig {
            root_directory: dir.path().to_path_buf(),
            ..Default::default()
        });
        mgr.initialize().await.unwrap();

        let p = mgr
            .create_rule_file("alpha", None::<&str>, "body")
            .await
            .unwrap();
        assert!(p.exists());

        let p2 = mgr
            .create_rule_file("beta", Some("cat"), "c")
            .await
            .unwrap();
        assert!(p2.exists());

        assert!(mgr.rule_file_exists("alpha", None::<&str>).await.unwrap());
        assert!(mgr.rule_file_exists("beta", Some("cat")).await.unwrap());

        let cats = mgr.get_categories().await.unwrap();
        assert!(cats.contains(&"cat".to_string()));

        let all = mgr.get_all_rule_files().await.unwrap();
        assert!(all.len() >= 2);

        mgr.delete_rule_file("alpha", None::<&str>).await.unwrap();
        assert!(!mgr.rule_file_exists("alpha", None::<&str>).await.unwrap());

        let err = mgr
            .delete_rule_file("nope", None::<&str>)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("File"));
    }

    #[tokio::test]
    async fn get_source_rule_files_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = RuleDirectoryManager::new(RuleDirectoryConfig {
            root_directory: dir.path().to_path_buf(),
            ..Default::default()
        });
        mgr.initialize().await.unwrap();
        let err = mgr.get_source_rule_files("missing").await.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn config_get_and_update() {
        let dir = tempfile::tempdir().unwrap();
        let mut cfg = RuleDirectoryConfig::default();
        cfg.root_directory = dir.path().to_path_buf();
        let mgr = RuleDirectoryManager::new(cfg);
        mgr.update_config(RuleDirectoryConfig {
            watch_for_changes: false,
            root_directory: dir.path().to_path_buf(),
            ..RuleDirectoryConfig::default()
        })
        .await;
        let g = mgr.get_config().await;
        assert!(!g.watch_for_changes);
    }

    #[tokio::test]
    async fn factory_and_helpers() {
        let _ = RuleDirectoryManagerFactory::default();
        let m = RuleDirectoryManagerFactory::create_manager();
        assert_eq!(
            m.get_config().await.root_directory,
            RuleDirectoryConfig::default().root_directory
        );

        let tmp = tempfile::tempdir().unwrap();
        let m2 = create_rule_directory_manager_with_root_dir(tmp.path());
        assert_eq!(m2.get_config().await.root_directory, tmp.path());

        let m3 = create_rule_directory_manager_with_config(RuleDirectoryConfig {
            root_directory: tmp.path().to_path_buf(),
            ..Default::default()
        });
        assert_eq!(m3.get_config().await.root_directory, tmp.path());

        let _ = create_rule_directory_manager();
    }
}
