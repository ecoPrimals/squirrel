//! Sandbox Profile Generation and Management
//!
//! This module handles the creation and management of macOS sandbox profiles
//! for different security contexts and permission levels.

use super::*;
use tokio::task;
use std::io::Write;
use std::fs::OpenOptions;

impl MacOsSandbox {
    /// Create a sandbox profile for a plugin
    pub async fn create_sandbox_profile(&self, plugin_id: Uuid, context: &SecurityContext) -> Result<PathBuf> {
        let profile_path = self.get_sandbox_profile_path(plugin_id);
        
        // Create sandbox profile
        let profile_content = self.generate_sandbox_profile(plugin_id, context).await?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = profile_path.parent() {
            let parent_path = parent.to_path_buf();
            task::spawn_blocking(move || {
                std::fs::create_dir_all(parent_path)
            }).await??;
        }
        
        // Write sandbox profile to file
        let profile_path_clone = profile_path.clone();
        let result = task::spawn_blocking(move || {
            let mut file = std::fs::File::create(&profile_path_clone)
                .map_err(|e| SandboxError::Creation(format!(
                    "Failed to create sandbox profile for plugin {}: {}",
                    plugin_id, e
                )))?;
                
            std::io::Write::write_all(&mut file, profile_content.as_bytes())
                .map_err(|e| SandboxError::Creation(format!(
                    "Failed to write sandbox profile for plugin {}: {}",
                    plugin_id, e
                )))?;
                
            Ok::<PathBuf, SandboxError>(profile_path_clone)
        }).await?;
        
        result.map_err(CoreError::from)
    }
    
    /// Generate a sandbox profile based on context
    pub async fn generate_sandbox_profile(&self, plugin_id: Uuid, context: &SecurityContext) -> Result<String> {
        // Create a enhanced sandbox profile with macOS syntax and better security
        let mut profile = String::new();
        
        // Add standard header with version and metadata
        profile.push_str(";  Generated Sandbox Profile for Squirrel Plugin\n");
        profile.push_str(&format!(";  Plugin ID: {}\n", plugin_id));
        profile.push_str(&format!(";  Permission Level: {:?}\n", context.permission_level));
        profile.push_str(&format!(";  Generated: {}\n", chrono::Utc::now()));
        profile.push_str("(version 1)\n\n");
        
        // Add debug section
        profile.push_str("; Debug Information\n");
        profile.push_str("(debug sandbox-profile)\n\n");
        
        // Different base profile depending on permission level
        match context.permission_level {
            PermissionLevel::System => {
                profile.push_str(&self.generate_system_level_profile(context).await?);
            },
            PermissionLevel::User => {
                profile.push_str(&self.generate_user_level_profile(context).await?);
            },
            PermissionLevel::Restricted => {
                profile.push_str(&self.generate_restricted_level_profile(context).await?);
            },
        }
        
        // Add resource limits
        profile.push_str("\n; Resource limits\n");
        profile.push_str(&super::resource_limits::generate_resource_limits_rules(context).await?);
        
        // Add entitlement rules
        profile.push_str("\n; Entitlements\n");
        profile.push_str(&self.generate_entitlement_rules(context).await?);
        
        // Add process restrictions
        profile.push_str("\n; Process management\n");
        profile.push_str(&self.generate_process_management_rules(context));
        
        // Add security summary at the end
        profile.push_str("\n; Profile Summary\n");
        profile.push_str(&format!("; Permission Level: {:?}\n", context.permission_level));
        profile.push_str(&format!("; Capabilities: {:?}\n", context.allowed_capabilities));
        profile.push_str(&format!("; Paths: {}\n", context.allowed_paths.len()));
        
        Ok(profile)
    }
    
    /// Generate system-level sandbox profile
    async fn generate_system_level_profile(&self, context: &SecurityContext) -> Result<String> {
        let mut profile = String::new();
        
        profile.push_str("; System Permission Level - High Access\n");
        
        // System permissions have extensive access with some guard rails
        profile.push_str("(allow default)\n");
        
        // Deny dangerous operations even for system-level
        profile.push_str("\n; Security restrictions even for system level\n");
        profile.push_str("(deny file-write-setugid)\n");
        profile.push_str("(deny nvram*)\n");
        profile.push_str("(deny system-privilege)\n");
        
        // App-specific paths
        profile.push_str("\n; App-specific paths\n");
        for path in &context.allowed_paths {
            if let Ok(canonical) = path.canonicalize() {
                if let Some(path_str) = canonical.to_str() {
                    profile.push_str(&format!("(allow file* (subpath \"{}\"))\n", path_str));
                }
            }
        }
        
        // Add TCC access permissions based on capabilities
        profile.push_str("\n; TCC Access Permissions\n");
        for capability in &context.allowed_capabilities {
            if capability.starts_with("hardware:") || capability.starts_with("data:") {
                profile.push_str(&format!("; Capability: {}\n", capability));
            }
        }
        
        Ok(profile)
    }
    
    /// Generate user-level sandbox profile
    async fn generate_user_level_profile(&self, context: &SecurityContext) -> Result<String> {
        let mut profile = String::new();
        
        profile.push_str("; User Permission Level - Standard Access\n");
        
        // Default to deny but allow basic functionality
        profile.push_str("(deny default)\n");
        
        // File system access
        profile.push_str("\n; Basic file system access\n");
        profile.push_str("(allow file-read-metadata)\n");
        profile.push_str("(allow file-read-data (subpath \"/usr/lib\"))\n");
        profile.push_str("(allow file-read-data (subpath \"/System/Library\"))\n");
        profile.push_str("(allow file-read-data (subpath \"/Library\"))\n");
        
        // Allow read-write access to app temp directory
        profile.push_str("\n; Temporary directory access\n");
        profile.push_str("(allow file-read* file-write* (subpath \"/tmp\"))\n");
        
        // Add user's home directory with controlled permissions
        if let Ok(home) = std::env::var("HOME") {
            profile.push_str("\n; Limited home directory access\n");
            
            // Always allow access to preference files
            profile.push_str(&format!("(allow file-read* (subpath \"{}/Library/Preferences\"))\n", home));
            
            // Allow read-only access to Documents
            profile.push_str(&format!("(allow file-read* (subpath \"{}/Documents\"))\n", home));
            
            // Deny access to sensitive user directories
            profile.push_str(&format!("(deny file* (subpath \"{}/Library/Keychains\"))\n", home));
            profile.push_str(&format!("(deny file* (subpath \"{}/Library/Cookies\"))\n", home));
        }
        
        // Network access
        profile.push_str("\n; Network access\n");
        if context.allowed_capabilities.contains(&"network:connect".to_string()) {
            profile.push_str("(allow network-outbound)\n");
        } else {
            profile.push_str("(deny network-outbound)\n");
        }
        
        if context.allowed_capabilities.contains(&"network:listen".to_string()) {
            profile.push_str("(allow network-inbound (local tcp))\n");
        } else {
            profile.push_str("(deny network-inbound)\n");
        }
        
        // App-specific paths with controlled access
        profile.push_str("\n; App-specific paths\n");
        for path in &context.allowed_paths {
            if let Ok(canonical) = path.canonicalize() {
                if let Some(path_str) = canonical.to_str() {
                    // Allow read access for all allowed paths
                    profile.push_str(&format!("(allow file-read* (subpath \"{}\"))\n", path_str));
                    
                    // Allow write access only if capability is granted
                    if context.allowed_capabilities.contains(&"file:write".to_string()) {
                        profile.push_str(&format!("(allow file-write* (subpath \"{}\"))\n", path_str));
                    }
                }
            }
        }
        
        Ok(profile)
    }
    
    /// Generate restricted-level sandbox profile
    async fn generate_restricted_level_profile(&self, context: &SecurityContext) -> Result<String> {
        let mut profile = String::new();
        
        profile.push_str("; Restricted Permission Level - Minimal Access\n");
        
        // Default deny everything
        profile.push_str("(deny default)\n");
        
        // Minimal filesystem read access
        profile.push_str("\n; Minimal system access\n");
        profile.push_str("(allow file-read-metadata)\n");
        profile.push_str("(allow file-read-data (subpath \"/usr/lib\"))\n");
        profile.push_str("(allow file-read-data (subpath \"/System/Library/Frameworks\"))\n");
        profile.push_str("(allow file-read-data (subpath \"/System/Library/PrivateFrameworks\"))\n");
        
        // Very limited tmp access
        profile.push_str("\n; Temporary directory access\n");
        profile.push_str("(allow file-read* file-write* (literal \"/tmp/squirrel-restricted\"))\n");
        
        // App-specific paths with strict read-only access
        profile.push_str("\n; App-specific paths (read-only)\n");
        for path in &context.allowed_paths {
            if let Ok(canonical) = path.canonicalize() {
                if let Some(path_str) = canonical.to_str() {
                    profile.push_str(&format!(
                        "(allow file-read-data file-read-metadata (subpath \"{}\"))\n",
                        path_str
                    ));
                }
            }
        }
        
        // Allow minimal system services for basic functionality
        profile.push_str("\n; Minimal system services\n");
        profile.push_str("(allow mach-lookup (global-name \"com.apple.system.logger\"))\n");
        
        // No write access by default
        profile.push_str("\n; Explicit write denial\n");
        profile.push_str("(deny file-write*)\n");
        
        // No network access by default
        profile.push_str("\n; Network restrictions\n");
        profile.push_str("(deny network-inbound)\n");
        profile.push_str("(deny network-outbound)\n");
        
        Ok(profile)
    }
    
    /// Generate entitlement rules for the sandbox profile
    async fn generate_entitlement_rules(&self, context: &SecurityContext) -> Result<String> {
        let mut rules = String::new();
        
        // Add common entitlements based on capabilities
        for capability in &context.allowed_capabilities {
            match capability.as_str() {
                "hardware:camera" => {
                    rules.push_str("(allow device-camera)\n");
                },
                "hardware:microphone" => {
                    rules.push_str("(allow device-microphone)\n");
                },
                "data:contacts" => {
                    rules.push_str("(allow file-read* (subpath \"~/Library/Application Support/AddressBook\"))\n");
                },
                "data:calendar" => {
                    rules.push_str("(allow file-read* (subpath \"~/Library/Calendars\"))\n");
                },
                "data:photos" => {
                    rules.push_str("(allow file-read* (subpath \"~/Pictures\"))\n");
                },
                _ => {
                    // Add comment for unhandled capabilities
                    rules.push_str(&format!("; Capability: {}\n", capability));
                }
            }
        }
        
        Ok(rules)
    }
    
    /// Generate process management rules
    fn generate_process_management_rules(&self, context: &SecurityContext) -> String {
        let mut rules = String::new();
        
        match context.permission_level {
            PermissionLevel::System => {
                rules.push_str("(allow process-fork)\n");
                rules.push_str("(allow process-exec)\n");
            },
            PermissionLevel::User => {
                if context.allowed_capabilities.contains(&"process:spawn".to_string()) {
                    rules.push_str("(allow process-fork)\n");
                    rules.push_str("(allow process-exec (subpath \"/usr/bin\"))\n");
                    rules.push_str("(allow process-exec (subpath \"/bin\"))\n");
                } else {
                    rules.push_str("(deny process-fork)\n");
                    rules.push_str("(deny process-exec (with no-log))\n");
                }
            },
            PermissionLevel::Restricted => {
                rules.push_str("(deny process-fork (with no-log))\n");
                rules.push_str("(deny process-exec (with no-log))\n");
            }
        };
        
        rules
    }
} 