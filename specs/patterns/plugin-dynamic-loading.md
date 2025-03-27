# Dynamic Plugin Loading Pattern

## Overview

This pattern describes the recommended approach for implementing dynamic library loading in the Squirrel Plugin System. It enables loading plugins from external shared libraries (.dll, .so, .dylib) at runtime, with proper platform abstraction, error handling, and version compatibility checking.

## Use Cases

- Loading third-party plugins from shared libraries
- Enabling plugin distribution separately from the main application
- Supporting plugin marketplaces
- Extending application functionality without recompilation

## Components

### 1. Dynamic Library Loader

An abstraction layer over platform-specific shared library loading:

```rust
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
```

### 2. Platform-Specific Implementations

Platform-specific implementations for Windows, Linux, and macOS:

```rust
#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct WindowsLibraryLoader {
    // Implementation details
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct LinuxLibraryLoader {
    // Implementation details
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct MacOSLibraryLoader {
    // Implementation details
}
```

### 3. Plugin Entry Point

A standardized entry point for dynamic plugins:

```rust
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    let plugin = MyPlugin::new();
    Box::into_raw(Box::new(plugin))
}

#[no_mangle]
pub extern "C" fn get_plugin_metadata() -> *mut PluginMetadata {
    let metadata = PluginMetadata {
        id: Uuid::new_v4(),
        name: "my-plugin".to_string(),
        version: "1.0.0".to_string(),
        // Other fields...
    };
    Box::into_raw(Box::new(metadata))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut dyn Plugin) {
    if !plugin.is_null() {
        unsafe {
            let _ = Box::from_raw(plugin);
        }
    }
}
```

### 4. Version Compatibility Checker

A component for checking plugin version compatibility:

```rust
#[derive(Debug)]
pub struct VersionCompatibilityChecker {
    app_version: semver::Version,
}

impl VersionCompatibilityChecker {
    pub fn new(app_version: impl Into<String>) -> Result<Self> {
        let app_version = semver::Version::parse(&app_version.into())?;
        Ok(Self { app_version })
    }
    
    pub fn check_compatibility(&self, plugin_version: &str, requirements: &str) -> Result<bool> {
        // Implementation details
    }
}
```

### 5. Plugin Registry Integration

Integration with the existing plugin registry:

```rust
impl PluginRegistry {
    pub async fn register_dynamic_plugin(&self, path: &Path) -> Result<Uuid> {
        let loader = self.get_loader_for_platform()?;
        let plugin = loader.load_plugin(path).await?;
        
        // Version compatibility check
        self.verify_plugin_compatibility(&plugin)?;
        
        // Register plugin
        let plugin_id = plugin.metadata().id;
        self.register_plugin(plugin).await?;
        
        Ok(plugin_id)
    }
}
```

## Implementation Approach

### 1. Platform Detection and Abstraction

```rust
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
```

### 2. Safe FFI Interface

```rust
impl WindowsLibraryLoader {
    async fn load_plugin(&self, path: &Path) -> Result<Box<dyn Plugin>> {
        // Load the library safely
        let lib = unsafe { Library::new(path)? };
        
        // Get plugin creation function
        let create_fn: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = 
            unsafe { lib.get(b"create_plugin")? };
        
        // Call the function
        let plugin_ptr = unsafe { create_fn() };
        if plugin_ptr.is_null() {
            return Err(PluginError::LoadingFailed("Plugin creation returned null".into()));
        }
        
        // Convert to Box
        let plugin = unsafe { Box::from_raw(plugin_ptr) };
        
        // Store the library for later unloading
        self.libraries.write().await.insert(plugin.metadata().id, lib);
        
        Ok(plugin)
    }
}
```

### 3. Validation and Safety Checks

```rust
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
    let lib = unsafe { Library::new(path)? };
    
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
    
    // Don't free the original pointer since we didn't allocated it
    std::mem::forget(metadata);
    
    Ok(result)
}
```

### 4. Version Compatibility Checking

```rust
fn verify_plugin_compatibility(&self, plugin: &Box<dyn Plugin>) -> Result<()> {
    let metadata = plugin.metadata();
    
    // Check API version compatibility
    let api_version = &metadata.api_version;
    if !self.version_checker.check_compatibility(api_version, &self.api_version_requirements)? {
        return Err(PluginError::IncompatibleVersion(format!(
            "Plugin API version {} is not compatible with required {}",
            api_version, self.api_version_requirements
        )));
    }
    
    // Check dependency compatibility
    for dep in &metadata.dependencies {
        if let Some(required_plugin) = self.plugins.read().await.get(&dep.plugin_id) {
            let required_version = &required_plugin.metadata().version;
            if !self.version_checker.check_compatibility(required_version, &dep.version_requirement)? {
                return Err(PluginError::DependencyVersionMismatch(format!(
                    "Plugin depends on {} version {}, but found {}",
                    dep.plugin_id, dep.version_requirement, required_version
                )));
            }
        } else {
            return Err(PluginError::MissingDependency(format!(
                "Plugin depends on {} which is not loaded",
                dep.plugin_id
            )));
        }
    }
    
    Ok(())
}
```

### 5. Resource Cleanup

```rust
async fn unload_plugin(&self, plugin_id: Uuid) -> Result<()> {
    let mut libraries = self.libraries.write().await;
    
    if let Some(lib) = libraries.remove(&plugin_id) {
        // Library will be automatically dropped, which will unload it
        debug!("Unloaded library for plugin {}", plugin_id);
        Ok(())
    } else {
        Err(PluginError::NotFound(format!("Library for plugin {} not found", plugin_id)))
    }
}
```

## Best Practices

1. **Use Safe Abstractions**: Always wrap unsafe code in safe abstractions with proper error handling.

2. **Handle Platform Differences**: Use conditional compilation to handle platform-specific differences.

3. **Check Version Compatibility**: Always verify plugin API version compatibility before loading.

4. **Resource Management**: Properly track and clean up library handles to avoid resource leaks.

5. **Validate Before Loading**: Always validate libraries before attempting to load them as plugins.

6. **Error Handling**: Provide detailed error messages for debugging issues with plugin loading.

7. **Lifecycle Integration**: Integrate with the plugin lifecycle system for proper initialization and shutdown.

8. **Thread Safety**: Use appropriate thread safety mechanisms when loading and unloading plugins.

## Anti-Patterns to Avoid

1. **Direct Symbol Access**: Avoid accessing symbols directly from plugins without validation.

2. **Skipping Validation**: Don't skip validation steps for performance reasons.

3. **Missing Error Handling**: Always handle errors properly in FFI code.

4. **Global State**: Avoid global state for plugin libraries and use proper dependency injection.

5. **Memory Leaks**: Ensure proper cleanup of resources when plugins are unloaded.

## Example Implementation

See the upcoming implementation in `src/plugins/dynamic.rs` for a complete example of this pattern.

<version>1.0.0</version> 