use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::ui::layout::{LayoutError, Rect, Size};
use std::time::Instant;

/// A key used to identify cached layout calculations.
#[derive(Debug, Clone, Eq)]
pub struct LayoutCacheKey {
    /// The available size for the layout.
    pub size: Size,
    /// The unique identifier of the component.
    pub component_id: String,
    /// Serialized constraints for the layout.
    pub constraints: Vec<u8>,
}

impl LayoutCacheKey {
    /// Creates a new cache key from a component ID, size, and constraints.
    ///
    /// # Arguments
    /// * `component_id` - The unique identifier for the component
    /// * `size` - The size of the component
    /// * `constraints` - The layout constraints for the component
    pub fn new(component_id: impl Into<String>, size: Size, constraints: &[u8]) -> Self {
        Self {
            component_id: component_id.into(),
            size,
            constraints: constraints.to_vec(),
        }
    }
}

impl PartialEq for LayoutCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.component_id == other.component_id &&
        self.size == other.size &&
        self.constraints == other.constraints
    }
}

impl Hash for LayoutCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.component_id.hash(state);
        self.size.hash(state);
        self.constraints.hash(state);
    }
}

/// A cache entry containing calculated layout information
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached rectangle dimensions and position.
    pub rect: Rect,
    /// The timestamp when this entry was created.
    pub timestamp: std::time::Instant,
}

impl CacheEntry {
    /// Create a new cache entry with the given rectangle
    #[must_use]
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            timestamp: Instant::now(),
        }
    }
}

/// A cache system for layout calculations to improve performance
#[derive(Debug, Clone)]
pub struct LayoutCache {
    /// The map of cache keys to their corresponding entries.
    entries: HashMap<LayoutCacheKey, CacheEntry>,
    /// The maximum number of entries to store in the cache.
    max_entries: usize,
    /// The time-to-live duration for cache entries.
    ttl: std::time::Duration,
}

impl LayoutCache {
    /// Creates a new layout cache with specified capacity and time-to-live.
    ///
    /// # Arguments
    ///
    /// * `max_entries` - Maximum number of entries to store in the cache
    /// * `ttl_secs` - Time-to-live in seconds for cache entries
    ///
    /// The cache will automatically remove entries that exceed their TTL when accessed
    /// and will limit the total number of entries to the specified maximum.
    pub fn new(max_entries: usize, ttl_secs: u64) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_entries,
            ttl: std::time::Duration::from_secs(ttl_secs),
        }
    }

    /// Gets a cached layout if available and not expired.
    /// 
    /// This method checks if a layout calculation exists in the cache for the given key
    /// and returns it if the entry hasn't expired.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The cache key to look up
    /// 
    /// # Returns
    /// 
    /// Returns `Some(Rect)` if a valid cached layout exists, `None` otherwise.
    pub fn get(&self, key: &LayoutCacheKey) -> Option<Rect> {
        if let Some(entry) = self.entries.get(key) {
            if entry.timestamp.elapsed() < self.ttl {
                return Some(entry.rect);
            }
        }
        None
    }

    /// Stores a layout calculation in the cache.
    /// 
    /// This method stores a layout calculation in the cache, automatically removing
    /// expired entries and the oldest entry if the cache is full.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The cache key for the layout calculation
    /// * `rect` - The calculated layout rectangle to store
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the layout was successfully cached, or a `LayoutError` if
    /// there was an error during the operation.
    pub fn insert(&mut self, key: LayoutCacheKey, rect: Rect) -> Result<(), LayoutError> {
        // Remove expired entries first
        self.cleanup();

        // Check if we need to remove old entries to make space
        if self.entries.len() >= self.max_entries {
            // Remove the oldest entry
            if let Some((oldest_key, _)) = self.entries
                .iter()
                .min_by_key(|(_, entry)| entry.timestamp)
            {
                let oldest_key = oldest_key.clone();
                self.entries.remove(&oldest_key);
            }
        }

        self.entries.insert(key, CacheEntry {
            rect,
            timestamp: std::time::Instant::now(),
        });

        Ok(())
    }

    /// Removes expired entries from the cache.
    /// 
    /// This method removes all cache entries that have exceeded their time-to-live.
    /// It is called automatically when inserting new entries.
    pub fn cleanup(&mut self) {
        let _now = Instant::now();
        self.entries.retain(|_, entry| {
            entry.timestamp.elapsed() < self.ttl
        });
    }

    /// Clears all entries from the cache.
    /// 
    /// This method removes all entries from the cache, regardless of their expiration status.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Gets the number of entries in the cache.
    /// 
    /// # Returns
    /// 
    /// Returns the current number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the cache is empty.
    /// 
    /// # Returns
    /// 
    /// Returns `true` if the cache contains no entries, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Sets a new time-to-live for cache entries.
    /// 
    /// # Arguments
    /// 
    /// * `ttl_secs` - The new time-to-live in seconds
    /// 
    /// This method updates the TTL and immediately removes any expired entries.
    pub fn set_ttl(&mut self, ttl_secs: u64) {
        self.ttl = std::time::Duration::from_secs(ttl_secs);
        self.cleanup();
    }

    /// Sets a new maximum number of entries for the cache.
    /// 
    /// # Arguments
    /// 
    /// * `max_entries` - The new maximum number of entries
    /// 
    /// If the new maximum is less than the current number of entries, the oldest
    /// entries are removed until the cache size is within the new limit.
    pub fn set_max_entries(&mut self, max_entries: usize) {
        self.max_entries = max_entries;
        while self.entries.len() > self.max_entries {
            if let Some((oldest_key, _)) = self.entries
                .iter()
                .min_by_key(|(_, entry)| entry.timestamp)
            {
                let oldest_key = oldest_key.clone();
                self.entries.remove(&oldest_key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layout::Position;

    #[test]
    fn test_cache_insertion() {
        let mut cache = LayoutCache::new(10, 60);
        let key = LayoutCacheKey {
            size: Size::new(100, 100),
            component_id: "test".to_string(),
            constraints: vec![1, 2, 3],
        };
        let rect = Rect::new(Position::new(0, 0), Size::new(50, 50));
        assert!(cache.insert(key.clone(), rect).is_ok());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_retrieval() {
        let mut cache = LayoutCache::new(10, 60);
        let key = LayoutCacheKey {
            size: Size::new(100, 100),
            component_id: "test".to_string(),
            constraints: vec![1, 2, 3],
        };
        let rect = Rect::new(Position::new(0, 0), Size::new(50, 50));
        assert!(cache.insert(key.clone(), rect).is_ok());

        let cached_rect = cache.get(&key);
        assert!(cached_rect.is_some());
        assert_eq!(cached_rect.unwrap(), rect);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = LayoutCache::new(10, 0); // 0 second TTL for testing
        let key = LayoutCacheKey {
            size: Size::new(100, 100),
            component_id: "test".to_string(),
            constraints: vec![1, 2, 3],
        };
        let rect = Rect::new(Position::new(0, 0), Size::new(50, 50));
        assert!(cache.insert(key.clone(), rect).is_ok());

        // Sleep briefly to ensure the entry expires
        std::thread::sleep(std::time::Duration::from_millis(10));

        let cached_rect = cache.get(&key);
        assert!(cached_rect.is_none());
    }

    #[test]
    fn test_cache_cleanup() {
        let mut cache = LayoutCache::new(10, 0); // 0 second TTL for testing
        let key = LayoutCacheKey {
            size: Size::new(100, 100),
            component_id: "test".to_string(),
            constraints: vec![1, 2, 3],
        };
        let rect = Rect::new(Position::new(0, 0), Size::new(50, 50));
        assert!(cache.insert(key.clone(), rect).is_ok());

        // Sleep briefly to ensure the entry expires
        std::thread::sleep(std::time::Duration::from_millis(10));

        cache.cleanup();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_max_entries() {
        let mut cache = LayoutCache::new(2, 60);
        
        // Insert 3 entries (exceeding max_entries)
        for i in 0..3 {
            let key = LayoutCacheKey {
                size: Size::new(100, 100),
                component_id: format!("test{}", i),
                constraints: vec![i as u8],
            };
            let rect = Rect::new(Position::new(0, 0), Size::new(50, 50));
            assert!(cache.insert(key.clone(), rect).is_ok());
        }

        assert_eq!(cache.len(), 2);
    }
} 