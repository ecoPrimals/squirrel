use std::path::{Path, PathBuf};
use std::fs;
use async_trait::async_trait;
use crate::error::Result;
use super::{Plugin, PluginMetadata, PluginManager};
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use futures::future::BoxFuture;
use std::any::Any;
use crate::plugin::PluginState;
use std::sync::RwLock as StdRwLock;
use std::sync::Arc;
use tokio::sync::RwLock;
use walkdir;
use anyhow;

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
    /// Security level for validation
    #[allow(dead_code)]
    security_level: SecurityLevel,
}

impl FileSystemDiscovery {
    /// Create a new file system discovery
    #[must_use]
    pub fn new(security_level: SecurityLevel) -> Self {
        Self {
            extensions: vec!["json".to_string(), "toml".to_string()],
            validation_rules: Vec::new(),
            security_level,
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
        Self::new(SecurityLevel::Basic)
    }
}

/// Plugin loader trait
#[async_trait]
pub trait PluginLoaderTrait: Send + Sync {
    /// Load a plugin from a manifest
    #[allow(unused)]
    async fn load_plugin(&self, manifest: &PluginManifest, path: &Path) -> Result<Box<dyn Plugin>>;
    
    /// Create plugin sandbox
    #[allow(unused)]
    async fn create_sandbox(&self, manifest: &PluginManifest) -> Result<PluginSandbox>;
}

/// Plugin sandbox for resource isolation
#[derive(Debug)]
pub struct PluginSandbox {
    /// Plugin ID
    #[allow(dead_code)]
    pub id: Uuid,
    /// Resource limits
    #[allow(dead_code)]
    pub limits: ResourceLimits,
    /// Security metadata
    #[allow(dead_code)]
    pub security: SecurityMetadata,
    /// Isolated working directory
    #[allow(dead_code)]
    pub work_dir: PathBuf,
    /// Resource usage tracking
    #[allow(dead_code)]
    pub usage: HashMap<String, usize>,
}

impl PluginSandbox {
    /// Create a new plugin sandbox
    #[must_use]
    pub fn new(id: Uuid, limits: ResourceLimits, security: SecurityMetadata, work_dir: PathBuf) -> Self {
        Self {
            id,
            limits,
            security,
            work_dir,
            usage: HashMap::new(),
        }
    }
    
    /// Check if an operation is allowed
    #[allow(dead_code)]
    #[must_use]
    pub fn is_operation_allowed(&self, operation: &str) -> bool {
        match operation {
            "file:read" | "file:write" => self.security.permissions.contains(&"file:access".to_string()),
            "network:connect" => self.security.permissions.contains(&"network:access".to_string()),
            "system:exec" => self.security.permissions.contains(&"system:exec".to_string()),
            _ => false,
        }
    }
    
    /// Check if a resource limit is exceeded
    #[allow(dead_code)]
    #[must_use]
    pub fn check_resource_limits(&self, resource: &str, amount: usize) -> bool {
        match resource {
            "memory" => amount <= self.limits.memory_mb,
            "cpu" => amount <= self.limits.cpu_percent,
            "disk" => amount <= self.limits.disk_mb,
            "network" => amount <= self.limits.network_mb,
            "threads" => amount <= self.limits.threads,
            "files" => amount <= self.limits.files,
            _ => false,
        }
    }
    
    /// Update resource usage
    #[allow(dead_code)]
    pub fn update_usage(&mut self, resource: &str, amount: usize) {
        self.usage.insert(resource.to_string(), amount);
    }
    
    /// Get current resource usage
    #[allow(dead_code)]
    #[must_use]
    pub fn get_usage(&self, resource: &str) -> usize {
        *self.usage.get(resource).unwrap_or(&0)
    }
}

/// Memory-based plugin loader for testing
#[derive(Debug)]
pub struct MemoryPluginLoader {
    /// In-memory collection of plugins mapped by name
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl MemoryPluginLoader {
    /// Create a new memory-based plugin loader
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    /// Register a plugin
    #[allow(dead_code)]
    pub fn register_plugin(&mut self, name: &str, plugin: Box<dyn Plugin>) {
        self.plugins.insert(name.to_string(), plugin);
    }
}

#[async_trait]
impl PluginLoaderTrait for MemoryPluginLoader {
    async fn load_plugin(&self, manifest: &PluginManifest, _path: &Path) -> Result<Box<dyn Plugin>> {
        // Check if plugin exists
        let plugin_name = manifest.name.clone();
        if !self.plugins.contains_key(&plugin_name) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Plugin not found: {plugin_name}")
            ).into());
        }

        // Create a mock plugin for testing
        let metadata = super::PluginMetadata {
            id: Uuid::new_v4(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            author: manifest.author.clone(),
            dependencies: manifest.dependencies.clone(),
            capabilities: manifest.capabilities.clone(),
        };

        // Create a new placeholder plugin
        let plugin = Box::new(PlaceholderPlugin { metadata });
        Ok(plugin)
    }
    
    async fn create_sandbox(&self, manifest: &PluginManifest) -> Result<PluginSandbox> {
        let id = Uuid::new_v4();
        let work_dir = std::env::temp_dir().join(format!("plugin-{id}"));
        
        // Create working directory if it doesn't exist
        if !work_dir.exists() {
            fs::create_dir_all(&work_dir)?;
        }
        
        Ok(PluginSandbox::new(
            id,
            manifest.resource_limits.clone(),
            manifest.security.clone(),
            work_dir,
        ))
    }
}

impl Default for MemoryPluginLoader {
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
    metadata: super::PluginMetadata,
}

impl Plugin for PlaceholderPlugin {
    fn metadata(&self) -> &super::PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    
    fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    
    fn get_state(&self) -> BoxFuture<'_, Result<Option<super::PluginState>>> {
        Box::pin(async move { Ok(None) })
    }
    
    fn set_state(&self, _state: super::PluginState) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(PlaceholderPlugin {
            metadata: self.metadata.clone()
        })
    }
}

/// Plugin manifest file format
#[derive(Debug, Deserialize)]
pub struct PluginManifest {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin entry point
    #[allow(dead_code)]
    pub entry_point: String,
    /// Plugin dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Plugin capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Plugin resource limits
    #[serde(default)]
    pub resource_limits: ResourceLimits,
    /// Plugin security metadata
    #[serde(default)]
    pub security: SecurityMetadata,
}

/// Resource limits for a plugin sandbox
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    #[serde(default = "default_memory_limit")]
    pub memory_mb: usize,
    
    /// Maximum CPU usage in percent
    #[serde(default = "default_cpu_limit")]
    pub cpu_percent: usize,
    
    /// Maximum disk space usage in MB
    #[serde(default = "default_disk_limit")]
    pub disk_mb: usize,
    
    /// Maximum network bandwidth in MB
    #[serde(default = "default_network_limit")]
    pub network_mb: usize,
    
    /// Maximum number of threads
    #[serde(default = "default_thread_limit")]
    pub threads: usize,
    
    /// Maximum number of open files
    #[serde(default = "default_file_limit")]
    pub files: usize,
}

/// Returns the default memory limit in MB.
/// 
/// Default is 100 MB.
fn default_memory_limit() -> usize {
    100 // 100 MB
}

/// Returns the default CPU usage limit in percent.
/// 
/// Default is 10%.
fn default_cpu_limit() -> usize {
    10 // 10%
}

/// Returns the default disk space limit in MB.
/// 
/// Default is 100 MB.
fn default_disk_limit() -> usize {
    100 // 100 MB
}

/// Returns the default network bandwidth limit in MB.
/// 
/// Default is 10 MB.
fn default_network_limit() -> usize {
    10 // 10 MB
}

/// Returns the default thread count limit.
/// 
/// Default is 5 threads.
fn default_thread_limit() -> usize {
    5 // 5 threads
}

/// Returns the default file count limit.
/// 
/// Default is 10 files.
fn default_file_limit() -> usize {
    10 // 10 files
}

/// Security metadata for a plugin
#[derive(Debug, Deserialize, Default, Clone)]
pub struct SecurityMetadata {
    /// Plugin permissions
    #[serde(default)]
    #[allow(dead_code)]
    pub permissions: Vec<String>,
    /// Plugin signature
    #[allow(dead_code)]
    pub signature: Option<String>,
    /// Plugin publisher
    #[allow(dead_code)]
    pub publisher: Option<String>,
    /// Plugin verification status
    #[serde(default)]
    #[allow(dead_code)]
    pub verified: bool,
    /// Plugin sandboxed
    #[serde(default = "default_sandboxed")]
    #[allow(dead_code)]
    pub sandboxed: bool,
}

/// Default for sandboxed
fn default_sandboxed() -> bool {
    true // Plugins are sandboxed by default
}

/// Security level for plugin verification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    /// No verification, high risk
    None,
    /// Basic verification, medium risk
    Basic,
    /// Full verification, low risk
    Full,
}

/// Enhanced plugin discovery with caching and monitoring
#[derive(Debug)]
pub struct EnhancedPluginDiscovery {
    /// Base directory for plugin discovery
    base_dir: PathBuf,
    /// Plugin metadata cache
    cache: RwLock<HashMap<String, PluginMetadata>>,
    /// Plugin path to ID mapping
    path_cache: RwLock<HashMap<PathBuf, Uuid>>,
    /// Last scan time
    last_scan: StdRwLock<Option<std::time::Instant>>,
    /// Scan interval in seconds
    scan_interval: u64,
}

impl EnhancedPluginDiscovery {
    /// Create a new enhanced plugin discovery
    /// 
    /// # Errors
    /// 
    /// Returns an error if the directory doesn't exist
    pub fn new(directory: &Path) -> Result<Self> {
        if !directory.exists() {
            fs::create_dir_all(directory)?;
        }
        
        Ok(Self {
            base_dir: directory.to_path_buf(),
            cache: RwLock::new(HashMap::new()),
            path_cache: RwLock::new(HashMap::new()),
            last_scan: StdRwLock::new(None),
            scan_interval: 60, // Default to 60 seconds
        })
    }
    
    /// Set the scan interval
    pub fn with_scan_interval(mut self, seconds: u64) -> Self {
        self.scan_interval = seconds;
        self
    }
    
    /// Check if a scan is needed
    async fn needs_scan(&self) -> bool {
        let last_scan = self.last_scan.read().unwrap();
        
        if let Some(time) = *last_scan {
            let elapsed = time.elapsed();
            elapsed.as_secs() > self.scan_interval
        } else {
            true
        }
    }
    
    /// Scan for plugins and update the cache
    pub async fn scan(&self) -> Result<Vec<PluginMetadata>> {
        // Update the last scan time
        {
            let mut last_scan = self.last_scan.write().unwrap();
            *last_scan = Some(std::time::Instant::now());
        }
        
        // Scan the base directory
        let mut plugins = Vec::new();
        
        // Create a new HashMap for the updated cache
        let mut new_cache = HashMap::new();
        let mut new_path_cache = HashMap::new();
        
        // Walk the directory tree
        let entries = walkdir::WalkDir::new(&self.base_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                let path = e.path();
                path.is_file() && 
                path.extension().is_some_and(|ext| ext == "json" || ext == "toml")
            });
        
        for entry in entries {
            let path = entry.path();
            
            // Try to load the plugin metadata
            match self.load_plugin_metadata(path).await {
                Ok(metadata) => {
                    // Add to the new cache
                    new_cache.insert(metadata.name.clone(), metadata.clone());
                    new_path_cache.insert(path.to_path_buf(), metadata.id);
                    plugins.push(metadata);
                }
                Err(e) => {
                    log::warn!("Failed to load plugin metadata from {}: {}", path.display(), e);
                    continue;
                }
            }
        }
        
        // Update the cache
        {
            let mut cache = self.cache.write().await;
            *cache = new_cache;
            
            let mut path_cache = self.path_cache.write().await;
            *path_cache = new_path_cache;
        }
        
        Ok(plugins)
    }
    
    /// Load plugin metadata from a file
    async fn load_plugin_metadata(&self, path: &Path) -> Result<PluginMetadata> {
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        
        match extension {
            "json" => {
                let content = std::fs::read_to_string(path)?;
                let metadata: PluginMetadata = serde_json::from_str(&content)?;
                Ok(metadata)
            }
            "toml" => {
                let content = std::fs::read_to_string(path)?;
                let metadata: PluginMetadata = toml::from_str(&content)?;
                Ok(metadata)
            }
            _ => {
                Err(anyhow::anyhow!("Unsupported file extension: {}", extension).into())
            }
        }
    }
    
    /// Clear the cache
    pub async fn clear_cache(&self) -> Result<()> {
        {
            let mut cache = self.cache.write().await;
            cache.clear();
        }
        {
            let mut path_cache = self.path_cache.write().await;
            path_cache.clear();
        }
        
        Ok(())
    }
    
    /// Get plugin metadata from path
    pub async fn get_metadata_from_path(&self, path: &Path) -> Option<PluginMetadata> {
        let path_cache = self.path_cache.read().await;
        
        if let Some(&id) = path_cache.get(path) {
            let cache = self.cache.read().await;
            return cache.values().find(|m| m.id == id).cloned();
        }
        
        None
    }
    
    /// Get plugin metadata by id
    pub async fn get_metadata_by_id(&self, id: Uuid) -> Option<PluginMetadata> {
        let cache = self.cache.read().await;
        cache.values().find(|m| m.id == id).cloned()
    }
    
    /// Get plugin metadata by name
    pub async fn get_metadata_by_name(&self, name: &str) -> Option<PluginMetadata> {
        let cache = self.cache.read().await;
        cache.values().find(|m| m.name == name).cloned()
    }
    
    /// Get all plugin metadata
    pub async fn get_all_metadata(&self) -> Vec<PluginMetadata> {
        let cache = self.cache.read().await;
        cache.values().cloned().collect()
    }
    
    /// Get the base directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

/// Enhanced plugin loader that supports more plugin types
pub struct EnhancedPluginLoader {
    /// Plugin discovery
    discovery: EnhancedPluginDiscovery,
    /// Loaded plugins
    loaded_plugins: RwLock<HashMap<Uuid, Arc<dyn Plugin>>>,
}

impl EnhancedPluginLoader {
    /// Create a new enhanced plugin loader
    pub fn new(discovery: EnhancedPluginDiscovery) -> Self {
        Self {
            discovery,
            loaded_plugins: RwLock::new(HashMap::new()),
        }
    }
    
    /// Load a plugin
    pub async fn load_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        // Check if already loaded
        {
            let loaded = self.loaded_plugins.read().await;
            if let Some(plugin) = loaded.get(&id) {
                return Ok(plugin.clone());
            }
        }
        
        // Get the plugin metadata
        let metadata = match self.discovery.get_metadata_by_id(id).await {
            Some(metadata) => metadata,
            None => return Err(anyhow::anyhow!("Plugin not found: {}", id).into()),
        };
        
        // Get the plugin path
        let plugin_path = Path::new(&self.discovery.base_dir())
            .join(metadata.id.to_string());
        
        // Load the plugin
        let plugin = self.load_plugin_from_path(&plugin_path).await?;
        
        // Cache it
        {
            let mut loaded = self.loaded_plugins.write().await;
            loaded.insert(id, plugin.clone());
        }
        
        Ok(plugin)
    }
    
    /// Load a plugin by name
    pub async fn load_plugin_by_name(&self, name: &str) -> Result<Arc<dyn Plugin>> {
        // Find the plugin
        let metadata = match self.discovery.get_metadata_by_name(name).await {
            Some(metadata) => metadata,
            None => return Err(anyhow::anyhow!("Plugin not found: {}", name).into()),
        };
        
        // Load it
        self.load_plugin(metadata.id).await
    }
    
    /// Load plugins by capability
    pub async fn load_plugins_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Plugin>>> {
        let metadata_list = self.discovery.get_all_metadata().await;
        
        let mut plugins = Vec::new();
        for metadata in metadata_list {
            match self.load_plugin(metadata.id).await {
                Ok(plugin) => {
                    plugins.push(plugin);
                }
                Err(e) => {
                    log::warn!("Failed to load plugin {}: {}", metadata.name, e);
                    continue;
                }
            }
        }
        
        Ok(plugins)
    }
    
    /// Load a plugin from a path
    async fn load_plugin_from_path(&self, path: &Path) -> Result<Arc<dyn Plugin>> {
        // Load the metadata
        let metadata = self.discovery.get_metadata_from_path(path).await.unwrap();
        
        // Create the appropriate plugin type based on capabilities
        if metadata.capabilities.contains(&"command".to_string()) {
            // Create a command plugin
            let plugin = crate::plugin::types::CommandPluginBuilder::new(metadata)
                .build();
            
            // Unbox the plugin and create a new Arc directly with the concrete implementation
            if let Some(impl_plugin) = plugin.as_any().downcast_ref::<crate::plugin::types::CommandPluginImpl>() {
                Ok(Arc::new(impl_plugin.clone()))
            } else {
                Err(anyhow::anyhow!("Failed to downcast command plugin").into())
            }
        } else if metadata.capabilities.contains(&"tool".to_string()) {
            // Create a tool plugin
            let plugin = crate::plugin::types::ToolPluginBuilder::new(metadata)
                .build();
            
            // Unbox the plugin and create a new Arc directly with the concrete implementation
            if let Some(impl_plugin) = plugin.as_any().downcast_ref::<crate::plugin::types::ToolPluginImpl>() {
                return Ok(Arc::new(impl_plugin.clone()));
            } else {
                return Err(anyhow::anyhow!("Failed to downcast tool plugin").into());
            }
        } else if metadata.capabilities.contains(&"mcp".to_string()) {
            // Create an MCP plugin
            let plugin = crate::plugin::types::McpPluginBuilder::new(metadata)
                .build();
            
            // Since this already returns an Arc<McpPluginImpl> which implements Plugin,
            // we can use as to cast it to Arc<dyn Plugin>
            return Ok(plugin as Arc<dyn Plugin>);
        } else {
            // Create a generic plugin
            let plugin = GenericPlugin::new(metadata);
            Ok(Arc::new(plugin) as Arc<dyn Plugin>)
        }
    }
    
    /// Get the discovery system
    pub fn discovery(&self) -> &EnhancedPluginDiscovery {
        &self.discovery
    }
}

/// A generic plugin implementation that doesn't have special functionality
#[derive(Debug)]
pub struct GenericPlugin {
    /// Plugin metadata
    metadata: PluginMetadata,
    /// Plugin state
    state: StdRwLock<Option<PluginState>>,
}

impl GenericPlugin {
    /// Create a new generic plugin
    #[must_use] pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            state: StdRwLock::new(None),
        }
    }
}

impl Plugin for GenericPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>> {
        Box::pin(async move { 
            let guard = self.state.read().unwrap();
            Ok(guard.clone())
        })
    }

    fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let mut guard = self.state.write().unwrap();
            *guard = Some(state);
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(Self {
            metadata: self.metadata.clone(),
            state: StdRwLock::new(None),
        })
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
        let mut discovery = FileSystemDiscovery::new(SecurityLevel::Basic);
        discovery.add_validation_rule(|_| Ok(()));
        
        let manager = PluginManager::new();
        let mut loader = PluginLoader::new(manager, Box::new(discovery));
        loader.add_directory(temp_dir.path());
        
        // Load plugins
        loader.load_all().await.unwrap();
    }
} 
