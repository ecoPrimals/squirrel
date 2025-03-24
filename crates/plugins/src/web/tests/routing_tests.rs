//! Tests for routing functionality
//!
//! This module contains tests for the path parameter extraction and route matching.

#[cfg(test)]
mod routing_tests {
    use std::collections::HashMap;
    use crate::web::routing::Route;
    use crate::web::{HttpMethod, WebRequest, WebResponse};
    use serde_json::json;

    #[test]
    fn test_route_basic_matching() {
        let route = Route::new("/api/users");
        
        assert!(route.matches("/api/users"));
        assert!(!route.matches("/api/users/123"));
        assert!(!route.matches("/api"));
    }
    
    #[test]
    fn test_route_with_single_parameter() {
        let route = Route::new("/api/users/{id}");
        
        assert!(route.matches("/api/users/123"));
        assert!(route.matches("/api/users/abc"));
        assert!(!route.matches("/api/users"));
        assert!(!route.matches("/api/users/123/profile"));
        
        let params = route.extract_params("/api/users/123").unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }
    
    #[test]
    fn test_route_with_multiple_parameters() {
        let route = Route::new("/api/users/{userId}/posts/{postId}");
        
        assert!(route.matches("/api/users/123/posts/456"));
        assert!(!route.matches("/api/users/123"));
        assert!(!route.matches("/api/users/123/posts"));
        
        let params = route.extract_params("/api/users/123/posts/456").unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params.get("userId"), Some(&"123".to_string()));
        assert_eq!(params.get("postId"), Some(&"456".to_string()));
    }
    
    #[test]
    fn test_route_with_complex_parameters() {
        let route = Route::new("/api/{version}/users/{userId}/posts/{postId}/comments");
        
        assert!(route.matches("/api/v1/users/123/posts/456/comments"));
        assert!(!route.matches("/api/v1/users/123/posts"));
        
        let params = route.extract_params("/api/v1/users/123/posts/456/comments").unwrap();
        assert_eq!(params.len(), 3);
        assert_eq!(params.get("version"), Some(&"v1".to_string()));
        assert_eq!(params.get("userId"), Some(&"123".to_string()));
        assert_eq!(params.get("postId"), Some(&"456".to_string()));
    }
    
    #[test]
    fn test_web_request_with_route_params() {
        // Create a WebRequest
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        
        let mut query_params = HashMap::new();
        query_params.insert("sort".to_string(), "desc".to_string());
        
        let request = WebRequest::new(
            HttpMethod::Get,
            "/api/users/123/posts/456".to_string(),
            query_params,
            headers,
            Some(json!({})),
            Some("user-1".to_string()),
            vec!["users.read".to_string()]
        ).with_route_params("/api/users/{userId}/posts/{postId}");
        
        // Verify route parameters were extracted correctly
        assert_eq!(request.route_params.len(), 2);
        assert_eq!(request.param("userId"), Some(&"123".to_string()));
        assert_eq!(request.param("postId"), Some(&"456".to_string()));
        
        // Test param_or_query
        assert_eq!(request.param_or_query("userId"), Some(&"123".to_string()));
        assert_eq!(request.param_or_query("sort"), Some(&"desc".to_string()));
        assert_eq!(request.param_or_query("nonexistent"), None);
    }
    
    #[test]
    fn test_non_matching_route() {
        let route = Route::new("/api/users/{id}");
        
        // Non-matching path shouldn't extract any parameters
        let params = route.extract_params("/api/posts/123").unwrap_or_default();
        assert_eq!(params.len(), 0);
        
        // WebRequest with non-matching route shouldn't have route params
        let request = WebRequest::new(
            HttpMethod::Get,
            "/api/posts/123".to_string(),
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            vec![]
        ).with_route_params("/api/users/{id}");
        
        assert_eq!(request.route_params.len(), 0);
        assert_eq!(request.param("id"), None);
    }
    
    #[test]
    fn test_route_with_special_characters() {
        let route = Route::new("/api/files/{filename}");
        
        assert!(route.matches("/api/files/document.pdf"));
        assert!(route.matches("/api/files/image-123.jpg"));
        
        let params = route.extract_params("/api/files/my-document.pdf").unwrap();
        assert_eq!(params.get("filename"), Some(&"my-document.pdf".to_string()));
        
        // Test with URL-encoded characters
        assert!(route.matches("/api/files/my%20document.pdf"));
        let params = route.extract_params("/api/files/my%20document.pdf").unwrap();
        assert_eq!(params.get("filename"), Some(&"my%20document.pdf".to_string()));
    }
    
    #[test]
    fn test_real_world_api_endpoints() {
        // Test common REST API patterns
        
        // 1. Collection endpoints
        let collection_route = Route::new("/api/v1/resources");
        assert!(collection_route.matches("/api/v1/resources"));
        
        // 2. Specific resource endpoint
        let resource_route = Route::new("/api/v1/resources/{id}");
        assert!(resource_route.matches("/api/v1/resources/123"));
        
        // 3. Nested resources
        let nested_route = Route::new("/api/v1/resources/{resourceId}/sub-resources/{subId}");
        assert!(nested_route.matches("/api/v1/resources/123/sub-resources/456"));
        
        // 4. Actions on resources
        let action_route = Route::new("/api/v1/resources/{id}/{action}");
        assert!(action_route.matches("/api/v1/resources/123/activate"));
        
        let params = action_route.extract_params("/api/v1/resources/123/activate").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("action"), Some(&"activate".to_string()));
        
        // 5. Version path parameter
        let versioned_route = Route::new("/api/{version}/resources");
        assert!(versioned_route.matches("/api/v1/resources"));
        assert!(versioned_route.matches("/api/v2/resources"));
        
        let params = versioned_route.extract_params("/api/v2/resources").unwrap();
        assert_eq!(params.get("version"), Some(&"v2".to_string()));
        
        // 6. File download endpoint
        let file_route = Route::new("/api/v1/files/{fileId}/download");
        assert!(file_route.matches("/api/v1/files/abc123/download"));
    }
} 