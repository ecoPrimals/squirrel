---
version: 1.0.0
status: proposed
last_updated: 2024-10-01
author: DataScienceBioLab
---

# ZFS-Based Storage Architecture for Context Management

## Overview

This specification defines a ZFS-based storage architecture for the Squirrel Context Management System. The architecture leverages ZFS's advanced features such as snapshots, checksumming, compression, and deduplication to provide a robust, high-performance storage backend for context data, enabling time-travel debugging, instant recovery, and efficient storage of large context datasets.

## Objectives

1. Implement a highly reliable storage backend for context data
2. Enable point-in-time recovery through ZFS snapshots
3. Optimize storage efficiency through compression and deduplication
4. Provide transparent data integrity through checksumming
5. Support high-performance time series data storage for learning systems
6. Create an abstraction layer allowing fallback to alternative storage systems

## Key Considerations

### License Compatibility

ZFS is licensed under the Common Development and Distribution License (CDDL), which is incompatible with GPL v2. To address this:

1. **Architecture Separation**: The ZFS storage implementation will be isolated in a separate binary/process, communicating via IPC/RPC
2. **Modular Design**: Storage interfaces will be designed for multiple backends (ZFS, traditional file systems, databases)
3. **Runtime Detection**: ZFS support will be detected at runtime, falling back to alternative storage if unavailable
4. **License Clarity**: Documentation will clearly state the licensing implications for users compiling with ZFS support

This approach allows us to maintain GPL v2 compatibility for core components while providing optional ZFS capabilities.

## Architecture

The ZFS storage system consists of several key components:

```
context-storage/
├── interface/          # Storage interface definitions
│   ├── traits.rs       # Storage trait definitions
│   └── types.rs        # Common type definitions
├── zfs/                # ZFS-specific implementation
│   ├── adapter.rs      # ZFS adapter implementation
│   ├── service.rs      # ZFS service process
│   ├── client.rs       # ZFS client implementation
│   └── proto/          # Service protocol definitions
├── fallback/           # Fallback storage implementations
│   ├── file.rs         # File-based storage
│   ├── sled.rs         # Sled DB implementation
│   └── memory.rs       # In-memory storage
└── factory.rs          # Factory for creating storage instances
```

## Core Components

### 1. Storage Interface

The storage interface provides a common abstraction layer for all storage backends:

```rust
/// Context Storage trait defining the interface for all storage backends
#[async_trait]
pub trait ContextStorage: Send + Sync {
    /// Store context data
    async fn store<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        data: &T,
        metadata: &StorageMetadata,
    ) -> Result<StorageInfo, StorageError>;
    
    /// Retrieve context data
    async fn retrieve<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<StorageItem<T>, StorageError>;
    
    /// Retrieve context data as of a specific timestamp
    async fn retrieve_as_of<T: DeserializeOwned + Send + Sync>(
        &self,
        key: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<StorageItem<T>, StorageError>;
    
    /// List all keys matching a pattern
    async fn list_keys(&self, pattern: &str) -> Result<Vec<String>, StorageError>;
    
    /// Delete a key
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
    
    /// Create a snapshot
    async fn create_snapshot(&self, name: &str) -> Result<SnapshotInfo, StorageError>;
    
    /// List available snapshots
    async fn list_snapshots(&self) -> Result<Vec<SnapshotInfo>, StorageError>;
    
    /// Restore from a snapshot
    async fn restore_snapshot(&self, name: &str) -> Result<(), StorageError>;
    
    /// Delete a snapshot
    async fn delete_snapshot(&self, name: &str) -> Result<(), StorageError>;
    
    /// Get storage statistics
    async fn get_stats(&self) -> Result<StorageStats, StorageError>;
}

/// Storage item with metadata
pub struct StorageItem<T> {
    /// The stored data
    pub data: T,
    /// Storage metadata
    pub metadata: StorageMetadata,
    /// Storage information
    pub info: StorageInfo,
}

/// Storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    /// Content type
    pub content_type: String,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last modified time
    pub modified_at: DateTime<Utc>,
    /// Version information
    pub version: String,
    /// Tags
    pub tags: HashMap<String, String>,
}

/// Storage information returned after storing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    /// Storage key
    pub key: String,
    /// Size in bytes
    pub size: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Storage-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Snapshot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    /// Snapshot name
    pub name: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Size in bytes
    pub size: u64,
    /// Snapshot-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total size in bytes
    pub total_size: u64,
    /// Used size in bytes
    pub used_size: u64,
    /// Available size in bytes
    pub available_size: u64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// Deduplication ratio
    pub deduplication_ratio: f64,
    /// Total number of items
    pub total_items: u64,
    /// Total number of snapshots
    pub total_snapshots: u64,
}
```

### 2. ZFS Adapter Service

The ZFS Adapter operates as a separate process to maintain license separation:

```rust
/// ZFS Storage Service
pub struct ZfsStorageService {
    /// ZFS configuration
    config: ZfsConfig,
    /// Pool name
    pool_name: String,
    /// Dataset path
    dataset_path: String,
    /// RPC server
    server: RpcServer,
}

impl ZfsStorageService {
    /// Create a new ZFS Storage Service
    pub fn new(config: ZfsConfig) -> Result<Self, ZfsError> {
        // Implementation
    }
    
    /// Start the service
    pub async fn start(&self) -> Result<(), ZfsError> {
        // Implementation
    }
    
    /// Stop the service
    pub async fn stop(&self) -> Result<(), ZfsError> {
        // Implementation
    }
    
    /// Store data in ZFS
    async fn store(
        &self,
        request: StoreRequest,
    ) -> Result<StoreResponse, ZfsError> {
        // Implementation
    }
    
    /// Retrieve data from ZFS
    async fn retrieve(
        &self,
        request: RetrieveRequest,
    ) -> Result<RetrieveResponse, ZfsError> {
        // Implementation
    }
    
    /// Create ZFS snapshot
    async fn create_snapshot(
        &self,
        request: CreateSnapshotRequest,
    ) -> Result<CreateSnapshotResponse, ZfsError> {
        // Implementation
    }
    
    /// List available snapshots
    async fn list_snapshots(
        &self,
        request: ListSnapshotsRequest,
    ) -> Result<ListSnapshotsResponse, ZfsError> {
        // Implementation
    }
    
    /// Restore from a snapshot
    async fn restore_snapshot(
        &self,
        request: RestoreSnapshotRequest,
    ) -> Result<RestoreSnapshotResponse, ZfsError> {
        // Implementation
    }
    
    /// Execute a ZFS command
    async fn execute_zfs_command(
        &self,
        cmd: &str,
        args: &[&str],
    ) -> Result<String, ZfsError> {
        // Implementation
    }
}
```

### 3. ZFS Client

The ZFS Client communicates with the ZFS service:

```rust
/// ZFS Storage Client implementing the ContextStorage trait
pub struct ZfsStorageClient {
    /// Client configuration
    config: ZfsClientConfig,
    /// RPC client
    client: RpcClient,
}

#[async_trait]
impl ContextStorage for ZfsStorageClient {
    async fn store<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        data: &T,
        metadata: &StorageMetadata,
    ) -> Result<StorageInfo, StorageError> {
        // Serialize data
        let serialized = serde_json::to_vec(data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
        // Create store request
        let request = StoreRequest {
            key: key.to_string(),
            data: serialized,
            metadata: serde_json::to_vec(metadata)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?,
        };
        
        // Send request to service
        let response = self.client.store(request).await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;
            
        // Convert response to StorageInfo
        let info = serde_json::from_slice(&response.info)
            .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
            
        Ok(info)
    }
    
    // Other method implementations...
}
```

### 4. Storage Factory

The Storage Factory creates appropriate storage implementations based on configuration and availability:

```rust
/// Factory for creating storage instances
pub struct StorageFactory {
    /// Configuration
    config: StorageConfig,
}

impl StorageFactory {
    /// Create a new storage factory
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }
    
    /// Create a context storage instance
    pub async fn create_storage(&self) -> Result<Arc<dyn ContextStorage>, StorageError> {
        // Check if ZFS is enabled and available
        if self.config.use_zfs && self.is_zfs_available().await {
            // Create ZFS storage
            let client_config = ZfsClientConfig {
                service_address: self.config.zfs_service_address.clone(),
                timeout: self.config.timeout,
            };
            
            let client = ZfsStorageClient::new(client_config)
                .map_err(|e| StorageError::InitializationError(e.to_string()))?;
                
            Ok(Arc::new(client))
        } else {
            // Fallback to file storage
            let file_config = FileStorageConfig {
                base_path: self.config.file_storage_path.clone(),
                create_dirs: true,
            };
            
            let file_storage = FileStorage::new(file_config)
                .map_err(|e| StorageError::InitializationError(e.to_string()))?;
                
            Ok(Arc::new(file_storage))
        }
    }
    
    /// Check if ZFS is available
    async fn is_zfs_available(&self) -> bool {
        // Implementation
    }
}
```

## ZFS Dataset Structure

The ZFS storage will use the following dataset structure:

```
squirrel-context/                    # Main pool
├── data/                            # Active data
│   ├── contexts/                    # Context objects
│   ├── rules/                       # Rule data
│   └── patterns/                    # Pattern data
├── time-series/                     # Time series data
│   ├── metrics/                     # Metric data
│   └── events/                      # Event data
└── snapshots/                       # Snapshot storage
    ├── hourly/                      # Hourly snapshots
    ├── daily/                       # Daily snapshots
    └── manual/                      # Manual snapshots
```

## ZFS Configuration

The ZFS configuration will include the following options:

```toml
[storage.zfs]
enabled = true
pool_name = "squirrel-context"
dataset_path = "data"
compression = "lz4"
deduplication = true
snapshot_retention = "7d"
autosnap_interval = "1h"
service_address = "127.0.0.1:9870"

[storage.fallback]
type = "file"
path = "data/context"
```

## Key Features

### 1. Time Travel Context Debugging

ZFS snapshots enable retrieving context state from any point in time:

```rust
// Get context as of a specific time
async fn get_context_as_of(
    storage: &dyn ContextStorage,
    context_id: &str,
    timestamp: DateTime<Utc>,
) -> Result<Context, ContextError> {
    let storage_item = storage.retrieve_as_of::<ContextData>(
        &format!("contexts/{}", context_id),
        timestamp,
    ).await?;
    
    // Convert to Context object
    let context = Context::from_storage_item(storage_item)?;
    
    Ok(context)
}
```

### 2. Automatic Snapshotting

The system will automatically create snapshots at configured intervals:

```rust
/// Snapshot scheduler
pub struct SnapshotScheduler {
    /// Storage reference
    storage: Arc<dyn ContextStorage>,
    /// Scheduler configuration
    config: SchedulerConfig,
}

impl SnapshotScheduler {
    /// Start the scheduler
    pub async fn start(&self) -> Result<(), SchedulerError> {
        // Implementation
    }
    
    /// Create a scheduled snapshot
    async fn create_scheduled_snapshot(&self, schedule_type: ScheduleType) -> Result<(), SchedulerError> {
        let snapshot_name = match schedule_type {
            ScheduleType::Hourly => format!("hourly-{}", Utc::now().format("%Y%m%d%H")),
            ScheduleType::Daily => format!("daily-{}", Utc::now().format("%Y%m%d")),
            ScheduleType::Weekly => format!("weekly-{}", Utc::now().format("%Y%W")),
        };
        
        self.storage.create_snapshot(&snapshot_name).await?;
        
        // Cleanup old snapshots if needed
        self.cleanup_old_snapshots(schedule_type).await?;
        
        Ok(())
    }
    
    /// Clean up old snapshots based on retention policy
    async fn cleanup_old_snapshots(&self, schedule_type: ScheduleType) -> Result<(), SchedulerError> {
        // Implementation
    }
}
```

### 3. Data Integrity

ZFS provides automatic data integrity checking:

```rust
/// Integrity checker
pub struct IntegrityChecker {
    /// Storage reference
    storage: Arc<dyn ContextStorage>,
    /// Checker configuration
    config: CheckerConfig,
}

impl IntegrityChecker {
    /// Run an integrity check
    pub async fn check_integrity(&self) -> Result<IntegrityReport, CheckerError> {
        // For ZFS, leverage built-in scrub functionality
        if let Some(zfs_storage) = self.storage.as_any().downcast_ref::<ZfsStorageClient>() {
            return zfs_storage.run_scrub().await.map_err(CheckerError::from);
        }
        
        // For other storage types, perform manual checks
        // Implementation
        
        Ok(IntegrityReport::default())
    }
}
```

## Integration with Context Management

### ContextManager Integration

```rust
/// Initialize context manager with ZFS storage
pub async fn initialize_context_manager() -> Result<ContextManager, ContextError> {
    // Create storage factory
    let storage_config = StorageConfig::from_env()?;
    let factory = StorageFactory::new(storage_config);
    
    // Create storage instance
    let storage = factory.create_storage().await?;
    
    // Create context manager with storage
    let manager = ContextManagerBuilder::new()
        .with_storage(storage)
        .build()?;
        
    Ok(manager)
}
```

### Learning System Integration

```rust
/// Initialize learning system with time series storage
pub async fn initialize_learning_system(
    context_manager: Arc<ContextManager>,
) -> Result<LearningSystem, LearningError> {
    // Create storage factory
    let storage_config = StorageConfig::from_env()?;
    let factory = StorageFactory::new(storage_config);
    
    // Create storage instance
    let storage = factory.create_storage().await?;
    
    // Create observation collector with storage
    let collector = ObservationCollector::new(context_manager.clone())
        .with_storage(storage.clone());
        
    // Create learning system
    let learning_system = LearningSystemBuilder::new()
        .with_observation_collector(collector)
        .with_storage(storage)
        .build()?;
        
    Ok(learning_system)
}
```

## Performance Considerations

1. **Caching**: Implement multi-level caching to minimize ZFS I/O operations
2. **Batch Operations**: Use batch operations for frequent updates
3. **Compression Settings**: Tune ZFS compression for optimal performance/space tradeoff
4. **Record Size**: Optimize ZFS record size based on typical data size
5. **Memory Management**: Configure appropriate ARC cache size based on available memory

## Future Enhancements

1. **Distributed ZFS**: Support for distributed ZFS deployments across multiple nodes
2. **Tiered Storage**: Implement automated tiering for hot/cold data
3. **Encrypted Datasets**: Support for encrypted ZFS datasets for sensitive data
4. **Advanced Analytics**: ZFS-aware analytics for storage optimization
5. **Cross-Platform Support**: Enhanced support for ZFS on different platforms

## Implementation Plan

1. **Phase 1**: Storage interface definition and file-based implementation (2 weeks)
2. **Phase 2**: ZFS service implementation (3 weeks)
3. **Phase 3**: ZFS client implementation (2 weeks)
4. **Phase 4**: Snapshot and time travel functionality (2 weeks)
5. **Phase 5**: Performance optimization and testing (3 weeks)
6. **Phase 6**: Documentation and examples (2 weeks)

## Licensing and Distribution Considerations

1. **Separate Packages**: Package ZFS functionality separately from core components
2. **Runtime Plugins**: Implement ZFS support as a runtime plugin
3. **Clear Documentation**: Provide clear guidance on license implications
4. **Alternative Options**: Always ensure non-ZFS alternatives are available
5. **Legal Review**: Conduct legal review of the architecture

## References

1. OpenZFS Documentation: https://openzfs.org/wiki/Documentation
2. ZFS on Linux: https://zfsonlinux.org/
3. CDDL License: https://opensource.org/licenses/CDDL-1.0
4. GPL Compatibility: https://www.gnu.org/licenses/license-list.html#CDDL 