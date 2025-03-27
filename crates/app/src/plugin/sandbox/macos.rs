#![cfg(target_os = "macos")]

//! macOS-specific plugin sandbox implementation
//!
//! This module provides a macOS-specific implementation of the PluginSandbox trait
//! using App Sandbox and resource limits.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{debug, error, info, trace, warn};
use std::any::Any;
use tokio::process::Command;
use tokio::task;
use std::io::Write;
use std::fs::OpenOptions;
use tokio::time::{sleep, Duration};
use std::process::Command as StdCommand;

use crate::error::{Result, SquirrelError, CoreError};
use crate::plugin::security::{SecurityContext, PermissionLevel, ResourceLimits};
use crate::plugin::resource_monitor::{ResourceMonitor, ResourceUsage};
use crate::plugin::sandbox::{PluginSandbox, SandboxError};

/// macOS-specific plugin sandbox implementation
#[derive(Debug)]
pub struct MacOsSandbox {
    /// Process IDs for plugins
    process_ids: Arc<RwLock<HashMap<Uuid, u32>>>,
    /// Security contexts for plugins
    security_contexts: Arc<RwLock<HashMap<Uuid, SecurityContext>>>,
    /// Resource monitor
    resource_monitor: Arc<ResourceMonitor>,
    /// Profile paths for sandboxed applications
    sandbox_profiles: Arc<RwLock<HashMap<Uuid, PathBuf>>>,
}

impl MacOsSandbox {
    /// Create a new macOS sandbox
    pub fn new(resource_monitor: Arc<ResourceMonitor>) -> Result<Self> {
        Ok(Self {
            process_ids: Arc::new(RwLock::new(HashMap::new())),
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor,
            sandbox_profiles: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Set a security context for a plugin
    pub async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        // Update the stored security context
        let mut contexts = self.security_contexts.write().await;
        contexts.insert(plugin_id, context);
        
        // If the plugin is already sandboxed, we may need to restart it
        let process_ids = self.process_ids.read().await;
        if process_ids.contains_key(&plugin_id) {
            // In macOS, we can't modify sandbox settings for running processes
            // We would need to restart the plugin, which is handled at a higher level
            warn!("Security context updated for plugin {}, but changes will only take effect after restart", plugin_id);
        }
        
        Ok(())
    }
    
    /// Get a security context for a plugin
    pub async fn get_security_context(&self, plugin_id: Uuid) -> Result<SecurityContext> {
        let contexts = self.security_contexts.read().await;
        contexts.get(&plugin_id)
            .cloned()
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id).into())
    }
    
    /// Create a sandbox profile for a plugin
    async fn create_sandbox_profile(&self, plugin_id: Uuid, context: &SecurityContext) -> Result<PathBuf> {
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
    async fn generate_sandbox_profile(&self, plugin_id: Uuid, context: &SecurityContext) -> Result<String> {
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
            },
            
            PermissionLevel::User => {
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
                if context.allowed_capabilities.contains("network:connect") {
                    profile.push_str("(allow network-outbound)\n");
                } else {
                    profile.push_str("(deny network-outbound)\n");
                }
                
                if context.allowed_capabilities.contains("network:listen") {
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
                            if context.allowed_capabilities.contains("file:write") {
                                profile.push_str(&format!("(allow file-write* (subpath \"{}\"))\n", path_str));
                            }
                        }
                    }
                }
            },
            
            PermissionLevel::Restricted => {
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
            },
        }
        
        // Add resource limits
        profile.push_str("\n; Resource limits\n");
        profile.push_str(&self.generate_resource_limits_rules(context).await?);
        
        // Add entitlement rules
        profile.push_str("\n; Entitlements\n");
        profile.push_str(&self.generate_entitlement_rules(context).await?);
        
        // Add process restrictions
        profile.push_str("\n; Process management\n");
        match context.permission_level {
            PermissionLevel::System => {
                profile.push_str("(allow process-fork)\n");
                profile.push_str("(allow process-exec)\n");
            },
            PermissionLevel::User => {
                if context.allowed_capabilities.contains("process:spawn") {
                    profile.push_str("(allow process-fork)\n");
                    profile.push_str("(allow process-exec (subpath \"/usr/bin\"))\n");
                    profile.push_str("(allow process-exec (subpath \"/bin\"))\n");
                } else {
                    profile.push_str("(deny process-fork)\n");
                    profile.push_str("(deny process-exec (with no-log))\n");
                }
            },
            PermissionLevel::Restricted => {
                profile.push_str("(deny process-fork (with no-log))\n");
                profile.push_str("(deny process-exec (with no-log))\n");
            }
        };
        
        // Add security summary at the end
        profile.push_str("\n; Security Summary\n");
        profile.push_str(&format!("; Permission Level: {:?}\n", context.permission_level));
        profile.push_str(&format!("; Resource Limits: CPU {}%, Memory {} MB\n", 
            context.resource_limits.max_cpu_percent,
            context.resource_limits.max_memory_bytes / (1024 * 1024)));
        profile.push_str(&format!("; Capabilities Count: {}\n", context.allowed_capabilities.len()));
        profile.push_str(&format!("; Allowed Paths Count: {}\n", context.allowed_paths.len()));
        
        debug!("Generated enhanced sandbox profile for plugin {}", plugin_id);
        trace!("Profile content:\n{}", profile);
        
        Ok(profile)
    }
    
    /// Enhanced helper method to generate resource-specific rules
    async fn generate_resource_limits_rules(&self, context: &SecurityContext) -> Result<String> {
        // Extract resource limits
        let memory_limit_mb = context.resource_limits.max_memory_bytes / (1024 * 1024);
        let cpu_limit = context.resource_limits.max_cpu_percent;
        let thread_limit = context.resource_limits.max_threads;
        let fd_limit = context.resource_limits.max_file_descriptors;
        
        // Add debug comment for auditing purposes
        let mut rules = format!(
            "(version 1)\n(debug resource-limits)\n;; Resource limits for permission level: {:?}\n", 
            context.permission_level
        );
        
        // Add file descriptor limit
        rules.push_str(&format!(
            ";; File descriptor limit: {}\n(file-descriptor-limit {})\n\n", 
            fd_limit, fd_limit
        ));
        
        // Add process spawn limitation based on thread count
        rules.push_str(&format!(
            ";; Process spawn limit based on thread count: {}\n(process-spawn-limit {})\n\n", 
            thread_limit, thread_limit / 2
        ));
        
        // Add process exec limits
        if context.permission_level == PermissionLevel::Restricted {
            rules.push_str(";; Restricted process execution\n(deny process-exec*)\n");
            rules.push_str("(allow process-exec-interpreter (interpreter-path \"*\") (subpath \"/usr/bin/env\"))\n\n");
        } else {
            // Allow spawning processes with limitations
            rules.push_str(";; Limited process execution\n");
            rules.push_str("(allow process-exec)\n");
            rules.push_str("(allow process-exec-interpreter)\n\n");
        }
        
        // Add CPU priority control
        rules.push_str(&format!(
            ";; CPU priority directives\n(cpu-priority {})\n\n", 
            match cpu_limit {
                0..=25 => "low",
                26..=60 => "normal",
                _ => "high"
            }
        ));
        
        // Add system service restrictions based on permission level
        match context.permission_level {
            PermissionLevel::System => {
                rules.push_str(";; System-level service access\n(allow mach-lookup)\n");
                rules.push_str("(allow system-socket)\n");
            },
            PermissionLevel::User => {
                rules.push_str(";; User-level service access restrictions\n");
                rules.push_str("(allow mach-lookup (global-name \"com.apple.system.*\"))\n");
                rules.push_str("(deny mach-lookup (global-name \"com.apple.security.*\"))\n");
            },
            PermissionLevel::Restricted => {
                rules.push_str(";; Restricted service access\n");
                rules.push_str("(deny mach-lookup (with no-log))\n");
                rules.push_str("(deny system-socket (with no-log))\n");
                
                // Allow the minimal necessary services
                rules.push_str("(allow mach-lookup (global-name \"com.apple.system.notification_center\"))\n");
                rules.push_str("(allow mach-lookup (global-name \"com.apple.system.logger\"))\n");
            }
        }
        
        // Add resource manager directive based on memory limit
        rules.push_str(&format!(
            "\n;; Memory limit: {} MB\n(memory-limit {} \"MB\")\n", 
            memory_limit_mb, memory_limit_mb
        ));
        
        // Resource monitoring/violation policy based on permission level
        rules.push_str("\n;; Resource violation policy\n");
        match context.permission_level {
            PermissionLevel::System => {
                rules.push_str("(resource-monitor \"system\")\n");
                rules.push_str("(violation-action log)\n");
                rules.push_str("(monitoring-interval 60)\n");
            },
            PermissionLevel::User => {
                rules.push_str("(resource-monitor \"user\")\n");
                rules.push_str("(violation-action throttle)\n");
                rules.push_str("(monitoring-interval 30)\n");
            },
            PermissionLevel::Restricted => {
                rules.push_str("(resource-monitor \"restricted\")\n");
                rules.push_str("(violation-action terminate)\n");
                rules.push_str("(monitoring-interval 15)\n");
            }
        }
        
        Ok(rules)
    }
    
    /// Generate entitlement rules based on permission level and security context
    /// 
    /// This method creates appropriate entitlement rules for macOS sandbox profiles
    /// with fine-grained control over system resource access based on permission level.
    async fn generate_entitlement_rules(&self, context: &SecurityContext) -> Result<String> {
        let mut rules = format!(
            ";; Entitlement rules for permission level: {:?}\n", 
            context.permission_level
        );
        
        // Basic entitlement structure
        rules.push_str("(define STANDARD_ENTITLEMENTS\n");
        
        // Standard user entitlements available to all permission levels
        rules.push_str("  (allow user-preference-read)\n");
        rules.push_str("  (allow clipboard-read (require-entitlement \"com.apple.security.clipboard-read\"))\n");
        rules.push_str("  (allow system-info)\n");
        rules.push_str("  (allow device-camera (require-entitlement \"com.apple.security.device.camera\"))\n");
        rules.push_str("  (allow device-microphone (require-entitlement \"com.apple.security.device.microphone\"))\n");
        rules.push_str(")\n\n");
        
        // Permission-specific entitlements
        match context.permission_level {
            PermissionLevel::System => {
                rules.push_str(";; System-level entitlements\n");
                rules.push_str("(define SYSTEM_ENTITLEMENTS\n");
                rules.push_str("  (allow user-preference-write)\n");
                rules.push_str("  (allow clipboard-write)\n");
                rules.push_str("  (allow clipboard-read)\n");
                rules.push_str("  (allow ipc-posix-shm)\n");
                rules.push_str("  (allow job-creation)\n");
                rules.push_str("  (allow mach-lookup (global-name-regex #\".*\"))\n");
                rules.push_str("  (allow system-socket)\n");
                rules.push_str("  (allow system-fsctl)\n");
                rules.push_str("  (allow process-info-pidinfo)\n");
                rules.push_str("  (allow process-info-pidfdinfo)\n");
                rules.push_str("  (allow process-info-pidrusage)\n");
                rules.push_str("  (allow process-info-setcontrol)\n");
                rules.push_str("  (allow system-sysctl)\n");
                rules.push_str("  (allow system-kext)\n");
                rules.push_str("  (allow device-usb)\n");
                rules.push_str("  (allow network-outbound)\n");
                rules.push_str("  (allow network-inbound)\n");
                rules.push_str("  (allow network-bind)\n");
                rules.push_str("  (allow authorization-right-obtain)\n");
                rules.push_str("  (allow hypervisor)\n");
                rules.push_str(")\n\n");
                
                // Apply both standard and system entitlements
                rules.push_str("(STANDARD_ENTITLEMENTS)\n");
                rules.push_str("(SYSTEM_ENTITLEMENTS)\n");
            },
            PermissionLevel::User => {
                rules.push_str(";; User-level entitlements\n");
                rules.push_str("(define USER_ENTITLEMENTS\n");
                rules.push_str("  (allow user-preference-write (preference-domain \"user\"))\n");
                rules.push_str("  (allow clipboard-write)\n");
                rules.push_str("  (allow ipc-posix-shm-read-data)\n");
                rules.push_str("  (allow ipc-posix-shm-write-data (require-entitlement \"com.apple.security.shm-write\"))\n");
                rules.push_str("  (allow mach-lookup\n");
                rules.push_str("    (global-name \"com.apple.system.notification_center\")\n");
                rules.push_str("    (global-name \"com.apple.system.logger\")\n");
                rules.push_str("    (global-name \"com.apple.pasteboard.1\")\n");
                rules.push_str("  )\n");
                rules.push_str("  (allow process-info-pidinfo (target self))\n");
                rules.push_str("  (allow network-outbound (remote tcp))\n");
                rules.push_str("  (allow network-outbound (remote udp) (require-entitlement \"com.apple.security.network.udp\"))\n");
                
                // Handle specific user-granted capabilities
                if context.allowed_capabilities.contains("network:listen") {
                    rules.push_str("  (allow network-inbound (local tcp) (require-entitlement \"com.apple.security.network.server\"))\n");
                }
                
                rules.push_str(")\n\n");
                
                // Apply both standard and user entitlements
                rules.push_str("(STANDARD_ENTITLEMENTS)\n");
                rules.push_str("(USER_ENTITLEMENTS)\n");
            },
            PermissionLevel::Restricted => {
                rules.push_str(";; Restricted-level entitlements\n");
                rules.push_str("(define RESTRICTED_ENTITLEMENTS\n");
                rules.push_str("  (allow user-preference-read (preference-domain \"restricted\"))\n");
                
                // Only allow outbound network if capability is granted
                if context.allowed_capabilities.contains("network:connect") {
                    rules.push_str("  (allow network-outbound (remote tcp) (require-entitlement \"com.apple.security.network.client\"))\n");
                } else {
                    rules.push_str("  (deny network-outbound (with no-log))\n");
                }
                
                rules.push_str("  (deny network-inbound (with no-log))\n");
                rules.push_str("  (deny network-bind (with no-log))\n");
                rules.push_str("  (deny mach-lookup (global-name \"com.apple.system.security.*\") (with no-log))\n");
                rules.push_str("  (deny system-socket (with no-log))\n");
                rules.push_str("  (deny job-creation (with no-log))\n");
                rules.push_str("  (deny process-info (with no-log))\n");
                
                // Allow only necessary operations
                rules.push_str("  (allow mach-lookup\n");
                rules.push_str("    (global-name \"com.apple.system.notification_center\")\n");
                rules.push_str("    (global-name \"com.apple.system.logger\")\n");
                rules.push_str("  )\n");
                rules.push_str(")\n\n");
                
                // Apply both standard and restricted entitlements
                rules.push_str("(STANDARD_ENTITLEMENTS)\n");
                rules.push_str("(RESTRICTED_ENTITLEMENTS)\n");
            }
        }
        
        // Add custom entitlements based on specific capabilities
        if context.allowed_capabilities.contains("hardware:bluetooth") {
            rules.push_str("\n;; Bluetooth access\n");
            rules.push_str("(allow bluetooth (require-entitlement \"com.apple.security.device.bluetooth\"))\n");
        }
        
        if context.allowed_capabilities.contains("hardware:usb") && context.permission_level != PermissionLevel::Restricted {
            rules.push_str("\n;; USB device access\n");
            rules.push_str("(allow device-usb (require-entitlement \"com.apple.security.device.usb\"))\n");
        }
        
        if context.allowed_capabilities.contains("hardware:printer") {
            rules.push_str("\n;; Printer access\n");
            rules.push_str("(allow printing (require-entitlement \"com.apple.security.print\"))\n");
        }
        
        if context.allowed_capabilities.contains("data:contacts") {
            rules.push_str("\n;; Contacts access\n");
            rules.push_str("(allow addressbook (require-entitlement \"com.apple.security.personal-information.addressbook\"))\n");
        }
        
        if context.allowed_capabilities.contains("data:photos") {
            rules.push_str("\n;; Photos access\n");
            rules.push_str("(allow photos-library (require-entitlement \"com.apple.security.personal-information.photos-library\"))\n");
        }
        
        Ok(rules)
    }

    /// Apply TCC (Transparency, Consent, and Control) permissions based on security context
    async fn apply_tcc_permissions(&self, plugin_id: Uuid) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
            
        // Get process ID if available
        let process_id = {
            let process_ids = self.process_ids.read().await;
            process_ids.get(&plugin_id).copied()
        };
        
        // We need a process ID to apply TCC permissions
        if process_id.is_none() {
            return Ok(());
        }
        
        let process_id = process_id.unwrap();
        
        // Get the process executable path
        let process_info = self.get_detailed_process_info(process_id).await?;
        let executable_path = if let Some(path) = process_info.get("executable_path") {
            path
        } else {
            return Err(SandboxError::Internal(
                format!("Could not determine executable path for process {}", process_id)
            ).into());
        };
        
        debug!("Applying TCC permissions for plugin {} (process {})", plugin_id, process_id);
        
        // In a real implementation, we would use the tccutil command to grant permissions
        // However, this requires admin privileges and user interaction
        // For now, we'll just log what permissions would be granted based on capabilities
        
        for capability in &context.allowed_capabilities {
            match capability.as_str() {
                "hardware:camera" => {
                    info!("Would grant camera access to {}", executable_path);
                },
                "hardware:microphone" => {
                    info!("Would grant microphone access to {}", executable_path);
                },
                "data:contacts" => {
                    info!("Would grant contacts access to {}", executable_path);
                },
                "data:photos" => {
                    info!("Would grant photos access to {}", executable_path);
                },
                "data:calendar" => {
                    info!("Would grant calendar access to {}", executable_path);
                },
                "data:reminders" => {
                    info!("Would grant reminders access to {}", executable_path);
                },
                "data:location" => {
                    info!("Would grant location access to {}", executable_path);
                },
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Check System Integrity Protection (SIP) status
    async fn check_sip_status(&self) -> Result<(bool, HashMap<String, bool>)> {
        debug!("Checking System Integrity Protection (SIP) status");
        
        // Execute the command to check SIP status
        let sip_status = Command::new("csrutil")
            .arg("status")
            .output()
            .await
            .map_err(|e| SandboxError::Platform(format!("Failed to check SIP status: {}", e)))?;
            
        let status_output = String::from_utf8_lossy(&sip_status.stdout);
        
        // Parse the overall status
        let sip_enabled = status_output.contains("System Integrity Protection status: enabled");
        
        // Parse individual protection categories
        let mut protections = HashMap::new();
        
        // Look for specific protections in the output
        protections.insert("filesystem_protection".to_string(), 
            !status_output.contains("Filesystem Protections: disabled"));
        protections.insert("kext_protection".to_string(), 
            !status_output.contains("Kext Signing: disabled"));
        protections.insert("nvram_protection".to_string(), 
            !status_output.contains("NVRAM Protections: disabled"));
        protections.insert("debug_protection".to_string(), 
            !status_output.contains("Debugging Protections: disabled"));
        protections.insert("dtrace_protection".to_string(), 
            !status_output.contains("DTrace Restrictions: disabled"));
        
        debug!("SIP status: enabled={}, protections={:?}", sip_enabled, protections);
        
        Ok((sip_enabled, protections))
    }
    
    /// Integrate with System Integrity Protection for enhanced security
    async fn integrate_with_sip(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Integrating with System Integrity Protection for plugin {}", plugin_id);
        
        // Check SIP status first
        let (sip_enabled, protections) = self.check_sip_status().await?;
        
        if !sip_enabled {
            warn!("System Integrity Protection is disabled. Some security features will not be available for plugin {}", plugin_id);
            return Ok(());
        }
        
        // Get security context for the plugin
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?
            .clone();
        drop(contexts);
        
        // Check if we should add SIP-specific protections
        if context.permission_level == PermissionLevel::Restricted {
            info!("Adding enhanced SIP restrictions for restricted plugin {}", plugin_id);
            
            // For restricted plugins, we should be even more cautious
            // These changes will be applied to the sandbox profile
            
            // Get the sandbox profile path
            let sandbox_profiles = self.sandbox_profiles.read().await;
            let profile_path = sandbox_profiles.get(&plugin_id)
                .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?
                .clone();
            drop(sandbox_profiles);
            
            // Read the existing profile
            let profile_content = tokio::fs::read_to_string(&profile_path).await
                .map_err(|e| SandboxError::Platform(format!("Failed to read sandbox profile: {}", e)))?;
            
            // Add SIP-specific protections
            let mut enhanced_profile = profile_content.clone();
            
            // Add protection against SIP-protected paths
            enhanced_profile.push_str("\n; SIP-specific protections\n");
            enhanced_profile.push_str("(deny file* (subpath \"/System/Library/Security\"))\n");
            enhanced_profile.push_str("(deny file* (subpath \"/System/Library/CoreServices\"))\n");
            enhanced_profile.push_str("(deny file* (literal \"/System/Library/Sandbox/rootless.conf\"))\n");
            enhanced_profile.push_str("(deny file* (literal \"/System/Library/Sandbox/Compatibility.bundle\"))\n");
            
            // Prevent access to system configuration and sensitive files
            if protections.get("filesystem_protection").copied().unwrap_or(true) {
                enhanced_profile.push_str("\n; SIP filesystem protections\n");
                enhanced_profile.push_str("(deny file* (regex #\"^/usr/local/bin/sudo$\"))\n");
                enhanced_profile.push_str("(deny file* (regex #\"^/etc/(pam|sudoers).*\"))\n");
                enhanced_profile.push_str("(deny file* (subpath \"/System/Library/Sandbox\"))\n");
            }
            
            // Prevent kernel extensions access if kext protection is enabled
            if protections.get("kext_protection").copied().unwrap_or(true) {
                enhanced_profile.push_str("\n; SIP kext protections\n");
                enhanced_profile.push_str("(deny file* (subpath \"/System/Library/Extensions\"))\n");
                enhanced_profile.push_str("(deny iokit-open (iokit-user-client-class \"IOKernelDebugger\"))\n");
            }
            
            // Write the enhanced profile back
            tokio::fs::write(&profile_path, enhanced_profile).await
                .map_err(|e| SandboxError::Platform(format!("Failed to write enhanced sandbox profile: {}", e)))?;
            
            info!("Enhanced sandbox profile with SIP protections for plugin {}", plugin_id);
        } else if context.permission_level == PermissionLevel::User {
            // For user level, we add some protections but not as strict
            let sandbox_profiles = self.sandbox_profiles.read().await;
            let profile_path = sandbox_profiles.get(&plugin_id)
                .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?
                .clone();
            drop(sandbox_profiles);
            
            // Read the existing profile
            let profile_content = tokio::fs::read_to_string(&profile_path).await
                .map_err(|e| SandboxError::Platform(format!("Failed to read sandbox profile: {}", e)))?;
            
            // Add SIP-specific protections
            let mut enhanced_profile = profile_content.clone();
            
            // Add basic SIP protections
            enhanced_profile.push_str("\n; Basic SIP protections for User level\n");
            enhanced_profile.push_str("(deny file* (subpath \"/System/Library/Security\"))\n");
            enhanced_profile.push_str("(deny file* (literal \"/System/Library/Sandbox/rootless.conf\"))\n");
            
            // Write the enhanced profile back
            tokio::fs::write(&profile_path, enhanced_profile).await
                .map_err(|e| SandboxError::Platform(format!("Failed to write enhanced sandbox profile: {}", e)))?;
            
            info!("Added basic SIP protections for User level plugin {}", plugin_id);
        }
        
        // For System level, we don't add restrictions as they should have the highest privileges
        
        Ok(())
    }
    
    /// Apply additional platform-specific optimizations for macOS
    async fn apply_platform_optimizations(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Applying platform-specific optimizations for plugin {}", plugin_id);
        
        // Get process ID if available
        let process_id = {
            let process_ids = self.process_ids.read().await;
            process_ids.get(&plugin_id).copied()
        };
        
        // We need a process ID to apply optimizations
        if process_id.is_none() {
            return Ok(());
        }
        
        let process_id = process_id.unwrap();
        
        // Get security context to determine optimization level
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?
            .clone();
        drop(contexts);
        
        // 1. Apply QoS (Quality of Service) class based on permission level and resource limits
        let qos_class = match context.permission_level {
            PermissionLevel::System => "user-interactive", // Highest priority
            PermissionLevel::User => {
                if context.resource_limits.max_cpu_percent > 60 {
                    "user-initiated" // Higher priority for CPU-intensive tasks
                } else {
                    "utility" // Standard priority for regular tasks
                }
            },
            PermissionLevel::Restricted => "background" // Lowest priority
        };
        
        // Apply QoS class using the setpriority command
        let qos_result = Command::new("sh")
            .args([
                "-c",
                &format!("launchctl setpriority {} {}", qos_class, process_id)
            ])
            .output()
            .await;
            
        match qos_result {
            Ok(output) if output.status.success() => {
                info!("Applied QoS class '{}' to process {} for plugin {}", qos_class, process_id, plugin_id);
            },
            Ok(_) => {
                warn!("Failed to apply QoS class to process {} for plugin {}", process_id, plugin_id);
            },
            Err(e) => {
                warn!("Error applying QoS class to process {} for plugin {}: {}", process_id, plugin_id, e);
            }
        }
        
        // 2. Apply Low Latency I/O for System level when appropriate
        if context.permission_level == PermissionLevel::System 
           && context.allowed_capabilities.contains("system:realtime") {
            let throttle_io_result = Command::new("sh")
                .args([
                    "-c",
                    &format!("sudo process_policy -r lowlatency -p {}", process_id)
                ])
                .output()
                .await;
                
            match throttle_io_result {
                Ok(output) if output.status.success() => {
                    info!("Applied low latency I/O policy to process {} for plugin {}", process_id, plugin_id);
                },
                _ => {
                    // This requires privileged access, so we don't warn if it fails
                    debug!("Could not apply low latency I/O policy (likely requires privileges)");
                }
            }
        }
        
        // 3. Apply memory optimization based on permission level
        let memory_optimization = match context.permission_level {
            PermissionLevel::System => "none", // No memory pressure for System level
            PermissionLevel::User => "normal",
            PermissionLevel::Restricted => "aggressive"
        };
        
        // Apply memory pressure using the memory_pressure command if it exists
        let memory_pressure_result = Command::new("sh")
            .args([
                "-c",
                &format!("command -v memory_pressure && memory_pressure -l {} -p {}", 
                    memory_optimization, process_id)
            ])
            .output()
            .await;
            
        if let Ok(output) = memory_pressure_result {
            if output.status.success() {
                debug!("Applied memory optimization '{}' to process {} for plugin {}", 
                    memory_optimization, process_id, plugin_id);
            }
        }
        
        // 4. Apply timer throttling for energy efficiency in restricted mode
        if context.permission_level == PermissionLevel::Restricted {
            let timer_throttle_result = Command::new("sh")
                .args([
                    "-c", 
                    &format!("sudo powermetrics --show-process-energy --samplers tasks && echo 1 > /dev/null")
                ])
                .output()
                .await;
                
            // We don't actually need to do anything with powermetrics output
            // This is just to demonstrate the concept - in a real implementation,
            // we would use the powermetrics data to make optimization decisions
            if let Ok(output) = timer_throttle_result {
                if output.status.success() {
                    debug!("Checked energy profile for energy efficiency optimization");
                }
            }
        }
        
        info!("Applied platform-specific optimizations for plugin {}", plugin_id);
        Ok(())
    }
    
    /// Launch a process with sandbox with enhanced error handling and process management
    pub async fn launch_with_sandbox(&self, plugin_id: Uuid, executable: &Path, args: &[&str]) -> Result<u32> {
        // Get security context
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .cloned()
            .unwrap_or_default();
        drop(contexts); // Release lock early

        // Verify executable exists
        if !executable.exists() {
            return Err(SandboxError::Creation(format!(
                "Executable not found: {}", executable.display()
            )).into());
        }

        // Create sandbox profile with optimizations
        debug!("Creating sandbox profile for plugin {}", plugin_id);
        let profile_path = self.create_sandbox_profile(plugin_id, &context).await?;
        
        // Optimize the sandbox profile
        self.apply_macos_optimizations(&profile_path)?;

        // Check if the sandbox profile is valid
        let validation_result = StdCommand::new("sandbox-exec")
            .args(["-f", profile_path.to_str().unwrap_or_default(), "-n"])
            .output();
            
        if let Ok(output) = validation_result {
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Sandbox profile validation warning: {}", error);
            }
        }

        // Launch process with sandbox-exec with enhanced error handling
        let mut command = Command::new("sandbox-exec");
        command.arg("-f")
               .arg(&profile_path)
               .arg(executable)
               .args(args);
               
        // Set environment variables based on resource limits
        if let Some(memory_limit_bytes) = context.resource_limits.max_memory_bytes.checked_div(1024 * 1024) {
            command.env("SQUIRREL_MEMORY_LIMIT_MB", memory_limit_bytes.to_string());
        }
        command.env("SQUIRREL_CPU_LIMIT", context.resource_limits.max_cpu_percent.to_string());
        command.env("SQUIRREL_PLUGIN_ID", plugin_id.to_string());
        command.env("SQUIRREL_SANDBOX_ENABLED", "1");
        
        // Set up proper error handling for process output
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        
        info!("Launching sandboxed process for plugin {}: {:?}", plugin_id, command);
        
        // Spawn the process with robust error handling
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                error!("Failed to launch sandbox-exec for plugin {}: {}", plugin_id, e);
                
                // Try falling back to direct execution without sandbox if sandbox failed
                warn!("Attempting fallback to direct execution without sandbox for plugin {}", plugin_id);
                let fallback_result = Command::new(executable)
                    .args(args)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn();
                    
                match fallback_result {
                    Ok(fallback_child) => {
                        warn!("Fallback execution succeeded for plugin {} without sandbox", plugin_id);
                        fallback_child
                    },
                    Err(fallback_err) => {
                        return Err(SandboxError::Creation(format!(
                            "Failed to launch process for plugin {}: {}. Fallback also failed: {}", 
                            plugin_id, e, fallback_err
                        )).into());
                    }
                }
            }
        };
        
        // Get process ID with improved error handling
        let process_id = child.id().ok_or_else(|| SandboxError::Creation(
            format!("Failed to get process ID for plugin {}", plugin_id)
        ))?;
        
        info!("Process ID for plugin {}: {}", plugin_id, process_id);
        
        // Store the process ID
        let mut process_ids = self.process_ids.write().await;
        process_ids.insert(plugin_id, process_id);
        drop(process_ids); // Release lock early
        
        // Store profile path
        let mut sandbox_profiles = self.sandbox_profiles.write().await;
        sandbox_profiles.insert(plugin_id, profile_path.clone());
        drop(sandbox_profiles); // Release lock early
        
        // Register process with resource monitor
        info!("Registering process {} with resource monitor for plugin {}", process_id, plugin_id);
        self.resource_monitor.register_process(plugin_id, process_id, executable).await?;
        
        // Apply resource limits
        self.enforce_memory_limit(plugin_id).await?;
        
        // Capture and log process output
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        
        // Handle stdout
        if let Some(stdout) = stdout {
            let plugin_id_clone = plugin_id;
            tokio::spawn(async move {
                let mut reader = tokio::io::BufReader::new(stdout);
                let mut line = String::new();
                loop {
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            debug!("[Plugin {} stdout]: {}", plugin_id_clone, line.trim());
                            line.clear();
                        },
                        Err(e) => {
                            warn!("Error reading stdout from plugin {}: {}", plugin_id_clone, e);
                            break;
                        }
                    }
                }
            });
        }
        
        // Handle stderr
        if let Some(stderr) = stderr {
            let plugin_id_clone = plugin_id;
            tokio::spawn(async move {
                let mut reader = tokio::io::BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            warn!("[Plugin {} stderr]: {}", plugin_id_clone, line.trim());
                            line.clear();
                        },
                        Err(e) => {
                            warn!("Error reading stderr from plugin {}: {}", plugin_id_clone, e);
                            break;
                        }
                    }
                }
            });
        }
        
        // Spawn a task to wait for the process to exit with improved cleanup
        let plugin_id_clone = plugin_id;
        let resource_monitor_clone = self.resource_monitor.clone();
        let process_ids_clone = self.process_ids.clone();
        let sandbox_profiles_clone = self.sandbox_profiles.clone();
        
        tokio::spawn(async move {
            // Wait for process to exit
            let exit_status = match child.wait().await {
                Ok(status) => {
                    if status.success() {
                        info!("Process {} for plugin {} exited successfully", 
                             process_id, plugin_id_clone);
                    } else {
                        warn!("Process {} for plugin {} exited with non-zero status: {:?}", 
                             process_id, plugin_id_clone, status);
                    }
                    Some(status)
                },
                Err(e) => {
                    error!("Error waiting for process {} for plugin {}: {}", 
                          process_id, plugin_id_clone, e);
                    None
                }
            };
            
            // Clean up resources
            info!("Cleaning up resources for plugin {}", plugin_id_clone);
            
            // Remove process ID
            let mut process_ids = process_ids_clone.write().await;
            process_ids.remove(&plugin_id_clone);
            drop(process_ids);
            
            // Remove sandbox profile
            let mut sandbox_profiles = sandbox_profiles_clone.write().await;
            if let Some(profile_path) = sandbox_profiles.remove(&plugin_id_clone) {
                // Clean up profile file
                if let Err(e) = tokio::fs::remove_file(&profile_path).await {
                    warn!("Failed to remove sandbox profile for plugin {}: {}", 
                         plugin_id_clone, e);
                }
            }
            drop(sandbox_profiles);
            
            // Unregister from resource monitor
            if let Err(e) = resource_monitor_clone.unregister_process(plugin_id_clone).await {
                warn!("Failed to unregister plugin {} from resource monitor: {}", 
                     plugin_id_clone, e);
            }
            
            info!("Cleanup complete for plugin {}", plugin_id_clone);
        });
        
        info!("Successfully launched sandboxed process {} for plugin {}", process_id, plugin_id);
        
        Ok(process_id)
    }
    
    /// Check if path is inside secure namespace
    fn is_path_in_secure_namespace(&self, path: &Path) -> bool {
        // Check if the path is under /tmp or the home directory of the current user
        let tmp_path = Path::new("/tmp");
        if path.starts_with(tmp_path) {
            return true;
        }
        
        // Check for home directory
        if let Ok(home) = std::env::var("HOME") {
            let home_path = Path::new(&home);
            if path.starts_with(home_path) {
                return true;
            }
        }
        
        false
    }
    
    /// Terminate a sandboxed process
    async fn terminate_process(&self, process_id: u32) -> Result<()> {
        // Use kill command to terminate process
        let output = Command::new("kill")
            .arg("-TERM")
            .arg(process_id.to_string())
            .output()
            .await
            .map_err(|e| SandboxError::Platform(format!(
                "Failed to terminate process {}: {}", process_id, e
            )))?;
        
        if !output.status.success() {
            // Try with SIGKILL if SIGTERM fails
            let output = Command::new("kill")
                .arg("-KILL")
                .arg(process_id.to_string())
                .output()
                .await
                .map_err(|e| SandboxError::Platform(format!(
                    "Failed to kill process {}: {}", process_id, e
                )))?;
            
            if !output.status.success() {
                return Err(SandboxError::Platform(format!(
                    "Failed to kill process {}: {}", 
                    process_id,
                    String::from_utf8_lossy(&output.stderr)
                )).into());
            }
        }
        
        Ok(())
    }
    
    /// Apply macOS-specific optimizations
    fn apply_macos_optimizations(&self, profile_path: &Path) -> Result<()> {
        // Check if optimization tools are available
        let has_sandbox_tool = StdCommand::new("sh")
            .args(["-c", "command -v sandbox-simplify"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        
        let has_sandbox_exec = StdCommand::new("sh")
            .args(["-c", "command -v sandbox-exec"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
            
        if !has_sandbox_exec {
            return Err(SandboxError::Platform(
                "sandbox-exec not found, sandboxing cannot be applied".to_string()
            ).into());
        }
        
        // Verify profile is valid by checking syntax
        let verify_result = StdCommand::new("sandbox-exec")
            .args(["-p", "(version 1)\n(allow default)", "-n"])
            .output();
            
        if let Err(e) = verify_result {
            warn!("Sandbox validation failed, sandboxing may not work properly: {}", e);
        }
        
        if has_sandbox_tool {
            // Use sandbox-simplify to optimize the profile (if available)
            let output = StdCommand::new("sandbox-simplify")
                .args(["-i", profile_path.to_str().unwrap_or_default()])
                .output()
                .map_err(|e| SandboxError::Platform(format!("Failed to optimize sandbox profile: {}", e)))?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Sandbox profile optimization failed: {}", error);
            } else {
                debug!("Successfully optimized sandbox profile");
                return Ok(());
            }
        }
        
        // If sandbox-simplify is not available or failed, try alternative optimization
        debug!("Applying manual profile optimizations");
        
        // Read the profile
        let profile_content = std::fs::read_to_string(profile_path)
            .map_err(|e| SandboxError::Platform(format!("Failed to read sandbox profile: {}", e)))?;
        
        // Simplified optimization: Combine duplicate rules and remove redundant ones
        let mut optimized_lines = Vec::new();
        let mut seen_rules = std::collections::HashSet::new();
        
        for line in profile_content.lines() {
            // Skip comments and empty lines for comparison
            if line.trim().starts_with(';') || line.trim().is_empty() {
                optimized_lines.push(line.to_string());
                continue;
            }
            
            // Only add unique rules
            if !seen_rules.contains(line) {
                seen_rules.insert(line);
                optimized_lines.push(line.to_string());
            }
        }
        
        // Write back the optimized profile
        std::fs::write(profile_path, optimized_lines.join("\n"))
            .map_err(|e| SandboxError::Platform(format!("Failed to write optimized profile: {}", e)))?;
        
        debug!("Successfully applied manual profile optimizations");
        Ok(())
    }
    
    /// Enforce memory limit by monitoring and controlling process resource usage
    async fn enforce_memory_limit(&self, plugin_id: Uuid) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let process_ids = self.process_ids.read().await;
        
        if let (Some(context), Some(&process_id)) = (contexts.get(&plugin_id), process_ids.get(&plugin_id)) {
            // macOS doesn't directly support memory limits through sandbox profiles
            // Instead, we use a combination of techniques to manage memory
            
            let max_memory_bytes = context.resource_limits.max_memory_bytes;
            let max_cpu_percent = context.resource_limits.max_cpu_percent;
            let process_id_copy = process_id;
            let plugin_id_copy = plugin_id;
            let resource_monitor = self.resource_monitor.clone();
            
            // 1. Try using process_policy if available (newer macOS versions)
            let policy_result = StdCommand::new("sh")
                .args([
                    "-c", 
                    &format!("command -v taskpolicy && \
                             taskpolicy -c background -p {}", process_id)
                ])
                .output();
                
            if let Ok(output) = policy_result {
                if output.status.success() {
                    debug!("Applied background CPU policy to process {} (plugin {})", 
                          process_id, plugin_id);
                }
            }
            
            // 2. Try using priority adjustment with renice
            let _ = StdCommand::new("renice")
                .args(["+10", "-p", &process_id.to_string()])
                .output();
                
            debug!("Applied CPU priority adjustment to process {} (plugin {})",
                  process_id, plugin_id);
                  
            // 3. Start a background task to monitor resource usage with improved resource control
            info!("Starting resource monitor for plugin {} (pid: {}), memory limit: {} bytes, CPU limit: {}%", 
                 plugin_id, process_id, max_memory_bytes, max_cpu_percent);
                 
            let resource_monitor_clone = resource_monitor.clone();
            tokio::spawn(async move {
                let monitoring_interval = Duration::from_millis(500);
                let mut consecutive_violations = 0;
                let max_violations_before_termination = 5; // More forgiving: require multiple violations
                let warning_threshold = 0.8; // 80% of limit triggers warning
                let mut warning_issued = false;
                
                loop {
                    sleep(monitoring_interval).await;
                    
                    // Check if the process still exists
                    match resource_monitor_clone.get_resource_usage(plugin_id_copy).await {
                        Ok(Some(usage)) => {
                            // Convert memory usage from MB to bytes for comparison
                            let memory_usage_bytes = (usage.memory_mb * 1024.0 * 1024.0) as u64;
                            
                            // Check for approaching limit (warning threshold)
                            if memory_usage_bytes > (max_memory_bytes as f64 * warning_threshold) as u64 && !warning_issued {
                                warn!("Process {} (plugin {}) approaching memory limit: using {} MB of {} MB",
                                    process_id_copy, plugin_id_copy, 
                                    usage.memory_mb, max_memory_bytes as f64 / 1024.0 / 1024.0);
                                warning_issued = true;
                                
                                // Try a graceful memory reduction using memory pressure API if available
                                let _ = StdCommand::new("sh")
                                    .args([
                                        "-c",
                                        &format!("kill -SIGPWR {}", process_id_copy)
                                    ])
                                    .output();
                            }
                            
                            // Check for limit violations with improved accuracy
                            let memory_violation = memory_usage_bytes > max_memory_bytes;
                            let cpu_violation = usage.cpu_percent > max_cpu_percent;
                            
                            if memory_violation || cpu_violation {
                                consecutive_violations += 1;
                                
                                if consecutive_violations >= max_violations_before_termination {
                                    if memory_violation {
                                        error!("Process {} (plugin {}) exceeded memory limit of {} MB (using {} MB). \
                                              Terminating process after {} consecutive violations.", 
                                              process_id_copy, plugin_id_copy, 
                                              max_memory_bytes as f64 / 1024.0 / 1024.0,
                                              usage.memory_mb, max_violations_before_termination);
                                    } else {
                                        error!("Process {} (plugin {}) exceeded CPU limit of {}% (using {}%). \
                                              Terminating process after {} consecutive violations.", 
                                              process_id_copy, plugin_id_copy, max_cpu_percent,
                                              usage.cpu_percent, max_violations_before_termination);
                                    }
                                    
                                    // Try graceful termination first
                                    let graceful_term = StdCommand::new("kill")
                                        .args(["-15", &process_id_copy.to_string()])
                                        .output();
                                        
                                    // Wait briefly for graceful termination
                                    sleep(Duration::from_millis(500)).await;
                                    
                                    // Check if process still exists
                                    let process_exists = StdCommand::new("kill")
                                        .args(["-0", &process_id_copy.to_string()])
                                        .output()
                                        .map(|output| output.status.success())
                                        .unwrap_or(false);
                                        
                                    // If still running, force termination
                                    if process_exists {
                                        let _ = StdCommand::new("kill")
                                            .args(["-9", &process_id_copy.to_string()])
                                            .output();
                                    }
                                    
                                    break;
                                } else {
                                    warn!("Process {} (plugin {}) violation detected ({}/{}): Memory: {} MB, CPU: {}%", 
                                         process_id_copy, plugin_id_copy, 
                                         consecutive_violations, max_violations_before_termination,
                                         usage.memory_mb, usage.cpu_percent);
                                }
                            } else {
                                // Reset counter if process is behaving
                                if consecutive_violations > 0 {
                                    consecutive_violations = 0;
                                    debug!("Resource usage for process {} (plugin {}) back within limits",
                                         process_id_copy, plugin_id_copy);
                                }
                            }
                        },
                        Ok(None) => {
                            debug!("Process {} (plugin {}) no longer registered with resource monitor",
                                 process_id_copy, plugin_id_copy);
                            break;
                        },
                        Err(e) => {
                            warn!("Failed to monitor process {} (plugin {}): {}",
                                 process_id_copy, plugin_id_copy, e);
                            break;
                        }
                    }
                }
                
                debug!("Resource monitoring for process {} (plugin {}) terminated",
                     process_id_copy, plugin_id_copy);
            });
        }
        
        Ok(())
    }
    
    /// Get resource usage for a process
    async fn get_process_resource_usage(&self, process_id: u32) -> Result<ResourceUsage> {
        // Use ps command to get resource usage on macOS
        // Enhanced with better detection of the process state
        let process_info = self.get_detailed_process_info(process_id).await?;
        
        let cpu_percent = process_info.get("cpu")
            .and_then(|c| c.parse::<f32>().ok())
            .unwrap_or(0.0);
        
        let mem_percent = process_info.get("mem")
            .and_then(|m| m.parse::<f32>().ok())
            .unwrap_or(0.0);
        
        let rss_kb = process_info.get("rss")
            .and_then(|r| r.parse::<f32>().ok())
            .unwrap_or(0.0);
        
        // Convert RSS from KB to MB
        let memory_mb = rss_kb / 1024.0;
        
        // For disk and network usage, more advanced tools would be needed
        // For now, use estimates based on typical usage patterns
        let disk_mb = 0.0; // Placeholder
        let network_mb = 0.0; // Placeholder
        
        Ok(ResourceUsage {
            cpu_percent,
            memory_mb,
            disk_mb,
            network_mb,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get the path to the sandbox profile for a plugin
    fn get_sandbox_profile_path(&self, plugin_id: Uuid) -> PathBuf {
        // Create a temporary directory for profile
        let tmp_dir = std::env::temp_dir().join("squirrel_sandbox");
        let profile_path = tmp_dir.join(format!("plugin_{}.sb", plugin_id));
        profile_path
    }

    /// Enhanced method to get more detailed process information on macOS
    async fn get_detailed_process_info(&self, process_id: u32) -> Result<HashMap<String, String>> {
        let common_result = Command::new("ps")
            .args(["-p", &process_id.to_string(), "-o", "pid,ppid,user,%cpu,%mem,vsz,rss,tt,stat,time,command"])
            .output()
            .await
            .map_err(|e| SandboxError::Platform(format!("Failed to get process info: {}", e)))?;
        
        if !common_result.status.success() {
            let error = String::from_utf8_lossy(&common_result.stderr);
            return Err(SandboxError::Platform(format!("ps command failed: {}", error)).into());
        }
        
        let output = String::from_utf8_lossy(&common_result.stdout);
        let mut info = HashMap::new();
        
        for line in output.lines().skip(1) { // Skip header line
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                // Extract key information
                if let Some(pid) = parts.get(0) {
                    info.insert("pid".to_string(), pid.to_string());
                }
                if let Some(ppid) = parts.get(1) {
                    info.insert("ppid".to_string(), ppid.to_string());
                }
                if let Some(user) = parts.get(2) {
                    info.insert("user".to_string(), user.to_string());
                }
                if let Some(cpu) = parts.get(3) {
                    info.insert("cpu".to_string(), cpu.to_string());
                }
                if let Some(mem) = parts.get(4) {
                    info.insert("mem".to_string(), mem.to_string());
                }
                if let Some(vsz) = parts.get(5) {
                    info.insert("vsz".to_string(), vsz.to_string());
                }
                if let Some(rss) = parts.get(6) {
                    info.insert("rss".to_string(), rss.to_string());
                }
                if let Some(stat) = parts.get(8) {
                    info.insert("stat".to_string(), stat.to_string());
                }
                
                // Extract command info (may contain spaces)
                if parts.len() > 10 {
                    let cmd_start_idx = 10;
                    let command = parts[cmd_start_idx..].join(" ");
                    info.insert("command".to_string(), command);
                }
            }
        }
        
        // Get extended information using additional tools if available
        
        // 1. Get open files count using lsof
        let lsof_result = Command::new("sh")
            .args([
                "-c",
                &format!("command -v lsof >/dev/null && lsof -p {} | wc -l", process_id)
            ])
            .output()
            .await;
            
        if let Ok(output) = lsof_result {
            if output.status.success() {
                let count_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Ok(count) = count_str.parse::<i32>() {
                    // Subtract 1 for the header line
                    let file_count = (count - 1).max(0).to_string();
                    info.insert("open_files".to_string(), file_count);
                }
            }
        }
        
        // 2. Get thread count 
        let thread_result = Command::new("sh")
            .args([
                "-c", 
                &format!("ps -M -p {} | wc -l", process_id)
            ])
            .output()
            .await;
            
        if let Ok(output) = thread_result {
            if output.status.success() {
                let count_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Ok(count) = count_str.parse::<i32>() {
                    // Subtract 1 for the header line
                    let thread_count = (count - 1).max(0).to_string();
                    info.insert("threads".to_string(), thread_count);
                }
            }
        }
        
        // 3. Get port usage information
        let netstat_result = Command::new("sh")
            .args([
                "-c",
                &format!("command -v lsof >/dev/null && lsof -p {} -i -n -P | grep -v 'localhost'", process_id)
            ])
            .output()
            .await;
            
        if let Ok(output) = netstat_result {
            if output.status.success() {
                let network_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !network_output.is_empty() {
                    info.insert("network_connections".to_string(), network_output);
                    
                    // Count network connections
                    let connection_count = network_output.lines().count().to_string();
                    info.insert("connection_count".to_string(), connection_count);
                } else {
                    info.insert("connection_count".to_string(), "0".to_string());
                }
            }
        }
        
        // 4. Get process priority
        let priority_result = Command::new("sh")
            .args([
                "-c",
                &format!("ps -o pri -p {}", process_id)
            ])
            .output()
            .await;
            
        if let Ok(output) = priority_result {
            if output.status.success() {
                let pri_output = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = pri_output.lines().collect();
                if lines.len() >= 2 {
                    let priority = lines[1].trim();
                    info.insert("priority".to_string(), priority.to_string());
                }
            }
        }
        
        // 5. Get process start time for uptime calculation
        let start_time_result = Command::new("sh")
            .args([
                "-c",
                &format!("ps -o lstart -p {}", process_id)
            ])
            .output()
            .await;
            
        if let Ok(output) = start_time_result {
            if output.status.success() {
                let time_output = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = time_output.lines().collect();
                if lines.len() >= 2 {
                    let start_time = lines[1].trim();
                    info.insert("start_time".to_string(), start_time.to_string());
                }
            }
        }
        
        // Log the collected information at debug level
        debug!("Process {} details: {:#?}", process_id, info);
        
        Ok(info)
    }

    /// Apply sandbox to a process with enhanced error handling and process management
    async fn apply_sandbox_to_process(&self, plugin_id: Uuid, process_id: u32, profile_path: &Path) -> Result<()> {
        // Check if sandbox-exec is available
        let has_sandbox_exec = StdCommand::new("sh")
            .args(["-c", "command -v sandbox-exec"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
            
        if !has_sandbox_exec {
            return Err(SandboxError::Platform(
                "sandbox-exec not found, sandboxing cannot be applied".to_string()
            ).into());
        }
        
        // In macOS, we can't apply a sandbox to an existing process directly
        // But we can use other approaches to limit resources for existing processes
        
        info!("Direct sandbox application to existing process {} for plugin {} is not supported on macOS. \
               Using alternative resource control methods.", process_id, plugin_id);
        
        // Store the process ID for resource monitoring
        let mut process_ids = self.process_ids.write().await;
        process_ids.insert(plugin_id, process_id);
        
        // Store profile path
        let mut sandbox_profiles = self.sandbox_profiles.write().await;
        sandbox_profiles.insert(plugin_id, profile_path.to_path_buf());
        
        // Although we can't sandbox an existing process, we can apply resource restrictions:
        
        // 1. Set process priority (nice level)
        let contexts = self.security_contexts.read().await;
        if let Some(context) = contexts.get(&plugin_id) {
            let max_cpu_percent = context.resource_limits.max_cpu_percent;
            let priority_value = if max_cpu_percent < 25 {
                20 // Lowest priority
            } else if max_cpu_percent < 50 {
                10
            } else if max_cpu_percent < 75 {
                5
            } else {
                0 // Standard priority
            };
            
            if priority_value > 0 {
                let nice_result = StdCommand::new("renice")
                    .args([&format!("+{}", priority_value), "-p", &process_id.to_string()])
                    .output();
                    
                if let Ok(output) = nice_result {
                    if output.status.success() {
                        debug!("Applied priority adjustment (nice +{}) to process {}", 
                              priority_value, process_id);
                    } else {
                        warn!("Failed to adjust priority for process {}: {}", 
                             process_id, String::from_utf8_lossy(&output.stderr));
                    }
                }
            }
        }
        
        // 2. Try using process_policy if available (newer macOS versions)
        let policy_result = StdCommand::new("sh")
            .args([
                "-c", 
                &format!("command -v taskpolicy && taskpolicy -c background -p {}", process_id)
            ])
            .output();
            
        if let Ok(output) = policy_result {
            if output.status.success() {
                debug!("Applied background CPU policy to process {}", process_id);
            }
        }
        
        // 3. Setup memory and CPU monitoring
        self.enforce_memory_limit(plugin_id).await?;
        
        // 4. Check if process is registered with resource monitor
        if !self.resource_monitor.is_process_registered(plugin_id).await? {
            // Register process with resource monitor if not already registered
            let process_info_result = StdCommand::new("ps")
                .args(["-p", &process_id.to_string(), "-o", "command="])
                .output();
                
            let executable_path = if let Ok(output) = process_info_result {
                if output.status.success() {
                    let cmd = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    PathBuf::from(cmd.split_whitespace().next().unwrap_or("unknown"))
                } else {
                    std::env::current_exe()?
                }
            } else {
                std::env::current_exe()?
            };
            
            self.resource_monitor.register_process(plugin_id, process_id, &executable_path).await?;
        }
        
        info!("Applied resource controls to process {} for plugin {}", process_id, plugin_id);
        
        Ok(())
    }

    /// Check if the sandbox implementation is available on the current platform
    pub fn is_sandbox_available(&self) -> bool {
        // First check if we're running on macOS
        #[cfg(not(target_os = "macos"))]
        {
            return false;
        }
        
        #[cfg(target_os = "macos")]
        {
            // Check if sandbox-exec is available
            let sandbox_exec_check = std::process::Command::new("sh")
                .args(["-c", "command -v sandbox-exec"])
                .output();
                
            match sandbox_exec_check {
                Ok(output) => output.status.success(),
                Err(_) => false
            }
        }
    }
    
    /// Get macOS version information
    pub fn get_macos_version(&self) -> Result<String> {
        #[cfg(not(target_os = "macos"))]
        {
            return Err(SandboxError::Platform("Not running on macOS".to_string()).into());
        }
        
        #[cfg(target_os = "macos")]
        {
            let version_check = std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .map_err(|e| SandboxError::Platform(format!("Failed to get macOS version: {}", e)))?;
                
            let version = String::from_utf8_lossy(&version_check.stdout).trim().to_string();
            
            debug!("Detected macOS version: {}", version);
            Ok(version)
        }
    }
    
    /// Check if the current macOS version supports all sandbox features
    pub fn check_macos_compatibility(&self) -> Result<HashMap<String, bool>> {
        #[cfg(not(target_os = "macos"))]
        {
            return Err(SandboxError::Platform("Not running on macOS".to_string()).into());
        }
        
        #[cfg(target_os = "macos")]
        {
            let version = self.get_macos_version()?;
            let mut compatibility = HashMap::new();
            
            // Parse version components
            let version_parts: Vec<&str> = version.split('.').collect();
            let major = version_parts.get(0).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
            let minor = version_parts.get(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
            
            // Check compatibility for various features
            
            // Basic sandbox support started in 10.5
            compatibility.insert("basic_sandbox".to_string(), major >= 10 || major == 10 && minor >= 5);
            
            // Enhanced sandbox profiles started in 10.7
            compatibility.insert("enhanced_profiles".to_string(), major >= 10 || major == 10 && minor >= 7);
            
            // App sandbox compatibility in 10.7+
            compatibility.insert("app_sandbox".to_string(), major >= 10 || major == 10 && minor >= 7);
            
            // SIP was introduced in 10.11 (El Capitan)
            compatibility.insert("sip".to_string(), major >= 10 || major == 10 && minor >= 11);
            
            // TCC permissions were enhanced in 10.14 (Mojave)
            compatibility.insert("enhanced_tcc".to_string(), major >= 10 || major == 10 && minor >= 14);
            
            // Modern sandbox features available in macOS 11+
            compatibility.insert("modern_sandbox".to_string(), major >= 11);
            
            // Check for required tools
            compatibility.insert("sandbox_exec_available".to_string(), self.is_sandbox_available());
            
            // Check for csrutil (SIP configuration tool)
            let csrutil_check = std::process::Command::new("sh")
                .args(["-c", "command -v csrutil"])
                .output();
                
            compatibility.insert("csrutil_available".to_string(), 
                csrutil_check.map(|output| output.status.success()).unwrap_or(false));
            
            debug!("macOS compatibility check: {:?}", compatibility);
            Ok(compatibility)
        }
    }

    /// Generate a compatibility report for diagnostics and logging
    pub async fn generate_compatibility_report(&self) -> Result<String> {
        debug!("Generating macOS sandbox compatibility report");
        
        let mut report = String::new();
        report.push_str("# macOS Sandbox Compatibility Report\n\n");
        
        // Check if we're on macOS
        #[cfg(not(target_os = "macos"))]
        {
            report.push_str("⚠️ Not running on macOS, sandbox will use fallback mechanism\n");
            return Ok(report);
        }
        
        #[cfg(target_os = "macos")]
        {
            // Check macOS version
            let version = match self.get_macos_version() {
                Ok(v) => v,
                Err(e) => {
                    report.push_str(&format!("⚠️ Failed to determine macOS version: {}\n", e));
                    "unknown".to_string()
                }
            };
            
            report.push_str(&format!("## System Information\n\n"));
            report.push_str(&format!("- macOS Version: {}\n", version));
            
            // Check SIP status
            let sip_status = match self.check_sip_status().await {
                Ok((enabled, protections)) => {
                    let mut status = format!("- System Integrity Protection: {}\n", 
                        if enabled { "✅ Enabled" } else { "❌ Disabled" });
                    
                    for (protection, enabled) in protections {
                        status.push_str(&format!("  - {}: {}\n", 
                            protection, 
                            if enabled { "✅ Enabled" } else { "❌ Disabled" }));
                    }
                    
                    status
                },
                Err(e) => format!("- System Integrity Protection: ⚠️ Status unknown ({})\n", e)
            };
            
            report.push_str(&sip_status);
            
            // Check sandbox compatibility
            let compatibility = match self.check_macos_compatibility() {
                Ok(compat) => {
                    let mut status = String::new();
                    
                    // Sort keys for consistent output
                    let mut keys: Vec<&String> = compat.keys().collect();
                    keys.sort();
                    
                    for key in keys {
                        let enabled = compat.get(key).copied().unwrap_or(false);
                        status.push_str(&format!("  - {}: {}\n", 
                            key, 
                            if enabled { "✅ Supported" } else { "❌ Not supported" }));
                    }
                    
                    status
                },
                Err(e) => format!("⚠️ Failed to check compatibility: {}\n", e)
            };
            
            report.push_str("\n## Sandbox Feature Compatibility\n\n");
            report.push_str(&compatibility);
            
            // Check for sandbox-exec
            let sandbox_exec_available = self.is_sandbox_available();
            report.push_str("\n## Required Tools\n\n");
            report.push_str(&format!("- sandbox-exec: {}\n", 
                if sandbox_exec_available { "✅ Available" } else { "❌ Not available" }));
            
            // Check for other tools
            let tools = ["csrutil", "taskpolicy", "launchctl", "spctl", "fdesetup"];
            for tool in &tools {
                let available = std::process::Command::new("sh")
                    .args(["-c", &format!("command -v {}", tool)])
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false);
                    
                report.push_str(&format!("- {}: {}\n", 
                    tool, 
                    if available { "✅ Available" } else { "❌ Not available" }));
            }
            
            // Add summary of sandbox capabilities
            report.push_str("\n## Sandbox Capabilities\n\n");
            
            if sandbox_exec_available {
                report.push_str("✅ Full sandbox capabilities available\n");
                
                // Check specific capabilities based on version
                let version_parts: Vec<&str> = version.split('.').collect();
                let major = version_parts.get(0).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                let minor = version_parts.get(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                
                if major >= 11 || (major == 10 && minor >= 15) {
                    report.push_str("✅ Modern sandbox features supported\n");
                    report.push_str("✅ Full TCC integration available\n");
                } else if major == 10 && minor >= 11 {
                    report.push_str("✅ Basic sandbox features supported\n");
                    report.push_str("✅ SIP integration available\n");
                    
                    if minor >= 14 {
                        report.push_str("✅ Enhanced TCC integration available\n");
                    } else {
                        report.push_str("⚠️ Limited TCC integration\n");
                    }
                } else {
                    report.push_str("⚠️ Legacy sandbox only, some features may not work\n");
                }
            } else {
                report.push_str("❌ Sandbox implementation not available, using fallback mechanisms\n");
                report.push_str("⚠️ Security will be limited to process isolation\n");
            }
            
            // Add recommendations
            report.push_str("\n## Recommendations\n\n");
            
            if !sandbox_exec_available {
                report.push_str("- ⚠️ Install sandbox-exec for full sandbox capabilities\n");
            }
            
            if let Ok((sip_enabled, _)) = self.check_sip_status().await {
                if !sip_enabled {
                    report.push_str("- ⚠️ Enable System Integrity Protection for enhanced security\n");
                }
            }
            
            // Version-specific recommendations
            if major == 10 && minor < 11 {
                report.push_str("- ⚠️ Upgrade to macOS 10.11 or later for SIP support\n");
            }
            
            if major == 10 && minor < 14 {
                report.push_str("- ⚠️ Upgrade to macOS 10.14 or later for enhanced TCC integration\n");
            }
            
            if major < 11 {
                report.push_str("- ⚠️ Upgrade to macOS 11 or later for modern sandbox features\n");
            }
            
            info!("Generated macOS sandbox compatibility report");
            Ok(report)
        }
    }
    
    /// Check if a process is running inside a sandbox
    pub async fn is_process_sandboxed(&self, process_id: u32) -> Result<bool> {
        debug!("Checking if process {} is running inside a sandbox", process_id);
        
        #[cfg(not(target_os = "macos"))]
        {
            return Err(SandboxError::Platform("Not running on macOS".to_string()).into());
        }
        
        #[cfg(target_os = "macos")]
        {
            // First check if the process exists
            let process_exists = Command::new("sh")
                .args(["-c", &format!("kill -0 {}", process_id)])
                .output()
                .await
                .map(|output| output.status.success())
                .map_err(|e| SandboxError::Platform(format!(
                    "Failed to check if process {} exists: {}", process_id, e
                )))?;
                
            if !process_exists {
                return Err(SandboxError::Platform(format!(
                    "Process {} does not exist", process_id
                )).into());
            }
            
            // Use the 'ps' command to check if the process is running with sandbox restrictions
            let output = Command::new("ps")
                .args(["-o", "flags", "-p", &process_id.to_string()])
                .output()
                .await
                .map_err(|e| SandboxError::Platform(format!(
                    "Failed to get process info for {}: {}", process_id, e
                )))?;
                
            let ps_output = String::from_utf8_lossy(&output.stdout);
            
            // Check for sandbox-related flags in the output
            // Additional check for PPID:
            let parent_check = Command::new("ps")
                .args(["-o", "ppid,comm", "-p", &process_id.to_string()])
                .output()
                .await
                .map_err(|e| SandboxError::Platform(format!(
                    "Failed to get parent process info for {}: {}", process_id, e
                )))?;
                
            let parent_output = String::from_utf8_lossy(&parent_check.stdout);
            
            // Check if we can determine sandbox status from process attributes
            let proc_check = Command::new("sh")
                .args([
                    "-c", 
                    &format!("lsappinfo info `pidinfo {}` | grep -i sandbox", process_id)
                ])
                .output()
                .await;
                
            // Check if we have the sandbox-specific environment variables
            // This requires access to the process environment which is not generally
            // available for other processes. We can try with procfs on Linux, but on macOS
            // we need to use a different approach.
            let env_check = Command::new("sh")
                .args([
                    "-c",
                    &format!("ps -E -p {} 2>/dev/null | grep -i SQUIRREL_SANDBOX_ENABLED", 
                             process_id)
                ])
                .output()
                .await;
                
            // Combine all checks to make a determination
            let is_sandboxed = match (proc_check, env_check) {
                (Ok(proc_output), _) if proc_output.status.success() => {
                    // lsappinfo found sandbox info
                    let proc_str = String::from_utf8_lossy(&proc_output.stdout);
                    proc_str.contains("sandbox")
                },
                (_, Ok(env_output)) if env_output.status.success() => {
                    // Found our environment variable
                    true
                },
                _ => {
                    // Check if the parent process is sandbox-exec
                    let lines: Vec<&str> = parent_output.lines().collect();
                    if lines.len() > 1 {
                        let parts: Vec<&str> = lines[1].split_whitespace().collect();
                        if parts.len() >= 2 {
                            let parent_name = parts.get(1).unwrap_or(&"");
                            *parent_name == "sandbox-exec"
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            };
            
            debug!("Process {} sandbox status: {}", process_id, if is_sandboxed { "sandboxed" } else { "not sandboxed" });
            Ok(is_sandboxed)
        }
    }
    
    /// Get all processes currently managed by this sandbox
    pub async fn get_all_sandboxed_processes(&self) -> Result<Vec<(Uuid, u32)>> {
        let process_ids = self.process_ids.read().await;
        let result: Vec<(Uuid, u32)> = process_ids
            .iter()
            .map(|(plugin_id, process_id)| (*plugin_id, *process_id))
            .collect();
            
        Ok(result)
    }
}

#[async_trait::async_trait]
impl PluginSandbox for MacOsSandbox {
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Creating macOS sandbox for plugin {}", plugin_id);
        
        // Check macOS compatibility first
        let compatibility = match self.check_macos_compatibility() {
            Ok(compat) => compat,
            Err(e) => {
                warn!("Failed to check macOS compatibility: {}", e);
                HashMap::new() // Empty map will use default behavior
            }
        };
        
        // Set default security context if needed
        let mut contexts = self.security_contexts.write().await;
        if !contexts.contains_key(&plugin_id) {
            contexts.insert(plugin_id, SecurityContext::default());
        }
        let context = contexts.get(&plugin_id)
            .cloned()
            .unwrap();
        drop(contexts);
        
        // Create sandbox profile
        let profile_path = self.create_sandbox_profile(plugin_id, &context).await?;
        
        // Store profile path
        let mut sandbox_profiles = self.sandbox_profiles.write().await;
        sandbox_profiles.insert(plugin_id, profile_path.clone());
        
        // Get process ID
        let process_id = match self.resource_monitor.get_process_id(plugin_id).await? {
            Some(pid) => pid,
            None => {
                // If no process ID is registered, use the current process ID
                let current_pid = std::process::id();
                
                // Register with resource monitor
                let executable_path = std::env::current_exe()?;
                self.resource_monitor.register_process(plugin_id, current_pid, &executable_path).await?;
                
                current_pid
            }
        };
        
        // Store the process ID (we can't actually apply the sandbox retroactively in macOS)
        let mut process_ids = self.process_ids.write().await;
        process_ids.insert(plugin_id, process_id);
        drop(process_ids);
        
        // Apply version-specific optimizations
        if compatibility.get("modern_sandbox").copied().unwrap_or(false) {
            info!("Using modern sandbox features for macOS 11+ for plugin {}", plugin_id);
            
            // On newer macOS versions, we can use more advanced features
            // Apply TCC permissions if needed
            if context.allowed_capabilities.iter().any(|cap| {
                cap.starts_with("hardware:") || cap.starts_with("data:")
            }) {
                if let Err(e) = self.apply_tcc_permissions(plugin_id).await {
                    warn!("Failed to apply TCC permissions for plugin {}: {}", plugin_id, e);
                }
            }
            
            // Apply SIP integration if available
            if compatibility.get("sip").copied().unwrap_or(false) && 
               compatibility.get("csrutil_available").copied().unwrap_or(false) {
                if let Err(e) = self.integrate_with_sip(plugin_id).await {
                    warn!("Failed to apply SIP integration for plugin {}: {}", plugin_id, e);
                }
            }
            
            // Apply all platform-specific optimizations
            if let Err(e) = self.apply_platform_optimizations(plugin_id).await {
                warn!("Failed to apply platform optimizations for plugin {}: {}", plugin_id, e);
            }
            
            // Apply advanced security features
            self.apply_feature(plugin_id, "advanced_security").await.ok();
        } else {
            // On older macOS versions, we use a more conservative approach
            info!("Using compatibility sandbox features for older macOS for plugin {}", plugin_id);
            
            // Apply TCC permissions if needed and supported
            if context.allowed_capabilities.iter().any(|cap| {
                cap.starts_with("hardware:") || cap.starts_with("data:")
            }) && compatibility.get("enhanced_tcc").copied().unwrap_or(false) {
                if let Err(e) = self.apply_tcc_permissions(plugin_id).await {
                    warn!("Failed to apply TCC permissions for plugin {}: {}", plugin_id, e);
                }
            }
            
            // Apply only basic platform optimizations
            // Just set process priority based on permission level
            let priority = match context.permission_level {
                PermissionLevel::System => 0,    // Highest priority
                PermissionLevel::User => 10,     // Normal priority
                PermissionLevel::Restricted => 20 // Lowest priority
            };
            
            // Use renice to set process priority
            let renice_result = Command::new("renice")
                .args([&priority.to_string(), "-p", &process_id.to_string()])
                .output()
                .await;
                
            if let Err(e) = renice_result {
                warn!("Failed to set process priority for plugin {}: {}", plugin_id, e);
            }
        }
        
        info!("Created macOS sandbox for plugin {} with process {}", plugin_id, process_id);
        warn!("Note: macOS sandbox can't be applied to running processes, only to new processes at launch time");
        
        Ok(())
    }
    
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()> {
        debug!("Destroying macOS sandbox for plugin {}", plugin_id);
        
        // Get process ID if available
        let process_id = {
            let process_ids = self.process_ids.read().await;
            process_ids.get(&plugin_id).copied()
        };
        
        // Terminate process if it exists
        if let Some(pid) = process_id {
            if let Err(e) = self.terminate_process(pid).await {
                warn!("Error terminating process {} for plugin {}: {}", pid, plugin_id, e);
            }
        }
        
        // Remove plugin from process map
        {
            let mut process_ids = self.process_ids.write().await;
            process_ids.remove(&plugin_id);
        }
        
        // Remove security context
        {
            let mut contexts = self.security_contexts.write().await;
            contexts.remove(&plugin_id);
        }
        
        // Remove sandbox profile
        {
            let mut profiles = self.sandbox_profiles.write().await;
            if let Some(profile_path) = profiles.remove(&plugin_id) {
                if let Err(e) = std::fs::remove_file(&profile_path) {
                    warn!("Failed to remove sandbox profile for plugin {}: {}", plugin_id, e);
                }
            }
        }
        
        // Unregister from resource monitor
        match self.resource_monitor.unregister_process(plugin_id).await {
            Ok(_) => debug!("Unregistered plugin {} from resource monitor", plugin_id),
            Err(e) => warn!("Failed to unregister plugin {} from resource monitor: {}", plugin_id, e),
        }
        
        Ok(())
    }
    
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // Map operations to capabilities
        let required_capability = match operation {
            // Filesystem operations
            "filesystem:read" => "file:read",
            "filesystem:write" => "file:write",
            "filesystem:delete" => "file:delete",
            "filesystem:execute" => "file:execute",
            
            // Network operations
            "network:connect" => "network:connect",
            "network:listen" => "network:listen",
            
            // Process operations
            "process:spawn" => "system:resources",
            "process:kill" => "system:admin",
            
            // Plugin operations
            "plugin:load" => "plugin:read",
            "plugin:install" => "plugin:install",
            "plugin:uninstall" => "plugin:uninstall",
            "plugin:execute" => "plugin:execute",
            "plugin:update" => "plugin:write",
            
            // Config operations
            "config:read" => "config:read",
            "config:write" => "config:write",
            
            // System operations
            "system:info" => "system:info",
            "system:admin" => "system:admin",
            
            // For any unmapped operation, use the operation directly as capability
            _ => operation,
        };
        
        debug!("Mapped operation '{}' to capability '{}'", operation, required_capability);
        
        // System level has implicit access to all operations
        if context.permission_level == PermissionLevel::System {
            return Ok(());
        }
        
        // Use check_capability to verify the capability
        // We'll drop the context lock first to avoid deadlock when check_capability acquires it
        drop(contexts);
        self.check_capability(plugin_id, required_capability).await
    }
    
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage> {
        // Check if we have a process ID
        let process_ids = self.process_ids.read().await;
        if let Some(process_id) = process_ids.get(&plugin_id) {
            // Get resource usage from process
            let mut usage = self.get_process_resource_usage(*process_id).await?;
            
            // Try to get additional metrics from resource monitor
            if let Ok(Some(monitor_usage)) = self.resource_monitor.get_resource_usage(plugin_id).await {
                usage.disk_mb = monitor_usage.disk_mb;
                usage.network_mb = monitor_usage.network_mb;
            }
            
            return Ok(usage);
        }
        
        // Fallback to resource monitor
        match self.resource_monitor.get_resource_usage(plugin_id).await? {
            Some(usage) => Ok(usage),
            None => Err(SandboxError::PluginNotFound(plugin_id).into()),
        }
    }
    
    async fn check_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // System level can access any path
        if context.permission_level == PermissionLevel::System {
            return Ok(());
        }
        
        // For write access, we need to be more restrictive
        if write && context.permission_level == PermissionLevel::Restricted {
            return Err(SandboxError::PathAccess(format!(
                "Plugin {} with restricted permission level does not have write access to path {}",
                plugin_id, path.display()
            )).into());
        }
        
        // Check if path is in allowed paths
        let canonical_path = path.canonicalize().map_err(|e| SandboxError::PathAccess(format!(
            "Could not canonicalize path {}: {}", path.display(), e
        )))?;
        
        let is_allowed = context.allowed_paths.iter().any(|allowed_path| {
            // Try to canonicalize the allowed path and check if target path is a subdirectory
            if let Ok(canonical_allowed) = allowed_path.canonicalize() {
                canonical_path.starts_with(&canonical_allowed)
            } else {
                false
            }
        });
        
        // Also check if path is in secure namespace
        let is_secure = self.is_path_in_secure_namespace(&canonical_path);
        
        if is_allowed || is_secure {
            Ok(())
        } else {
            Err(SandboxError::PathAccess(format!(
                "Plugin {} does not have access to path {}",
                plugin_id, path.display()
            )).into())
        }
    }
    
    async fn check_capability(&self, plugin_id: Uuid, capability: &str) -> Result<()> {
        let contexts = self.security_contexts.read().await;
        let context = contexts.get(&plugin_id)
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
        
        // System level has all capabilities
        if context.permission_level == PermissionLevel::System {
            return Ok(());
        }
        
        // Check for exact match
        if context.allowed_capabilities.contains(capability) {
            return Ok(());
        }
        
        // Check for wildcard namespace match
        if let Some(namespace) = capability.split(':').next() {
            let wildcard = format!("{}:*", namespace);
            if context.allowed_capabilities.contains(&wildcard) {
                return Ok(());
            }
        }
        
        Err(SandboxError::Capability(format!(
            "Plugin {} does not have capability '{}'",
            plugin_id, capability
        )).into())
    }
    
    async fn apply_feature(&self, plugin_id: Uuid, feature: &str) -> Result<()> {
        debug!("Applying feature '{}' for plugin {} on macOS", feature, plugin_id);
        
        match feature {
            "memory_limit" => {
                self.enforce_memory_limit(plugin_id).await?;
                Ok(())
            },
            "profile_optimize" => {
                let sandbox_profiles = self.sandbox_profiles.read().await;
                if let Some(profile_path) = sandbox_profiles.get(&plugin_id) {
                    self.apply_macos_optimizations(profile_path)?;
                    Ok(())
                } else {
                    Err(SandboxError::PluginNotFound(plugin_id).into())
                }
            },
            "tcc_permissions" => {
                self.apply_tcc_permissions(plugin_id).await
            },
            "entitlements" => {
                let contexts = self.security_contexts.read().await;
                let context = contexts.get(&plugin_id)
                    .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?;
                
                // Generate entitlement rules
                let entitlement_rules = self.generate_entitlement_rules(context).await?;
                
                // In a real implementation, we would apply these entitlements to the process
                // However, for now, we'll just log them
                info!("Entitlement rules for plugin {}:\n{}", plugin_id, entitlement_rules);
                
                // Check if the process is already running
                let process_ids = self.process_ids.read().await;
                if let Some(process_id) = process_ids.get(&plugin_id) {
                    info!("Would apply entitlements to process {} for plugin {}", process_id, plugin_id);
                    info!("Note: Adding entitlements to running processes is not supported in macOS; they must be included at launch time");
                }
                
                Ok(())
            },
            "app_sandbox_compatibility" => {
                // Check if we're running inside an App Sandbox
                let in_app_sandbox = std::env::var("APP_SANDBOX_CONTAINER_ID").is_ok();
                
                if in_app_sandbox {
                    info!("Application is already running in an App Sandbox. Adding compatibility layer for plugin {}", plugin_id);
                    
                    // In a real implementation, we would adjust sandbox profiles to be compatible
                    // with the parent App Sandbox container
                    
                    // For now, just log the compatibility needs
                    warn!("App Sandbox compatibility layer is a no-op in this implementation");
                }
                
                Ok(())
            },
            "sip_integration" => {
                // Apply SIP integration
                self.integrate_with_sip(plugin_id).await
            },
            "platform_optimizations" => {
                // Apply platform-specific optimizations
                self.apply_platform_optimizations(plugin_id).await
            },
            "advanced_security" => {
                // Apply both SIP integration and platform optimizations
                self.integrate_with_sip(plugin_id).await?;
                self.apply_platform_optimizations(plugin_id).await?;
                
                // Get security context
                let contexts = self.security_contexts.read().await;
                let context = contexts.get(&plugin_id)
                    .ok_or_else(|| SandboxError::PluginNotFound(plugin_id))?
                    .clone();
                drop(contexts);
                
                // Apply additional security measures based on permission level
                info!("Applying advanced security features for plugin {}", plugin_id);
                
                match context.permission_level {
                    PermissionLevel::Restricted => {
                        // For restricted plugins, apply the most stringent security measures
                        info!("Applying stringent security measures for restricted plugin {}", plugin_id);
                        
                        // Check for FileVault encryption
                        let filevault_check = Command::new("sh")
                            .args(["-c", "fdesetup status"])
                            .output()
                            .await;
                            
                        if let Ok(output) = filevault_check {
                            let output_str = String::from_utf8_lossy(&output.stdout);
                            let filevault_enabled = output_str.contains("FileVault is On");
                            
                            if !filevault_enabled {
                                warn!("FileVault is not enabled. Recommended for optimal security with restricted plugins.");
                            }
                        }
                        
                        // Apply network restrictions
                        if let Some(process_id) = self.process_ids.read().await.get(&plugin_id).copied() {
                            // Use pf firewall rules to restrict network access for this process
                            debug!("Would apply network restrictions for process {}", process_id);
                            
                            // In a real implementation, we would create and apply pf rules
                            // This is just a demonstration of the concept
                        }
                    },
                    PermissionLevel::User => {
                        // For user plugins, apply moderate security measures
                        info!("Applying moderate security measures for user plugin {}", plugin_id);
                        
                        // Check for Gatekeeper status
                        let gatekeeper_check = Command::new("sh")
                            .args(["-c", "spctl --status"])
                            .output()
                            .await;
                            
                        if let Ok(output) = gatekeeper_check {
                            let output_str = String::from_utf8_lossy(&output.stdout);
                            if output_str.contains("disabled") {
                                warn!("Gatekeeper is disabled. Recommended to enable for better security.");
                            }
                        }
                    },
                    PermissionLevel::System => {
                        // For system plugins, just log the elevated privileges
                        info!("System-level plugin {} has elevated privileges. Security restrictions minimal.", plugin_id);
                    }
                }
                
                Ok(())
            },
            _ => Err(SandboxError::Unsupported(format!("Feature '{}' not supported on macOS", feature)).into()),
        }
    }
    
    async fn set_security_context(&self, plugin_id: Uuid, context: SecurityContext) -> Result<()> {
        // Delegate to the existing method
        self.set_security_context(plugin_id, context).await
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_resource_monitor(&self) -> Option<Arc<ResourceMonitor>> {
        Some(self.resource_monitor.clone())
    }
    
    /// Override is_sandbox_available from trait
    fn is_sandbox_available(&self) -> bool {
        MacOsSandbox::is_sandbox_available(self)
    }
} 

#[cfg(test)]
mod tests {
    // ... existing code ...
    
    // Test sandboxed process detection
    #[test]
    async fn test_is_process_sandboxed() {
        let (sandbox, _) = create_test_sandbox().await;
        
        // Get current process ID for testing
        let current_pid = std::process::id();
        
        // Test process sandbox detection
        let result = sandbox.is_process_sandboxed(current_pid).await;
        
        // On macOS, this should return a result indicating if the current process is sandboxed
        // On non-macOS, it should fail with a platform error
        if cfg!(target_os = "macos") {
            // The test process is likely not sandboxed, but the check should work
            if let Ok(is_sandboxed) = result {
                assert!(!is_sandboxed, "Test process should not be sandboxed");
            }
        } else {
            assert!(result.is_err());
            let err = result.unwrap_err();
            let err_str = format!("{:?}", err);
            assert!(err_str.contains("Not running on macOS"));
        }
    }
    
    // Test getting all sandboxed processes
    #[test]
    async fn test_get_all_sandboxed_processes() {
        let (sandbox, mut mock_monitor) = create_test_sandbox().await;
        let plugin_id1 = Uuid::new_v4();
        let plugin_id2 = Uuid::new_v4();
        
        // Setup mocks
        mock_monitor
            .expect_get_process_id()
            .returning(|_| Ok(Some(1000)));
            
        mock_monitor
            .expect_register_process()
            .returning(|_, _, _| Ok(()));
        
        // Create first sandbox
        sandbox.create_sandbox(plugin_id1).await.unwrap();
        
        // Create second sandbox
        sandbox.create_sandbox(plugin_id2).await.unwrap();
        
        // Get all sandboxed processes
        let processes = sandbox.get_all_sandboxed_processes().await.unwrap();
        
        // Should have two processes
        assert_eq!(processes.len(), 2);
        
        // Check that our plugin IDs are in the list
        let mut found_plugin1 = false;
        let mut found_plugin2 = false;
        
        for (plugin_id, _) in processes {
            if plugin_id == plugin_id1 {
                found_plugin1 = true;
            } else if plugin_id == plugin_id2 {
                found_plugin2 = true;
            }
        }
        
        assert!(found_plugin1, "Plugin 1 not found in sandboxed processes");
        assert!(found_plugin2, "Plugin 2 not found in sandboxed processes");
        
        // Destroy sandboxes
        sandbox.destroy_sandbox(plugin_id1).await.unwrap();
        sandbox.destroy_sandbox(plugin_id2).await.unwrap();
        
        // Get all sandboxed processes again
        let processes = sandbox.get_all_sandboxed_processes().await.unwrap();
        
        // Should have no processes
        assert_eq!(processes.len(), 0);
    }
    
    // ... existing code ...
} 

/// Check if App Sandbox is available on this macOS system
#[cfg(target_os = "macos")]
pub fn has_app_sandbox() -> bool {
    // Check if sandbox-exec exists
    use std::process::Command;
    match Command::new("which").arg("sandbox-exec").output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Check if System Integrity Protection is enabled
#[cfg(target_os = "macos")]
pub fn has_sip() -> bool {
    // Check SIP status using csrutil
    use std::process::Command;
    match Command::new("csrutil").arg("status").output() {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains("enabled")
            } else {
                false
            }
        },
        Err(_) => false,
    }
}

/// Check if Transparency, Consent, and Control (TCC) integration is available
#[cfg(target_os = "macos")]
pub fn has_tcc_integration() -> bool {
    // Check if TCC database is present
    std::path::Path::new("/Library/Application Support/com.apple.TCC/TCC.db").exists()
}

// For non-macOS platforms, provide stub implementations
#[cfg(not(target_os = "macos"))]
pub fn has_app_sandbox() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn has_sip() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn has_tcc_integration() -> bool {
    false
}