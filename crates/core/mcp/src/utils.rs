// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP Utility Functions
//!
//! This module provides utility functions for the Machine Context Protocol implementation.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{MCPError, Result};

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

const fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Generate a unique message ID
pub fn generate_message_id() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a timestamp in milliseconds since epoch
pub fn generate_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis() as u64
}

/// Validate a message ID format
pub fn validate_message_id(id: &str) -> bool {
    Uuid::parse_str(id).is_ok()
}

/// Create a hash from a string
pub fn hash_string(input: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

/// Utility for handling JSON serialization/deserialization
pub struct JsonUtils;

impl JsonUtils {
    /// Serialize object to JSON string
    #[must_use = "serialization errors should be handled"]
    pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
        serde_json::to_string(value)
            .map_err(|e| MCPError::Serialization(format!("JSON serialization failed: {e}")))
    }

    /// Serialize object to pretty JSON string
    #[must_use = "serialization errors should be handled"]
    pub fn to_pretty_string<T: Serialize>(value: &T) -> Result<String> {
        serde_json::to_string_pretty(value)
            .map_err(|e| MCPError::Serialization(format!("JSON serialization failed: {e}")))
    }

    /// Deserialize JSON string to object
    #[must_use = "deserialization errors should be handled"]
    pub fn from_string<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T> {
        serde_json::from_str(json)
            .map_err(|e| MCPError::Serialization(format!("JSON deserialization failed: {e}")))
    }

    /// Validate JSON string
    pub fn validate_json(json: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(json).is_ok()
    }

    /// Merge two JSON objects
    pub fn merge_json(
        base: &serde_json::Value,
        overlay: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        match (base, overlay) {
            (serde_json::Value::Object(base_map), serde_json::Value::Object(overlay_map)) => {
                let mut merged = base_map.clone();
                for (key, value) in overlay_map {
                    merged.insert(key.clone(), value.clone());
                }
                Ok(serde_json::Value::Object(merged))
            }
            _ => Err(MCPError::Serialization(
                "Cannot merge non-object JSON values".to_string(),
            )),
        }
    }
}

/// Utility for handling collections
pub struct CollectionUtils;

impl CollectionUtils {
    /// Merge two hashmaps
    pub fn merge_hashmaps<K, V>(base: HashMap<K, V>, overlay: HashMap<K, V>) -> HashMap<K, V>
    where
        K: Eq + Hash,
    {
        let mut merged = base;
        for (key, value) in overlay {
            merged.insert(key, value);
        }
        merged
    }

    /// Get keys from hashmap as vector
    pub fn get_keys<K, V>(map: &HashMap<K, V>) -> Vec<&K>
    where
        K: Eq + Hash,
    {
        map.keys().collect()
    }

    /// Get values from hashmap as vector
    pub fn get_values<K, V>(map: &HashMap<K, V>) -> Vec<&V>
    where
        K: Eq + Hash,
    {
        map.values().collect()
    }

    /// Filter hashmap by predicate
    pub fn filter_hashmap<K, V, F>(map: HashMap<K, V>, predicate: F) -> HashMap<K, V>
    where
        K: Eq + Hash,
        F: Fn(&K, &V) -> bool,
    {
        map.into_iter().filter(|(k, v)| predicate(k, v)).collect()
    }
}

/// Utility for string operations
pub struct StringUtils;

impl StringUtils {
    /// Truncate string to specified length
    pub fn truncate(s: &str, max_length: usize) -> String {
        if s.len() <= max_length {
            s.to_string()
        } else {
            format!("{}...", &s[..max_length.saturating_sub(3)])
        }
    }

    /// Check if string is empty or whitespace
    pub fn is_empty_or_whitespace(s: &str) -> bool {
        s.trim().is_empty()
    }

    /// Split string by delimiter and trim each part
    pub fn split_and_trim(s: &str, delimiter: &str) -> Vec<String> {
        s.split(delimiter)
            .map(|part| part.trim().to_string())
            .filter(|part| !part.is_empty())
            .collect()
    }

    /// Join strings with delimiter
    pub fn join_with_delimiter(strings: &[String], delimiter: &str) -> String {
        strings.join(delimiter)
    }

    /// Convert string to title case
    pub fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Remove special characters from string
    pub fn sanitize_string(s: &str) -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .collect()
    }
}

/// Utility for time operations
pub struct TimeUtils;

impl TimeUtils {
    /// Get current timestamp in seconds
    pub fn current_timestamp_seconds() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs()
    }

    /// Get current timestamp in milliseconds
    pub fn current_timestamp_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_millis() as u64
    }

    /// Format duration for display
    pub fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();

        if secs >= 60 {
            let minutes = secs / 60;
            let remaining_secs = secs % 60;
            format!("{minutes}m{remaining_secs}s")
        } else if secs > 0 {
            format!("{secs}.{millis:03}s")
        } else {
            format!("{millis}ms")
        }
    }

    /// Parse duration from string (e.g., "30s", "5m", "1h")
    pub fn parse_duration(duration_str: &str) -> Result<Duration> {
        let duration_str = duration_str.trim();

        if duration_str.is_empty() {
            return Err(MCPError::InvalidArgument(
                "Empty duration string".to_string(),
            ));
        }

        let (value_str, unit) = if let Some(stripped) = duration_str.strip_suffix("ms") {
            (stripped, "ms")
        } else if let Some(stripped) = duration_str.strip_suffix('s') {
            (stripped, "s")
        } else if let Some(stripped) = duration_str.strip_suffix('m') {
            (stripped, "m")
        } else if let Some(stripped) = duration_str.strip_suffix('h') {
            (stripped, "h")
        } else {
            return Err(MCPError::InvalidArgument(format!(
                "Invalid duration format: {duration_str}"
            )));
        };

        let value: u64 = value_str.parse().map_err(|_| {
            MCPError::InvalidArgument(format!("Invalid duration value: {value_str}"))
        })?;

        let duration = match unit {
            "ms" => Duration::from_millis(value),
            "s" => Duration::from_secs(value),
            "m" => Duration::from_secs(value * 60),
            "h" => Duration::from_secs(value * 3600),
            _ => {
                return Err(MCPError::InvalidArgument(format!(
                    "Invalid duration unit: {unit}"
                )));
            }
        };

        Ok(duration)
    }
}

/// Utility for validation
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate email format
    pub fn is_valid_email(email: &str) -> bool {
        // Use a static regex pattern to avoid compilation at runtime
        match regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$") {
            Ok(email_regex) => email_regex.is_match(email),
            Err(_) => {
                // If regex compilation fails, fall back to basic validation
                email.contains('@') && email.contains('.') && email.len() > 5
            }
        }
    }

    /// Validate URL format (basic check without external `url` crate)
    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("unix://")
            || url.starts_with("ws://")
            || url.starts_with("wss://")
    }

    /// Validate port number
    pub const fn is_valid_port(port: u16) -> bool {
        port > 0
    }

    /// Validate IP address
    pub fn is_valid_ip(ip: &str) -> bool {
        ip.parse::<std::net::IpAddr>().is_ok()
    }

    /// Validate required fields
    pub fn validate_required_fields(fields: &[(&str, &str)]) -> Result<()> {
        for (field_name, field_value) in fields {
            if field_value.trim().is_empty() {
                return Err(MCPError::Validation(format!(
                    "Required field '{field_name}' is empty"
                )));
            }
        }
        Ok(())
    }

    /// Validate string length
    pub fn validate_string_length(value: &str, min: usize, max: usize) -> Result<()> {
        let length = value.len();
        if length < min || length > max {
            return Err(MCPError::Validation(format!(
                "String length {length} is not between {min} and {max}"
            )));
        }
        Ok(())
    }
}

/// Utility for encoding/decoding
pub struct EncodingUtils;

impl EncodingUtils {
    /// Encode string to base64
    pub fn encode_base64(input: &str) -> String {
        base64::engine::general_purpose::STANDARD.encode(input)
    }

    /// Decode base64 string
    pub fn decode_base64(input: &str) -> Result<String> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(input)
            .map_err(|e| MCPError::InvalidArgument(format!("Base64 decode error: {e}")))?;
        String::from_utf8(bytes)
            .map_err(|e| MCPError::InvalidArgument(format!("UTF-8 decode error: {e}")))
    }

    /// URL encode string (pure Rust, no external crate)
    pub fn url_encode(input: &str) -> String {
        let mut encoded = String::with_capacity(input.len() * 3);
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                _ => {
                    encoded.push('%');
                    encoded.push(char::from(HEX_CHARS[(byte >> 4) as usize]));
                    encoded.push(char::from(HEX_CHARS[(byte & 0x0f) as usize]));
                }
            }
        }
        encoded
    }

    /// URL decode string (pure Rust, no external crate)
    pub fn url_decode(input: &str) -> Result<String> {
        let mut bytes = Vec::with_capacity(input.len());
        let mut chars = input.bytes();
        while let Some(b) = chars.next() {
            if b == b'%' {
                let hi = chars.next().ok_or_else(|| {
                    MCPError::InvalidArgument("Incomplete percent-encoding".into())
                })?;
                let lo = chars.next().ok_or_else(|| {
                    MCPError::InvalidArgument("Incomplete percent-encoding".into())
                })?;
                let val = hex_val(hi)
                    .and_then(|h| hex_val(lo).map(|l| (h << 4) | l))
                    .ok_or_else(|| {
                        MCPError::InvalidArgument("Invalid hex in percent-encoding".into())
                    })?;
                bytes.push(val);
            } else if b == b'+' {
                bytes.push(b' ');
            } else {
                bytes.push(b);
            }
        }
        String::from_utf8(bytes)
            .map_err(|e| MCPError::InvalidArgument(format!("URL decode UTF-8 error: {e}")))
    }

    /// Hex encode bytes (pure Rust, no external crate)
    pub fn hex_encode(input: &[u8]) -> String {
        let mut s = String::with_capacity(input.len() * 2);
        for &b in input {
            s.push(char::from(HEX_CHARS[(b >> 4) as usize]));
            s.push(char::from(HEX_CHARS[(b & 0x0f) as usize]));
        }
        s
    }

    /// Hex decode string (pure Rust, no external crate)
    pub fn hex_decode(input: &str) -> Result<Vec<u8>> {
        if !input.len().is_multiple_of(2) {
            return Err(MCPError::InvalidArgument("Odd-length hex string".into()));
        }
        input
            .as_bytes()
            .chunks(2)
            .map(|pair| {
                hex_val(pair[0])
                    .and_then(|h| hex_val(pair[1]).map(|l| (h << 4) | l))
                    .ok_or_else(|| MCPError::InvalidArgument("Invalid hex character".into()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_message_id() {
        let id = generate_message_id();
        assert!(validate_message_id(&id));
    }

    #[test]
    fn test_generate_timestamp() {
        let timestamp = generate_timestamp();
        assert!(timestamp > 0);
    }

    #[test]
    fn test_hash_string() {
        let hash1 = hash_string("hello");
        let hash2 = hash_string("hello");
        let hash3 = hash_string("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_json_utils() {
        let value = serde_json::json!({"key": "value"});
        let json_str = JsonUtils::to_string(&value).unwrap();
        let parsed: serde_json::Value = JsonUtils::from_string(&json_str).unwrap();
        assert_eq!(value, parsed);
    }

    #[test]
    fn test_string_utils() {
        assert_eq!(StringUtils::truncate("hello world", 5), "he...");
        assert!(StringUtils::is_empty_or_whitespace("   "));
        assert_eq!(StringUtils::to_title_case("hello world"), "Hello World");
    }

    #[test]
    fn test_time_utils() {
        let duration = TimeUtils::parse_duration("30s").unwrap();
        assert_eq!(duration, Duration::from_secs(30));

        let formatted = TimeUtils::format_duration(Duration::from_secs(90));
        assert_eq!(formatted, "1m30s");
    }

    #[test]
    fn test_validation_utils() {
        assert!(ValidationUtils::is_valid_email("test@example.com"));
        assert!(!ValidationUtils::is_valid_email("invalid-email"));
        assert!(ValidationUtils::is_valid_port(8080));
        assert!(!ValidationUtils::is_valid_port(0));
    }

    #[test]
    fn test_encoding_utils() {
        let encoded = EncodingUtils::encode_base64("hello");
        let decoded = EncodingUtils::decode_base64(&encoded).unwrap();
        assert_eq!(decoded, "hello");
    }
}
