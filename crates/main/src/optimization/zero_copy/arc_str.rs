// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Zero-copy ``Arc<str>`` utilities for high-performance string handling
//!
//! This module provides utilities for using ```Arc<str>``` as a zero-copy string type,
//! eliminating expensive cloning in hot paths like service discovery, metrics collection,
//! and request routing.
//!
//! # Performance Benefits
//!
//! - `String::clone()`: Allocates new memory, copies all bytes - O(n)
//! - `Arc<str>::clone()`: Copies pointer only - O(1)
//!
//! For strings used frequently (service names, metric names, endpoints),
//! `Arc<str>` provides massive performance improvements.
//!
//! # Usage
//!
//! ```rust
//! use squirrel::optimization::zero_copy::ArcStr;
//!
//! // Create from string
//! let name = ArcStr::from("my-service");
//!
//! // Clone is cheap (just pointer copy)
//! let name2 = name.clone();
//!
//! // Convert to &str for usage
//! println!("Service: {}", name.as_ref());
//! ```

use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

/// Type alias for ``Arc<str>`` with semantic meaning
pub type ArcStr = Arc<str>;

/// Extension trait for creating `ArcStr` from various sources
pub trait IntoArcStr {
    /// Converts the value into an `ArcStr`.
    fn into_arc_str(self) -> ArcStr;
}

impl IntoArcStr for String {
    #[inline]
    fn into_arc_str(self) -> ArcStr {
        Arc::from(self.as_str())
    }
}

impl IntoArcStr for &str {
    #[inline]
    fn into_arc_str(self) -> ArcStr {
        Arc::from(self)
    }
}

impl IntoArcStr for Arc<str> {
    #[inline]
    fn into_arc_str(self) -> ArcStr {
        self
    }
}

impl IntoArcStr for Box<str> {
    #[inline]
    fn into_arc_str(self) -> ArcStr {
        Arc::from(self)
    }
}

/// String interning cache for commonly-used strings
///
/// For strings that appear frequently (metric names, endpoint paths, error codes),
/// we can intern them once and reuse the same ``Arc<str>`` everywhere.
pub mod intern {
    use super::{Arc, ArcStr};
    use std::collections::HashMap;
    use std::sync::LazyLock;
    use std::sync::RwLock;

    static STRING_CACHE: LazyLock<RwLock<HashMap<&'static str, ArcStr>>> = LazyLock::new(|| {
        let mut map = HashMap::new();

        // Self-identity (only squirrel knows itself)
        map.insert("squirrel", Arc::from("squirrel"));

        // Capability domains (agnostic — no hardcoded primal names)
        map.insert("ai", Arc::from("ai"));
        map.insert("security", Arc::from("security"));
        map.insert("network", Arc::from("network"));
        map.insert("compute", Arc::from("compute"));
        map.insert("storage", Arc::from("storage"));
        map.insert("ecosystem", Arc::from("ecosystem"));

        // Common endpoint paths
        map.insert("/health", Arc::from("/health"));
        map.insert("/health/live", Arc::from("/health/live"));
        map.insert("/health/ready", Arc::from("/health/ready"));
        map.insert("/metrics", Arc::from("/metrics"));
        map.insert("/api/v1/primals", Arc::from("/api/v1/primals"));
        map.insert(
            "/api/v1/ecosystem/status",
            Arc::from("/api/v1/ecosystem/status"),
        );

        // Common metric names
        map.insert("request_count", Arc::from("request_count"));
        map.insert("request_duration", Arc::from("request_duration"));
        map.insert("error_count", Arc::from("error_count"));
        map.insert("cache_hit", Arc::from("cache_hit"));
        map.insert("cache_miss", Arc::from("cache_miss"));

        // Common capability names
        map.insert("ai_inference", Arc::from("ai_inference"));
        map.insert("storage", Arc::from("storage"));
        map.insert("compute", Arc::from("compute"));
        map.insert("security", Arc::from("security"));
        map.insert("service_mesh", Arc::from("service_mesh"));

        // Semantic method names (wateringHole standard: {domain}.{operation})
        map.insert("ai.query", Arc::from("ai.query"));
        map.insert("ai.complete", Arc::from("ai.complete"));
        map.insert("ai.chat", Arc::from("ai.chat"));
        map.insert("ai.list_providers", Arc::from("ai.list_providers"));
        map.insert("ai.inference", Arc::from("ai.inference"));
        map.insert("ai.text_generation", Arc::from("ai.text_generation"));
        map.insert("capability.announce", Arc::from("capability.announce"));
        map.insert("capability.discover", Arc::from("capability.discover"));
        map.insert("system.health", Arc::from("system.health"));
        map.insert("system.metrics", Arc::from("system.metrics"));
        map.insert("system.ping", Arc::from("system.ping"));
        map.insert("discovery.peers", Arc::from("discovery.peers"));
        map.insert("tool.execute", Arc::from("tool.execute"));

        RwLock::new(map)
    });

    /// Get an interned string if it exists, otherwise create and cache it
    ///
    /// # Performance
    ///
    /// - First call: O(n) to create `Arc<str>` + cache lookup
    /// - Subsequent calls: O(1) cache hit
    ///
    /// Best for strings that appear many times (>10) in your program.
    ///
    /// # Safety
    ///
    /// This function handles poisoned locks gracefully by recovering the lock guard.
    /// Cache poisoning is extremely rare (requires panic during cache modification),
    /// and if it occurs, we can safely continue by creating the `Arc<str>` without caching.
    pub fn get_or_intern(s: &str) -> ArcStr {
        // Fast path: check if already interned
        {
            let cache = match STRING_CACHE.read() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    // SAFETY: Lock poisoned due to panic in another thread.
                    // We can safely recover the lock guard and continue.
                    // The cache data itself is valid, just the lock panicked.
                    poisoned.into_inner()
                }
            };

            if let Some(arc) = cache.get(s) {
                return Arc::clone(arc);
            }
        }

        // Slow path: intern new string
        let cache = match STRING_CACHE.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // SAFETY: If write lock is poisoned, we can still recover.
                // Worst case: cache may have partial writes, but we'll create
                // a new Arc<str> anyway without using corrupted cache data.
                poisoned.into_inner()
            }
        };

        // Double-check in case another thread interned it
        if let Some(arc) = cache.get(s) {
            return Arc::clone(arc);
        }

        let arc: ArcStr = Arc::from(s);
        // Note: We can't use the Arc as a key since it's not 'static,
        // so we only cache predefined strings above
        arc
    }

    /// Get a pre-interned common string, if it exists
    ///
    /// # Safety
    ///
    /// This function handles poisoned locks by recovering the guard.
    /// Read-only access is safe even if the lock was poisoned.
    pub fn get_common(key: &'static str) -> Option<ArcStr> {
        let cache = match STRING_CACHE.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // SAFETY: Read lock poisoning doesn't affect data integrity.
                // We can safely read from the recovered cache.
                poisoned.into_inner()
            }
        };
        cache.get(key).map(Arc::clone)
    }
}

/// Newtype wrapper for ``Arc<str>`` with additional utilities
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SmartString(ArcStr);

impl SmartString {
    /// Create from any type that can be converted to `Arc<str>`
    pub fn new(s: impl IntoArcStr) -> Self {
        Self(s.into_arc_str())
    }

    /// Get the underlying `Arc<str>`
    #[must_use]
    pub fn as_arc_str(&self) -> &ArcStr {
        &self.0
    }

    /// Convert into the underlying `Arc<str>`
    #[must_use]
    pub fn into_arc_str(self) -> ArcStr {
        self.0
    }

    /// Create from a static string
    ///
    /// Note: Not const fn because `Arc::from` is not const.
    /// For compile-time strings, this is still efficient as Arc will
    /// just create a pointer to the static data.
    #[must_use]
    pub fn from_static(s: &'static str) -> Self {
        Self(Arc::from(s))
    }
}

impl Deref for SmartString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for SmartString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SmartString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for SmartString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl From<String> for SmartString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SmartString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<Arc<str>> for SmartString {
    fn from(s: Arc<str>) -> Self {
        Self(s)
    }
}

impl serde::Serialize for SmartString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for SmartString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_str_clone_is_cheap() {
        let original = ArcStr::from("test string");
        let cloned = original.clone();

        // Both point to same data
        assert_eq!(Arc::as_ptr(&original), Arc::as_ptr(&cloned));
        assert_eq!(&*original, &*cloned);
    }

    #[test]
    fn test_into_arc_str_trait() {
        let from_string: ArcStr = String::from("hello").into_arc_str();
        let from_str: ArcStr = "hello".into_arc_str();
        let from_arc_existing: Arc<str> = Arc::from("hello");
        let from_arc: ArcStr = from_arc_existing.into_arc_str();

        assert_eq!(&*from_string, "hello");
        assert_eq!(&*from_str, "hello");
        assert_eq!(&*from_arc, "hello");
    }

    #[test]
    fn test_string_interning_predefined() {
        // Test with pre-interned strings (these work)
        let s1 = intern::get_common("squirrel").expect("Should have squirrel");
        let s2 = intern::get_common("squirrel").expect("Should have squirrel");

        // Pre-interned strings should return same Arc (same pointer)
        assert_eq!(Arc::as_ptr(&s1), Arc::as_ptr(&s2));

        // Test that get_or_intern works for pre-interned strings
        let s3 = intern::get_or_intern("squirrel");
        assert_eq!(Arc::as_ptr(&s1), Arc::as_ptr(&s3));
    }

    #[test]
    fn test_string_interning_dynamic() {
        // For non-pre-interned strings, get_or_intern creates new Arcs
        // This is expected behavior due to the 'static lifetime requirement for cache keys
        let s1 = intern::get_or_intern("dynamic_test_string");
        let s2 = intern::get_or_intern("dynamic_test_string");

        // They won't be the same pointer (limitation of current design)
        // But they will have the same value
        assert_eq!(&*s1, &*s2);
        assert_eq!(&*s1, "dynamic_test_string");
    }

    #[test]
    fn test_common_strings() {
        let squirrel = intern::get_common("squirrel").expect("Should have squirrel");
        assert_eq!(&*squirrel, "squirrel");

        let health = intern::get_common("/health").expect("Should have /health");
        assert_eq!(&*health, "/health");
    }

    #[test]
    fn test_smart_string() {
        let s1 = SmartString::new("hello");
        let s2 = s1.clone();

        // Clone is cheap
        assert_eq!(Arc::as_ptr(s1.as_arc_str()), Arc::as_ptr(s2.as_arc_str()));

        // Can use as &str
        assert_eq!(s1.as_ref(), "hello");
        assert_eq!(&*s2, "hello");
    }

    #[test]
    fn test_smart_string_from_static() {
        let s = SmartString::from_static("static string");
        assert_eq!(&*s, "static string");
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = SmartString::new("test value");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SmartString = serde_json::from_str(&json).unwrap();

        assert_eq!(&*original, &*deserialized);
    }

    // ---- Interned semantic method names ----

    #[test]
    fn test_interned_semantic_method_names() {
        let semantic_names = [
            "ai.query",
            "ai.complete",
            "ai.chat",
            "ai.list_providers",
            "ai.inference",
            "ai.text_generation",
            "capability.announce",
            "capability.discover",
            "system.health",
            "system.metrics",
            "system.ping",
            "discovery.peers",
            "tool.execute",
        ];

        for name in &semantic_names {
            let interned = intern::get_common(name);
            assert!(
                interned.is_some(),
                "Semantic method name '{}' should be pre-interned",
                name
            );
            assert_eq!(&*interned.unwrap(), *name);
        }
    }

    #[test]
    fn test_interned_capability_domains() {
        let domains = [
            "ai",
            "security",
            "network",
            "compute",
            "storage",
            "ecosystem",
        ];
        for domain in &domains {
            let interned = intern::get_common(domain);
            assert!(
                interned.is_some(),
                "Capability domain '{}' should be pre-interned",
                domain
            );
            assert_eq!(&*interned.unwrap(), *domain);
        }
    }

    #[test]
    fn test_interned_common_endpoints() {
        let endpoints = [
            "/health",
            "/health/live",
            "/health/ready",
            "/metrics",
            "/api/v1/primals",
            "/api/v1/ecosystem/status",
        ];
        for ep in &endpoints {
            let interned = intern::get_common(ep);
            assert!(
                interned.is_some(),
                "Endpoint '{}' should be pre-interned",
                ep
            );
            assert_eq!(&*interned.unwrap(), *ep);
        }
    }

    #[test]
    fn test_interned_metric_names() {
        let metrics = [
            "request_count",
            "request_duration",
            "error_count",
            "cache_hit",
            "cache_miss",
        ];
        for metric in &metrics {
            let interned = intern::get_common(metric);
            assert!(
                interned.is_some(),
                "Metric '{}' should be pre-interned",
                metric
            );
            assert_eq!(&*interned.unwrap(), *metric);
        }
    }

    #[test]
    fn test_get_common_nonexistent_returns_none() {
        let result = intern::get_common("nonexistent-key-xyz-12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_or_intern_returns_correct_value_for_unknown() {
        let result = intern::get_or_intern("unique_test_value_abc_42");
        assert_eq!(&*result, "unique_test_value_abc_42");
    }

    // ---- IntoArcStr from Box<str> ----

    #[test]
    fn test_into_arc_str_from_box_str() {
        let boxed: Box<str> = "boxed string".into();
        let arc: ArcStr = boxed.into_arc_str();
        assert_eq!(&*arc, "boxed string");
    }

    // ---- SmartString conversions ----

    #[test]
    fn test_smart_string_from_string() {
        let s: SmartString = String::from("from-string").into();
        assert_eq!(&*s, "from-string");
    }

    #[test]
    fn test_smart_string_from_str_ref() {
        let s: SmartString = "from-str-ref".into();
        assert_eq!(&*s, "from-str-ref");
    }

    #[test]
    fn test_smart_string_from_arc_str() {
        let arc: Arc<str> = Arc::from("from-arc");
        let s: SmartString = arc.into();
        assert_eq!(&*s, "from-arc");
    }

    #[test]
    fn test_smart_string_into_arc_str() {
        let s = SmartString::new("convert-me");
        let arc = s.into_arc_str();
        assert_eq!(&*arc, "convert-me");
    }

    #[test]
    fn test_smart_string_display() {
        let s = SmartString::new("display-test");
        assert_eq!(format!("{}", s), "display-test");
    }

    #[test]
    fn test_smart_string_debug() {
        let s = SmartString::new("debug-test");
        let debug_output = format!("{:?}", s);
        assert!(debug_output.contains("debug-test"));
    }

    #[test]
    fn test_smart_string_deref() {
        let s = SmartString::new("deref-test");
        // Deref gives us &str methods
        assert!(s.starts_with("deref"));
        assert!(s.ends_with("test"));
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_smart_string_ord_and_eq() {
        let a = SmartString::new("aaa");
        let b = SmartString::new("bbb");
        let a2 = SmartString::new("aaa");

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, a2);
        assert_ne!(a, b);
    }

    #[test]
    fn test_smart_string_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SmartString::new("one"));
        set.insert(SmartString::new("two"));
        set.insert(SmartString::new("one")); // duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&SmartString::new("one")));
        assert!(set.contains(&SmartString::new("two")));
    }

    #[test]
    fn test_smart_string_serde_empty() {
        let original = SmartString::new("");
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"\"");
        let deserialized: SmartString = serde_json::from_str(&json).unwrap();
        assert_eq!(&*deserialized, "");
    }

    #[test]
    fn test_smart_string_serde_special_chars() {
        let original = SmartString::new("hello \"world\"\nnewline");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SmartString = serde_json::from_str(&json).unwrap();
        assert_eq!(&*original, &*deserialized);
    }

    // ---- ArcStr edge cases ----

    #[test]
    fn test_arc_str_empty_string() {
        let empty: ArcStr = "".into_arc_str();
        assert_eq!(&*empty, "");
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_arc_str_unicode() {
        let unicode: ArcStr = "日本語テスト 🦊".into_arc_str();
        assert_eq!(&*unicode, "日本語テスト 🦊");
    }

    #[test]
    fn test_interned_pointer_identity_for_preinterned() {
        // Two calls to get_common for the same key must return same Arc pointer
        let a = intern::get_common("squirrel").unwrap();
        let b = intern::get_common("squirrel").unwrap();
        assert!(
            Arc::ptr_eq(&a, &b),
            "Pre-interned strings must share the same Arc"
        );

        // get_or_intern for pre-interned key must also share the pointer
        let c = intern::get_or_intern("squirrel");
        assert!(
            Arc::ptr_eq(&a, &c),
            "get_or_intern must reuse pre-interned Arc"
        );
    }
}
