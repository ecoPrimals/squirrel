// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! MCP Protocol Performance Benchmarks
//!
//! This benchmark suite tests the performance of MCP protocol components:
//! - Message serialization and deserialization
//! - Protocol type operations
//! - Session management
//! - Connection handling

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

use squirrel::protocol::types::*;
use squirrel::session::*;
use squirrel::transport::types::*;

/// Benchmark protocol message serialization
fn bench_message_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialization");

    // Benchmark AuthCredentials serialization
    group.bench_function("serialize_auth_credentials", |b| {
        let credentials = AuthCredentials {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            token: Some("test_token".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        b.iter(|| black_box(serde_json::to_string(&credentials).unwrap()))
    });

    // Benchmark SecurityMetadata serialization
    group.bench_function("serialize_security_metadata", |b| {
        let metadata = SecurityMetadata {
            version: "1.0".to_string(),
            token: Some("test_token".to_string()),
            encrypted: true,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        b.iter(|| black_box(serde_json::to_string(&metadata).unwrap()))
    });

    group.finish();
}

/// Benchmark protocol type operations
fn bench_protocol_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_operations");

    // Benchmark credential validation
    group.bench_function("validate_credentials", |b| {
        let credentials = AuthCredentials {
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            token: Some("test_token".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        b.iter(|| {
            let is_valid = !credentials.username.is_empty()
                && !credentials.password.is_empty()
                && credentials.token.is_some();
            black_box(is_valid)
        })
    });

    // Benchmark metadata lookup
    group.bench_function("metadata_lookup", |b| {
        let mut metadata = std::collections::HashMap::new();
        for i in 0..100 {
            metadata.insert(format!("key_{}", i), format!("value_{}", i));
        }

        b.iter(|| black_box(metadata.get("key_50")))
    });

    group.finish();
}

/// Benchmark session operations
fn bench_session_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_operations");

    // Benchmark session creation
    group.bench_function("create_session", |b| {
        b.iter(|| {
            let session = SessionMetadata {
                session_id: Uuid::new_v4().to_string(),
                user_id: "test_user".to_string(),
                created_at: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };
            black_box(session)
        })
    });

    // Benchmark session validation
    group.bench_function("validate_session", |b| {
        let session = SessionMetadata {
            session_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        b.iter(|| {
            let is_valid = !session.session_id.is_empty() && !session.user_id.is_empty();
            black_box(is_valid)
        })
    });

    group.finish();
}

/// Benchmark connection operations
fn bench_connection_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_operations");

    // Benchmark connection metadata creation
    group.bench_function("create_connection_metadata", |b| {
        b.iter(|| {
            let metadata = ConnectionMetadata {
                connection_id: Uuid::new_v4().to_string(),
                remote_address: Some("127.0.0.1:8080".to_string()),
                local_address: Some("127.0.0.1:8081".to_string()),
                connected_at: chrono::Utc::now(),
                protocol_version: "1.0".to_string(),
                metadata: std::collections::HashMap::new(),
            };
            black_box(metadata)
        })
    });

    // Benchmark frame processing
    group.bench_function("process_frame", |b| {
        let frame = FrameMetadata {
            frame_id: Uuid::new_v4().to_string(),
            frame_type: "data".to_string(),
            size: 1024,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };

        b.iter(|| {
            let processed = frame.size > 0 && !frame.frame_type.is_empty();
            black_box(processed)
        })
    });

    group.finish();
}

/// Benchmark concurrent protocol operations
fn bench_concurrent_protocol(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_protocol");
    let rt = Runtime::new().unwrap();

    // Benchmark concurrent session creation
    group.bench_function("concurrent_session_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..10 {
                handles.push(tokio::spawn(async move {
                    let session = SessionMetadata {
                        session_id: Uuid::new_v4().to_string(),
                        user_id: format!("user_{}", i),
                        created_at: chrono::Utc::now(),
                        last_activity: chrono::Utc::now(),
                        metadata: std::collections::HashMap::new(),
                    };
                    black_box(session)
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap());
            }
        })
    });

    // Benchmark concurrent message processing
    group.bench_function("concurrent_message_processing", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..20 {
                handles.push(tokio::spawn(async move {
                    let metadata = SecurityMetadata {
                        version: "1.0".to_string(),
                        token: Some(format!("token_{}", i)),
                        encrypted: true,
                        timestamp: chrono::Utc::now(),
                        metadata: std::collections::HashMap::new(),
                    };

                    let serialized = serde_json::to_string(&metadata).unwrap();
                    black_box(serialized)
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap());
            }
        })
    });

    group.finish();
}

/// Benchmark memory usage
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // Benchmark large message creation
    group.bench_function("large_message_creation", |b| {
        b.iter(|| {
            let mut metadata = std::collections::HashMap::new();
            for i in 0..1000 {
                metadata.insert(
                    format!("key_{}", i),
                    format!("value_with_long_content_{}", i),
                );
            }

            let large_metadata = SecurityMetadata {
                version: "1.0".to_string(),
                token: Some("large_token_with_lots_of_content".to_string()),
                encrypted: true,
                timestamp: chrono::Utc::now(),
                metadata,
            };

            black_box(large_metadata)
        })
    });

    // Benchmark bulk session creation
    group.bench_function("bulk_session_creation", |b| {
        b.iter(|| {
            let mut sessions = Vec::new();

            for i in 0..100 {
                let session = SessionMetadata {
                    session_id: Uuid::new_v4().to_string(),
                    user_id: format!("user_{}", i),
                    created_at: chrono::Utc::now(),
                    last_activity: chrono::Utc::now(),
                    metadata: std::collections::HashMap::new(),
                };
                sessions.push(session);
            }

            black_box(sessions)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_message_serialization,
    bench_protocol_operations,
    bench_session_operations,
    bench_connection_operations,
    bench_concurrent_protocol,
    bench_memory_usage
);

criterion_main!(benches);
