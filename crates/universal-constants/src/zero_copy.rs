// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Zero-copy string utilities using `ArcStr`
//!
//! This module provides utilities for efficient string handling with zero-copy semantics.

use arcstr::ArcStr;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Common strings used throughout the system
static COMMON_STRINGS: LazyLock<HashMap<&'static str, ArcStr>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // HTTP methods
    map.insert("GET", ArcStr::from("GET"));
    map.insert("POST", ArcStr::from("POST"));
    map.insert("PUT", ArcStr::from("PUT"));
    map.insert("DELETE", ArcStr::from("DELETE"));
    map.insert("PATCH", ArcStr::from("PATCH"));

    // Common hosts
    map.insert("localhost", ArcStr::from("localhost"));
    map.insert("127.0.0.1", ArcStr::from("127.0.0.1"));

    // Common protocols
    map.insert("http", ArcStr::from("http"));
    map.insert("https", ArcStr::from("https"));
    map.insert("ws", ArcStr::from("ws"));
    map.insert("wss", ArcStr::from("wss"));

    // MCP message types
    map.insert("request", ArcStr::from("request"));
    map.insert("response", ArcStr::from("response"));
    map.insert("notification", ArcStr::from("notification"));
    map.insert("ack", ArcStr::from("ack"));
    map.insert("error", ArcStr::from("error"));

    map
});

/// Intern a string with zero-copy for common values
///
/// For frequently used strings, this returns a cached `ArcStr` instance.
/// For uncommon strings, it creates a new `ArcStr`.
///
/// # Examples
///
/// ```
/// use universal_constants::zero_copy::intern;
///
/// let host = intern("localhost"); // Returns cached instance
/// let custom = intern("my-custom-host"); // Creates new instance
/// ```
pub fn intern(s: &str) -> ArcStr {
    COMMON_STRINGS
        .get(s)
        .cloned()
        .unwrap_or_else(|| ArcStr::from(s))
}

/// Create `ArcStr` from owned `String`.
///
/// This consumes the `String` and converts it to `ArcStr` efficiently.
#[must_use]
pub fn from_string(s: String) -> ArcStr {
    ArcStr::from(s)
}

/// Create `ArcStr` from static string.
///
/// For compile-time known strings, this is zero-cost.
#[must_use]
pub fn from_static(s: &'static str) -> ArcStr {
    ArcStr::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_common() {
        let s1 = intern("localhost");
        let s2 = intern("localhost");

        // Should be same instance (pointer equality)
        assert_eq!(s1.as_ptr(), s2.as_ptr());
    }

    #[test]
    fn test_intern_custom() {
        let s1 = intern("custom-value-12345");
        let s2 = intern("custom-value-12345");

        // Different instances but same content
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_from_string() {
        let owned = String::from("test");
        let arc = from_string(owned);
        assert_eq!(&*arc, "test");
    }

    #[test]
    fn test_from_static() {
        let arc = from_static("static");
        assert_eq!(&*arc, "static");
    }
}
