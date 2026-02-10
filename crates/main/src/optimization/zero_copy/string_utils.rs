// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Zero-copy string utilities

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

/// String interning cache for common values
///
/// Provides efficient access to commonly used strings by maintaining a cache
/// of ``Arc<str>`` instances. This eliminates repeated string allocations for
/// frequently accessed values like provider names, status strings, etc.
///
/// # Performance
///
/// - Cache hits avoid string allocation completely
/// - ``Arc<str>`` sharing enables zero-copy string passing
/// - Pre-populated with common Squirrel ecosystem values
/// - Thread-safe for concurrent read access
///
/// # Examples
///
/// ```rust
/// use squirrel::optimization::zero_copy::string_utils::StaticStrings;
///
/// let strings = StaticStrings::new();
///
/// // Efficient access to cached strings
/// let provider = strings.get("openai").unwrap();
/// let status = strings.get("running").unwrap();
///
/// // Create new cached string
/// let mut strings = StaticStrings::new();
/// let custom = strings.get_or_create("custom_value");
/// ```
///
/// # Thread Safety
///
/// While the underlying `HashMap` requires mutable access for insertions,
/// read operations are lock-free once strings are cached. For high-concurrency
/// scenarios, consider pre-populating all needed strings during initialization.
#[derive(Debug)]
pub struct StaticStrings {
    cache: HashMap<String, Arc<str>>,
}

impl StaticStrings {
    /// Create a new `StaticStrings` cache with common ecosystem values pre-populated
    ///
    /// Pre-populated values include:
    /// - AI providers: "openai", "anthropic", "local", "local-server"
    /// - Status strings: "running", "stopped", "error", "initializing"
    /// - Operation types: "inference", "training", "analysis"
    /// - Response codes: "success", "failure", "timeout", "retry"
    #[must_use]
    pub fn new() -> Self {
        let mut cache = HashMap::new();

        // Pre-populate with common Squirrel ecosystem values
        let common_values = [
            // AI providers
            "openai",
            "anthropic",
            "local",
            "local-server",
            // Status strings
            "running",
            "stopped",
            "error",
            "initializing",
            "healthy",
            "unhealthy",
            // Operation types
            "inference",
            "training",
            "analysis",
            "session",
            "context",
            // Response codes
            "success",
            "failure",
            "timeout",
            "retry",
            "pending",
            // Common HTTP methods
            "GET",
            "POST",
            "PUT",
            "DELETE",
            "PATCH",
            // Content types
            "application/json",
            "text/plain",
            "application/x-protobuf",
        ];

        for value in &common_values {
            let arc_str: Arc<str> = Arc::from(*value);
            cache.insert((*value).to_string(), arc_str);
        }

        Self { cache }
    }

    /// Get a cached string if it exists
    ///
    /// Returns `Some(Arc<str>)` if the string is cached, `None` otherwise.
    /// This method is very fast for cached values as it only requires a `HashMap` lookup.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let strings = StaticStrings::new();
    ///
    /// if let Some(cached) = strings.get("openai") {
    ///     println!("Found cached string: {}", cached);
    /// }
    /// ```
    #[must_use]
    pub fn get(&self, key: &str) -> Option<Arc<str>> {
        self.cache.get(key).cloned()
    }

    /// Get a cached string or create and cache it if it doesn't exist
    ///
    /// This method is useful when you want to ensure a string is interned.
    /// If the string already exists in the cache, it returns the cached version.
    /// Otherwise, it creates a new `Arc<str>`, adds it to the cache, and returns it.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut strings = StaticStrings::new();
    /// let interned = strings.get_or_create("custom_provider");
    /// ```
    pub fn get_or_create(&mut self, key: &str) -> Arc<str> {
        if let Some(cached) = self.cache.get(key) {
            cached.clone()
        } else {
            let arc_str: Arc<str> = Arc::from(key);
            self.cache.insert(key.to_string(), arc_str.clone());
            arc_str
        }
    }

    /// Check if a string is cached
    ///
    /// Returns `true` if the string exists in the cache, `false` otherwise.
    /// This is useful for optimization decisions.
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Get the number of cached strings
    ///
    /// Returns the total number of strings currently cached.
    /// Useful for monitoring and debugging.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear all cached strings
    ///
    /// Removes all strings from the cache. Note that any `Arc<str>` instances
    /// that are still in use elsewhere will remain valid due to Arc semantics.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for StaticStrings {
    fn default() -> Self {
        Self::new()
    }
}

/// Efficient string concatenation utilities
pub struct StringConcat;

impl StringConcat {
    /// Concatenate multiple string references efficiently
    ///
    /// Uses a single allocation to build the final string, avoiding
    /// intermediate allocations that would occur with repeated string concatenation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use squirrel::optimization::zero_copy::string_utils::StringConcat;
    ///
    /// let parts = vec!["Hello", " ", "world", "!"];
    /// let result = StringConcat::concat(&parts);
    /// assert_eq!(result, "Hello world!");
    /// ```
    #[must_use]
    pub fn concat(parts: &[&str]) -> String {
        let total_len: usize = parts.iter().map(|s| s.len()).sum();
        let mut result = String::with_capacity(total_len);

        for part in parts {
            result.push_str(part);
        }

        result
    }

    /// Concatenate with a separator
    ///
    /// Efficiently joins string parts with a separator, using a single allocation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use squirrel::optimization::zero_copy::string_utils::StringConcat;
    ///
    /// let parts = vec!["apple", "banana", "cherry"];
    /// let result = StringConcat::concat_with_separator(&parts, ", ");
    /// assert_eq!(result, "apple, banana, cherry");
    /// ```
    #[must_use]
    pub fn concat_with_separator(parts: &[&str], separator: &str) -> String {
        if parts.is_empty() {
            return String::new();
        }

        if parts.len() == 1 {
            return parts[0].to_string();
        }

        let content_len: usize = parts.iter().map(|s| s.len()).sum();
        let separator_len = separator.len() * (parts.len() - 1);
        let total_len = content_len + separator_len;

        let mut result = String::with_capacity(total_len);

        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                result.push_str(separator);
            }
            result.push_str(part);
        }

        result
    }
}

/// Copy-on-write string utilities
pub struct CowString;

impl CowString {
    /// Create a `Cow<str>` from various string types
    ///
    /// Efficiently handles different string types without unnecessary allocations.
    /// Useful for APIs that can accept both owned and borrowed strings.
    #[must_use]
    pub fn from_string(s: String) -> Cow<'static, str> {
        Cow::Owned(s)
    }

    /// Create a `Cow<str>` from a string slice
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Cow<'_, str> {
        Cow::Borrowed(s)
    }

    /// Conditionally clone a `Cow<str>` only if needed
    ///
    /// Returns the owned version only if the Cow contains borrowed data,
    /// avoiding unnecessary cloning for already-owned strings.
    #[must_use]
    pub fn into_owned_if_needed(cow: Cow<'_, str>) -> String {
        match cow {
            Cow::Borrowed(s) => s.to_owned(),
            Cow::Owned(s) => s,
        }
    }
}
