//! Routing utilities for web plugins
//!
//! This module provides utilities for routing and path parameter extraction.

use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref PARAM_REGEX: Regex = Regex::new(r"\{([a-zA-Z0-9_]+)\}").unwrap();
}

/// Represents a route with parameter extraction capabilities
#[derive(Debug, Clone)]
pub struct Route {
    /// Original route pattern with {param} placeholders
    pattern: String,
    /// Regular expression for matching paths
    regex: Regex,
    /// Parameter names in order of appearance
    param_names: Vec<String>,
}

impl Route {
    /// Create a new route from a pattern
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_plugins::web::routing::Route;
    ///
    /// let route = Route::new("/api/users/{id}/posts/{post_id}");
    /// ```
    pub fn new(pattern: &str) -> Self {
        // Extract parameter names
        let param_names: Vec<String> = PARAM_REGEX
            .captures_iter(pattern)
            .map(|cap| cap[1].to_string())
            .collect();

        // Create regex pattern by replacing {param} with named capture groups
        let regex_pattern = PARAM_REGEX.replace_all(pattern, r"(?P<$1>[^/]+)");
        
        // Anchor the regex to match full paths
        let regex_string = format!("^{}$", regex_pattern);
        let regex = Regex::new(&regex_string).unwrap();

        Self {
            pattern: pattern.to_string(),
            regex,
            param_names,
        }
    }

    /// Check if path matches the route pattern
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_plugins::web::routing::Route;
    ///
    /// let route = Route::new("/api/users/{id}");
    /// assert!(route.matches("/api/users/123"));
    /// assert!(!route.matches("/api/users"));
    /// ```
    pub fn matches(&self, path: &str) -> bool {
        self.regex.is_match(path)
    }

    /// Extract parameters from path
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_plugins::web::routing::Route;
    ///
    /// let route = Route::new("/api/users/{id}/posts/{post_id}");
    /// let params = route.extract_params("/api/users/123/posts/456").unwrap();
    /// assert_eq!(params.get("id"), Some(&"123".to_string()));
    /// assert_eq!(params.get("post_id"), Some(&"456".to_string()));
    /// ```
    pub fn extract_params(&self, path: &str) -> Option<HashMap<String, String>> {
        let mut params = HashMap::new();
        
        if let Some(captures) = self.regex.captures(path) {
            for name in &self.param_names {
                if let Some(value) = captures.name(name) {
                    params.insert(name.clone(), value.as_str().to_string());
                }
            }
            Some(params)
        } else {
            None
        }
    }

    /// Get the original pattern
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Get parameter names
    pub fn param_names(&self) -> &[String] {
        &self.param_names
    }
}

/// Extension trait for WebRequest to handle route parameters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_matches() {
        let route = Route::new("/api/users/{id}");
        assert!(route.matches("/api/users/123"));
        assert!(route.matches("/api/users/abc"));
        assert!(!route.matches("/api/users"));
        assert!(!route.matches("/api/users/123/posts"));
    }

    #[test]
    fn test_route_extract_params() {
        let route = Route::new("/api/users/{id}/posts/{post_id}");
        let params = route.extract_params("/api/users/123/posts/456").unwrap();
        
        assert_eq!(params.len(), 2);
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("post_id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_route_with_multiple_params() {
        let route = Route::new("/api/{resource}/{id}/{action}");
        let params = route.extract_params("/api/users/123/activate").unwrap();
        
        assert_eq!(params.len(), 3);
        assert_eq!(params.get("resource"), Some(&"users".to_string()));
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("action"), Some(&"activate".to_string()));
    }
    
    #[test]
    fn test_route_with_no_params() {
        let route = Route::new("/api/health");
        let params = route.extract_params("/api/health").unwrap();
        
        assert_eq!(params.len(), 0);
    }
    
    #[test]
    fn test_non_matching_route() {
        let route = Route::new("/api/users/{id}");
        let params = route.extract_params("/api/posts/123");
        
        assert!(params.is_none());
    }
} 