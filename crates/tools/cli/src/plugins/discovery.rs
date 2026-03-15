// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && Self::is_plugin_library(&path) {
                    plugin_paths.push(path);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_discover_nonexistent_dir() {
        let discovery = DefaultPluginDiscovery;
        let result = discovery.discover(Path::new("/nonexistent/path"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_discover_empty_dir() {
        let dir = TempDir::new().expect("create temp dir");
        let discovery = DefaultPluginDiscovery;
        let result = discovery.discover(dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_discover_dir_with_non_plugin_files() {
        let dir = TempDir::new().expect("create temp dir");
        std::fs::write(dir.path().join("readme.txt"), "hello").expect("write");
        std::fs::write(dir.path().join("config.toml"), "key = 1").expect("write");

        let discovery = DefaultPluginDiscovery;
        let result = discovery.discover(dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_discover_dir_with_so_files() {
        let dir = TempDir::new().expect("create temp dir");
        std::fs::write(dir.path().join("plugin.so"), b"ELF").expect("write");

        let discovery = DefaultPluginDiscovery;
        let result = discovery.discover(dir.path());
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].to_string_lossy().contains("plugin.so"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_is_plugin_library_linux() {
        assert!(DefaultPluginDiscovery::is_plugin_library(Path::new(
            "lib.so"
        )));
        assert!(!DefaultPluginDiscovery::is_plugin_library(Path::new(
            "lib.txt"
        )));
        assert!(!DefaultPluginDiscovery::is_plugin_library(Path::new(
            "noext"
        )));
    }
}
