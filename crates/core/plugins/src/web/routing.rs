// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Routing utilities for web plugins

use std::collections::HashMap;

/// Simple route structure for matching paths
#[derive(Debug, Clone)]
pub struct Route {
    /// The route pattern (e.g., "/api/v1/users/{id}")
    pub pattern: String,
    /// Whether this route has parameters
    pub has_params: bool,
    /// Parameter names found in the pattern
    pub param_names: Vec<String>,
}

impl Route {
    /// Create a new route from a pattern
    pub fn new(pattern: &str) -> Self {
        let param_names = Self::extract_param_names(pattern);
        let has_params = !param_names.is_empty();

        Self {
            pattern: pattern.to_string(),
            has_params,
            param_names,
        }
    }

    /// Extract parameter names from a route pattern
    fn extract_param_names(pattern: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut in_param = false;
        let mut current_param = String::new();

        for ch in pattern.chars() {
            match ch {
                '{' => {
                    in_param = true;
                    current_param.clear();
                }
                '}' => {
                    if in_param {
                        params.push(current_param.clone());
                        current_param.clear();
                        in_param = false;
                    }
                }
                _ => {
                    if in_param {
                        current_param.push(ch);
                    }
                }
            }
        }

        params
    }

    /// Check if a path matches this route pattern
    pub fn matches(&self, path: &str) -> bool {
        if !self.has_params {
            return self.pattern == path;
        }

        // Simple matching for parameterized routes
        let pattern_parts: Vec<&str> = self.pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
                // This is a parameter, skip validation
                continue;
            }

            if pattern_part != path_part {
                return false;
            }
        }

        true
    }

    /// Extract parameters from a path using this route pattern
    pub fn extract_params(&self, path: &str) -> Option<HashMap<String, String>> {
        let mut params = HashMap::new();

        if !self.has_params {
            return Some(params);
        }

        let pattern_parts: Vec<&str> = self.pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return None;
        }

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if let Some(param_name) = pattern_part
                .strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
            {
                params.insert(param_name.to_string(), path_part.to_string());
            }
        }

        Some(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_creation() {
        let route = Route::new("/api/v1/users/{id}");
        assert!(route.has_params);
        assert_eq!(route.param_names, vec!["id"]);
    }

    #[test]
    fn test_route_matching() {
        let route = Route::new("/api/v1/users/{id}");
        assert!(route.matches("/api/v1/users/123"));
        assert!(!route.matches("/api/v1/users"));
        assert!(!route.matches("/api/v1/users/123/posts"));
    }

    #[test]
    fn test_param_extraction() {
        let route = Route::new("/api/v1/users/{id}");
        let params = route.extract_params("/api/v1/users/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_static_route() {
        let route = Route::new("/api/v1/status");
        assert!(!route.has_params);
        assert!(route.matches("/api/v1/status"));
        assert!(!route.matches("/api/v1/status/health"));
    }
}
