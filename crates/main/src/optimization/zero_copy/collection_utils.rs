//! Zero-copy collection utilities

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// HashMap that efficiently stores Arc<str> keys and values
pub type ZeroCopyMap<V> = HashMap<Arc<str>, V>;

/// HashSet that efficiently stores Arc<str> values
pub type ZeroCopySet = HashSet<Arc<str>>;

/// Extension trait for efficient HashMap operations
pub trait ZeroCopyMapExt<V> {
    /// Insert with Arc<str> key conversion
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

/// Extension trait for efficient HashSet operations
pub trait ZeroCopySetExt {
    /// Insert with Arc<str> conversion
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
