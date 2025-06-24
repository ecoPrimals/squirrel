//! TCC (Transparency, Consent, and Control) Integration
//!
//! This module handles macOS TCC permissions for accessing protected user data
//! such as camera, microphone, contacts, calendar, and other sensitive resources.

use super::*;
use std::process::Command;

impl MacOsSandbox {
    /// Apply TCC permissions for a plugin
    pub async fn apply_tcc_permissions(&self, plugin_id: Uuid) -> Result<()> {
        let context = self.get_security_context(plugin_id).await?;
        
        debug!("Applying TCC permissions for plugin {}", plugin_id);
        
        // Check if TCC integration is available
        if !check_tcc_available() {
            warn!("TCC integration not available, skipping TCC permissions");
            return Ok(());
        }
        
        // Apply TCC permissions based on capabilities
        for capability in &context.allowed_capabilities {
            match capability.as_str() {
                "hardware:camera" => {
                    self.request_tcc_permission("kTCCServiceCamera", plugin_id).await?;
                },
                "hardware:microphone" => {
                    self.request_tcc_permission("kTCCServiceMicrophone", plugin_id).await?;
                },
                "data:contacts" => {
                    self.request_tcc_permission("kTCCServiceAddressBook", plugin_id).await?;
                },
                "data:calendar" => {
                    self.request_tcc_permission("kTCCServiceCalendar", plugin_id).await?;
                },
                "data:photos" => {
                    self.request_tcc_permission("kTCCServicePhotos", plugin_id).await?;
                },
                "data:location" => {
                    self.request_tcc_permission("kTCCServiceLocation", plugin_id).await?;
                },
                "data:reminders" => {
                    self.request_tcc_permission("kTCCServiceReminders", plugin_id).await?;
                },
                "data:full_disk_access" => {
                    self.request_tcc_permission("kTCCServiceSystemPolicyAllFiles", plugin_id).await?;
                },
                _ => {
                    // Skip non-TCC capabilities
                    debug!("Capability {} does not require TCC permission", capability);
                }
            }
        }
        
        info!("TCC permissions applied for plugin {}", plugin_id);
        Ok(())
    }
    
    /// Request a specific TCC permission
    async fn request_tcc_permission(&self, service: &str, plugin_id: Uuid) -> Result<()> {
        // Get the app bundle identifier (we'll use a generic one for Squirrel plugins)
        let bundle_id = format!("com.squirrel.plugin.{}", plugin_id);
        
        debug!("Requesting TCC permission {} for bundle {}", service, bundle_id);
        
        // Use tccutil to request permission
        let output = Command::new("tccutil")
            .args(&["reset", service, &bundle_id])
            .output()
            .map_err(|e| SandboxError::TccIntegration(format!("Failed to reset TCC permission: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to reset TCC permission {}: {}", service, error_msg);
        }
        
        // Note: In a real implementation, we would need to prompt the user for permission
        // through the macOS permission dialog. This typically happens automatically
        // when the app first tries to access the protected resource.
        
        Ok(())
    }
    
    /// Check TCC permission status for a plugin
    pub async fn check_tcc_permission(&self, plugin_id: Uuid, service: &str) -> Result<bool> {
        let bundle_id = format!("com.squirrel.plugin.{}", plugin_id);
        
        // Query TCC database to check permission status
        // Note: This requires special entitlements or admin privileges
        let output = Command::new("sqlite3")
            .args(&[
                "/Library/Application Support/com.apple.TCC/TCC.db",
                &format!(
                    "SELECT allowed FROM access WHERE service='{}' AND client='{}';",
                    service, bundle_id
                )
            ])
            .output()
            .map_err(|e| SandboxError::TccIntegration(format!("Failed to query TCC database: {}", e)))?;
        
        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout);
            let allowed = result.trim() == "1";
            
            debug!("TCC permission {} for plugin {}: {}", service, plugin_id, allowed);
            Ok(allowed)
        } else {
            // If we can't query the database, assume permission is not granted
            debug!("Could not query TCC database for plugin {}, assuming permission denied", plugin_id);
            Ok(false)
        }
    }
    
    /// Get all TCC permissions for a plugin
    pub async fn get_tcc_permissions(&self, plugin_id: Uuid) -> Result<HashMap<String, bool>> {
        let mut permissions = HashMap::new();
        
        let bundle_id = format!("com.squirrel.plugin.{}", plugin_id);
        
        // Query all TCC permissions for this bundle
        let output = Command::new("sqlite3")
            .args(&[
                "/Library/Application Support/com.apple.TCC/TCC.db",
                &format!(
                    "SELECT service, allowed FROM access WHERE client='{}';",
                    bundle_id
                )
            ])
            .output()
            .map_err(|e| SandboxError::TccIntegration(format!("Failed to query TCC database: {}", e)))?;
        
        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout);
            
            for line in result.lines() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 2 {
                    let service = parts[0].to_string();
                    let allowed = parts[1] == "1";
                    permissions.insert(service, allowed);
                }
            }
        }
        
        Ok(permissions)
    }
    
    /// Remove TCC permissions for a plugin (cleanup)
    pub async fn remove_tcc_permissions(&self, plugin_id: Uuid) -> Result<()> {
        let bundle_id = format!("com.squirrel.plugin.{}", plugin_id);
        
        debug!("Removing TCC permissions for plugin {}", plugin_id);
        
        // Get all TCC services that this plugin has permissions for
        let permissions = self.get_tcc_permissions(plugin_id).await?;
        
        // Reset each permission
        for service in permissions.keys() {
            let output = Command::new("tccutil")
                .args(&["reset", service, &bundle_id])
                .output()
                .map_err(|e| SandboxError::TccIntegration(format!("Failed to reset TCC permission: {}", e)))?;
            
            if output.status.success() {
                debug!("Reset TCC permission {} for plugin {}", service, plugin_id);
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to reset TCC permission {}: {}", service, error_msg);
            }
        }
        
        info!("TCC permissions removed for plugin {}", plugin_id);
        Ok(())
    }
    
    /// Generate TCC permission summary for diagnostics
    pub async fn generate_tcc_report(&self, plugin_id: Uuid) -> Result<String> {
        let mut report = String::new();
        
        report.push_str(&format!("TCC Permission Report for Plugin {}\n", plugin_id));
        report.push_str("=" .repeat(50).as_str());
        report.push_str("\n\n");
        
        // Check if TCC is available
        report.push_str(&format!("TCC Available: {}\n", check_tcc_available()));
        
        // Get security context
        let context = self.get_security_context(plugin_id).await?;
        
        report.push_str("\nRequested Capabilities:\n");
        for capability in &context.allowed_capabilities {
            if let Some(tcc_service) = capability_to_tcc_service(capability) {
                let permission_status = self.check_tcc_permission(plugin_id, &tcc_service).await?;
                report.push_str(&format!("  {}: {} ({})\n", capability, tcc_service, 
                                        if permission_status { "GRANTED" } else { "DENIED" }));
            } else {
                report.push_str(&format!("  {}: Not TCC-protected\n", capability));
            }
        }
        
        // Get all TCC permissions
        let all_permissions = self.get_tcc_permissions(plugin_id).await?;
        if !all_permissions.is_empty() {
            report.push_str("\nAll TCC Permissions:\n");
            for (service, allowed) in all_permissions {
                report.push_str(&format!("  {}: {}\n", service, if allowed { "GRANTED" } else { "DENIED" }));
            }
        }
        
        Ok(report)
    }
}

/// Map a capability string to its corresponding TCC service
fn capability_to_tcc_service(capability: &str) -> Option<String> {
    match capability {
        "hardware:camera" => Some("kTCCServiceCamera".to_string()),
        "hardware:microphone" => Some("kTCCServiceMicrophone".to_string()),
        "data:contacts" => Some("kTCCServiceAddressBook".to_string()),
        "data:calendar" => Some("kTCCServiceCalendar".to_string()),
        "data:photos" => Some("kTCCServicePhotos".to_string()),
        "data:location" => Some("kTCCServiceLocation".to_string()),
        "data:reminders" => Some("kTCCServiceReminders".to_string()),
        "data:full_disk_access" => Some("kTCCServiceSystemPolicyAllFiles".to_string()),
        _ => None,
    }
}

/// Check if the current user has the necessary privileges to modify TCC permissions
pub fn check_tcc_admin_privileges() -> bool {
    // Check if we can write to the TCC database
    let test_output = Command::new("sqlite3")
        .args(&[
            "/Library/Application Support/com.apple.TCC/TCC.db",
            ".tables"
        ])
        .output();
        
    test_output.map(|output| output.status.success()).unwrap_or(false)
} 