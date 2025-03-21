use std::path::{Path, PathBuf};
use std::fs;
use async_trait::async_trait;
use crate::error::Result;
use super::{Plugin, PluginMetadata, PluginManager};

/// Plugin discovery strategy trait
#[async_trait]
pub trait PluginDiscovery: Send + Sync {
    /// Discover plugins in the given directory
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The directory cannot be read
    /// - Plugin metadata cannot be loaded
    /// - Plugin validation fails
    async fn discover_plugins(&self, directory: &Path) -> Result<Vec<Box<dyn Plugin>>>;
    
    /// Load plugin metadata from a file
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file content cannot be parsed as JSON or TOML
    fn load_metadata(&self, path: &Path) -> Result<PluginMetadata>;
    
    /// Validate plugin compatibility
    /// 
    /// # Errors
    /// Returns an error if:
    /// - The plugin metadata is invalid
    /// - The plugin is incompatible with the current system
    fn validate_plugin(&self, metadata: &PluginMetadata) -> Result<()>;
}

/// Type alias for validation rules
type ValidationRule = Box<dyn Fn(&PluginMetadata) -> Result<()> + Send + Sync>;

/// File system based plugin discovery
#[allow(missing_debug_implementations)]
pub struct FileSystemDiscovery {
    /// Supported plugin file extensions
    extensions: Vec<String>,
    /// Plugin validation rules
    validation_rules: Vec<ValidationRule>,
}

impl FileSystemDiscovery {
    /// Create a new file system discovery
    #[must_use]
    pub fn new() -> Self {
        Self {
            extensions: vec!["json".to_string(), "toml".to_string()],
            validation_rules: Vec::new(),
        }
    }
    
    /// Add a supported file extension
    pub fn add_extension(&mut self, extension: String) {
        self.extensions.push(extension);
    }
    
    /// Add a validation rule
    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&PluginMetadata) -> Result<()> + Send + Sync + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }
    
    /// Check if file extension is supported
    fn is_supported_extension(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| self.extensions.contains(&ext.to_string()))
    }
}

#[async_trait]
impl PluginDiscovery for FileSystemDiscovery {
    async fn discover_plugins(&self, directory: &Path) -> Result<Vec<Box<dyn Plugin>>> {
        let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();
        
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && self.is_supported_extension(&path) {
                let metadata = self.load_metadata(&path)?;
                self.validate_plugin(&metadata)?;
                
                // Here we would actually load the plugin based on the metadata
                // For now, we'll just create a placeholder plugin
                let plugin: Box<dyn Plugin> = Box::new(PlaceholderPlugin { metadata });
                plugins.push(plugin);
            }
        }
        
        Ok(plugins)
    }
    
    fn load_metadata(&self, path: &Path) -> Result<PluginMetadata> {
        let content = fs::read_to_string(path)?;
        
        // Parse based on file extension
        let metadata = if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            serde_json::from_str::<PluginMetadata>(&content)?
        } else {
            toml::from_str::<PluginMetadata>(&content)?
        };
        
        Ok(metadata)
    }
    
    fn validate_plugin(&self, metadata: &PluginMetadata) -> Result<()> {
        for rule in &self.validation_rules {
            rule(metadata)?;
        }
        Ok(())
    }
}

impl Default for FileSystemDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin loader that handles plugin discovery and loading
#[allow(missing_debug_implementations)]
pub struct PluginLoader {
    /// Plugin manager
    manager: PluginManager,
    /// Plugin discovery strategy
    discovery: Box<dyn PluginDiscovery>,
    /// Plugin directories
    directories: Vec<PathBuf>,
}

impl PluginLoader {
    /// Create a new plugin loader
    #[must_use]
    pub fn new(manager: PluginManager, discovery: Box<dyn PluginDiscovery>) -> Self {
        Self {
            manager,
            discovery,
            directories: Vec::new(),
        }
    }
    
    /// Add a plugin directory
    pub fn add_directory<P: AsRef<Path>>(&mut self, directory: P) {
        self.directories.push(directory.as_ref().to_path_buf());
    }
    
    /// Load plugins from all registered directories
    /// 
    /// # Errors
    /// Returns an error if:
    /// - Any plugin directory cannot be read
    /// - Plugin discovery fails
    /// - Plugin registration fails
    pub async fn load_all(&self) -> Result<()> {
        for directory in &self.directories {
            let plugins = self.discovery.discover_plugins(directory).await?;
            for plugin in plugins {
                self.manager.register_plugin(plugin).await?;
            }
        }
        Ok(())
    }
}

/// Placeholder plugin for testing
#[derive(Debug)]
struct PlaceholderPlugin {
    /// Plugin metadata containing identification and capability information
    metadata: PluginMetadata,
}

#[async_trait]
impl Plugin for PlaceholderPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    async fn get_state(&self) -> Result<Option<super::PluginState>> {
        Ok(None)
    }
    
    async fn set_state(&self, _state: super::PluginState) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_plugin_discovery() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let plugin_path = temp_dir.path().join("test_plugin.json");
        
        // Create test plugin metadata
        let metadata = PluginMetadata {
            id: Uuid::new_v4(),
            name: "test_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            dependencies: vec![],
            capabilities: vec![],
        };
        
        // Write metadata to file
        let mut file = File::create(&plugin_path).unwrap();
        write!(file, "{}", serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
        
        // Create discovery and loader
        let mut discovery = FileSystemDiscovery::new();
        discovery.add_validation_rule(|_| Ok(()));
        
        let manager = PluginManager::new();
        let mut loader = PluginLoader::new(manager, Box::new(discovery));
        loader.add_directory(temp_dir.path());
        
        // Load plugins
        loader.load_all().await.unwrap();
    }
} 