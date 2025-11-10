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

    /// Creates a new rule directory manager with the default configuration
    #[must_use] 
    pub fn default() -> Self {
        Self::new(RuleDirectoryConfig::default())
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

            if path.is_dir() {
                if let Some(category) = path.file_name() {
                    if let Some(category_str) = category.to_str() {
                        categories.push(category_str.to_string());
                    }
                }
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
        let mut config = RuleDirectoryConfig::default();
        config.root_directory = root_dir.into();
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
