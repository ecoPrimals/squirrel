//! Directory structure management for rules
use crate::rules::error::{Result, RuleError};
use async_recursion::async_recursion;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Default rules directory name
pub const DEFAULT_RULES_DIR: &str = ".rules";

/// Rule directory manager for managing rule files
#[derive(Debug)]
pub struct RuleDirectoryManager {
    /// Path to the rules directory
    rules_dir: PathBuf,
}

impl RuleDirectoryManager {
    /// Create a new rule directory manager
    pub fn new(rules_dir: impl AsRef<Path>) -> Self {
        Self {
            rules_dir: rules_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a new rule directory manager with the default rules directory
    pub async fn with_default(base_dir: impl Into<PathBuf>) -> Result<Self> {
        let base_dir = base_dir.into();
        let rules_dir = base_dir.join(DEFAULT_RULES_DIR);

        // Create the rules directory if it doesn't exist
        Self::ensure_directory(&rules_dir).await?;

        Ok(Self { rules_dir })
    }

    /// Get the rules directory
    pub fn rules_dir(&self) -> &Path {
        &self.rules_dir
    }

    /// Ensure a directory exists
    pub async fn ensure_directory<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();

        // If the path doesn't exist, create it
        if !path.exists() {
            fs::create_dir_all(path).await.map_err(|e| {
                RuleError::DirectoryError(format!("Failed to create directory: {e}"))
            })?;
        } else if !path.is_dir() {
            // If the path exists but is not a directory, return an error
            return Err(RuleError::DirectoryError(format!(
                "Path exists but is not a directory: {}",
                path.display()
            )));
        }

        Ok(())
    }

    /// List all rule files in a category
    #[async_recursion]
    pub async fn list_rule_files(&self, category: Option<&str>) -> Result<Vec<PathBuf>> {
        let base_path = match category {
            Some(cat) => self.rules_dir.join(cat),
            None => self.rules_dir.clone(),
        };

        // Ensure the directory exists
        Self::ensure_directory(&base_path).await?;

        let mut result = Vec::new();
        let mut dirs = vec![base_path];

        // Read all directories recursively
        while let Some(dir) = dirs.pop() {
            let mut entries = fs::read_dir(&dir)
                .await
                .map_err(|e| RuleError::DirectoryError(format!("Failed to read directory: {e}")))?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                RuleError::DirectoryError(format!("Failed to read directory entry: {e}"))
            })? {
                let path = entry.path();

                if path.is_dir() {
                    dirs.push(path);
                } else if let Some(ext) = path.extension() {
                    if ext == "mdc" {
                        result.push(path);
                    }
                }
            }
        }

        Ok(result)
    }

    /// Get the path for a category
    pub fn category_path(&self, category: &str) -> PathBuf {
        self.rules_dir.join(category)
    }

    /// Get the path for a rule
    pub fn rule_path(&self, category: &str, rule_id: &str) -> PathBuf {
        self.category_path(category).join(format!("{rule_id}.mdc"))
    }

    /// Create a category directory
    pub async fn create_category(&self, category: &str) -> Result<PathBuf> {
        let category_path = self.category_path(category);

        Self::ensure_directory(&category_path).await?;

        Ok(category_path)
    }

    /// Check if a path is a rule file
    pub fn is_rule_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            ext == "mdc" || ext == "md"
        } else {
            false
        }
    }

    /// Check if a rule exists
    pub async fn rule_exists(&self, category: &str, rule_id: &str) -> Result<bool> {
        let rule_path = self.rule_path(category, rule_id);

        Ok(rule_path.exists())
    }

    /// Create a rule file
    pub async fn create_rule_file(
        &self,
        category: &str,
        rule_id: &str,
        content: &str,
    ) -> Result<PathBuf> {
        let category_path = self.create_category(category).await?;
        let rule_path = category_path.join(format!("{rule_id}.mdc"));

        // Check if the rule already exists
        if rule_path.exists() {
            return Err(RuleError::AlreadyExists(rule_id.to_string()));
        }

        // Write the content
        fs::write(&rule_path, content)
            .await
            .map_err(|e| RuleError::DirectoryError(format!("Failed to write rule file: {e}")))?;

        Ok(rule_path)
    }

    /// Read a rule file
    pub async fn read_rule_file(&self, category: &str, id: &str) -> Result<String> {
        let rule_path = self.rule_path(category, id);

        if !rule_path.exists() {
            return Err(RuleError::NotFound(id.to_string()));
        }

        fs::read_to_string(&rule_path)
            .await
            .map_err(|e| RuleError::DirectoryError(format!("Failed to read rule file: {e}")))
    }

    /// Update a rule file
    pub async fn update_rule_file(&self, category: &str, id: &str, content: &str) -> Result<()> {
        let _category_path = self.create_category(category).await?;

        let rule_path = self.rule_path(category, id);

        // Create parent directories if they don't exist
        if let Some(parent) = rule_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                RuleError::DirectoryError(format!("Failed to create directory: {e}"))
            })?;
        }

        fs::write(&rule_path, content)
            .await
            .map_err(|e| RuleError::DirectoryError(format!("Failed to write rule file: {e}")))?;

        Ok(())
    }

    /// Delete a rule file
    pub async fn delete_rule_file(&self, category: &str, id: &str) -> Result<()> {
        let rule_path = self.rule_path(category, id);

        // Delete the file if it exists
        if rule_path.exists() {
            fs::remove_file(&rule_path).await.map_err(|e| {
                RuleError::DirectoryError(format!("Failed to delete rule file: {e}"))
            })?;
        }

        Ok(())
    }
}
