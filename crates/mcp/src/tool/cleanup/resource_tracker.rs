use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Internal tracking information for resources
#[derive(Debug)]
struct ResourceTrackerData {
    /// Tool ID
    tool_id: String,
    /// Current memory usage in bytes
    memory_bytes: AtomicU64,
    /// Current CPU time used in milliseconds
    cpu_time_ms: AtomicU64,
    /// Currently open file handles
    file_handles: Arc<RwLock<HashSet<String>>>,
    /// Currently open network connections
    network_connections: Arc<RwLock<HashSet<String>>>,
}

/// Resource tracker for monitoring a tool's resource usage
#[derive(Debug)]
pub struct ResourceTracker {
    data: Arc<ResourceTrackerData>,
}

/// Current resource tracking data
#[derive(Debug)]
pub struct ResourceTrackerUsage {
    /// Current memory usage in bytes
    pub memory_bytes: u64,
    /// Current CPU time used in milliseconds
    pub cpu_time_ms: u64,
    /// Set of open file handles
    pub file_handles: HashSet<String>,
    /// Set of open network connections
    pub network_connections: HashSet<String>,
}

impl ResourceTracker {
    /// Creates a new resource tracker for the specified tool
    pub fn new(tool_id: &str) -> Self {
        Self {
            data: Arc::new(ResourceTrackerData {
                tool_id: tool_id.to_string(),
                memory_bytes: AtomicU64::new(0),
                cpu_time_ms: AtomicU64::new(0),
                file_handles: Arc::new(RwLock::new(HashSet::new())),
                network_connections: Arc::new(RwLock::new(HashSet::new())),
            }),
        }
    }

    /// Track memory allocation
    pub async fn track_memory_allocation(&self, bytes: u64) -> Result<(), String> {
        let current = self.data.memory_bytes.fetch_add(bytes, Ordering::SeqCst);
        debug!(
            "Tool {} allocated {}B (now at {}B)",
            self.data.tool_id,
            bytes,
            current + bytes
        );
        Ok(())
    }

    /// Track memory deallocation
    pub async fn track_memory_deallocation(&self, bytes: u64) -> Result<(), String> {
        let current = self.data.memory_bytes.fetch_sub(bytes, Ordering::SeqCst);
        debug!(
            "Tool {} deallocated {}B (now at {}B)",
            self.data.tool_id,
            bytes,
            current - bytes
        );
        Ok(())
    }

    /// Track CPU time usage
    pub async fn track_cpu_time(&self, milliseconds: u64) -> Result<(), String> {
        let current = self
            .data
            .cpu_time_ms
            .fetch_add(milliseconds, Ordering::SeqCst);
        debug!(
            "Tool {} used {}ms CPU time (now at {}ms)",
            self.data.tool_id,
            milliseconds,
            current + milliseconds
        );
        Ok(())
    }

    /// Track file handle open
    pub async fn track_file_handle_open(&self, path: &str) -> Result<(), String> {
        let mut handles = self.data.file_handles.write().await;
        handles.insert(path.to_string());
        debug!(
            "Tool {} opened file {} (now has {} open files)",
            self.data.tool_id,
            path,
            handles.len()
        );
        Ok(())
    }

    /// Track file handle close
    pub async fn track_file_handle_close(&self, path: &str) -> Result<(), String> {
        let mut handles = self.data.file_handles.write().await;
        if handles.remove(path) {
            debug!(
                "Tool {} closed file {} (now has {} open files)",
                self.data.tool_id,
                path,
                handles.len()
            );
            Ok(())
        } else {
            Err(format!(
                "File handle {} not tracked for tool {}",
                path, self.data.tool_id
            ))
        }
    }

    /// Track network connection open
    pub async fn track_network_connection_open(&self, endpoint: &str) -> Result<(), String> {
        let mut connections = self.data.network_connections.write().await;
        connections.insert(endpoint.to_string());
        debug!(
            "Tool {} opened connection to {} (now has {} open connections)",
            self.data.tool_id,
            endpoint,
            connections.len()
        );
        Ok(())
    }

    /// Track network connection close
    pub async fn track_network_connection_close(&self, endpoint: &str) -> Result<(), String> {
        let mut connections = self.data.network_connections.write().await;
        if connections.remove(endpoint) {
            debug!(
                "Tool {} closed connection to {} (now has {} open connections)",
                self.data.tool_id,
                endpoint,
                connections.len()
            );
            Ok(())
        } else {
            Err(format!(
                "Network connection {} not tracked for tool {}",
                endpoint, self.data.tool_id
            ))
        }
    }

    /// Reset all resource tracking
    pub async fn reset(&self) -> Result<(), String> {
        // Reset memory and CPU counters
        self.data.memory_bytes.store(0, Ordering::SeqCst);
        self.data.cpu_time_ms.store(0, Ordering::SeqCst);

        // Reset file handles
        {
            let mut handles = self.data.file_handles.write().await;
            handles.clear();
        }

        // Reset network connections
        {
            let mut connections = self.data.network_connections.write().await;
            connections.clear();
        }

        info!("Reset all resource tracking for tool {}", self.data.tool_id);
        Ok(())
    }

    /// Get current resource usage
    pub async fn get_current_usage(&self) -> Result<ResourceTrackerUsage, String> {
        let file_handles = {
            let handles = self.data.file_handles.read().await;
            handles.iter().cloned().collect()
        };

        let network_connections = {
            let connections = self.data.network_connections.read().await;
            connections.iter().cloned().collect()
        };

        Ok(ResourceTrackerUsage {
            memory_bytes: self.data.memory_bytes.load(Ordering::SeqCst),
            cpu_time_ms: self.data.cpu_time_ms.load(Ordering::SeqCst),
            file_handles,
            network_connections,
        })
    }

    /// Perform cleanup of resources
    pub async fn cleanup(&self) -> Result<(), String> {
        // Here we would actually close/clean up resources
        // For now we just log and reset tracking

        // Get current resources to clean
        let usage = self.get_current_usage().await?;

        if !usage.file_handles.is_empty() {
            warn!(
                "Tool {} has {} unclosed file handles during cleanup",
                self.data.tool_id,
                usage.file_handles.len()
            );
            // Here we would close files
        }

        if !usage.network_connections.is_empty() {
            warn!(
                "Tool {} has {} unclosed network connections during cleanup",
                self.data.tool_id,
                usage.network_connections.len()
            );
            // Here we would close connections
        }

        // Reset counters
        self.reset().await?;

        Ok(())
    }
}
