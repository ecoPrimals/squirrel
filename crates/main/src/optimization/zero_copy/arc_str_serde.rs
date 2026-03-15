// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Arc<str> wrapper with serde support for zero-copy string optimization
//!
//! This module provides `ArcStr`, a wrapper around `Arc<str>` that implements
//! serde serialization/deserialization, enabling zero-copy string patterns
//! throughout the codebase while maintaining compatibility with JSON APIs.
//!
//! ## Usage
//! ```rust
//! use squirrel_primal::optimization::zero_copy::ArcStr;
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Config {
//!     name: ArcStr,
//!     endpoint: ArcStr,
//! }
//!
//! let config = Config {
//!     name: ArcStr::from("service"),
//!     endpoint: ArcStr::from("http://localhost:8080"),
//! };
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

/// Zero-copy string type with serde support
///
/// This type wraps `Arc<str>` and provides automatic serialization/deserialization,
/// enabling efficient string sharing across the application while maintaining
/// compatibility with JSON APIs and configuration files.
///
/// # Benefits
/// - **Zero-copy cloning**: Cloning is O(1) atomic increment
/// - **Memory efficient**: Shared ownership with automatic cleanup
/// - **Serde compatible**: Works with JSON, TOML, YAML serialization
/// - **Type safe**: Strong typing prevents accidental conversions
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArcStr(Arc<str>);

impl ArcStr {
    /// Create a new ArcStr from a string slice
    #[inline]
    pub fn new(s: &str) -> Self {
        Self(Arc::from(s))
    }

    /// Get the underlying Arc<str>
    #[inline]
    pub fn as_arc(&self) -> &Arc<str> {
        &self.0
    }

    /// Convert into the underlying Arc<str>
    #[inline]
    pub fn into_arc(self) -> Arc<str> {
        self.0
    }

    /// Get the string slice
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the length of the string
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the string is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// Deref to str for easy usage
impl Deref for ArcStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// AsRef<str> for generic string operations
impl AsRef<str> for ArcStr {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// From conversions
impl From<String> for ArcStr {
    #[inline]
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

impl From<&str> for ArcStr {
    #[inline]
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl From<Arc<str>> for ArcStr {
    #[inline]
    fn from(arc: Arc<str>) -> Self {
        Self(arc)
    }
}

impl From<ArcStr> for Arc<str> {
    #[inline]
    fn from(arc_str: ArcStr) -> Self {
        arc_str.0
    }
}

// Display for formatting
impl fmt::Display for ArcStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &*self.0)
    }
}

// PartialEq with &str for easy comparisons
impl PartialEq<str> for ArcStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        &*self.0 == other
    }
}

impl PartialEq<&str> for ArcStr {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &*self.0 == *other
    }
}

impl PartialEq<String> for ArcStr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        &*self.0 == other
    }
}

// Allow HashMap lookups with &str when key is ArcStr
impl std::borrow::Borrow<str> for ArcStr {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::hash::Hash for ArcStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

// Serde support
impl Serialize for ArcStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ArcStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ArcStr::from(s))
    }
}

// Default implementation
impl Default for ArcStr {
    fn default() -> Self {
        ArcStr::from("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_arcstr_creation() {
        let s = ArcStr::from("test");
        assert_eq!(s.as_str(), "test");
        assert_eq!(s.len(), 4);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_arcstr_empty() {
        let s = ArcStr::default();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_arcstr_clone() {
        let s1 = ArcStr::from("test");
        let s2 = s1.clone();

        assert_eq!(s1, s2);
        // Verify they share the same Arc (pointer equality)
        assert!(Arc::ptr_eq(s1.as_arc(), s2.as_arc()));
    }

    #[test]
    fn test_arcstr_comparisons() {
        let s = ArcStr::from("test");

        assert_eq!(s, "test");
        assert_eq!(s, "test"); // Compare with str directly
        assert_eq!(s, String::from("test"));
        assert_ne!(s, "other");
    }

    #[test]
    fn test_arcstr_display() {
        let s = ArcStr::from("test");
        assert_eq!(format!("{}", s), "test");
    }

    #[test]
    fn test_arcstr_deref() {
        let s = ArcStr::from("test");
        let len: usize = s.len(); // Uses Deref to str
        assert_eq!(len, 4);
    }

    #[test]
    fn test_arcstr_serialize() {
        let s = ArcStr::from("test");
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"test\"");
    }

    #[test]
    fn test_arcstr_deserialize() {
        let json = "\"test\"";
        let s: ArcStr = serde_json::from_str(json).unwrap();
        assert_eq!(s, "test");
    }

    #[test]
    fn test_arcstr_roundtrip() {
        let original = ArcStr::from("hello world");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ArcStr = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        name: ArcStr,
        description: ArcStr,
    }

    #[test]
    fn test_arcstr_in_struct() {
        let data = TestStruct {
            name: ArcStr::from("service"),
            description: ArcStr::from("A test service"),
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: TestStruct = serde_json::from_str(&json).unwrap();

        assert_eq!(data, parsed);
    }

    #[test]
    fn test_arcstr_from_conversions() {
        // From &str
        let s1: ArcStr = "test".into();
        assert_eq!(s1, "test");

        // From String
        let s2: ArcStr = String::from("test").into();
        assert_eq!(s2, "test");

        // From Arc<str>
        let arc: Arc<str> = Arc::from("test");
        let s3: ArcStr = arc.into();
        assert_eq!(s3, "test");
    }

    #[test]
    fn test_arcstr_ordering() {
        let s1 = ArcStr::from("a");
        let s2 = ArcStr::from("b");
        let s3 = ArcStr::from("a");

        assert!(s1 < s2);
        assert!(s2 > s1);
        assert_eq!(s1, s3);
    }
}
