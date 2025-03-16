use std::path::PathBuf;
use std::fs;
use std::time::Duration;
use super::{ContextState, ContextError, ContextSnapshot};

pub trait Storage: Send + Sync {
    fn save(&self, key: &str, data: &[u8]) -> Result<(), ContextError>;
    fn load(&self, key: &str) -> Result<Vec<u8>, ContextError>;
    fn delete(&self, key: &str) -> Result<(), ContextError>;
    fn exists(&self, key: &str) -> bool;
}

pub trait Serializer: Send + Sync {
    fn serialize_state(&self, state: &ContextState) -> Result<Vec<u8>, ContextError>;
    fn deserialize_state(&self, data: &[u8]) -> Result<ContextState, ContextError>;
    fn serialize_snapshot(&self, snapshot: &ContextSnapshot) -> Result<Vec<u8>, ContextError>;
    fn deserialize_snapshot(&self, data: &[u8]) -> Result<ContextSnapshot, ContextError>;
}

#[derive(Debug)]
pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    pub fn new(base_path: PathBuf) -> Result<Self, ContextError> {
        fs::create_dir_all(&base_path).map_err(|e| {
            ContextError::PersistenceError(format!("Failed to create directory: {e}"))
        })?;
        Ok(Self { base_path })
    }

    fn get_path(&self, key: &str) -> PathBuf {
        self.base_path.join(format!("{key}.json"))
    }
}

impl Storage for FileStorage {
    fn save(&self, key: &str, data: &[u8]) -> Result<(), ContextError> {
        fs::write(self.get_path(key), data).map_err(|e| {
            ContextError::PersistenceError(format!("Failed to write file: {e}"))
        })
    }

    fn load(&self, key: &str) -> Result<Vec<u8>, ContextError> {
        fs::read(self.get_path(key)).map_err(|e| {
            ContextError::PersistenceError(format!("Failed to read file: {e}"))
        })
    }

    fn delete(&self, key: &str) -> Result<(), ContextError> {
        fs::remove_file(self.get_path(key)).map_err(|e| {
            ContextError::PersistenceError(format!("Failed to delete file: {e}"))
        })
    }

    fn exists(&self, key: &str) -> bool {
        self.get_path(key).exists()
    }
}

pub struct JsonSerializer;

impl JsonSerializer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer for JsonSerializer {
    fn serialize_state(&self, state: &ContextState) -> Result<Vec<u8>, ContextError> {
        serde_json::to_vec(state).map_err(|e| {
            ContextError::PersistenceError(format!("State serialization failed: {e}"))
        })
    }

    fn deserialize_state(&self, data: &[u8]) -> Result<ContextState, ContextError> {
        serde_json::from_slice(data).map_err(|e| {
            ContextError::PersistenceError(format!("State deserialization failed: {e}"))
        })
    }

    fn serialize_snapshot(&self, snapshot: &ContextSnapshot) -> Result<Vec<u8>, ContextError> {
        serde_json::to_vec(snapshot).map_err(|e| {
            ContextError::PersistenceError(format!("Snapshot serialization failed: {e}"))
        })
    }

    fn deserialize_snapshot(&self, data: &[u8]) -> Result<ContextSnapshot, ContextError> {
        serde_json::from_slice(data).map_err(|e| {
            ContextError::PersistenceError(format!("Snapshot deserialization failed: {e}"))
        })
    }
}

#[derive(Debug)]
pub struct Cache {
    entries: std::collections::HashMap<String, CacheEntry>,
    max_size: usize,
    ttl: Duration,
}

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: std::time::SystemTime,
}

impl Cache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            entries: std::collections::HashMap::with_capacity(max_size),
            max_size,
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.entries.get(key).and_then(|entry| {
            if entry.expires_at > std::time::SystemTime::now() {
                Some(entry.data.as_slice())
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, key: String, data: Vec<u8>) {
        if self.entries.len() >= self.max_size {
            // Remove expired entries
            self.entries.retain(|_, entry| {
                entry.expires_at > std::time::SystemTime::now()
            });

            // If still at capacity, remove oldest entry
            if self.entries.len() >= self.max_size {
                if let Some(oldest_key) = self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.expires_at)
                    .map(|(key, _)| key.clone())
                {
                    self.entries.remove(&oldest_key);
                }
            }
        }

        self.entries.insert(key, CacheEntry {
            data,
            expires_at: std::time::SystemTime::now() + self.ttl,
        });
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.remove(key);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

pub struct ContextPersistence {
    storage: Box<dyn Storage>,
    serializer: Box<dyn Serializer>,
    cache: Cache,
}

impl ContextPersistence {
    pub fn new(
        storage: Box<dyn Storage>,
        serializer: Box<dyn Serializer>,
        cache_size: usize,
        cache_ttl: Duration,
    ) -> Self {
        Self {
            storage,
            serializer,
            cache: Cache::new(cache_size, cache_ttl),
        }
    }

    pub fn save_state(&mut self, state: &ContextState) -> Result<(), ContextError> {
        let key = format!("state_{}", state.version);
        let data = self.serializer.serialize_state(state)?;
        self.storage.save(&key, &data)?;
        self.cache.set(key, data);
        Ok(())
    }

    pub fn load_state(&self, version: u64) -> Result<ContextState, ContextError> {
        let key = format!("state_{version}");
        
        // Try cache first
        if let Some(data) = self.cache.get(&key) {
            return self.serializer.deserialize_state(data);
        }

        // Load from storage
        let data = self.storage.load(&key)?;
        self.serializer.deserialize_state(&data)
    }

    pub fn save_snapshot(&mut self, snapshot: &ContextSnapshot) -> Result<(), ContextError> {
        let serialized = self.serializer.serialize_snapshot(snapshot)
            .map_err(|e| ContextError::PersistenceError(e.to_string()))?;
        self.storage.save(&snapshot.id, &serialized)
            .map_err(|e| ContextError::PersistenceError(e.to_string()))
    }

    pub fn delete_snapshot(&mut self, id: &str) -> Result<(), ContextError> {
        self.storage.delete(id)
            .map_err(|e| ContextError::PersistenceError(e.to_string()))
    }

    pub fn load_snapshot(&self, id: &str) -> Result<ContextSnapshot, ContextError> {
        // Try cache first
        if let Some(data) = self.cache.get(id) {
            return self.serializer.deserialize_snapshot(data);
        }

        // Load from storage
        let data = self.storage.load(id)?;
        self.serializer.deserialize_snapshot(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_storage() {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf()).unwrap();

        // Test save and load
        let test_data = b"test data";
        assert!(storage.save("test_key", test_data).is_ok());
        assert!(storage.exists("test_key"));

        let loaded_data = storage.load("test_key").unwrap();
        assert_eq!(loaded_data, test_data);
    }
} 