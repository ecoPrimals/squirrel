//! Tests for zero-copy collection utilities

#[cfg(test)]
mod tests {
    use super::super::collection_utils::*;
    use std::sync::Arc;

    #[test]
    fn test_zero_copy_map_insert_arc() {
        let mut map = ZeroCopyMap::new();

        let result = map.insert_arc("key1".to_string(), "value1");
        assert!(result.is_none());

        let result = map.insert_arc("key1".to_string(), "value2");
        assert_eq!(result, Some("value1"));

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_zero_copy_map_get_str() {
        let mut map = ZeroCopyMap::new();

        map.insert_arc("openai".to_string(), "provider1");
        map.insert_arc("anthropic".to_string(), "provider2");

        assert_eq!(map.get_str("openai"), Some(&"provider1"));
        assert_eq!(map.get_str("anthropic"), Some(&"provider2"));
        assert_eq!(map.get_str("nonexistent"), None);
    }

    #[test]
    fn test_zero_copy_map_contains_str() {
        let mut map = ZeroCopyMap::new();

        map.insert_arc("key1".to_string(), 42);
        map.insert_arc("key2".to_string(), 84);

        assert!(map.contains_str("key1"));
        assert!(map.contains_str("key2"));
        assert!(!map.contains_str("key3"));
    }

    #[test]
    fn test_zero_copy_map_multiple_types() {
        let mut map1 = ZeroCopyMap::<String>::new();
        let mut map2 = ZeroCopyMap::<i32>::new();
        let mut map3 = ZeroCopyMap::<Vec<u8>>::new();

        map1.insert_arc("a".to_string(), "string_value".to_string());
        map2.insert_arc("b".to_string(), 123);
        map3.insert_arc("c".to_string(), vec![1, 2, 3]);

        assert_eq!(map1.get_str("a"), Some(&"string_value".to_string()));
        assert_eq!(map2.get_str("b"), Some(&123));
        assert_eq!(map3.get_str("c"), Some(&vec![1, 2, 3]));
    }

    #[test]
    fn test_zero_copy_set_insert_arc() {
        let mut set = ZeroCopySet::new();

        assert!(set.insert_arc("value1".to_string()));
        assert!(set.insert_arc("value2".to_string()));
        assert!(!set.insert_arc("value1".to_string())); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_zero_copy_set_contains_str() {
        let mut set = ZeroCopySet::new();

        set.insert_arc("openai".to_string());
        set.insert_arc("anthropic".to_string());
        set.insert_arc("ollama".to_string());

        assert!(set.contains_str("openai"));
        assert!(set.contains_str("anthropic"));
        assert!(set.contains_str("ollama"));
        assert!(!set.contains_str("nonexistent"));
    }

    #[test]
    fn test_zero_copy_set_operations() {
        let mut set = ZeroCopySet::new();

        // Insert multiple values
        for i in 0..10 {
            set.insert_arc(format!("key{}", i));
        }

        assert_eq!(set.len(), 10);

        // Verify all exist
        for i in 0..10 {
            assert!(set.contains_str(&format!("key{}", i)));
        }
    }

    #[test]
    fn test_zero_copy_map_empty() {
        let map = ZeroCopyMap::<i32>::new();

        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert!(map.get_str("anything").is_none());
        assert!(!map.contains_str("anything"));
    }

    #[test]
    fn test_zero_copy_set_empty() {
        let set = ZeroCopySet::new();

        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
        assert!(!set.contains_str("anything"));
    }

    #[test]
    fn test_zero_copy_map_with_arc_keys() {
        let mut map = ZeroCopyMap::<String>::new();

        let key1: Arc<str> = Arc::from("key1");
        let key2: Arc<str> = Arc::from("key2");

        map.insert(key1.clone(), "value1".to_string());
        map.insert(key2.clone(), "value2".to_string());

        assert_eq!(map.get_str("key1"), Some(&"value1".to_string()));
        assert_eq!(map.get_str("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_zero_copy_set_with_arc_values() {
        let mut set = ZeroCopySet::new();

        let val1: Arc<str> = Arc::from("value1");
        let val2: Arc<str> = Arc::from("value2");

        set.insert(val1);
        set.insert(val2);

        assert!(set.contains_str("value1"));
        assert!(set.contains_str("value2"));
    }

    #[test]
    fn test_zero_copy_map_unicode_keys() {
        let mut map = ZeroCopyMap::<String>::new();

        map.insert_arc("你好".to_string(), "chinese".to_string());
        map.insert_arc("مرحبا".to_string(), "arabic".to_string());
        map.insert_arc("🚀".to_string(), "emoji".to_string());

        assert_eq!(map.get_str("你好"), Some(&"chinese".to_string()));
        assert_eq!(map.get_str("مرحبا"), Some(&"arabic".to_string()));
        assert_eq!(map.get_str("🚀"), Some(&"emoji".to_string()));
    }

    #[test]
    fn test_zero_copy_set_unicode_values() {
        let mut set = ZeroCopySet::new();

        set.insert_arc("你好".to_string());
        set.insert_arc("مرحبا".to_string());
        set.insert_arc("🚀".to_string());

        assert!(set.contains_str("你好"));
        assert!(set.contains_str("مرحبا"));
        assert!(set.contains_str("🚀"));
    }

    #[test]
    fn test_zero_copy_map_long_keys() {
        let mut map = ZeroCopyMap::<i32>::new();

        let long_key = "a".repeat(10000);
        map.insert_arc(long_key.clone(), 42);

        assert_eq!(map.get_str(&long_key), Some(&42));
        assert!(map.contains_str(&long_key));
    }

    #[test]
    fn test_zero_copy_set_long_values() {
        let mut set = ZeroCopySet::new();

        let long_value = "b".repeat(10000);
        set.insert_arc(long_value.clone());

        assert!(set.contains_str(&long_value));
    }

    #[test]
    fn test_zero_copy_map_update_value() {
        let mut map = ZeroCopyMap::<i32>::new();

        map.insert_arc("counter".to_string(), 1);
        map.insert_arc("counter".to_string(), 2);
        map.insert_arc("counter".to_string(), 3);

        assert_eq!(map.get_str("counter"), Some(&3));
        assert_eq!(map.len(), 1);
    }
}
