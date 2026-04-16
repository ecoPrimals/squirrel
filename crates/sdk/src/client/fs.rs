// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

    /// Check if a file exists.
    ///
    /// WASM sandbox stub: returns `false` until a host filesystem is wired
    /// via the plugin bridge. Real implementations route through the host's
    /// `fs.exists` capability.
    #[wasm_bindgen]
    pub async fn exists(&self, path: String) -> Result<bool, JsValue> {
        self.check_file_permission(&path, false)?;
        Ok(false)
    }

    /// Get file metadata.
    ///
    /// WASM sandbox stub: returns zeroed metadata until a host filesystem is
    /// wired. Uses `serde_wasm_bindgen` (wasm32-only).
    #[wasm_bindgen]
    pub async fn metadata(&self, path: String) -> Result<JsValue, JsValue> {
        self.check_file_permission(&path, false)?;

        let metadata = FileMetadata {
            name: path.split('/').next_back().unwrap_or(&path).to_string(),
            size: 0,
            file_type: "application/octet-stream".to_string(),
            last_modified: 0,
            is_directory: false,
            permissions: None,
        };

        serde_wasm_bindgen::to_value(&metadata)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }

    /// List directory contents.
    ///
    /// WASM sandbox stub: returns empty listing until a host filesystem is wired.
    #[wasm_bindgen]
    pub async fn list_directory(&self, path: String) -> Result<JsValue, JsValue> {
        self.check_file_permission(&path, false)?;

        let entries: Vec<DirectoryEntry> = vec![];
        serde_wasm_bindgen::to_value(&entries)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }

    /// Create a directory (WASM sandbox stub: no-op until host wired).
    #[wasm_bindgen]
    pub async fn create_directory(&self, path: String) -> Result<(), JsValue> {
        self.check_file_permission(&path, true)?;
        Ok(())
    }

    /// Delete a file or directory (WASM sandbox stub: no-op until host wired).
    #[wasm_bindgen]
    pub async fn delete(&self, path: String) -> Result<(), JsValue> {
        self.check_file_permission(&path, true)?;
        Ok(())
    }

    /// Copy a file (WASM sandbox stub: no-op until host wired).
    #[wasm_bindgen]
    pub async fn copy(&self, source: String, destination: String) -> Result<(), JsValue> {
        self.check_file_permission(&source, false)?;
        self.check_file_permission(&destination, true)?;
        Ok(())
    }

    /// Move/rename a file (WASM sandbox stub: no-op until host wired).
    #[wasm_bindgen]
    pub async fn move_file(&self, source: String, destination: String) -> Result<(), JsValue> {
        self.check_file_permission(&source, true)?;
        self.check_file_permission(&destination, true)?;
        Ok(())
    }
}

// Internal implementation (not exposed to WASM)
impl FileSystem {
    /// Internal file reading (WASM sandbox stub: returns empty content
    /// until a host filesystem is wired via the plugin bridge).
    async fn read_file_internal(&self, path: &str, write: bool) -> PluginResult<FileContent> {
        self.check_file_permission(path, write)?;
        Ok(FileContent::Binary(Vec::new()))
    }

    /// Internal file writing (WASM sandbox stub: no-op until host wired).
    async fn write_file_internal(&self, path: &str, _data: Vec<u8>) -> PluginResult<()> {
        self.check_file_permission(path, true)?;
        Ok(())
    }

    /// Check file system permissions
    fn check_file_permission(&self, path: &str, write: bool) -> Result<(), JsValue> {
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
        filename: String,
        _mime_type: String,
    ) -> PluginResult<String> {
        Err(PluginError::McpError {
            message: format!("WASM sandbox: filesystem not connected — cannot upload '{filename}'"),
        })
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
        path.split('.').next_back()
    }

    /// Get file name from path
    pub fn get_file_name(path: &str) -> &str {
        path.split('/').next_back().unwrap_or(path)
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

    #[test]
    fn test_file_content_text() {
        let content = FileContent::Text("Hello World".to_string());

        assert_eq!(content.as_text().expect("should succeed"), "Hello World");
        assert_eq!(content.as_binary(), b"Hello World");
        assert_eq!(content.size(), 11);
        assert_eq!(content.len(), 11);
        assert!(!content.is_empty());
        assert!(content.is_text());
        assert_eq!(
            content.as_text_ref().expect("should succeed"),
            "Hello World"
        );
        assert_eq!(content.as_text_lossy(), "Hello World");
    }

    #[test]
    fn test_file_content_binary_valid_utf8() {
        let content = FileContent::Binary(b"hi".to_vec());
        assert!(!content.is_text());
        assert_eq!(content.as_text().expect("should succeed"), "hi");
        assert_eq!(content.as_text_ref().expect("should succeed"), "hi");
        assert_eq!(content.as_text_lossy(), "hi");
    }

    #[test]
    fn test_file_content_binary_invalid_utf8() {
        let bad = vec![0xff, 0xfe, 0xfd];
        let content = FileContent::Binary(bad);
        assert!(content.as_text().is_err());
        assert!(content.as_text_ref().is_err());
        assert!(!content.as_text_lossy().is_empty());
    }

    #[test]
    fn test_file_content_empty() {
        let t = FileContent::Text(String::new());
        assert!(t.is_empty());
        let b = FileContent::Binary(vec![]);
        assert!(b.is_empty());
    }

    #[test]
    fn test_path_utilities() {
        assert_eq!(utils::get_file_extension("test.txt"), Some("txt"));
        assert_eq!(utils::get_file_name("/path/to/file.txt"), "file.txt");
        assert_eq!(utils::get_file_name("single"), "single");
        assert_eq!(utils::get_directory("/path/to/file.txt"), "/path/to");
        assert_eq!(utils::get_directory("relative"), ".");
        assert_eq!(utils::join_path("/path", "file.txt"), "/path/file.txt");
        assert_eq!(utils::join_path("/path/", "file.txt"), "/path/file.txt");
        assert_eq!(utils::normalize_path("/path/../file.txt"), "/file.txt");
        assert_eq!(utils::normalize_path("//a/./b/../c"), "/a/c");
        assert_eq!(utils::normalize_path(".."), "/");
        assert!(utils::is_safe_path("file.txt"));
        assert!(!utils::is_safe_path("../file.txt"));
        assert!(!utils::is_safe_path("/abs"));
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(utils::get_mime_type("TXT"), "text/plain");
        assert_eq!(utils::get_mime_type("htm"), "text/html");
        assert_eq!(utils::get_mime_type("jpeg"), "image/jpeg");
        assert_eq!(utils::get_mime_type("svg"), "image/svg+xml");
        assert_eq!(utils::get_mime_type("unknown"), "application/octet-stream");
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(utils::format_file_size(0), "0 B");
        assert_eq!(utils::format_file_size(512), "512 B");
        assert_eq!(utils::format_file_size(1536), "1.5 KB");
        assert_eq!(utils::format_file_size(1_048_576), "1.0 MB");
        assert_eq!(utils::format_file_size(5 * 1024_u64.pow(4)), "5.0 TB");
    }

    #[test]
    fn test_file_metadata_serde_roundtrip() {
        let meta = FileMetadata {
            name: "a.txt".to_string(),
            size: 10,
            file_type: "text/plain".to_string(),
            last_modified: 1,
            is_directory: false,
            permissions: Some("rw".to_string()),
        };
        let json = serde_json::to_string(&meta).expect("should succeed");
        let back: FileMetadata = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(meta.name, back.name);
        assert_eq!(meta.permissions, back.permissions);
    }

    #[test]
    fn test_directory_entry_serde_roundtrip() {
        let entry = DirectoryEntry {
            name: "n".to_string(),
            path: "/p/n".to_string(),
            metadata: FileMetadata {
                name: "n".to_string(),
                size: 0,
                file_type: "x".to_string(),
                last_modified: 0,
                is_directory: true,
                permissions: None,
            },
        };
        let v = serde_json::to_value(&entry).expect("should succeed");
        let back: DirectoryEntry = serde_json::from_value(v).expect("should succeed");
        assert_eq!(entry.path, back.path);
    }

    #[test]
    fn test_file_info_and_permissions_serde() {
        let info = FileInfo {
            path: "/a".to_string(),
            name: "a".to_string(),
            size: 3,
            created: "t0".to_string(),
            modified: "t1".to_string(),
            is_directory: false,
            mime_type: "text/plain".to_string(),
            permissions: FilePermissions::ReadWrite,
        };
        let s = serde_json::to_string(&info).expect("should succeed");
        let back: FileInfo = serde_json::from_str(&s).expect("should succeed");
        assert!(matches!(back.permissions, FilePermissions::ReadWrite));
    }

    #[test]
    fn test_fs_proxy_new() {
        let p = FsProxy::new("https://fs.example/".to_string());
        assert_eq!(p.base_url, "https://fs.example/");
    }

    #[test]
    fn test_filesystem_default_and_new() {
        let fs = FileSystem::new();
        let _: FileSystem = FileSystem::default();
        assert_eq!(format!("{:?}", fs), "FileSystem");
    }

    #[tokio::test]
    async fn test_filesystem_read_returns_empty_without_host() {
        let fs = FileSystem::new();
        let bytes = fs
            .read_bytes("blob.bin".to_string())
            .await
            .expect("should succeed");
        assert!(bytes.is_empty(), "stub returns empty data until host wired");
    }

    #[tokio::test]
    async fn test_filesystem_read_text_empty_stub() {
        let fs = FileSystem::new();
        let text = fs
            .read_text("notes.txt".to_string())
            .await
            .expect("read_text");
        assert!(text.is_empty());
    }

    #[test]
    fn test_path_utils_additional_edge_cases() {
        assert_eq!(utils::normalize_path("/a/b/../../c"), "/c");
        assert_eq!(utils::get_file_extension(".hidden"), Some("hidden"));
        assert_eq!(utils::format_file_size(10 * 1024_u64.pow(4)), "10.0 TB");
    }

    #[test]
    fn test_file_permissions_enum_serde() {
        let p = FilePermissions::None;
        let s = serde_json::to_string(&p).expect("serde");
        let back: FilePermissions = serde_json::from_str(&s).expect("de");
        assert!(matches!(back, FilePermissions::None));
    }

    #[tokio::test]
    async fn test_filesystem_ops_stub_behavior() {
        let fs = FileSystem::new();
        fs.write_text("out.txt".to_string(), "x".to_string())
            .await
            .expect("write stub is no-op");
        assert!(
            !fs.exists("any".to_string()).await.expect("should succeed"),
            "exists returns false until host wired"
        );
        fs.create_directory("d".to_string())
            .await
            .expect("stub no-op");
        fs.delete("d".to_string()).await.expect("stub no-op");
        fs.copy("a".to_string(), "b".to_string())
            .await
            .expect("stub no-op");
        fs.move_file("a".to_string(), "b".to_string())
            .await
            .expect("stub no-op");
    }

    /// `metadata` / `list_directory` use `serde_wasm_bindgen::to_value`, which requires wasm.
    #[cfg(target_arch = "wasm32")]
    #[tokio::test]
    async fn test_filesystem_metadata_and_list_wasm() {
        let fs = FileSystem::new();
        let meta = fs
            .metadata("x.txt".to_string())
            .await
            .expect("should succeed");
        let parsed: FileMetadata = serde_wasm_bindgen::from_value(meta).expect("should succeed");
        assert_eq!(parsed.file_type, "text/plain");
        let list = fs
            .list_directory("/".to_string())
            .await
            .expect("should succeed");
        let entries: Vec<DirectoryEntry> =
            serde_wasm_bindgen::from_value(list).expect("should succeed");
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_file_upload_handler_errors_without_host() {
        let h = FileUploadHandler::new();
        let _: FileUploadHandler = FileUploadHandler::default();
        let err = h
            .upload_file(
                b"data",
                "f.bin".to_string(),
                "application/octet-stream".to_string(),
            )
            .await;
        assert!(err.is_err(), "upload errors without host fs");
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_create_download_url_without_window_errors() {
        let r = create_download_url(b"x", "text/plain");
        assert!(r.is_err());
        match r {
            Err(crate::error::PluginError::JsError { message }) => {
                assert!(
                    message.contains("window")
                        || message.contains("Failed")
                        || message.contains("not implemented")
                );
            }
            _ => unreachable!("expected JsError"),
        }
    }
}
