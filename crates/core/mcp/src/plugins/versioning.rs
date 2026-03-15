// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

// Protocol Versioning for MCP Plugin System
//
// This module implements protocol versioning for MCP plugins, ensuring
// compatibility between different components and managing version constraints.

use anyhow::{Result, anyhow};
use semver::{Version, VersionReq};
use serde::{Serialize, Deserialize};
use std::fmt;
use std::str::FromStr;

/// Protocol version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolVersion {
    /// Major version (incompatible changes)
    pub major: u64,
    /// Minor version (compatible features)
    pub minor: u64,
    /// Patch version (bug fixes)
    pub patch: u64,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Get the current stable version
    pub fn current() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }

    /// Convert to a semver::Version for comparison
    pub fn to_semver(&self) -> Version {
        Version::new(
            self.major,
            self.minor,
            self.patch
        )
    }

    /// Create from a semver::Version
    pub fn from_semver(version: &Version) -> Self {
        Self {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
        }
    }
    
    /// Check if this version is compatible with the requirement
    /// 
    /// # Errors
    /// 
    /// Returns an error if the version requirement format is invalid
    pub fn is_compatible_with(&self, req: &VersionRequirement) -> Result<bool> {
        // Validate the requirement first
        let _ = req.to_semver_req()?;
        
        // Check compatibility
        Ok(req.is_satisfied_by(self))
    }
}

// Implement Display for ProtocolVersion
impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for ProtocolVersion {
    type Err = semver::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let version = Version::parse(s)?;
        Ok(Self {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
        })
    }
}

/// Protocol version requirement 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRequirement {
    /// Requirement string in semver format (e.g., ">=1.0.0, <2.0.0")
    pub requirement: String,
    /// Parsed semver requirement
    #[serde(skip)]
    parsed: Option<VersionReq>,
}

impl VersionRequirement {
    /// Create a new version requirement
    pub fn new(requirement: &str) -> Self {
        let parsed = VersionReq::parse(requirement).ok();
        Self {
            requirement: requirement.to_string(),
            parsed,
        }
    }
    
    /// Convert to a semver::VersionReq for checking compatibility
    /// 
    /// # Errors
    /// 
    /// Returns an error if the requirement string contains invalid version
    /// operators or version numbers.
    pub fn to_semver_req(&self) -> Result<VersionReq> {
        self.parsed.as_ref().cloned().ok_or_else(|| anyhow!("Invalid version requirement format"))
    }

    /// Check if a version satisfies this requirement
    pub fn is_satisfied_by(&self, version: &ProtocolVersion) -> bool {
        if let Some(req) = &self.parsed {
            let semver_version = version.to_semver();
            req.matches(&semver_version)
        } else {
            // If we couldn't parse the requirement, assume it's satisfied
            true
        }
    }
}

impl FromStr for VersionRequirement {
    type Err = semver::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let req = VersionReq::parse(s)?;
        Ok(Self {
            requirement: s.to_string(),
            parsed: Some(req),
        })
    }
}

impl fmt::Display for VersionRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.requirement)
    }
}

/// Protocol version manager
#[derive(Debug, Clone)]
pub struct ProtocolVersionManager {
    /// Current protocol version
    current_version: ProtocolVersion,
    /// Supported protocol versions
    supported_versions: Vec<ProtocolVersion>,
}

impl ProtocolVersionManager {
    /// Create a new protocol version manager
    pub fn new(current: ProtocolVersion, supported: Vec<ProtocolVersion>) -> Self {
        Self {
            current_version: current,
            supported_versions: supported,
        }
    }

    /// Create a default protocol version manager
    pub fn default() -> Self {
        let current = ProtocolVersion::current();
        let supported = vec![current.clone()];
        Self {
            current_version: current,
            supported_versions: supported,
        }
    }

    /// Get the current protocol version
    pub fn current_version(&self) -> &ProtocolVersion {
        &self.current_version
    }

    /// Get the supported protocol versions
    pub fn supported_versions(&self) -> &[ProtocolVersion] {
        &self.supported_versions
    }

    /// Check if a version requirement is satisfied by any supported version
    pub fn is_compatible_with(&self, requirement: &VersionRequirement) -> bool {
        self.supported_versions.iter()
            .any(|version| requirement.is_satisfied_by(version))
    }

    /// Check if a version requirement is satisfied by any supported version and return a Result
    /// 
    /// # Errors
    /// 
    /// Returns an error if the version requirement format is invalid
    pub fn is_compatible_with_requirement(&self, requirement: &VersionRequirement) -> Result<bool> {
        // Ensure the requirement is valid first
        let _ = requirement.to_semver_req()?;
        
        // Then check compatibility
        Ok(self.is_compatible_with(requirement))
    }
    
    /// Check if a JSON message with version information is compatible
    /// 
    /// # Errors
    /// 
    /// Returns an error if the version information in the message is invalid
    pub fn check_message_compatibility(&self, message: &serde_json::Value) -> Result<bool> {
        // If the message doesn't have version info, assume it's compatible
        let version_value = match message.get("protocol_version") {
            Some(v) => v,
            None => return Ok(true),
        };
        
        // Try to deserialize the version
        let version: ProtocolVersion = serde_json::from_value(version_value.clone())
            .map_err(|e| anyhow!("Invalid protocol version format: {}", e))?;
        
        // Check if any of our supported versions is compatible with this version
        // For simple compatibility, we'll just check if the major version matches
        Ok(self.supported_versions.iter().any(|v| v.major == version.major))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_version_semver() {
        let version = ProtocolVersion::new(1, 2, 3);
        let semver = version.to_semver();
        
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
    
    #[test]
    fn test_protocol_version_serialization() {
        let version = ProtocolVersion::new(1, 0, 0);
        let json = serde_json::to_value(&version).unwrap();
        
        assert_eq!(json, serde_json::json!({
            "major": 1,
            "minor": 0,
            "patch": 0,
        }));
    }
    
    #[test]
    fn test_protocol_version_display() {
        let version = ProtocolVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }
    
    #[test]
    fn test_version_requirement() {
        let req = VersionRequirement::new(">=1.0.0, <2.0.0");
        let v1 = ProtocolVersion::new(1, 0, 0);
        let v2 = ProtocolVersion::new(2, 0, 0);
        
        assert!(req.is_satisfied_by(&v1));
        assert!(!req.is_satisfied_by(&v2));
    }
    
    #[test]
    fn test_protocol_version_manager() {
        let v1 = ProtocolVersion::new(1, 0, 0);
        let v2 = ProtocolVersion::new(1, 1, 0);
        let manager = ProtocolVersionManager::new(v1.clone(), vec![v1, v2]);
        
        let req1 = VersionRequirement::new(">=1.0.0, <2.0.0");
        let req2 = VersionRequirement::new(">=2.0.0");
        
        assert!(manager.is_compatible_with(&req1));
        assert!(!manager.is_compatible_with(&req2));
    }
} 