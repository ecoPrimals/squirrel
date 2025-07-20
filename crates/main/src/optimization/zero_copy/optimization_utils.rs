//! General optimization utilities

use std::sync::Arc;

/// General zero-copy optimization utilities
pub struct ZeroCopyUtils;

impl ZeroCopyUtils {
    /// Concatenate strings efficiently
    pub fn concat_strings(parts: &[&str]) -> String {
        let total_len: usize = parts.iter().map(|s| s.len()).sum();
        let mut result = String::with_capacity(total_len);

        for part in parts {
            result.push_str(part);
        }

        result
    }

    /// Format key-value pairs efficiently
    pub fn format_key_value_pairs(pairs: &[(Arc<str>, Arc<str>)]) -> String {
        if pairs.is_empty() {
            return String::new();
        }

        let estimated_len = pairs.len() * 20; // Rough estimate
        let mut result = String::with_capacity(estimated_len);

        for (i, (key, value)) in pairs.iter().enumerate() {
            if i > 0 {
                result.push(',');
            }
            result.push_str(key);
            result.push('=');
            result.push_str(value);
        }

        result
    }

    /// Build URL with parameters efficiently
    pub fn build_url_with_params(
        base: &str,
        path: &str,
        params: &[(Arc<str>, Arc<str>)],
    ) -> String {
        let mut url = String::with_capacity(base.len() + path.len() + params.len() * 20);

        url.push_str(base);
        url.push_str(path);

        if !params.is_empty() {
            url.push('?');
            for (i, (key, value)) in params.iter().enumerate() {
                if i > 0 {
                    url.push('&');
                }
                url.push_str(key);
                url.push('=');
                url.push_str(value);
            }
        }

        url
    }
}
