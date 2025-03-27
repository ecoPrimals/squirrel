//! Asset management for the Squirrel Web UI.
//!
//! This module provides functionality for managing static assets.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Include the generated assets file
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// Asset manager
#[derive(Debug)]
pub struct AssetManager {
    /// Asset directories
    asset_dirs: Vec<PathBuf>,
    /// Asset cache
    cache: HashMap<String, Vec<u8>>,
}

impl AssetManager {
    /// Create a new asset manager
    pub fn new() -> Self {
        Self {
            asset_dirs: Vec::new(),
            cache: HashMap::new(),
        }
    }
    
    /// Add an asset directory
    pub fn add_asset_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.asset_dirs.push(path.as_ref().to_path_buf());
    }
    
    /// Get an asset by path
    pub fn get_asset(&mut self, path: &str) -> Option<&[u8]> {
        if let Some(data) = self.cache.get(path) {
            return Some(data);
        }
        
        for dir in &self.asset_dirs {
            let asset_path = dir.join(path);
            if asset_path.exists() {
                match std::fs::read(&asset_path) {
                    Ok(data) => {
                        self.cache.insert(path.to_string(), data);
                        return self.cache.get(path).map(|d| d.as_slice());
                    }
                    Err(_) => continue,
                }
            }
        }
        
        None
    }
    
    /// Get the MIME type for an asset
    pub fn get_mime_type(&self, path: &str) -> &'static str {
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match extension {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "otf" => "font/otf",
            "eot" => "application/vnd.ms-fontobject",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "ogg" => "audio/ogg",
            "pdf" => "application/pdf",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            "txt" => "text/plain",
            "xml" => "application/xml",
            _ => "application/octet-stream",
        }
    }
    
    /// Load all assets from a directory
    pub fn preload_assets(&mut self, directory: &str) -> Result<(), std::io::Error> {
        for dir in &self.asset_dirs {
            let assets_dir = dir.join(directory);
            if assets_dir.exists() && assets_dir.is_dir() {
                self.preload_directory(&assets_dir, directory)?;
            }
        }
        
        Ok(())
    }
    
    /// Preload assets from a directory recursively
    fn preload_directory(&mut self, dir: &Path, prefix: &str) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let new_prefix = format!("{}/{}", prefix, path.file_name().unwrap().to_string_lossy());
                self.preload_directory(&path, &new_prefix)?;
            } else {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let asset_path = format!("{}/{}", prefix, file_name);
                let data = std::fs::read(&path)?;
                self.cache.insert(asset_path, data);
            }
        }
        
        Ok(())
    }
    
    /// Clear the asset cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Get list of all available assets
    pub fn list_assets(&self) -> Vec<String> {
        WEB_ASSETS.iter().map(|s| s.to_string()).collect()
    }
    
    /// Preload all available assets
    pub fn preload_all(&mut self) -> Result<(), std::io::Error> {
        for asset in WEB_ASSETS {
            for dir in &self.asset_dirs {
                let asset_path = dir.join(asset);
                if asset_path.exists() {
                    let data = std::fs::read(&asset_path)?;
                    self.cache.insert(asset.to_string(), data);
                    break;
                }
            }
        }
        
        Ok(())
    }
}

/// Default asset directories for the Squirrel Web UI
pub fn default_asset_directories() -> Vec<PathBuf> {
    vec![
        PathBuf::from("./web/assets"),
        PathBuf::from("./web/css"),
        PathBuf::from("./web/js"),
        PathBuf::from("./web"),
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/web/assets")),
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/web/css")),
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/web/js")),
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/web")),
        PathBuf::from(concat!(env!("OUT_DIR"), "/web")),
    ]
}

/// Create a new asset manager with default directories
pub fn create_asset_manager() -> AssetManager {
    let mut manager = AssetManager::new();
    for dir in default_asset_directories() {
        manager.add_asset_directory(dir);
    }
    manager
} 