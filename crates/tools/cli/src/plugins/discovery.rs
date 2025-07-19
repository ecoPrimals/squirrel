use crate::plugins::error::PluginError;
use std::path::{Path, PathBuf};

pub trait PluginDiscovery {
    fn discover(&self, plugin_dir: &Path) -> Result<Vec<PathBuf>, PluginError>;
}

pub struct DefaultPluginDiscovery;

impl PluginDiscovery for DefaultPluginDiscovery {
    fn discover(&self, plugin_dir: &Path) -> Result<Vec<PathBuf>, PluginError> {
        // Default implementation
        let mut plugin_paths = Vec::new();
        if !plugin_dir.exists() {
            return Ok(plugin_paths);
        }

        // Find all plugin library files in the directory
        if let Ok(entries) = std::fs::read_dir(plugin_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && Self::is_plugin_library(&path) {
                        plugin_paths.push(path);
                    }
                }
            }
        }

        Ok(plugin_paths)
    }
}

impl DefaultPluginDiscovery {
    fn is_plugin_library(path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            #[cfg(target_os = "windows")]
            return extension == "dll";
            #[cfg(target_os = "linux")]
            return extension == "so";
            #[cfg(target_os = "macos")]
            return extension == "dylib";
        }
        false
    }
}
