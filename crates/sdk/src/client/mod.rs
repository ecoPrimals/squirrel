//! Client module for HTTP and file system operations

// HTTP client
pub mod http;
pub use http::{HttpClient, HttpMethod, HttpRequest, HttpResponse, RequestBuilder};

// File system operations
pub mod fs;
pub use fs::{
    create_download_url, DirectoryEntry, FileContent, FileInfo, FileMetadata, FilePermissions,
    FileSystem, FileUploadHandler, FsProxy,
};
