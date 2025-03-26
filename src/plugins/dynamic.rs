// Dynamic Library Plugin Loader
//
// This module provides functionality for loading plugins from external
// shared libraries (.dll, .so, .dylib).

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use libloading::{Library, Symbol};
use semver::{Version, VersionReq};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use lru::LruCache;
use serde::{Serialize, Deserialize};

use crate::plugins::errors::{PluginError, Result};
use crate::plugins::interfaces::Plugin;
use crate::plugins::management::PluginRegistry;

/// Dynamic library loader interface
#[async_trait]
pub trait DynamicLibraryLoader: Send + Sync + Debug {
    /// Load a plugin from a shared library
    async fn load_plugin(&self, path: &Path) -> Result<Box<dyn Plugin>>;
    
    /// Check if a library contains a valid plugin
    async fn validate_library(&self, path: &Path) -> Result<PluginMetadata>;
    
    /// Unload a plugin library
    async fn unload_plugin(&self, plugin_id: Uuid) -> Result<()>;
}

/// Plugin metadata with version information
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: Uuid,
    
    /// Plugin name
    pub name: String,
    
    /// Plugin version
    pub version: String,
    
    /// Plugin API version
    pub api_version: String,
    
    /// Plugin description
    pub description: String,
    
    /// Plugin author
    pub author: String,
    
    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,
}

/// Plugin dependency information
#[derive(Debug, Clone)]
pub struct PluginDependency {
    /// Dependency plugin ID
    pub plugin_id: Uuid,
    
    /// Required version constraint
    pub version_requirement: String,
}

/// Cache entry for plugin library metadata
#[derive(Debug)]
struct CachedPluginMetadata {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// When this entry was cached
    cached_at: Instant,
    
    /// TTL for this cache entry
    ttl: Duration,
}

impl CachedPluginMetadata {
    /// Create a new cache entry
    fn new(metadata: PluginMetadata, ttl: Duration) -> Self {
        Self {
            metadata,
            cached_at: Instant::now(),
            ttl,
        }
    }
    
    /// Check if this cache entry is still valid
    fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
}

/// Loader with caching capabilities
#[derive(Debug)]
pub struct CachedDynamicLibraryLoader<T: DynamicLibraryLoader> {
    /// Inner loader
    inner: T,
    
    /// Metadata cache
    metadata_cache: RwLock<LruCache<PathBuf, CachedPluginMetadata>>,
    
    /// Cache TTL
    cache_ttl: Duration,
    
    /// Cache size
    cache_size: usize,
}

impl<T: DynamicLibraryLoader> CachedDynamicLibraryLoader<T> {
    /// Create a new cached loader
    pub fn new(inner: T, cache_size: usize, cache_ttl_secs: u64) -> Self {
        Self {
            inner,
            metadata_cache: RwLock::new(LruCache::new(cache_size.try_into().unwrap_or(100))),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            cache_size,
        }
    }
    
    /// Set cache TTL
    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.cache_ttl = Duration::from_secs(ttl_secs);
        self
    }
    
    /// Set cache size
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self.metadata_cache = RwLock::new(LruCache::new(size.try_into().unwrap_or(100)));
        self
    }
    
    /// Clear the cache
    pub async fn clear_cache(&self) {
        let mut cache = self.metadata_cache.write().await;
        cache.clear();
    }
    
    /// Get cache stats
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.metadata_cache.read().await;
        
        CacheStats {
            size: cache.len(),
            capacity: cache.cap(),
            ttl_seconds: self.cache_ttl.as_secs(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CacheStats {
    /// Current number of entries in the cache
    pub size: usize,
    
    /// Maximum capacity of the cache
    pub capacity: usize,
    
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
}

/// Windows-specific library loader
#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct WindowsLibraryLoader {
    /// Loaded libraries
    libraries: RwLock<HashMap<Uuid, Library>>,
}

#[cfg(target_os = "windows")]
impl WindowsLibraryLoader {
    /// Create a new Windows library loader
    pub fn new() -> Self {
        Self {
            libraries: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get the expected library extension
    fn expected_extension(&self) -> &str {
        "dll"
    }
}

/// Linux-specific library loader
#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct LinuxLibraryLoader {
    /// Loaded libraries
    libraries: RwLock<HashMap<Uuid, Library>>,
}

#[cfg(target_os = "linux")]
impl LinuxLibraryLoader {
    /// Create a new Linux library loader
    pub fn new() -> Self {
        Self {
            libraries: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get the expected library extension
    fn expected_extension(&self) -> &str {
        "so"
    }
}

/// macOS-specific library loader
#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct MacOSLibraryLoader {
    /// Loaded libraries
    libraries: RwLock<HashMap<Uuid, Library>>,
}

#[cfg(target_os = "macos")]
impl MacOSLibraryLoader {
    /// Create a new macOS library loader
    pub fn new() -> Self {
        Self {
            libraries: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get the expected library extension
    fn expected_extension(&self) -> &str {
        "dylib"
    }
}

/// Implementation for Windows library loader
#[cfg(target_os = "windows")]
#[async_trait]
impl DynamicLibraryLoader for WindowsLibraryLoader {
    async fn load_plugin(&self, path: &Path) -> Result<Box<dyn Plugin>> {
        // Validate the library first
        let metadata = self.validate_library(path).await?;
        
        // Load the library
        let path_clone = path.to_path_buf();
        let lib = tokio::task::spawn_blocking(move || -> Result<Library> {
            unsafe { Ok(Library::new(path_clone)?) }
        }).await??;
        
        // Get plugin creation function
        let lib_clone = lib.clone();
        let plugin_ptr = tokio::task::spawn_blocking(move || -> Result<*mut dyn Plugin> {
            let create_fn: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = 
                unsafe { lib_clone.get(b"create_plugin")? };
            
            // Call the function
            let plugin_ptr = unsafe { create_fn() };
            if plugin_ptr.is_null() {
                return Err(PluginError::LoadingFailed("Plugin creation returned null".into()));
            }
            
            Ok(plugin_ptr)
        }).await??;
        
        // Convert to Box
        let plugin = unsafe { Box::from_raw(plugin_ptr) };
        
        // Verify the metadata
        if plugin.metadata().id != metadata.id {
            return Err(PluginError::MetadataMismatch(
                "Plugin ID in metadata doesn't match loaded plugin".into()
            ));
        }
        
        // Store the library for later unloading
        self.libraries.write().await.insert(plugin.metadata().id, lib);
        
        debug!("Loaded plugin {} from {}", plugin.metadata().id, path.display());
        Ok(plugin)
    }
    
    async fn validate_library(&self, path: &Path) -> Result<PluginMetadata> {
        // Check file existence
        if !path.exists() {
            return Err(PluginError::NotFound(format!("Library not found: {}", path.display())));
        }
        
        // Check file extension
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if extension != self.expected_extension() {
            return Err(PluginError::InvalidFormat(
                format!("Invalid library extension: {}", extension)
            ));
        }
        
        // Try loading the library
        let path_clone = path.to_path_buf();
        let metadata = tokio::task::spawn_blocking(move || -> Result<PluginMetadata> {
            let lib = unsafe { Library::new(&path_clone)? };
            
            // Check for required symbols
            let has_create = unsafe { lib.get::<unsafe extern "C" fn() -> *mut dyn Plugin>(b"create_plugin").is_ok() };
            let has_metadata = unsafe { lib.get::<unsafe extern "C" fn() -> *mut PluginMetadata>(b"get_plugin_metadata").is_ok() };
            let has_destroy = unsafe { lib.get::<unsafe extern "C" fn(*mut dyn Plugin)>(b"destroy_plugin").is_ok() };
            
            if !has_create || !has_metadata || !has_destroy {
                return Err(PluginError::InvalidFormat("Missing required plugin symbols".into()));
            }
            
            // Get metadata
            let get_metadata_fn: Symbol<unsafe extern "C" fn() -> *mut PluginMetadata> = 
                unsafe { lib.get(b"get_plugin_metadata")? };
            
            let metadata_ptr = unsafe { get_metadata_fn() };
            if metadata_ptr.is_null() {
                return Err(PluginError::InvalidFormat("Plugin metadata function returned null".into()));
            }
            
            // Convert to PluginMetadata and clone it
            let metadata = unsafe { Box::from_raw(metadata_ptr) };
            let result = metadata.clone();
            
            // Don't free the original pointer to avoid double free
            std::mem::forget(metadata);
            
            Ok(result)
        }).await??;
        
        debug!("Validated plugin library at {}", path.display());
        Ok(metadata)
    }
    
    async fn unload_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let mut libraries = self.libraries.write().await;
        
        if let Some(_lib) = libraries.remove(&plugin_id) {
            // Library will be automatically dropped, which will unload it
            debug!("Unloaded library for plugin {}", plugin_id);
            Ok(())
        } else {
            Err(PluginError::NotFound(format!("Library for plugin {} not found", plugin_id)))
        }
    }
}

// Implement the same trait for Linux and macOS with platform-specific adjustments
#[cfg(target_os = "linux")]
#[async_trait]
impl DynamicLibraryLoader for LinuxLibraryLoader {
    async fn load_plugin(&self, path: &Path) -> Result<Box<dyn Plugin>> {
        // Implementation similar to Windows but with Linux-specific adjustments
        // Validate the library first
        let metadata = self.validate_library(path).await?;
        
        // Load the library
        let path_clone = path.to_path_buf();
        let lib = tokio::task::spawn_blocking(move || -> Result<Library> {
            unsafe { Ok(Library::new(path_clone)?) }
        }).await??;
        
        // Get plugin creation function
        let lib_clone = lib.clone();
        let plugin_ptr = tokio::task::spawn_blocking(move || -> Result<*mut dyn Plugin> {
            let create_fn: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = 
                unsafe { lib_clone.get(b"create_plugin")? };
            
            // Call the function
            let plugin_ptr = unsafe { create_fn() };
            if plugin_ptr.is_null() {
                return Err(PluginError::LoadingFailed("Plugin creation returned null".into()));
            }
            
            Ok(plugin_ptr)
        }).await??;
        
        // Convert to Box
        let plugin = unsafe { Box::from_raw(plugin_ptr) };
        
        // Verify the metadata
        if plugin.metadata().id != metadata.id {
            return Err(PluginError::MetadataMismatch(
                "Plugin ID in metadata doesn't match loaded plugin".into()
            ));
        }
        
        // Store the library for later unloading
        self.libraries.write().await.insert(plugin.metadata().id, lib);
        
        debug!("Loaded plugin {} from {}", plugin.metadata().id, path.display());
        Ok(plugin)
    }
    
    async fn validate_library(&self, path: &Path) -> Result<PluginMetadata> {
        // Implementation similar to Windows but with Linux-specific adjustments
        // Check file existence
        if !path.exists() {
            return Err(PluginError::NotFound(format!("Library not found: {}", path.display())));
        }
        
        // Check file extension
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if extension != self.expected_extension() {
            return Err(PluginError::InvalidFormat(
                format!("Invalid library extension: {}", extension)
            ));
        }
        
        // Try loading the library
        let path_clone = path.to_path_buf();
        let metadata = tokio::task::spawn_blocking(move || -> Result<PluginMetadata> {
            let lib = unsafe { Library::new(&path_clone)? };
            
            // Check for required symbols
            let has_create = unsafe { lib.get::<unsafe extern "C" fn() -> *mut dyn Plugin>(b"create_plugin").is_ok() };
            let has_metadata = unsafe { lib.get::<unsafe extern "C" fn() -> *mut PluginMetadata>(b"get_plugin_metadata").is_ok() };
            let has_destroy = unsafe { lib.get::<unsafe extern "C" fn(*mut dyn Plugin)>(b"destroy_plugin").is_ok() };
            
            if !has_create || !has_metadata || !has_destroy {
                return Err(PluginError::InvalidFormat("Missing required plugin symbols".into()));
            }
            
            // Get metadata
            let get_metadata_fn: Symbol<unsafe extern "C" fn() -> *mut PluginMetadata> = 
                unsafe { lib.get(b"get_plugin_metadata")? };
            
            let metadata_ptr = unsafe { get_metadata_fn() };
            if metadata_ptr.is_null() {
                return Err(PluginError::InvalidFormat("Plugin metadata function returned null".into()));
            }
            
            // Convert to PluginMetadata and clone it
            let metadata = unsafe { Box::from_raw(metadata_ptr) };
            let result = metadata.clone();
            
            // Don't free the original pointer to avoid double free
            std::mem::forget(metadata);
            
            Ok(result)
        }).await??;
        
        debug!("Validated plugin library at {}", path.display());
        Ok(metadata)
    }
    
    async fn unload_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let mut libraries = self.libraries.write().await;
        
        if let Some(_lib) = libraries.remove(&plugin_id) {
            // Library will be automatically dropped, which will unload it
            debug!("Unloaded library for plugin {}", plugin_id);
            Ok(())
        } else {
            Err(PluginError::NotFound(format!("Library for plugin {} not found", plugin_id)))
        }
    }
}

#[cfg(target_os = "macos")]
#[async_trait]
impl DynamicLibraryLoader for MacOSLibraryLoader {
    async fn load_plugin(&self, path: &Path) -> Result<Box<dyn Plugin>> {
        // Implementation similar to Windows but with macOS-specific adjustments
        // Validate the library first
        let metadata = self.validate_library(path).await?;
        
        // Load the library
        let path_clone = path.to_path_buf();
        let lib = tokio::task::spawn_blocking(move || -> Result<Library> {
            unsafe { Ok(Library::new(path_clone)?) }
        }).await??;
        
        // Get plugin creation function
        let lib_clone = lib.clone();
        let plugin_ptr = tokio::task::spawn_blocking(move || -> Result<*mut dyn Plugin> {
            let create_fn: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = 
                unsafe { lib_clone.get(b"create_plugin")? };
            
            // Call the function
            let plugin_ptr = unsafe { create_fn() };
            if plugin_ptr.is_null() {
                return Err(PluginError::LoadingFailed("Plugin creation returned null".into()));
            }
            
            Ok(plugin_ptr)
        }).await??;
        
        // Convert to Box
        let plugin = unsafe { Box::from_raw(plugin_ptr) };
        
        // Verify the metadata
        if plugin.metadata().id != metadata.id {
            return Err(PluginError::MetadataMismatch(
                "Plugin ID in metadata doesn't match loaded plugin".into()
            ));
        }
        
        // Store the library for later unloading
        self.libraries.write().await.insert(plugin.metadata().id, lib);
        
        debug!("Loaded plugin {} from {}", plugin.metadata().id, path.display());
        Ok(plugin)
    }
    
    async fn validate_library(&self, path: &Path) -> Result<PluginMetadata> {
        // Implementation similar to Windows but with macOS-specific adjustments
        // Check file existence
        if !path.exists() {
            return Err(PluginError::NotFound(format!("Library not found: {}", path.display())));
        }
        
        // Check file extension
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if extension != self.expected_extension() {
            return Err(PluginError::InvalidFormat(
                format!("Invalid library extension: {}", extension)
            ));
        }
        
        // Try loading the library
        let path_clone = path.to_path_buf();
        let metadata = tokio::task::spawn_blocking(move || -> Result<PluginMetadata> {
            let lib = unsafe { Library::new(&path_clone)? };
            
            // Check for required symbols
            let has_create = unsafe { lib.get::<unsafe extern "C" fn() -> *mut dyn Plugin>(b"create_plugin").is_ok() };
            let has_metadata = unsafe { lib.get::<unsafe extern "C" fn() -> *mut PluginMetadata>(b"get_plugin_metadata").is_ok() };
            let has_destroy = unsafe { lib.get::<unsafe extern "C" fn(*mut dyn Plugin)>(b"destroy_plugin").is_ok() };
            
            if !has_create || !has_metadata || !has_destroy {
                return Err(PluginError::InvalidFormat("Missing required plugin symbols".into()));
            }
            
            // Get metadata
            let get_metadata_fn: Symbol<unsafe extern "C" fn() -> *mut PluginMetadata> = 
                unsafe { lib.get(b"get_plugin_metadata")? };
            
            let metadata_ptr = unsafe { get_metadata_fn() };
            if metadata_ptr.is_null() {
                return Err(PluginError::InvalidFormat("Plugin metadata function returned null".into()));
            }
            
            // Convert to PluginMetadata and clone it
            let metadata = unsafe { Box::from_raw(metadata_ptr) };
            let result = metadata.clone();
            
            // Don't free the original pointer to avoid double free
            std::mem::forget(metadata);
            
            Ok(result)
        }).await??;
        
        debug!("Validated plugin library at {}", path.display());
        Ok(metadata)
    }
    
    async fn unload_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let mut libraries = self.libraries.write().await;
        
        if let Some(_lib) = libraries.remove(&plugin_id) {
            // Library will be automatically dropped, which will unload it
            debug!("Unloaded library for plugin {}", plugin_id);
            Ok(())
        } else {
            Err(PluginError::NotFound(format!("Library for plugin {} not found", plugin_id)))
        }
    }
}

/// Create a dynamic library loader appropriate for the current platform
pub fn create_library_loader() -> Arc<dyn DynamicLibraryLoader> {
    #[cfg(target_os = "windows")]
    {
        Arc::new(WindowsLibraryLoader::new())
    }
    
    #[cfg(target_os = "linux")]
    {
        Arc::new(LinuxLibraryLoader::new())
    }
    
    #[cfg(target_os = "macos")]
    {
        Arc::new(MacOSLibraryLoader::new())
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        compile_error!("Unsupported target OS");
    }
}

/// Version compatibility checker
#[derive(Debug)]
pub struct VersionCompatibilityChecker {
    app_version: Version,
}

impl VersionCompatibilityChecker {
    /// Create a new version compatibility checker
    pub fn new(app_version: impl Into<String>) -> Result<Self> {
        let app_version = Version::parse(&app_version.into())
            .map_err(|e| PluginError::VersionError(e.to_string()))?;
        
        Ok(Self { app_version })
    }
    
    /// Check if a version is compatible with requirements
    pub fn check_compatibility(&self, version: &str, requirement: &str) -> Result<bool> {
        let version = Version::parse(version)
            .map_err(|e| PluginError::VersionError(format!("Invalid version '{}': {}", version, e)))?;
        
        let req = VersionReq::parse(requirement)
            .map_err(|e| PluginError::VersionError(format!("Invalid requirement '{}': {}", requirement, e)))?;
        
        Ok(req.matches(&version))
    }
    
    /// Check if the app version is compatible with requirements
    pub fn check_app_compatibility(&self, requirement: &str) -> Result<bool> {
        let req = VersionReq::parse(requirement)
            .map_err(|e| PluginError::VersionError(format!("Invalid requirement '{}': {}", requirement, e)))?;
        
        Ok(req.matches(&self.app_version))
    }
}

// Extend the Plugin Registry with dynamic loading capabilities
impl PluginRegistry {
    /// Register a dynamic plugin from a shared library
    pub async fn register_dynamic_plugin(&self, path: &Path) -> Result<Uuid> {
        let loader = create_library_loader();
        let plugin = loader.load_plugin(path).await?;
        
        // Register plugin
        let plugin_id = plugin.metadata().id;
        self.register_plugin(plugin).await?;
        
        Ok(plugin_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use crate::plugins::management::PluginRegistryImpl;
    
    #[tokio::test]
    async fn test_version_compatibility() {
        let checker = VersionCompatibilityChecker::new("1.2.3").unwrap();
        
        // Test compatibility checks
        assert!(checker.check_compatibility("1.2.3", "=1.2.3").unwrap());
        assert!(checker.check_compatibility("1.2.3", ">=1.0.0").unwrap());
        assert!(!checker.check_compatibility("1.2.3", ">=2.0.0").unwrap());
        
        // Test app version compatibility
        assert!(checker.check_app_compatibility("=1.2.3").unwrap());
        assert!(checker.check_app_compatibility(">=1.0.0").unwrap());
        assert!(!checker.check_app_compatibility(">=2.0.0").unwrap());
    }
    
    // More tests would be added for platform-specific library loading
    // These would need sample plugin libraries to be available for testing
}

#[async_trait]
impl<T: DynamicLibraryLoader + Send + Sync> DynamicLibraryLoader for CachedDynamicLibraryLoader<T> {
    async fn load_plugin(&self, path: &Path) -> Result<Box<dyn Plugin>> {
        // Loading always goes through the inner loader
        // We don't cache loaded plugins for safety reasons
        self.inner.load_plugin(path).await
    }
    
    async fn validate_library(&self, path: &Path) -> Result<PluginMetadata> {
        // Check cache first
        {
            let mut cache = self.metadata_cache.write().await;
            
            if let Some(entry) = cache.get(path) {
                if entry.is_valid() {
                    debug!("Plugin metadata cache hit for {}", path.display());
                    return Ok(entry.metadata.clone());
                } else {
                    // Entry expired, remove it
                    cache.pop(path);
                }
            }
        }
        
        // Cache miss, load metadata from library
        let metadata = self.inner.validate_library(path).await?;
        
        // Cache the result
        {
            let mut cache = self.metadata_cache.write().await;
            let entry = CachedPluginMetadata::new(metadata.clone(), self.cache_ttl);
            cache.put(path.to_path_buf(), entry);
        }
        
        Ok(metadata)
    }
    
    async fn unload_plugin(&self, plugin_id: Uuid) -> Result<()> {
        // Delegate to inner loader
        self.inner.unload_plugin(plugin_id).await
    }
}

/// Create a cached loader for the current platform
pub fn create_cached_library_loader(cache_size: usize, cache_ttl_secs: u64) -> Arc<dyn DynamicLibraryLoader> {
    let inner = create_library_loader();
    
    Arc::new(CachedDynamicLibraryLoader::new(
        inner.as_ref().clone(),
        cache_size,
        cache_ttl_secs,
    ))
} 