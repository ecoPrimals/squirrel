// Protocol Versioning for MCP Plugin System
//
// This module implements protocol versioning for MCP plugins, ensuring
// compatibility between different components and managing version constraints.

use anyhow::{Result, anyhow};
use semver::{Version, VersionReq};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fmt;

/// Protocol version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Major version (incompatible changes)
    pub major: u64,
    /// Minor version (compatible features)
    pub minor: u64,
    /// Patch version (bug fixes)
    pub patch: u64,
    /// Optional pre-release identifier (e.g., "alpha.1")
    pub pre_release: Option<String>,
    /// Optional build metadata
    pub build: Option<String>,
}

impl ProtocolVersion {
    /// Create a new protocol version
    #[must_use] pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build: None,
        }
    }
    
    /// Create a new protocol version with pre-release information
    pub fn with_pre_release(
        major: u64,
        minor: u64,
        patch: u64,
        pre_release: impl Into<String>
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release.into()),
            build: None,
        }
    }
    
    /// Convert to a semver Version
    ///
    /// # Errors
    ///
    /// Returns an error if the version string cannot be parsed as a valid semver version.
    /// This could happen if the pre-release or build metadata contains invalid characters.
    pub fn to_semver(&self) -> Result<Version> {
        let version_str = match (&self.pre_release, &self.build) {
            (Some(pre), Some(build)) => format!("{}.{}.{}-{}+{}", self.major, self.minor, self.patch, pre, build),
            (Some(pre), None) => format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre),
            (None, Some(build)) => format!("{}.{}.{}+{}", self.major, self.minor, self.patch, build),
            (None, None) => format!("{}.{}.{}", self.major, self.minor, self.patch),
        };
        
        Version::parse(&version_str).map_err(|e| anyhow!("Invalid version format: {}", e))
    }
    
    /// Parse from a semver Version
    #[must_use] pub fn from_semver(version: &Version) -> Self {
        Self {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            pre_release: if version.pre.is_empty() {
                None
            } else {
                Some(version.pre.to_string())
            },
            build: if version.build.is_empty() {
                None
            } else {
                Some(version.build.to_string())
            },
        }
    }
    
    /// Check if this version is compatible with a version requirement
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version cannot be converted to a semver Version
    /// - The requirement cannot be parsed as a valid semver requirement
    pub fn is_compatible_with(&self, requirement: &VersionRequirement) -> Result<bool> {
        let version = self.to_semver()?;
        let req = requirement.to_semver_req()?;
        
        Ok(req.matches(&version))
    }
}

// Implement Display for ProtocolVersion
impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.pre_release, &self.build) {
            (Some(pre), Some(build)) => write!(f, "{}.{}.{}-{}+{}", self.major, self.minor, self.patch, pre, build),
            (Some(pre), None) => write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre),
            (None, Some(build)) => write!(f, "{}.{}.{}+{}", self.major, self.minor, self.patch, build),
            (None, None) => write!(f, "{}.{}.{}", self.major, self.minor, self.patch),
        }
    }
}

/// Protocol version requirement 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRequirement {
    /// Requirement string in semver format (e.g., ">=1.0.0, <2.0.0")
    pub requirement: String,
}

impl VersionRequirement {
    /// Create a new version requirement
    pub fn new(requirement: impl Into<String>) -> Self {
        Self {
            requirement: requirement.into(),
        }
    }
    
    /// Convert to a semver `VersionReq`
    ///
    /// # Errors
    ///
    /// Returns an error if the requirement string cannot be parsed as a valid semver
    /// requirement. This can happen if the syntax is incorrect or contains invalid 
    /// operators or version numbers.
    pub fn to_semver_req(&self) -> Result<VersionReq> {
        VersionReq::parse(&self.requirement)
            .map_err(|e| anyhow!("Invalid version requirement format: {}", e))
    }
}

/// Protocol versioning manager for MCP plugins
#[derive(Debug, Clone)]
pub struct ProtocolVersionManager {
    /// Current protocol version 
    current_version: ProtocolVersion,
    
    /// Minimum supported version
    min_supported_version: ProtocolVersion,
    
    /// Plugin version requirements
    plugin_requirements: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<uuid::Uuid, VersionRequirement>>>,
}

impl ProtocolVersionManager {
    /// Create a new protocol version manager
    #[must_use] pub fn new(current: ProtocolVersion, min_supported: ProtocolVersion) -> Self {
        Self {
            current_version: current,
            min_supported_version: min_supported,
            plugin_requirements: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// Get the current protocol version
    #[must_use] pub const fn current_version(&self) -> &ProtocolVersion {
        &self.current_version
    }
    
    /// Get the minimum supported protocol version
    #[must_use] pub const fn min_supported_version(&self) -> &ProtocolVersion {
        &self.min_supported_version
    }
    
    /// Register a plugin with version requirements
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The current protocol version is not compatible with the plugin's requirements
    /// - There is an error comparing versions (invalid format)
    pub async fn register_plugin(&self, plugin_id: uuid::Uuid, requirement: VersionRequirement) -> Result<()> {
        // Verify compatibility
        if !self.current_version.is_compatible_with(&requirement)? {
            return Err(anyhow!(
                "Plugin {} requires protocol version {}, but current version is {}.{}.{}",
                plugin_id,
                requirement.requirement,
                self.current_version.major,
                self.current_version.minor,
                self.current_version.patch
            ));
        }
        
        // Store requirement - directly insert without holding lock longer than necessary
        self.plugin_requirements.write().await.insert(plugin_id, requirement);
        
        Ok(())
    }
    
    /// Check if a plugin is compatible with the current protocol version
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - There is an error comparing versions (invalid format)
    /// - The version requirement cannot be parsed
    pub async fn is_plugin_compatible(&self, plugin_id: uuid::Uuid) -> Result<bool> {
        let requirements = self.plugin_requirements.read().await;
        
        requirements.get(&plugin_id).map_or_else(
            || Ok(true), // If no requirements are specified, assume compatible
            |req| self.current_version.is_compatible_with(req)
        )
    }
    
    /// Get all plugins compatible with a specific version
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - There is an error checking compatibility for any plugin
    /// - The version requirement cannot be parsed
    pub async fn get_compatible_plugins(&self, version: &ProtocolVersion) -> Result<Vec<uuid::Uuid>> {
        // Create a clone of all requirements with a short-lived lock
        let requirements = {
            let requirements_guard = self.plugin_requirements.read().await;
            requirements_guard.clone()
        };
        
        let mut compatible_plugins = Vec::new();
        
        for (plugin_id, req) in &requirements {
            if version.is_compatible_with(req)? {
                compatible_plugins.push(*plugin_id);
            }
        }
        
        Ok(compatible_plugins)
    }
    
    /// Check version compatibility from message
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The message does not contain a `protocol_version` field
    /// - The `protocol_version` has an invalid format
    /// - There is an error converting versions to semver format
    pub fn check_message_compatibility(&self, message: &Value) -> Result<bool> {
        // Extract version from message
        let version_value = message.get("protocol_version")
            .ok_or_else(|| anyhow!("Missing protocol_version in message"))?;
            
        let version: ProtocolVersion = serde_json::from_value(version_value.clone())
            .map_err(|e| anyhow!("Invalid protocol version format: {}", e))?;
            
        // Check against min supported
        let min_semver = self.min_supported_version.to_semver()?;
        let version_semver = version.to_semver()?;
        
        if version_semver < min_semver {
            return Ok(false);
        }
        
        // Check against current version
        let current_semver = self.current_version.to_semver()?;
        
        // Allow messages from same major version or older
        Ok(version_semver.major == current_semver.major)
    }
    
    /// Add version information to a message
    #[must_use] pub fn add_version_to_message(&self, mut message: Value) -> Value {
        let version_value = serde_json::to_value(&self.current_version).unwrap_or_default();
        
        if let Some(obj) = message.as_object_mut() {
            obj.insert("protocol_version".to_string(), version_value);
            message
        } else {
            // If not an object, create a new object with the version
            serde_json::json!({
                "protocol_version": self.current_version,
                "original_message": message
            })
        }
    }
    
    /// Check if the current version is compatible with a requirement
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - There is an error converting versions to semver format
    /// - The semver requirement cannot be parsed
    pub fn is_compatible_with_requirement(&self, requirement: &VersionRequirement) -> Result<bool> {
        self.current_version.is_compatible_with(requirement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_version_semver() {
        let version = ProtocolVersion::new(1, 2, 3);
        let semver = version.to_semver().unwrap();
        
        assert_eq!(semver.major, 1);
        assert_eq!(semver.minor, 2);
        assert_eq!(semver.patch, 3);
        
        let version2 = ProtocolVersion::from_semver(&semver);
        assert_eq!(version2.major, 1);
        assert_eq!(version2.minor, 2);
        assert_eq!(version2.patch, 3);
    }
    
    #[test]
    fn test_version_compatibility() {
        let version = ProtocolVersion::new(1, 2, 3);
        let req = VersionRequirement::new(">=1.0.0, <2.0.0");
        
        assert!(version.is_compatible_with(&req).unwrap());
        
        let req2 = VersionRequirement::new(">=2.0.0");
        assert!(!version.is_compatible_with(&req2).unwrap());
    }
    
    #[tokio::test]
    async fn test_protocol_version_manager() {
        let manager = ProtocolVersionManager::new(
            ProtocolVersion::new(1, 2, 0),
            ProtocolVersion::new(1, 0, 0)
        );
        
        let plugin_id = uuid::Uuid::new_v4();
        manager.register_plugin(plugin_id, VersionRequirement::new(">=1.0.0, <2.0.0")).await.unwrap();
        
        assert!(manager.is_plugin_compatible(plugin_id).await.unwrap());
        
        let compatible = manager.get_compatible_plugins(&ProtocolVersion::new(1, 1, 0)).await.unwrap();
        assert!(compatible.contains(&plugin_id));
    }
    
    #[test]
    fn test_message_versioning() {
        let manager = ProtocolVersionManager::new(
            ProtocolVersion::new(1, 2, 0),
            ProtocolVersion::new(1, 0, 0)
        );
        
        let message = serde_json::json!({
            "command": "test"
        });
        
        let versioned = manager.add_version_to_message(message);
        assert!(versioned.get("protocol_version").is_some());
        
        let compatible = manager.check_message_compatibility(&versioned).unwrap();
        assert!(compatible);
        
        let incompatible = serde_json::json!({
            "protocol_version": {
                "major": 2,
                "minor": 0,
                "patch": 0,
                "pre_release": null,
                "build": null
            }
        });
        
        let result = manager.check_message_compatibility(&incompatible).unwrap();
        assert!(!result);
    }
    
    #[test]
    fn test_protocol_version_display() {
        let version = ProtocolVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
        
        let version_pre = ProtocolVersion::with_pre_release(1, 2, 3, "alpha.1");
        assert_eq!(version_pre.to_string(), "1.2.3-alpha.1");
    }
    
    #[test]
    fn test_version_manager_compatibility_check() {
        let manager = ProtocolVersionManager::new(
            ProtocolVersion::new(1, 5, 0),
            ProtocolVersion::new(1, 0, 0)
        );
        
        let req1 = VersionRequirement::new(">=1.0.0, <2.0.0");
        assert!(manager.is_compatible_with_requirement(&req1).unwrap());
        
        let req2 = VersionRequirement::new(">=2.0.0");
        assert!(!manager.is_compatible_with_requirement(&req2).unwrap());
    }
} 