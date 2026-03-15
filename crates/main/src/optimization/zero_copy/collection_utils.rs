// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Zero-copy collection utilities

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// `HashMap` that efficiently stores ``Arc<str>`` keys and values
pub type ZeroCopyMap<V> = HashMap<Arc<str>, V>;

/// `HashSet` that efficiently stores ``Arc<str>`` values
pub type ZeroCopySet = HashSet<Arc<str>>;

/// Extension trait for efficient `HashMap` operations
pub trait ZeroCopyMapExt<V> {
    /// Insert with `Arc<str>` key conversion
    fn insert_arc(&mut self, key: String, value: V) -> Option<V>;

    /// Get value by string slice without allocation
    fn get_str(&self, key: &str) -> Option<&V>;

    /// Check if key exists without allocation
    fn contains_str(&self, key: &str) -> bool;
}

impl<V> ZeroCopyMapExt<V> for HashMap<Arc<str>, V> {
    fn insert_arc(&mut self, key: String, value: V) -> Option<V> {
        let arc_key: Arc<str> = Arc::from(key);
        self.insert(arc_key, value)
    }

    fn get_str(&self, key: &str) -> Option<&V> {
        // Efficient lookup without Arc allocation
        self.iter().find(|(k, _)| k.as_ref() == key).map(|(_, v)| v)
    }

    fn contains_str(&self, key: &str) -> bool {
        self.keys().any(|k| k.as_ref() == key)
    }
}

/// Extension trait for efficient `HashSet` operations
pub trait ZeroCopySetExt {
    /// Insert with `Arc<str>` conversion
    fn insert_arc(&mut self, value: String) -> bool;

    /// Check if value exists without allocation
    fn contains_str(&self, value: &str) -> bool;
}

impl ZeroCopySetExt for HashSet<Arc<str>> {
    fn insert_arc(&mut self, value: String) -> bool {
        let arc_value: Arc<str> = Arc::from(value);
        self.insert(arc_value)
    }

    fn contains_str(&self, value: &str) -> bool {
        self.iter().any(|v| v.as_ref() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_map_insert_arc() {
        let mut map: ZeroCopyMap<i32> = HashMap::new();
        assert!(map.insert_arc("key1".to_string(), 42).is_none());
        assert_eq!(map.len(), 1);

        // Overwriting returns previous value
        let prev = map.insert_arc("key1".to_string(), 99);
        assert_eq!(prev, Some(42));
    }

    #[test]
    fn test_zero_copy_map_get_str() {
        let mut map: ZeroCopyMap<i32> = HashMap::new();
        map.insert_arc("hello".to_string(), 1);
        map.insert_arc("world".to_string(), 2);

        assert_eq!(map.get_str("hello"), Some(&1));
        assert_eq!(map.get_str("world"), Some(&2));
        assert_eq!(map.get_str("missing"), None);
    }

    #[test]
    fn test_zero_copy_map_contains_str() {
        let mut map: ZeroCopyMap<String> = HashMap::new();
        map.insert_arc("exists".to_string(), "value".to_string());

        assert!(map.contains_str("exists"));
        assert!(!map.contains_str("not_exists"));
    }

    #[test]
    fn test_zero_copy_set_insert_arc() {
        let mut set: ZeroCopySet = HashSet::new();
        assert!(set.insert_arc("item1".to_string()));
        assert!(set.insert_arc("item2".to_string()));

        // Duplicate insert returns false
        assert!(!set.insert_arc("item1".to_string()));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_zero_copy_set_contains_str() {
        let mut set: ZeroCopySet = HashSet::new();
        set.insert_arc("present".to_string());

        assert!(set.contains_str("present"));
        assert!(!set.contains_str("absent"));
    }

    #[test]
    fn test_zero_copy_map_multiple_types() {
        let mut map: ZeroCopyMap<Vec<u8>> = HashMap::new();
        map.insert_arc("data".to_string(), vec![1, 2, 3]);

        let val = map.get_str("data");
        assert_eq!(val, Some(&vec![1, 2, 3]));
    }

    #[test]
    fn test_zero_copy_set_empty() {
        let set: ZeroCopySet = HashSet::new();
        assert!(!set.contains_str("anything"));
        assert!(set.is_empty());
    }

    #[test]
    fn test_zero_copy_map_empty() {
        let map: ZeroCopyMap<i32> = HashMap::new();
        assert!(map.get_str("anything").is_none());
        assert!(!map.contains_str("anything"));
    }
}
