//! Linux-specific plugin sandbox implementation using cgroups v2

use std::collections::HashMap;
use std::fs::{self};
use std::io::{Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::Result;
use crate::plugin::security::SecurityContext;
use crate::plugin::resource_monitor::ResourceMonitor;
use crate::plugin::sandbox::SandboxError;

use super::config::SeccompConfig;

const CGROUP_BASE_PATH: &str = "/sys/fs/cgroup/squirrel";

/// Linux-specific plugin sandbox implementation using cgroups v2
#[derive(Debug)]
pub struct LinuxCgroupSandbox {
    /// Cgroup paths for plugins
    pub cgroup_paths: Arc<RwLock<HashMap<Uuid, PathBuf>>>,
    /// Security contexts for plugins
    pub security_contexts: Arc<RwLock<HashMap<Uuid, SecurityContext>>>,
    /// Resource monitor
    pub resource_monitor: Arc<ResourceMonitor>,
    /// Base path for cgroups
    pub base_path: PathBuf,
    /// Seccomp configurations per plugin
    pub seccomp_configs: RwLock<HashMap<Uuid, SeccompConfig>>,
}

impl LinuxCgroupSandbox {
    /// Create a new Linux cgroup sandbox
    pub fn new(resource_monitor: Arc<ResourceMonitor>) -> Result<Self> {
        // Create base path for cgroups if it doesn't exist
        let base_path = PathBuf::from(CGROUP_BASE_PATH);
        
        // Ensure cgroup filesystem is mounted
        Self::ensure_cgroup_mounted()?;
        
        // Create squirrel cgroup if it doesn't exist
        if !base_path.exists() {
            fs::create_dir_all(&base_path)
                .map_err(|e| SandboxError::Creation(format!("Failed to create cgroup directory: {}", e)))?;
        }
        
        Ok(Self {
            cgroup_paths: Arc::new(RwLock::new(HashMap::new())),
            security_contexts: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor,
            base_path,
            seccomp_configs: RwLock::new(HashMap::new()),
        })
    }
    
    /// Ensure cgroup filesystem is mounted
    fn ensure_cgroup_mounted() -> Result<()> {
        // Check if cgroup2 is mounted
        let cgroup_mount = Path::new("/sys/fs/cgroup");
        if !cgroup_mount.exists() {
            return Err(SandboxError::Platform(
                "Cgroup filesystem is not mounted at /sys/fs/cgroup".to_string()
            ).into());
        }
        
        // Check if it's cgroup v2
        let cgroup_type_path = cgroup_mount.join("cgroup.controllers");
        if !cgroup_type_path.exists() {
            return Err(SandboxError::Platform(
                "Cgroup v2 is required but not available on this system".to_string()
            ).into());
        }
        
        Ok(())
    }
    
    /// Get cgroup path for a plugin
    pub fn get_cgroup_path(&self, plugin_id: &Uuid) -> PathBuf {
        self.base_path.join(plugin_id.to_string())
    }

    // Get a security context for a plugin
    pub async fn get_security_context(&self, plugin_id: Uuid) -> Result<SecurityContext> {
        let contexts = self.security_contexts.read().await;
        contexts.get(&plugin_id)
            .cloned()
            .ok_or_else(|| SandboxError::PluginNotFound(plugin_id).into())
    }
    
    // Helper to get plugin ID from cgroup path
    pub fn get_plugin_id_from_cgroup_path(&self, cgroup_path: &Path) -> Option<Uuid> {
        if let Some(file_name) = cgroup_path.file_name() {
            if let Some(name) = file_name.to_str() {
                if let Ok(id) = Uuid::parse_str(name) {
                    return Some(id);
                }
            }
        }
        None
    }
   
    /// Check if path is inside secure namespace
    pub fn is_path_in_secure_namespace(&self, path: &Path) -> bool {
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
} 