// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! General optimization utilities

use std::sync::Arc;

/// General zero-copy optimization utilities
pub struct ZeroCopyUtils;

impl ZeroCopyUtils {
    /// Concatenate strings efficiently
    #[must_use]
    pub fn concat_strings(parts: &[&str]) -> String {
        let total_len: usize = parts.iter().map(|s| s.len()).sum();
        let mut result = String::with_capacity(total_len);

        for part in parts {
            result.push_str(part);
        }

        result
    }

    /// Format key-value pairs efficiently
    #[must_use]
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
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_strings_empty() {
        let result = ZeroCopyUtils::concat_strings(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_concat_strings_single() {
        let result = ZeroCopyUtils::concat_strings(&["hello"]);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_concat_strings_multiple() {
        let result = ZeroCopyUtils::concat_strings(&["hello", " ", "world", "!"]);
        assert_eq!(result, "hello world!");
    }

    #[test]
    fn test_concat_strings_with_empty_parts() {
        let result = ZeroCopyUtils::concat_strings(&["hello", "", "world"]);
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_format_key_value_pairs_empty() {
        let result = ZeroCopyUtils::format_key_value_pairs(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_key_value_pairs_single() {
        let pairs = vec![(Arc::from("key1"), Arc::from("value1"))];
        let result = ZeroCopyUtils::format_key_value_pairs(&pairs);
        assert_eq!(result, "key1=value1");
    }

    #[test]
    fn test_format_key_value_pairs_multiple() {
        let pairs = vec![
            (Arc::from("name"), Arc::from("squirrel")),
            (Arc::from("type"), Arc::from("primal")),
            (Arc::from("version"), Arc::from("0.1.0")),
        ];
        let result = ZeroCopyUtils::format_key_value_pairs(&pairs);
        assert_eq!(result, "name=squirrel,type=primal,version=0.1.0");
    }

    #[test]
    fn test_format_key_value_pairs_with_special_chars() {
        let pairs = vec![
            (Arc::from("url"), Arc::from("http://example.com")),
            (Arc::from("path"), Arc::from("/api/v1")),
        ];
        let result = ZeroCopyUtils::format_key_value_pairs(&pairs);
        assert_eq!(result, "url=http://example.com,path=/api/v1");
    }

    #[test]
    fn test_build_url_with_params_no_params() {
        let result = ZeroCopyUtils::build_url_with_params("http://example.com", "/api/users", &[]);
        assert_eq!(result, "http://example.com/api/users");
    }

    #[test]
    fn test_build_url_with_params_single_param() {
        let params = vec![(Arc::from("id"), Arc::from("123"))];
        let result =
            ZeroCopyUtils::build_url_with_params("http://example.com", "/api/users", &params);
        assert_eq!(result, "http://example.com/api/users?id=123");
    }

    #[test]
    fn test_build_url_with_params_multiple_params() {
        let params = vec![
            (Arc::from("page"), Arc::from("1")),
            (Arc::from("limit"), Arc::from("10")),
            (Arc::from("sort"), Arc::from("name")),
        ];
        let result =
            ZeroCopyUtils::build_url_with_params("http://example.com", "/api/users", &params);
        assert_eq!(
            result,
            "http://example.com/api/users?page=1&limit=10&sort=name"
        );
    }

    #[test]
    fn test_build_url_with_params_empty_base() {
        let params = vec![(Arc::from("q"), Arc::from("search"))];
        let result = ZeroCopyUtils::build_url_with_params("", "/search", &params);
        assert_eq!(result, "/search?q=search");
    }

    #[test]
    fn test_build_url_with_params_empty_path() {
        let params = vec![(Arc::from("debug"), Arc::from("true"))];
        let result = ZeroCopyUtils::build_url_with_params("http://example.com", "", &params);
        assert_eq!(result, "http://example.com?debug=true");
    }

    #[test]
    fn test_build_url_with_params_with_slash() {
        let params = vec![(Arc::from("path"), Arc::from("/data/file.txt"))];
        let result =
            ZeroCopyUtils::build_url_with_params("http://example.com", "/api/files", &params);
        assert_eq!(result, "http://example.com/api/files?path=/data/file.txt");
    }
}
