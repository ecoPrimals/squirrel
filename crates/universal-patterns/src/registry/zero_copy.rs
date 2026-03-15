// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Zero-copy optimizations for the universal registry
//!
//! This module provides `Arc<str>`-based alternatives to String for frequently cloned
//! identifiers, reducing memory allocations and improving performance.

use std::collections::HashMap;
use std::sync::Arc;

/// String cache for common registry identifiers
///
 Maintains a cache of ``Arc<str>`` instances for frequently used strings like
/// primal types, capabilities, and instance IDs. This eliminates repeated
/// string allocations in hot paths.
#[derive(Debug, Clone, Default)]
pub struct RegistryStringCache {
     Cached ``Arc<str>`` instances
    cache: Arc<std::sync::RwLock<HashMap<String, Arc<str>>>>,
}

impl RegistryStringCache {
    /// Create a new string cache with common values pre-populated
    pub fn new() -> Self {
        let cache = HashMap::new();
        Self {
            cache: Arc::new(std::sync::RwLock::new(cache)),
        }
    }

    /// Pre-populate with common registry values
    pub fn with_common_values(self) -> Self {
        let common_values = [
            // Primal types
            "squirrel",
            "nestgate",
            "toadstool",
            "beardog",
            "songbird",
            "biomeos",
            // Common capabilities
            "mcp",
            "ai",
            "storage",
            "compute",
            "security",
            "orchestration",
            // Status values
            "running",
            "stopped",
            "initializing",
            "healthy",
            "unhealthy",
        ];

        if let Ok(mut cache) = self.cache.write() {
            for value in &common_values {
                let arc_str: Arc<str> = Arc::from(*value);
                cache.insert(value.to_string(), arc_str);
            }
        }

        self
    }

     Get or create an ``Arc<str>`` for the given string
    ///
    /// This is the main API for zero-copy string handling. If the string is already
     cached, returns the cached ``Arc<str>``. Otherwise, creates and caches a new one.
    ///
    /// # Performance
    ///
    /// - Cache hits: O(1) HashMap lookup + Arc::clone (just pointer increment)
    /// - Cache miss: O(1) HashMap insertion + String allocation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use universal_patterns::registry::zero_copy::RegistryStringCache;
    ///
    /// let cache = RegistryStringCache::new().with_common_values();
    ///
    /// // Fast path - cached value
    /// let primal_type = cache.get_or_create("squirrel");
    ///
    /// // Still fast - caches new value
    /// let custom_id = cache.get_or_create("custom-instance-123");
    /// ```
    pub fn get_or_create(&self, key: &str) -> Arc<str> {
        // Try read lock first (fast path for cache hits)
        if let Ok(cache) = self.cache.read() {
            if let Some(cached) = cache.get(key) {
                return Arc::clone(cached);
            }
        }

        // Cache miss - need write lock
        if let Ok(mut cache) = self.cache.write() {
            // Double-check after acquiring write lock (another thread may have inserted)
            if let Some(cached) = cache.get(key) {
                return Arc::clone(cached);
            }

            // Create and cache new Arc<str>
            let arc_str: Arc<str> = Arc::from(key);
            cache.insert(key.to_string(), Arc::clone(&arc_str));
            arc_str
        } else {
            // Fallback if lock is poisoned - create without caching
            Arc::from(key)
        }
    }

    /// Get a cached string without creating it
    ///
     Returns `Some(`Arc<str>`)` if cached, `None` otherwise.
    /// Useful for checking if a string is already interned.
    pub fn get(&self, key: &str) -> Option<Arc<str>> {
        self.cache.read().ok()?.get(key).cloned()
    }

    /// Pre-cache multiple strings at once
    ///
    /// Efficiently caches multiple strings in a single write lock acquisition.
    /// Useful during initialization to populate the cache with known values.
    pub fn pre_cache<I, S>(&self, keys: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        if let Ok(mut cache) = self.cache.write() {
            for key in keys {
                let key_str = key.as_ref();
                if !cache.contains_key(key_str) {
                    let arc_str: Arc<str> = Arc::from(key_str);
                    cache.insert(key_str.to_string(), arc_str);
                }
            }
        }
    }

    /// Get the number of cached strings
    pub fn len(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear the cache
    ///
     Removes all cached strings. Note that ``Arc<str>`` instances still in use
    /// elsewhere will remain valid due to Arc reference counting.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}

/// Zero-copy instance ID wrapper
///
 Provides an efficient representation of instance IDs using ``Arc<str>``
/// instead of String. This eliminates cloning overhead when passing instance
/// IDs around the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstanceId(Arc<str>);

impl InstanceId {
    /// Create a new instance ID from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(Arc::from(id.into().as_str()))
    }

     Create from an existing ``Arc<str>``
    pub fn from_arc(arc: Arc<str>) -> Self {
        Self(arc)
    }

     Get the underlying ``Arc<str>``
    pub fn as_arc(&self) -> &Arc<str> {
        &self.0
    }

    /// Get the string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for InstanceId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for InstanceId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<Arc<str>> for InstanceId {
    fn from(arc: Arc<str>) -> Self {
        Self::from_arc(arc)
    }
}

impl AsRef<str> for InstanceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Zero-copy capability name wrapper
///
/// Similar to InstanceId but specifically for capability names.
/// Provides type safety and zero-copy cloning.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityName(Arc<str>);

impl CapabilityName {
    /// Create a new capability name
    pub fn new(name: impl Into<String>) -> Self {
        Self(Arc::from(name.into().as_str()))
    }

     Create from an existing ``Arc<str>``
    pub fn from_arc(arc: Arc<str>) -> Self {
        Self(arc)
    }

     Get the underlying ``Arc<str>``
    pub fn as_arc(&self) -> &Arc<str> {
        &self.0
    }

    /// Get the string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CapabilityName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CapabilityName {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for CapabilityName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<Arc<str>> for CapabilityName {
    fn from(arc: Arc<str>) -> Self {
        Self::from_arc(arc)
    }
}

impl AsRef<str> for CapabilityName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_cache_basic() {
        let cache = RegistryStringCache::new();

        let id1 = cache.get_or_create("test-id");
        let id2 = cache.get_or_create("test-id");

        // Same Arc<str> instance (pointer equality)
        assert!(Arc::ptr_eq(&id1, &id2));
        assert_eq!(id1.as_ref(), "test-id");
    }

    #[test]
    fn test_string_cache_common_values() {
        let cache = RegistryStringCache::new().with_common_values();

        let squirrel = cache.get("squirrel");
        assert!(squirrel.is_some());
        assert_eq!(squirrel.unwrap().as_ref(), "squirrel");
    }

    #[test]
    fn test_instance_id_creation() {
        let id1 = InstanceId::new("instance-123");
        let id2 = InstanceId::new("instance-123".to_string());

        assert_eq!(id1.as_str(), "instance-123");
        assert_eq!(id2.as_str(), "instance-123");
    }

    #[test]
    fn test_instance_id_display() {
        let id = InstanceId::new("test-instance");
        assert_eq!(format!("{}", id), "test-instance");
    }

    #[test]
    fn test_capability_name() {
        let cap = CapabilityName::new("mcp");
        assert_eq!(cap.as_str(), "mcp");
    }

    #[test]
    fn test_pre_cache() {
        let cache = RegistryStringCache::new();
        let values = vec!["key1", "key2", "key3"];

        cache.pre_cache(values);

        assert_eq!(cache.len(), 3);
        assert!(cache.get("key1").is_some());
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_cache_clear() {
        let cache = RegistryStringCache::new();
        cache.get_or_create("test");

        assert!(!cache.is_empty());
        cache.clear();
        assert!(cache.is_empty());
    }
}
