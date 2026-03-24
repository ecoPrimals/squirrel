// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Zero-copy string utilities using Arc<str>
//!
//! This module provides efficient string handling that minimizes cloning
//! and memory allocations by using reference-counted string slices.

use std::sync::Arc;

/// A zero-copy string type that can be cheaply cloned
pub type ZeroCopyStr = Arc<str>;

 Create a zero-copy string from a `/// Create a zero-copy string from a &strstr`
#[inline]
pub fn zc_str(s: &str) -> ZeroCopyStr {
    Arc::from(s)
}

/// Create a zero-copy string from a String
#[inline]
pub fn zc_string(s: String) -> ZeroCopyStr {
    Arc::from(s.into_boxed_str())
}

/// Zero-copy string cache for commonly used strings
pub struct StringCache {
    cache: std::collections::HashMap<String, ZeroCopyStr>,
}

impl StringCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    /// Get or insert a string in the cache
    pub fn get_or_insert(&mut self, s: &str) -> ZeroCopyStr {
        if let Some(cached) = self.cache.get(s) {
            Arc::clone(cached)
        } else {
            let zc = zc_str(s);
            self.cache.insert(s.to_string(), Arc::clone(&zc));
            zc
        }
    }

    /// Pre-populate cache with common strings
    pub fn with_common_strings() -> Self {
        let mut cache = Self::new();

        // Common status strings
        for s in &["active", "inactive", "pending", "failed", "success"] {
            cache.get_or_insert(s);
        }

        // Primal self-identity and capability domains
        for s in &["squirrel", "ai", "security", "network", "compute", "storage"] {
            cache.get_or_insert(s);
        }

        cache
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for StringCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zc_str_creation() {
        let s = zc_str("hello");
        assert_eq!(&*s, "hello");
    }

    #[test]
    fn test_zc_string_creation() {
        let s = zc_string("world".to_string());
        assert_eq!(&*s, "world");
    }

    #[test]
    fn test_zc_str_cheap_clone() {
        let s1 = zc_str("test");
        let s2 = Arc::clone(&s1);

        assert_eq!(&*s1, &*s2);
        assert_eq!(Arc::strong_count(&s1), 2);
    }

    #[test]
    fn test_string_cache() {
        let mut cache = StringCache::new();

        let s1 = cache.get_or_insert("hello");
        let s2 = cache.get_or_insert("hello");

        assert_eq!(&*s1, &*s2);
        assert_eq!(Arc::strong_count(&s1), 3); // cache + s1 + s2
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_with_common_strings() {
        let cache = StringCache::with_common_strings();
        assert!(cache.len() >= 10); // At least 10 common strings
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = StringCache::with_common_strings();
        assert!(!cache.is_empty());

        cache.clear();
        assert!(cache.is_empty());
    }
}
