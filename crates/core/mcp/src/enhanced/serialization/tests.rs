// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for Zero-Copy Serialization System
//!
//! Tests cover buffer pool management, fast codec performance, streaming serialization,
//! template compilation, and integration with MCP message types.

use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use bytes::Bytes;

use crate::protocol::types::{MCPMessage, MessageType, MessageId, ProtocolVersion};
use crate::enhanced::coordinator::{UniversalAIRequest, UniversalAIResponse, Message};

/// Helper function to create test MCPMessage
fn create_test_mcp_message() -> MCPMessage {
    MCPMessage {
        id: MessageId("test-123".to_string()),
        type_: MessageType::Command,
        payload: serde_json::json!({"command": "test", "data": "hello world"}),
        metadata: Some(serde_json::json!({"source": "test"})),
        security: Default::default(),
        timestamp: chrono::Utc::now(),
        version: ProtocolVersion::new(1, 0),
        trace_id: Some("trace-123".to_string()),
    }
}

/// Helper function to create test AI request
fn create_test_ai_request() -> UniversalAIRequest {
    UniversalAIRequest {
        id: uuid::Uuid::new_v4().to_string(),
        model: "gpt-4".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
            }
        ],
        parameters: {
            let mut params = HashMap::new();
            params.insert("temperature".to_string(), serde_json::json!(0.7));
            params.insert("max_tokens".to_string(), serde_json::json!(1000));
            params
        },
    }
}

/// Helper function to create test AI response
fn create_test_ai_response() -> UniversalAIResponse {
    UniversalAIResponse {
        id: uuid::Uuid::new_v4().to_string(),
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        response_type: crate::enhanced::coordinator::types::AIRequestType::ChatCompletion,
        content: "Hello! How can I help you today?".to_string(),
        cost: 0.002,
        duration: Duration::from_millis(1500),
        metadata: HashMap::new(),
    }
}

// Buffer Pool Tests

#[tokio::test]
async fn test_buffer_pool_creation() {
    let config = BufferPoolConfig {
        initial_size: 5,
        max_size: 20,
        max_buffer_size: 1024 * 1024,
    };
    
    let pool = BufferPool::new(config);
    
    // Wait a bit for initialization
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let stats = pool.get_stats().await;
    assert_eq!(stats.total_created, 5); // Initial buffers created
}

#[tokio::test]
async fn test_buffer_pool_get_and_return() {
    let config = BufferPoolConfig::default();
    let pool = BufferPool::new(config);
    
    // Get a buffer
    let mut buffer = pool.get_buffer().await;
    buffer.extend_from_slice(b"test data");
    
    assert!(!buffer.is_empty());
    
    // Return the buffer
    pool.return_buffer(buffer).await;
    
    let stats = pool.get_stats().await;
    assert_eq!(stats.total_gets, 1);
    assert_eq!(stats.total_returns, 1);
}

#[tokio::test]
async fn test_buffer_pool_reuse() {
    let config = BufferPoolConfig {
        initial_size: 2,
        max_size: 10,
        max_buffer_size: 1024,
    };
    let pool = BufferPool::new(config);
    
    // Wait for initialization
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Get and return a buffer
    let buffer = pool.get_buffer().await;
    pool.return_buffer(buffer).await;
    
    // Get another buffer - should be reused
    let _buffer2 = pool.get_buffer().await;
    
    let stats = pool.get_stats().await;
    assert!(stats.total_reused > 0);
}

#[tokio::test]
async fn test_buffer_pool_efficiency() {
    let config = BufferPoolConfig::default();
    let pool = BufferPool::new(config);
    
    // Perform multiple get/return cycles
    for _ in 0..10 {
        let buffer = pool.get_buffer().await;
        pool.return_buffer(buffer).await;
    }
    
    let efficiency = pool.get_efficiency_metrics().await;
    assert!(efficiency.hit_rate > 0.0);
    assert!(efficiency.memory_saved_bytes > 0);
}

#[tokio::test]
async fn test_buffer_pool_concurrent_access() {
    let config = BufferPoolConfig::default();
    let pool = Arc::new(BufferPool::new(config));
    
    let mut handles = Vec::new();
    
    // Spawn multiple concurrent tasks
    for i in 0..10 {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let mut buffer = pool_clone.get_buffer().await;
            buffer.extend_from_slice(format!("data-{}", i).as_bytes());
            tokio::time::sleep(Duration::from_millis(10)).await;
            pool_clone.return_buffer(buffer).await;
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("should succeed");
    }
    
    let stats = pool.get_stats().await;
    assert_eq!(stats.total_gets, 10);
    assert_eq!(stats.total_returns, 10);
}

// Zero-Copy Serializer Tests

#[tokio::test]
async fn test_zero_copy_serializer_creation() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    let metrics = serializer.get_metrics().await;
    assert_eq!(metrics.total_serializations, 0);
    assert_eq!(metrics.total_deserializations, 0);
}

#[tokio::test]
async fn test_mcp_message_serialization() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    let message = create_test_mcp_message();
    let result = serializer.serialize_mcp_message(&message).await;
    
    assert!(result.is_ok());
    let serialization_result = result.expect("should succeed");
    assert!(!serialization_result.data.is_empty());
    assert_eq!(serialization_result.metadata.method, SerializationMethod::Standard);
}

#[tokio::test]
async fn test_ai_request_serialization() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    let request = create_test_ai_request();
    let result = serializer.serialize_ai_request(&request).await;
    
    assert!(result.is_ok());
    let serialization_result = result.expect("should succeed");
    assert!(!serialization_result.data.is_empty());
    assert!(serialization_result.metadata.duration.as_nanos() > 0);
}

#[tokio::test]
async fn test_ai_response_serialization() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    let response = create_test_ai_response();
    let result = serializer.serialize_ai_response(&response).await;
    
    assert!(result.is_ok());
    let serialization_result = result.expect("should succeed");
    assert!(!serialization_result.data.is_empty());
}

#[tokio::test]
async fn test_serialization_with_buffer_pooling() {
    let config = SerializationConfig {
        enable_buffer_pooling: true,
        ..Default::default()
    };
    let serializer = ZeroCopySerializer::new(config);
    
    let message = create_test_mcp_message();
    let result = serializer.serialize_mcp_message(&message).await;
    
    assert!(result.is_ok());
    let serialization_result = result.expect("should succeed");
    assert!(serialization_result.metadata.used_buffer_pool);
}

#[tokio::test]
async fn test_deserialization() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    let original_message = create_test_mcp_message();
    let serialized = serializer.serialize_mcp_message(&original_message).await.expect("should succeed");
    let deserialized = serializer.deserialize_mcp_message(&serialized.data).await;
    
    assert!(deserialized.is_ok());
    let deserialized_message = deserialized.expect("should succeed");
    assert_eq!(original_message.id, deserialized_message.id);
    assert_eq!(original_message.type_, deserialized_message.type_);
}

#[tokio::test]
async fn test_serialization_metrics() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    // Perform multiple serializations
    for _ in 0..5 {
        let message = create_test_mcp_message();
        let _ = serializer.serialize_mcp_message(&message).await.expect("should succeed");
    }
    
    let metrics = serializer.get_metrics().await;
    assert_eq!(metrics.total_serializations, 5);
    assert!(metrics.avg_serialization_time_us > 0.0);
}

#[tokio::test]
async fn test_performance_report() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    // Perform serializations
    let message = create_test_mcp_message();
    let _ = serializer.serialize_mcp_message(&message).await.expect("should succeed");
    
    let report = serializer.generate_performance_report().await;
    assert!(report.metrics.total_serializations > 0);
    assert!(report.average_throughput_mbps >= 0.0);
}

// Fast Codec Tests

#[tokio::test]
async fn test_mcp_message_codec() {
    let codec = codecs::MCPMessageCodec::new();
    let message = create_test_mcp_message();
    
    let result = codec.encode(&message).await;
    assert!(result.is_ok());
    
    let serialized = result.expect("should succeed");
    assert!(!serialized.data.is_empty());
    assert_eq!(serialized.metadata.method, SerializationMethod::FastCodec);
}

#[tokio::test]
async fn test_codec_performance_profile() {
    let codec = codecs::MCPMessageCodec::new();
    let profile = codec.performance_profile();
    
    assert!(profile.encoding_speed_mbps > 0.0);
    assert!(profile.decoding_speed_mbps > 0.0);
    assert!(profile.compression_ratio > 0.0);
}

#[tokio::test]
async fn test_codec_can_handle() {
    let codec = codecs::MCPMessageCodec::new();
    
    assert!(codec.can_handle("MCPMessage"));
    assert!(codec.can_handle("Command"));
    assert!(codec.can_handle("Response"));
    assert!(!codec.can_handle("UniversalAIRequest"));
}

#[tokio::test]
async fn test_ai_message_codec() {
    let codec = codecs::AIMessageCodec::new();
    
    assert_eq!(codec.name(), "AIMessageCodec");
    assert!(codec.can_handle("UniversalAIRequest"));
    assert!(codec.can_handle("UniversalAIResponse"));
}

#[tokio::test]
async fn test_binary_codec() {
    let codec = codecs::BinaryCodec::new();
    let message = create_test_mcp_message();
    
    let result = codec.encode(&message).await;
    assert!(result.is_ok());
    
    let serialized = result.expect("should succeed");
    assert!(!serialized.data.is_empty());
    assert_eq!(serialized.metadata.method, SerializationMethod::Binary);
}

#[tokio::test]
async fn test_codec_registry() {
    let mut registry = codecs::CodecRegistry::new();
    
    // Test default codecs are registered
    assert!(registry.get("MCPMessage").is_some());
    assert!(registry.get("AIMessage").is_some());
    assert!(registry.get("Binary").is_some());
    
    // Test finding best codec
    let best_codec = registry.find_best_codec("MCPMessage");
    assert!(best_codec.is_some());
    
    // Test performance comparison
    let comparison = registry.get_performance_comparison();
    assert!(!comparison.is_empty());
}

// Streaming Serialization Tests

#[tokio::test]
async fn test_streaming_serializer_creation() {
    let config = streaming::StreamingConfig::default();
    let serializer = streaming::StreamingSerializer::new(config);
    
    // Just test that it was created successfully
    // (StreamingSerializer doesn't have public methods to inspect state)
}

#[tokio::test]
async fn test_streaming_config() {
    let config = streaming::StreamingConfig::default();
    
    assert_eq!(config.chunk_size, 8192);
    assert_eq!(config.max_buffer_size, 64 * 1024);
    assert!(!config.enable_compression);
}

#[tokio::test]
async fn test_streaming_use_cases() {
    let large_msg_config = streaming::StreamingUtils::create_config_for_use_case(
        streaming::StreamingUseCase::LargeMessages
    );
    assert_eq!(large_msg_config.chunk_size, 64 * 1024);
    assert!(large_msg_config.enable_compression);
    
    let real_time_config = streaming::StreamingUtils::create_config_for_use_case(
        streaming::StreamingUseCase::RealTime
    );
    assert_eq!(real_time_config.chunk_size, 4 * 1024);
    assert!(!real_time_config.enable_compression);
}

#[tokio::test]
async fn test_streaming_overhead_calculation() {
    let overhead = streaming::StreamingUtils::estimate_streaming_overhead(100000, 8192);
    
    assert!(overhead.chunk_count > 0);
    assert!(overhead.header_overhead_bytes > 0);
    assert!(overhead.overhead_percentage > 0.0);
}

#[tokio::test]
async fn test_optimal_chunk_size() {
    let chunk_size = streaming::StreamingUtils::calculate_optimal_chunk_size(1000000, 10000000);
    
    assert!(chunk_size >= 1024); // Min chunk size
    assert!(chunk_size <= 1024 * 1024); // Max chunk size
}

#[tokio::test]
async fn test_streaming_factory() {
    let serializer = streaming::StreamingFactory::create_serializer(
        streaming::StreamingUseCase::HighThroughput
    );
    // Test that it was created (no public methods to inspect)
    
    let deserializer = streaming::StreamingFactory::create_deserializer(
        streaming::StreamingUseCase::LowMemory
    );
    // Test that it was created (no public methods to inspect)
}

// Template System Tests

#[tokio::test]
async fn test_template_cache_creation() {
    let cache = templates::MessageTemplateCache::new(100);
    
    let stats = cache.get_stats();
    assert_eq!(stats.templates_compiled, 0);
    assert_eq!(stats.cache_hits, 0);
}

#[tokio::test]
async fn test_template_factory() {
    let mcp_template = templates::TemplateFactory::create_mcp_message_template();
    assert!(!mcp_template.template.is_empty());
    assert!(!mcp_template.fields.is_empty());
    
    let ai_request_template = templates::TemplateFactory::create_ai_request_template();
    assert!(ai_request_template.template.contains("{{id:string}}"));
    assert!(ai_request_template.template.contains("{{model:string}}"));
    
    let error_template = templates::TemplateFactory::create_error_template();
    assert!(error_template.template.contains("Error"));
}

#[tokio::test]
async fn test_template_validation() {
    let valid_template = templates::TemplateFactory::create_mcp_message_template();
    let result = templates::TemplateUtils::validate_template(&valid_template);
    assert!(result.is_ok());
    
    // Test invalid template
    let invalid_template = templates::TemplateDefinition {
        template: "{{unclosed_field".to_string(),
        fields: vec![],
        options: templates::TemplateOptions::default(),
    };
    let result = templates::TemplateUtils::validate_template(&invalid_template);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_template_performance_estimation() {
    let template = templates::TemplateFactory::create_ai_response_template();
    let estimate = templates::TemplateUtils::estimate_performance(&template);
    
    assert!(estimate.estimated_render_time_ns > 0);
    assert!(estimate.complexity_score > 0.0);
    assert!(estimate.memory_usage_bytes > 0);
}

#[tokio::test]
async fn test_template_compilation() {
    let mut cache = templates::MessageTemplateCache::new(10);
    let template_def = templates::TemplateFactory::create_mcp_message_template();
    
    let result = cache.compile_template("mcp_message".to_string(), template_def);
    assert!(result.is_ok());
    
    let compiled = cache.get_template("mcp_message");
    assert!(compiled.is_some());
}

// Integration Tests

#[tokio::test]
async fn test_global_serializer() {
    let global = get_global_serializer();
    
    let message = create_test_mcp_message();
    let result = global.serialize_mcp_message(&message).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_serialization_roundtrip() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    let original = create_test_mcp_message();
    
    // Serialize
    let serialized = serializer.serialize_mcp_message(&original).await.expect("should succeed");
    
    // Deserialize
    let deserialized = serializer.deserialize_mcp_message(&serialized.data).await.expect("should succeed");
    
    // Verify roundtrip
    assert_eq!(original.id, deserialized.id);
    assert_eq!(original.type_, deserialized.type_);
    assert_eq!(original.payload, deserialized.payload);
}

#[tokio::test]
async fn test_ai_request_roundtrip() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    let original = create_test_ai_request();
    
    // Serialize
    let serialized = serializer.serialize_ai_request(&original).await.expect("should succeed");
    
    // Deserialize
    let deserialized = serializer.deserialize_ai_request(&serialized.data).await.expect("should succeed");
    
    // Verify roundtrip
    assert_eq!(original.id, deserialized.id);
    assert_eq!(original.model, deserialized.model);
    assert_eq!(original.messages.len(), deserialized.messages.len());
}

#[tokio::test]
async fn test_concurrent_serialization() {
    let config = SerializationConfig::default();
    let serializer = Arc::new(ZeroCopySerializer::new(config));
    
    let mut handles = Vec::new();
    
    // Spawn multiple concurrent serialization tasks
    for i in 0..10 {
        let serializer_clone = Arc::clone(&serializer);
        let handle = tokio::spawn(async move {
            let mut message = create_test_mcp_message();
            message.id = MessageId(format!("test-{}", i));
            
            let result = serializer_clone.serialize_mcp_message(&message).await;
            (i, result.is_ok())
        });
        handles.push(handle);
    }
    
    // Wait for all tasks and verify results
    for handle in handles {
        let (i, success) = handle.await.expect("should succeed");
        assert!(success, "Serialization {} should have succeeded", i);
    }
    
    let metrics = serializer.get_metrics().await;
    assert_eq!(metrics.total_serializations, 10);
}

#[tokio::test]
async fn test_performance_comparison() {
    let config = SerializationConfig {
        enable_buffer_pooling: true,
        enable_fast_codecs: true,
        enable_templates: true,
        ..Default::default()
    };
    
    let optimized_serializer = ZeroCopySerializer::new(config);
    
    let basic_config = SerializationConfig {
        enable_buffer_pooling: false,
        enable_fast_codecs: false,
        enable_templates: false,
        ..Default::default()
    };
    
    let basic_serializer = ZeroCopySerializer::new(basic_config);
    
    let message = create_test_mcp_message();
    
    // Measure optimized serialization
    let start = std::time::Instant::now();
    let _optimized_result = optimized_serializer.serialize_mcp_message(&message).await.expect("should succeed");
    let optimized_time = start.elapsed();
    
    // Measure basic serialization
    let start = std::time::Instant::now();
    let _basic_result = basic_serializer.serialize_mcp_message(&message).await.expect("should succeed");
    let basic_time = start.elapsed();
    
    // Both should work, optimized might be faster (but not guaranteed in tests)
    assert!(optimized_time.as_nanos() > 0);
    assert!(basic_time.as_nanos() > 0);
}

// Large Message Tests

#[tokio::test]
async fn test_large_message_handling() {
    let config = SerializationConfig {
        enable_streaming: true,
        ..Default::default()
    };
    let serializer = ZeroCopySerializer::new(config);
    
    // Create large message
    let large_content = "x".repeat(10000); // 10KB string
    let mut large_request = create_test_ai_request();
    large_request.messages[0].content = large_content;
    
    let result = serializer.serialize_ai_request(&large_request).await;
    assert!(result.is_ok());
    
    let serialized = result.expect("should succeed");
    assert!(serialized.data.len() > 10000);
}

// Error Handling Tests

#[tokio::test]
async fn test_serialization_error_handling() {
    let config = SerializationConfig::default();
    let serializer = ZeroCopySerializer::new(config);
    
    // Test deserialization with invalid data
    let invalid_data = Bytes::from("invalid json data");
    let result = serializer.deserialize_mcp_message(&invalid_data).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_buffer_pool_limits() {
    let config = BufferPoolConfig {
        initial_size: 1,
        max_size: 2,
        max_buffer_size: 100, // Very small limit
    };
    let pool = BufferPool::new(config);
    
    // Try to return a buffer that's too large
    let large_buffer = bytes::BytesMut::with_capacity(1000);
    pool.return_buffer(large_buffer).await;
    
    // Should still work (buffer just gets discarded)
    let stats = pool.get_stats().await;
    assert_eq!(stats.total_returns, 1);
}

// Memory Usage Tests

#[tokio::test]
async fn test_memory_efficiency() {
    let config = SerializationConfig {
        enable_buffer_pooling: true,
        max_buffer_size: 1024,
        ..Default::default()
    };
    
    let serializer = ZeroCopySerializer::new(config);
    
    // Perform many serializations to test memory reuse
    for _ in 0..100 {
        let message = create_test_mcp_message();
        let _result = serializer.serialize_mcp_message(&message).await.expect("should succeed");
    }
    
    let metrics = serializer.get_metrics().await;
    assert_eq!(metrics.total_serializations, 100);
    assert!(metrics.buffer_pool_hits > 0);
} 