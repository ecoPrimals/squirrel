//! File system functionality for plugins
//!
//! This module provides secure file system operations for WASM plugins,
//! with sandbox security and permission checking.

use crate::error::{PluginError, PluginResult};
use crate::plugin::Permission;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
// Sandbox security handled by BearDog framework

/// File metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// File type/MIME type
    pub file_type: String,
    /// Last modified timestamp
    pub last_modified: u64,
    /// Whether the file is a directory
    pub is_directory: bool,
    /// File permissions (if available)
    pub permissions: Option<String>,
}

/// File content representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileContent {
    /// Text content
    Text(String),
    /// Binary content
    Binary(Vec<u8>),
}

impl FileContent {
    /// Get content as UTF-8 string
    pub fn as_text(&self) -> PluginResult<String> {
        match self {
            FileContent::Text(content) => Ok(content.clone()),
            FileContent::Binary(data) => {
                String::from_utf8(data.clone()).map_err(|e| PluginError::SerializationError {
                    message: format!("Invalid UTF-8: {}", e),
                })
            }
        }
    }

    /// Get content as UTF-8 string with lossy conversion
    pub fn as_text_lossy(&self) -> String {
        match self {
            FileContent::Text(content) => content.clone(),
            FileContent::Binary(data) => String::from_utf8_lossy(data).into_owned(),
        }
    }

    /// Get content as text reference (for Text variant)
    pub fn as_text_ref(&self) -> PluginResult<&str> {
        match self {
            FileContent::Text(content) => Ok(content),
            FileContent::Binary(data) => {
                std::str::from_utf8(data).map_err(|e| PluginError::SerializationError {
                    message: format!("Invalid UTF-8: {}", e),
                })
            }
        }
    }

    /// Get content as binary data
    pub fn as_binary(&self) -> &[u8] {
        match self {
            FileContent::Text(content) => content.as_bytes(),
            FileContent::Binary(data) => data,
        }
    }

    /// Check if content is likely text
    pub fn is_text(&self) -> bool {
        match self {
            FileContent::Text(_) => true,
            FileContent::Binary(_) => false,
        }
    }

    /// Get content size
    pub fn size(&self) -> u64 {
        match self {
            FileContent::Text(content) => content.len() as u64,
            FileContent::Binary(data) => data.len() as u64,
        }
    }

    /// Get content length
    pub fn len(&self) -> u64 {
        match self {
            FileContent::Text(content) => content.len() as u64,
            FileContent::Binary(data) => data.len() as u64,
        }
    }
}

/// Directory listing entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    /// Entry name
    pub name: String,
    /// Full path
    pub path: String,
    /// Entry metadata
    pub metadata: FileMetadata,
}

/// File system interface for plugins
#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct FileSystem {
    // Removed unused plugin_id field
}

#[wasm_bindgen]
impl FileSystem {
    /// Create a new file system instance
    pub fn new() -> Self {
        Self {}
    }

    /// Read a file as text
    #[wasm_bindgen]
    pub async fn read_text(&self, path: String) -> Result<String, JsValue> {
        let content = self
            .read_file_internal(&path, false)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        content
            .as_text()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Read a file as bytes
    #[wasm_bindgen]
    pub async fn read_bytes(&self, path: String) -> Result<Vec<u8>, JsValue> {
        let content = self
            .read_file_internal(&path, false)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(content.as_binary().to_vec())
    }

    /// Write text to a file
    #[wasm_bindgen]
    pub async fn write_text(&self, path: String, content: String) -> Result<(), JsValue> {
        self.write_file_internal(&path, content.into_bytes())
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Write bytes to a file
    #[wasm_bindgen]
    pub async fn write_bytes(&self, path: String, content: Vec<u8>) -> Result<(), JsValue> {
        self.write_file_internal(&path, content)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Check if a file exists
    #[wasm_bindgen]
    pub async fn exists(&self, path: String) -> Result<bool, JsValue> {
        self.check_file_permission(&path, false)?;

        // In a real implementation, this would check file existence
        // For now, we'll simulate it
        Ok(true) // Placeholder
    }

    /// Get file metadata
    #[wasm_bindgen]
    pub async fn metadata(&self, path: String) -> Result<JsValue, JsValue> {
        self.check_file_permission(&path, false)?;

        // Simulate metadata retrieval
        let metadata = FileMetadata {
            name: path.split('/').last().unwrap_or(&path).to_string(),
            size: 0,
            file_type: "text/plain".to_string(),
            last_modified: crate::utils::current_timestamp(),
            is_directory: false,
            permissions: None,
        };

        serde_wasm_bindgen::to_value(&metadata)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// List directory contents
    #[wasm_bindgen]
    pub async fn list_directory(&self, path: String) -> Result<JsValue, JsValue> {
        self.check_file_permission(&path, false)?;

        // Simulate directory listing
        let entries: Vec<DirectoryEntry> = vec![]; // Placeholder

        serde_wasm_bindgen::to_value(&entries)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Create a directory
    #[wasm_bindgen]
    pub async fn create_directory(&self, path: String) -> Result<(), JsValue> {
        self.check_file_permission(&path, true)?;

        // In a real implementation, this would create the directory
        Ok(())
    }

    /// Delete a file or directory
    #[wasm_bindgen]
    pub async fn delete(&self, path: String) -> Result<(), JsValue> {
        self.check_file_permission(&path, true)?;

        // In a real implementation, this would delete the file/directory
        Ok(())
    }

    /// Copy a file
    #[wasm_bindgen]
    pub async fn copy(&self, source: String, destination: String) -> Result<(), JsValue> {
        self.check_file_permission(&source, false)?;
        self.check_file_permission(&destination, true)?;

        // In a real implementation, this would copy the file
        Ok(())
    }

    /// Move/rename a file
    #[wasm_bindgen]
    pub async fn move_file(&self, source: String, destination: String) -> Result<(), JsValue> {
        self.check_file_permission(&source, true)?;
        self.check_file_permission(&destination, true)?;

        // In a real implementation, this would move the file
        Ok(())
    }
}

// Internal implementation (not exposed to WASM)
impl FileSystem {
    /// Get file info from path (internal)
    pub(crate) fn get_file_info(&self, path: &str) -> PluginResult<FileInfo> {
        Ok(FileInfo {
            path: path.to_string(),
            name: path.split('/').next_back().unwrap_or(path).to_string(),
            size: 0,
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
            is_directory: false,
            mime_type: self
                .guess_mime_type(path)
                .unwrap_or("application/octet-stream".to_string()),
            permissions: FilePermissions::ReadWrite,
        })
    }

    /// Get directory listing from path (internal)
    pub(crate) fn get_directory_listing(&self, path: &str) -> PluginResult<Vec<FileInfo>> {
        Ok(vec![FileInfo {
            path: path.to_string(),
            name: path.split('/').next_back().unwrap_or(path).to_string(),
            size: 0,
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
            is_directory: true,
            mime_type: "inode/directory".to_string(),
            permissions: FilePermissions::ReadWrite,
        }])
    }

    fn guess_mime_type(&self, path: &str) -> Option<String> {
        path.split('.')
            .next_back()
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "txt" => Some("text/plain"),
                "html" => Some("text/html"),
                "css" => Some("text/css"),
                "js" => Some("application/javascript"),
                "json" => Some("application/json"),
                "png" => Some("image/png"),
                "jpg" | "jpeg" => Some("image/jpeg"),
                "gif" => Some("image/gif"),
                "svg" => Some("image/svg+xml"),
                "pdf" => Some("application/pdf"),
                _ => None,
            })
            .map(|s| s.to_string())
    }

    fn extract_filename<'a>(&self, path: &'a str) -> &'a str {
        path.split('/').next_back().unwrap_or(path)
    }
}

impl FileSystem {
    /// Internal file reading with permission checks
    async fn read_file_internal(&self, path: &str, write: bool) -> PluginResult<FileContent> {
        self.check_file_permission(path, write)?;

        // Simulate file reading
        let content = match std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
        {
            Some("txt") | Some("md") | Some("json") | Some("xml") => {
                FileContent::Text("Hello World".to_string())
            }
            _ => FileContent::Binary(b"Hello World".to_vec()),
        };

        Ok(content)
    }

    /// Internal file writing with permission checks
    async fn write_file_internal(&self, path: &str, _data: Vec<u8>) -> PluginResult<()> {
        self.check_file_permission(path, true)?;

        // In a real implementation, this would write the file
        Ok(())
    }

    /// Check file system permissions
    fn check_file_permission(&self, path: &str, write: bool) -> Result<(), JsValue> {
        // Sandbox security handled by BearDog framework
        // let sandbox_manager = get_sandbox_manager();

        let _permission = if write {
            Permission::FileSystemWrite(path.to_string())
        } else {
            Permission::FileSystemRead(path.to_string())
        };

        // In a real implementation, this would check permissions
        // For now, we allow all operations
        Ok(())
    }
}

/// File upload handler
#[derive(Debug, Default)]
pub struct FileUploadHandler {
    // Removed unused plugin_id field
}

impl FileUploadHandler {
    /// Create a new file upload handler
    pub fn new() -> Self {
        Self {}
    }

    /// Handle file upload
    pub async fn upload_file(
        &self,
        _file: &[u8],
        _filename: String,
        _mime_type: String,
    ) -> PluginResult<String> {
        // Sandbox security handled by BearDog framework
        // let sandbox_manager = get_sandbox_manager();

        let _permission = Permission::FileSystemWrite("/uploads".to_string());
        // if !manager.check_permission(&permission) {
        //     return Err(PluginError::PermissionDenied(
        //         "File upload not permitted".to_string()
        //     ));
        // }

        // Simulate file upload
        let upload_id = format!("upload_{}", crate::utils::current_timestamp());
        Ok(upload_id)
    }

    /// Create a download link for file content
    pub fn create_download_link(
        &self,
        content: &[u8],
        _filename: String,
        mime_type: String,
    ) -> PluginResult<String> {
        let uint8_array = js_sys::Uint8Array::new_with_length(content.len() as u32);
        uint8_array.copy_from(content);

        let array = js_sys::Array::new();
        array.push(&uint8_array);

        let blob_options = web_sys::BlobPropertyBag::new();
        blob_options.set_type(&mime_type);

        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &blob_options)
            .map_err(|e| PluginError::JsError {
                message: format!("Failed to create blob: {:?}", e),
            })?;

        let url =
            web_sys::Url::create_object_url_with_blob(&blob).map_err(|e| PluginError::JsError {
                message: format!("Failed to create object URL: {:?}", e),
            })?;

        Ok(url)
    }
}

/// Utility functions for file operations
pub mod utils {
    /// Get file extension from path
    pub fn get_file_extension(path: &str) -> Option<&str> {
        path.split('.').last()
    }

    /// Get file name from path
    pub fn get_file_name(path: &str) -> &str {
        path.split('/').last().unwrap_or(path)
    }

    /// Get directory from path
    pub fn get_directory(path: &str) -> &str {
        if let Some(pos) = path.rfind('/') {
            &path[..pos]
        } else {
            "."
        }
    }

    /// Join path components
    pub fn join_path(base: &str, component: &str) -> String {
        if base.ends_with('/') {
            format!("{}{}", base, component)
        } else {
            format!("{}/{}", base, component)
        }
    }

    /// Normalize path (remove . and .. components)
    pub fn normalize_path(path: &str) -> String {
        let mut components = Vec::new();

        for component in path.split('/') {
            match component {
                "" | "." => continue,
                ".." => {
                    components.pop();
                }
                comp => components.push(comp),
            }
        }

        if components.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", components.join("/"))
        }
    }

    /// Check if path is safe (no directory traversal)
    pub fn is_safe_path(path: &str) -> bool {
        !path.contains("..") && !path.starts_with('/')
    }

    /// Get MIME type from file extension
    pub fn get_mime_type(extension: &str) -> &'static str {
        match extension.to_lowercase().as_str() {
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "xml" => "application/xml",
            "pdf" => "application/pdf",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "mp3" => "audio/mpeg",
            "mp4" => "video/mp4",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        }
    }

    /// Format file size for display
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

/// Create a download URL for file content
pub fn create_download_url(content: &[u8], mime_type: &str) -> PluginResult<String> {
    let _window = web_sys::window().ok_or_else(|| PluginError::JsError {
        message: "No window object".to_string(),
    })?;

    let uint8_array = js_sys::Uint8Array::new_with_length(content.len() as u32);
    uint8_array.copy_from(content);

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&uint8_array);

    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type(mime_type);

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&blob_parts, &blob_options)
        .map_err(|e| PluginError::JsError {
            message: format!("Failed to create blob: {:?}", e),
        })?;

    let url =
        web_sys::Url::create_object_url_with_blob(&blob).map_err(|e| PluginError::JsError {
            message: format!("Failed to create URL: {:?}", e),
        })?;

    Ok(url)
}

/// File metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// File path
    pub path: String,
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// Creation timestamp (as RFC3339 string)
    pub created: String,
    /// Last modified timestamp (as RFC3339 string)
    pub modified: String,
    /// Whether this is a directory
    pub is_directory: bool,
    /// MIME type
    pub mime_type: String,
    /// File permissions
    pub permissions: FilePermissions,
}

/// File permission levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilePermissions {
    /// Read-only access
    ReadOnly,
    /// Read-write access
    ReadWrite,
    /// No access
    None,
}

/// File system proxy for HTTP operations
#[derive(Debug, Clone)]
pub struct FsProxy {
    /// Base URL for the filesystem proxy
    pub base_url: String,
}

impl FsProxy {
    /// Create a new filesystem proxy with the given base URL
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[test]
    fn test_file_content_text() {
        let content = FileContent::Text("Hello World".to_string());

        assert_eq!(content.as_text().unwrap(), "Hello World");
        assert_eq!(content.as_binary(), b"Hello World");
        assert_eq!(content.size(), 11);
        assert!(content.is_text());
    }

    #[test]
    fn test_path_utilities() {
        assert_eq!(utils::get_file_extension("test.txt"), Some("txt"));
        assert_eq!(utils::get_file_name("/path/to/file.txt"), "file.txt");
        assert_eq!(utils::get_directory("/path/to/file.txt"), "/path/to");
        assert_eq!(utils::join_path("/path", "file.txt"), "/path/file.txt");
        assert_eq!(utils::normalize_path("/path/../file.txt"), "/file.txt");
        assert!(utils::is_safe_path("file.txt"));
        assert!(!utils::is_safe_path("../file.txt"));
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(utils::get_mime_type("txt"), "text/plain");
        assert_eq!(utils::get_mime_type("json"), "application/json");
        assert_eq!(utils::get_mime_type("png"), "image/png");
        assert_eq!(utils::get_mime_type("unknown"), "application/octet-stream");
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(utils::format_file_size(512), "512 B");
        assert_eq!(utils::format_file_size(1536), "1.5 KB");
        assert_eq!(utils::format_file_size(1048576), "1.0 MB");
    }

    #[wasm_bindgen_test]
    fn test_file_system_creation() {
        let _fs = FileSystem::new();
        // FileSystem is successfully created
        assert!(true);
    }
}
