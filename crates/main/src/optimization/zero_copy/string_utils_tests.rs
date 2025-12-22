//! Tests for zero-copy string utilities

#[cfg(test)]
mod tests {
    use super::super::string_utils::*;
    use std::sync::Arc;

    #[test]
    fn test_static_strings_new() {
        let strings = StaticStrings::new();

        // Should be pre-populated with common values
        assert!(strings.get("openai").is_some());
        assert!(strings.get("anthropic").is_some());
        assert!(strings.get("running").is_some());
        assert!(strings.get("success").is_some());
    }

    #[test]
    fn test_static_strings_get_cached() {
        let strings = StaticStrings::new();

        // Get cached string
        let openai1 = strings.get("openai").unwrap();
        let openai2 = strings.get("openai").unwrap();

        // Should be same Arc (pointer equality)
        assert!(Arc::ptr_eq(&openai1, &openai2));
        assert_eq!(&*openai1, "openai");
    }

    #[test]
    fn test_static_strings_get_missing() {
        let strings = StaticStrings::new();

        // Non-existent key should return None
        assert!(strings.get("nonexistent").is_none());
    }

    #[test]
    fn test_static_strings_get_or_create() {
        let mut strings = StaticStrings::new();

        // Create new string
        let custom = strings.get_or_create("custom_value");
        assert_eq!(&*custom, "custom_value");

        // Should now be cached
        let custom2 = strings.get("custom_value").unwrap();
        assert!(Arc::ptr_eq(&custom, &custom2));
    }

    #[test]
    fn test_static_strings_get_or_create_existing() {
        let mut strings = StaticStrings::new();

        // Get existing value via get_or_create
        let openai = strings.get_or_create("openai");
        assert_eq!(&*openai, "openai");

        // Should be from cache, not newly created
        let openai2 = strings.get("openai").unwrap();
        assert!(Arc::ptr_eq(&openai, &openai2));
    }

    #[test]
    fn test_static_strings_contains() {
        let strings = StaticStrings::new();

        assert!(strings.contains("openai"));
        assert!(strings.contains("running"));
        assert!(!strings.contains("nonexistent"));
    }

    #[test]
    fn test_static_strings_len() {
        let strings = StaticStrings::new();

        // Should have pre-populated values
        assert!(strings.len() > 20);
    }

    #[test]
    fn test_static_strings_is_empty() {
        let strings = StaticStrings::new();

        // Should not be empty (has pre-populated values)
        assert!(!strings.is_empty());
    }

    #[test]
    fn test_static_strings_all_providers() {
        let strings = StaticStrings::new();

        // All common providers should be cached
        assert!(strings.contains("openai"));
        assert!(strings.contains("anthropic"));
        assert!(strings.contains("ollama"));
        assert!(strings.contains("local"));
    }

    #[test]
    fn test_static_strings_all_status() {
        let strings = StaticStrings::new();

        // All status strings should be cached
        assert!(strings.contains("running"));
        assert!(strings.contains("stopped"));
        assert!(strings.contains("error"));
        assert!(strings.contains("initializing"));
        assert!(strings.contains("healthy"));
        assert!(strings.contains("unhealthy"));
    }

    #[test]
    fn test_static_strings_all_operations() {
        let strings = StaticStrings::new();

        // All operation types should be cached
        assert!(strings.contains("inference"));
        assert!(strings.contains("training"));
        assert!(strings.contains("analysis"));
        assert!(strings.contains("session"));
        assert!(strings.contains("context"));
    }

    #[test]
    fn test_static_strings_all_responses() {
        let strings = StaticStrings::new();

        // All response codes should be cached
        assert!(strings.contains("success"));
        assert!(strings.contains("failure"));
        assert!(strings.contains("timeout"));
        assert!(strings.contains("retry"));
        assert!(strings.contains("pending"));
    }

    #[test]
    fn test_static_strings_http_methods() {
        let strings = StaticStrings::new();

        // HTTP methods should be cached
        assert!(strings.contains("GET"));
        assert!(strings.contains("POST"));
        assert!(strings.contains("PUT"));
        assert!(strings.contains("DELETE"));
        assert!(strings.contains("PATCH"));
    }

    #[test]
    fn test_static_strings_content_types() {
        let strings = StaticStrings::new();

        // Content types should be cached
        assert!(strings.contains("application/json"));
        assert!(strings.contains("text/plain"));
        assert!(strings.contains("application/x-protobuf"));
    }

    #[test]
    fn test_static_strings_zero_copy_benefit() {
        let strings = StaticStrings::new();

        // Get same string multiple times
        let str1 = strings.get("openai").unwrap();
        let str2 = strings.get("openai").unwrap();
        let str3 = strings.get("openai").unwrap();

        // All should point to same allocation (zero-copy)
        assert!(Arc::ptr_eq(&str1, &str2));
        assert!(Arc::ptr_eq(&str2, &str3));

        // Strong count should be 4 (3 clones + 1 in cache)
        assert_eq!(Arc::strong_count(&str1), 4);
    }

    #[test]
    fn test_static_strings_multiple_creates() {
        let mut strings = StaticStrings::new();

        // Create multiple custom strings
        let custom1 = strings.get_or_create("custom1");
        let custom2 = strings.get_or_create("custom2");
        let custom3 = strings.get_or_create("custom3");

        assert_eq!(&*custom1, "custom1");
        assert_eq!(&*custom2, "custom2");
        assert_eq!(&*custom3, "custom3");

        // All should be cached
        assert!(strings.contains("custom1"));
        assert!(strings.contains("custom2"));
        assert!(strings.contains("custom3"));

        // Length should have increased
        assert!(strings.len() > 23);
    }

    #[test]
    fn test_static_strings_empty_string() {
        let mut strings = StaticStrings::new();

        // Can cache empty strings
        let empty = strings.get_or_create("");
        assert_eq!(&*empty, "");
        assert!(strings.contains(""));
    }

    #[test]
    fn test_static_strings_unicode() {
        let mut strings = StaticStrings::new();

        // Can cache unicode strings
        let emoji = strings.get_or_create("🚀");
        let chinese = strings.get_or_create("你好");
        let arabic = strings.get_or_create("مرحبا");

        assert_eq!(&*emoji, "🚀");
        assert_eq!(&*chinese, "你好");
        assert_eq!(&*arabic, "مرحبا");
    }

    #[test]
    fn test_static_strings_long_string() {
        let mut strings = StaticStrings::new();

        // Can cache long strings
        let long = "a".repeat(10000);
        let cached = strings.get_or_create(&long);

        assert_eq!(cached.len(), 10000);
        assert!(strings.contains(&long));
    }

    #[test]
    fn test_default_impl() {
        let strings = StaticStrings::default();

        // Default should work same as new()
        assert!(strings.contains("openai"));
        assert!(strings.contains("running"));
        assert!(!strings.is_empty());
    }
}
