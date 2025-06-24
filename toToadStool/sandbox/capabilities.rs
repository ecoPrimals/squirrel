//! Platform Capability Detection
//!
//! This module provides functions to detect platform-specific sandbox capabilities
//! and features available on the current system.

/// Windows capability detection
pub mod windows {
    /// Check if Windows supports integrity levels
    pub fn has_integrity_levels() -> bool {
        #[cfg(target_os = "windows")]
        {
            // This is supported on Windows Vista+
            true
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
    
    /// Check if Windows supports desktop isolation
    pub fn has_desktop_isolation() -> bool {
        #[cfg(target_os = "windows")]
        {
            // This is supported on Windows 8+
            true
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
    
    /// Check if Windows supports network isolation
    pub fn has_network_isolation() -> bool {
        #[cfg(target_os = "windows")]
        {
            // This is supported on Windows 10+
            true
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
    
    /// Check if Windows supports app containers
    pub fn has_app_container() -> bool {
        #[cfg(target_os = "windows")]
        {
            // This is supported on Windows 8+
            true
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}

/// Linux capability detection
#[cfg(target_os = "linux")]
pub mod linux {
    /// Check if Linux supports cgroups v2
    pub fn has_cgroups_v2() -> bool {
        // Placeholder for actual Linux capability check
        std::path::Path::new("/sys/fs/cgroup/cgroup.controllers").exists()
    }
    
    /// Check if Linux supports seccomp
    pub fn has_seccomp() -> bool {
        // Placeholder for actual Linux capability check
        std::path::Path::new("/proc/sys/kernel/seccomp").exists()
    }
    
    /// Check if Linux supports namespaces
    pub fn has_namespaces() -> bool {
        // Placeholder for actual Linux capability check
        std::path::Path::new("/proc/self/ns").exists()
    }
    
    /// Get available namespace types
    pub fn get_available_namespaces() -> Vec<String> {
        let mut namespaces = Vec::new();
        
        // Check for common namespace types
        let ns_types = ["user", "pid", "net", "mnt", "ipc", "uts", "cgroup"];
        
        for ns_type in &ns_types {
            let ns_path = format!("/proc/self/ns/{}", ns_type);
            if std::path::Path::new(&ns_path).exists() {
                namespaces.push(ns_type.to_string());
            }
        }
        
        namespaces
    }
}

/// macOS capability detection
#[cfg(target_os = "macos")]
pub mod macos {
    /// Check if macOS supports App Sandbox
    pub fn has_app_sandbox() -> bool {
        // Check if sandbox-exec is available
        std::process::Command::new("which")
            .arg("sandbox-exec")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// Check if macOS has System Integrity Protection (SIP)
    pub fn has_sip() -> bool {
        // SIP is available on macOS 10.11+
        std::process::Command::new("csrutil")
            .arg("status")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// Check if macOS has Transparency, Consent, and Control (TCC) integration
    pub fn has_tcc_integration() -> bool {
        // Check if TCC database exists
        let tcc_path = "/Library/Application Support/com.apple.TCC/TCC.db";
        std::path::Path::new(tcc_path).exists()
    }
}

/// Cross-platform capability detection
pub mod cross_platform {
    use std::collections::HashSet;
    
    /// Get all available capabilities for the current platform
    pub fn get_available_capabilities() -> HashSet<String> {
        let mut capabilities = HashSet::new();
        
        // Add common capabilities
        capabilities.insert("basic_isolation".to_string());
        capabilities.insert("resource_monitoring".to_string());
        capabilities.insert("path_validation".to_string());
        capabilities.insert("plugin_lifecycle".to_string());
        
        #[cfg(target_os = "windows")]
        {
            if super::windows::has_integrity_levels() {
                capabilities.insert("integrity_levels".to_string());
            }
            if super::windows::has_desktop_isolation() {
                capabilities.insert("desktop_isolation".to_string());
            }
            if super::windows::has_network_isolation() {
                capabilities.insert("network_isolation".to_string());
            }
            if super::windows::has_app_container() {
                capabilities.insert("app_container".to_string());
            }
            capabilities.insert("windows_job_objects".to_string());
        }
        
        #[cfg(target_os = "linux")]
        {
            if super::linux::has_cgroups_v2() {
                capabilities.insert("cgroups_v2".to_string());
            }
            if super::linux::has_seccomp() {
                capabilities.insert("seccomp".to_string());
            }
            if super::linux::has_namespaces() {
                capabilities.insert("namespaces".to_string());
                
                // Add specific namespace capabilities
                for ns in super::linux::get_available_namespaces() {
                    capabilities.insert(format!("namespace_{}", ns));
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if super::macos::has_app_sandbox() {
                capabilities.insert("app_sandbox".to_string());
            }
            if super::macos::has_sip() {
                capabilities.insert("system_integrity_protection".to_string());
            }
            if super::macos::has_tcc_integration() {
                capabilities.insert("transparency_consent_control".to_string());
            }
        }
        
        capabilities
    }
    
    /// Check if a specific capability is available on this platform
    pub fn has_capability(capability: &str) -> bool {
        get_available_capabilities().contains(capability)
    }
} 