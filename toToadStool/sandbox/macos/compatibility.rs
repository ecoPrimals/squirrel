//! Compatibility and Trait Implementation
//!
//! This module handles macOS compatibility checks and implements the PluginSandbox trait.

use super::*;

impl MacOsSandbox {
    /// Check macOS compatibility for sandbox features
    pub fn check_macos_compatibility(&self) -> Result<HashMap<String, bool>> {
        let mut compatibility = HashMap::new();
        
        // Check macOS version
        let version = get_macos_version()?;
        compatibility.insert("macos_version".to_string(), !version.is_empty());
        
        // Check for required tools and features
        compatibility.insert("app_sandbox".to_string(), check_app_sandbox_available());
        compatibility.insert("sip".to_string(), check_sip_enabled().unwrap_or(true));
        compatibility.insert("tcc".to_string(), check_tcc_available());
        
        Ok(compatibility)
    }
    
    /// Generate compatibility report
    pub async fn generate_compatibility_report(&self) -> Result<String> {
        let mut report = String::new();
        
        report.push_str("macOS Sandbox Compatibility Report\n");
        report.push_str("==================================\n\n");
        
        // macOS Version
        match get_macos_version() {
            Ok(version) => report.push_str(&format!("macOS Version: {}\n", version)),
            Err(_) => report.push_str("macOS Version: Unknown\n"),
        }
        
        // Compatibility checks
        let compatibility = self.check_macos_compatibility()?;
        report.push_str("\nFeature Compatibility:\n");
        for (feature, available) in compatibility {
            report.push_str(&format!("  {}: {}\n", feature, if available { "✓" } else { "✗" }));
        }
        
        Ok(report)
    }
    
    /// Check if sandbox is available on this system
    pub fn is_sandbox_available(&self) -> bool {
        check_app_sandbox_available()
    }
}

/// Implement the PluginSandbox trait for macOS
#[async_trait::async_trait]
impl PluginSandbox for MacOsSandbox {
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Creating sandbox for plugin {}", plugin_id);
        
        // Get or create default security context
        let context = self.get_security_context(plugin_id).await
            .unwrap_or_else(|_| SecurityContext::default());
        
        // Create sandbox profile
        let _profile_path = self.create_sandbox_profile(plugin_id, &context).await?;
        
        // Apply platform optimizations
        self.apply_platform_optimizations(plugin_id).await?;
        
        info!("Sandbox created for plugin {}", plugin_id);
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Destroying sandbox for plugin {}", plugin_id);
        
        // Terminate any running processes
        if let Ok(process_id) = {
            let process_ids = self.process_ids.read().await;
            process_ids.get(&plugin_id).copied()
                .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))
        } {
            self.terminate_process(process_id).await.ok(); // Ignore errors
        }
        
        // Remove TCC permissions
        self.remove_tcc_permissions(plugin_id).await.ok(); // Ignore errors
        
        // Clean up files
        cleanup_sandbox_files(plugin_id).ok(); // Ignore errors
        
        // Remove from tracking
        let mut process_ids = self.process_ids.write().await;
        process_ids.remove(&plugin_id);
        
        let mut contexts = self.security_contexts.write().await;
        contexts.remove(&plugin_id);
        
        let mut profiles = self.sandbox_profiles.write().await;
        profiles.remove(&plugin_id);
        
        info!("Sandbox destroyed for plugin {}", plugin_id);
        Ok(())
    }
    
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        let context = self.get_security_context(plugin_id).await?;
        
        // Check if the operation is allowed based on capabilities
        let allowed = match operation {
            "file:read" => true, // Usually allowed
            "file:write" => context.allowed_capabilities.contains(&"file:write".to_string()),
            "network:connect" => context.allowed_capabilities.contains(&"network:connect".to_string()),
            "network:listen" => context.allowed_capabilities.contains(&"network:listen".to_string()),
            "process:spawn" => context.allowed_capabilities.contains(&"process:spawn".to_string()),
            _ => false,
        };
        
        if allowed {
            Ok(())
        } else {
            Err(SandboxError::PermissionDenied(format!("Operation '{}' not permitted for plugin {}", operation, plugin_id)).into())
        }
    }
    
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        let process_ids = self.process_ids.read().await;
        let process_id = process_ids.get(&plugin_id)
            .copied()
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        self.get_process_resource_usage(process_id).await
    }
    
    async fn check_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()> {
        let context = self.get_security_context(plugin_id).await?;
        
        // Check if path is in allowed paths
        let path_allowed = context.allowed_paths.iter().any(|allowed_path| {
            path.starts_with(allowed_path)
        });
        
        if !path_allowed {
            return Err(SandboxError::PathAccessDenied(format!("Path {:?} not allowed for plugin {}", path, plugin_id)).into());
        }
        
        // Check write permission if needed
        if write && !context.allowed_capabilities.contains(&"file:write".to_string()) {
            return Err(SandboxError::PermissionDenied(format!("Write access denied for plugin {}", plugin_id)).into());
        }
        
        Ok(())
    }
    
    async fn check_capability(&self, plugin_id: Uuid, capability: &str) -> Result<bool> {
        let context = self.get_security_context(plugin_id).await?;
        
        if context.allowed_capabilities.contains(&capability.to_string()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    async fn apply_feature(&self, plugin_id: Uuid, feature: &str) -> Result<()> {
        debug!("Applying feature '{}' to plugin {}", feature, plugin_id);
        
        match feature {
            "tcc_integration" => self.apply_tcc_permissions(plugin_id).await,
            "sip_integration" => self.integrate_with_sip(plugin_id).await,
            "memory_limits" => self.enforce_memory_limit(plugin_id).await,
            "platform_optimization" => self.apply_platform_optimizations(plugin_id).await,
            _ => Err(SandboxError::UnsupportedFeature(format!("Feature '{}' not supported", feature)).into()),
        }
    }
    
    async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        self.set_security_context(plugin_id, context).await
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn get_resource_monitor(&self) -> Option<Arc<ResourceMonitor>> {
        Some(Arc::clone(&self.resource_monitor))
    }
    
    fn is_sandbox_available(&self) -> bool {
        self.is_sandbox_available()
    }
}

/// Feature availability checks for compatibility
pub fn has_app_sandbox() -> bool {
    check_app_sandbox_available()
}

pub fn has_sip() -> bool {
    check_sip_enabled().unwrap_or(true)
}

pub fn has_tcc_integration() -> bool {
    check_tcc_available()
} 