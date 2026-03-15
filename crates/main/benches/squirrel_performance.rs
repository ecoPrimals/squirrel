// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive performance benchmarks for ecoPrimals Squirrel
//!
//! This benchmark suite tests the performance of key components:
//! - Error handling and type conversions
//! - Protocol message processing
//! - Session management
//! - Transport layer operations
//! - Enhanced MCP functionality

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use uuid::Uuid;

use squirrel::enhanced::{ClientInfo, EnhancedMCPConfig, EnhancedMCPServer, MCPRequest};
use squirrel::error::PrimalError;
use squirrel::protocol::types::*;
use squirrel::session::*;
use squirrel::transport::types::*;

/// Benchmark error creation and handling
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    // Benchmark error creation
    group.bench_function("create_validation_error", |b| {
        b.iter(|| {
            black_box(MCPError::ValidationFailed(
                "Test validation error".to_string(),
            ))
        })
    });

    group.bench_function("create_operation_error", |b| {
        b.iter(|| {
            black_box(MCPError::OperationFailed(
                "Test operation error".to_string(),
            ))
        })
    });

    group.bench_function("error_code_lookup", |b| {
        let error = MCPError::ValidationFailed("Test".to_string());
        b.iter(|| black_box(error.error_code()))
    });

    group.finish();
}

/// Benchmark protocol types operations
fn bench_protocol_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_types");

    // NOTE: AuthCredentials and SecurityMetadata types needed for full benchmark
    // Benchmark AuthCredentials creation
    // group.bench_function("create_auth_credentials", |b| {
    //     b.iter(|| {
    //         black_box(AuthCredentials {
    //             username: "test_user".to_string(),
    //             password: "test_pass".to_string(),
    //             token: Some("test_token".to_string()),
    //             metadata: std::collections::HashMap::new(),
    //         })
    //     })
    // });

    // Benchmark SecurityMetadata creation
    // group.bench_function("create_security_metadata", |b| {
    //     b.iter(|| {
    //         black_box(SecurityMetadata {
    //             version: "1.0".to_string(),
    //             token: Some("test_token".to_string()),
    //             encrypted: true,
    //             timestamp: chrono::Utc::now(),
    //         })
    //     })
    // });

    group.finish();
}

/// Benchmark session management
fn bench_session_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_management");

    // Benchmark session config creation
    group.bench_function("create_session_config", |b| {
        b.iter(|| black_box(SessionConfig::default()))
    });

    // Benchmark session metadata creation
    group.bench_function("create_session_metadata", |b| {
        b.iter(|| black_box(SessionMetadata::default()))
    });

    group.finish();
}

/// Benchmark transport layer operations
fn bench_transport_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("transport_operations");

    // Benchmark connection metadata creation
    group.bench_function("create_connection_metadata", |b| {
        b.iter(|| black_box(ConnectionMetadata::default()))
    });

    // Benchmark transport config creation
    group.bench_function("create_transport_config", |b| {
        b.iter(|| black_box(TransportConfig::default()))
    });

    // Benchmark frame metadata creation
    group.bench_function("create_frame_metadata", |b| {
        b.iter(|| black_box(FrameMetadata::default()))
    });

    group.finish();
}

/// Benchmark enhanced MCP operations
fn bench_enhanced_mcp(c: &mut Criterion) {
    let mut group = c.benchmark_group("enhanced_mcp");

    // Benchmark server creation
    group.bench_function("create_enhanced_server", |b| {
        b.iter(|| {
            let config = EnhancedMCPConfig::default();
            black_box(EnhancedMCPServer::new(config))
        })
    });

    // Benchmark session creation
    group.bench_function("create_session", |b| {
        let config = EnhancedMCPConfig::default();
        let server = EnhancedMCPServer::new(config);
        let client_info = ClientInfo {
            name: "test_client".to_string(),
            version: "1.0".to_string(),
            platform: Some("test".to_string()),
        };

        b.iter(|| black_box(server.create_session_sync(client_info.clone()).unwrap()))
    });

    // Benchmark request handling
    group.bench_function("handle_mcp_request", |b| {
        let config = EnhancedMCPConfig::default();
        let server = EnhancedMCPServer::new(config);
        let request = MCPRequest::GetStatus;

        b.iter(|| {
            black_box(
                server
                    .handle_mcp_request_sync("test_session", request.clone())
                    .unwrap(),
            )
        })
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    // Benchmark concurrent session creation (simulated)
    group.bench_function("concurrent_sessions", |b| {
        let config = EnhancedMCPConfig::default();
        let server = EnhancedMCPServer::new(config);

        b.iter(|| {
            let mut results = Vec::new();

            for i in 0..10 {
                let client_info = ClientInfo {
                    name: format!("client_{}", i),
                    version: "1.0".to_string(),
                    platform: Some("benchmark".to_string()),
                };

                let result = server.create_session_sync(client_info).unwrap();
                results.push(result);
            }

            black_box(results)
        })
    });

    // Benchmark concurrent request handling (simulated)
    group.bench_function("concurrent_requests", |b| {
        let config = EnhancedMCPConfig::default();
        let server = EnhancedMCPServer::new(config);

        b.iter(|| {
            let mut results = Vec::new();

            for i in 0..20 {
                let session_id = format!("session_{}", i);
                let request = MCPRequest::GetStatus;

                let result = server
                    .handle_mcp_request_sync(&session_id, request)
                    .unwrap();
                results.push(result);
            }

            black_box(results)
        })
    });

    group.finish();
}

/// Benchmark memory operations
fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");

    // Benchmark large metadata creation
    group.bench_function("large_metadata_creation", |b| {
        b.iter(|| {
            let mut metadata = std::collections::HashMap::new();
            for i in 0..100 {
                metadata.insert(format!("key_{}", i), format!("value_{}", i));
            }
            black_box(metadata)
        })
    });

    // Benchmark UUID generation
    group.bench_function("uuid_generation", |b| {
        b.iter(|| black_box(Uuid::new_v4().to_string()))
    });

    // Benchmark timestamp generation
    group.bench_function("timestamp_generation", |b| {
        b.iter(|| black_box(chrono::Utc::now()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_error_handling,
    bench_protocol_types,
    bench_session_management,
    bench_transport_operations,
    bench_enhanced_mcp,
    bench_concurrent_operations,
    bench_memory_operations
);

criterion_main!(benches);
