// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Beardog Security Integration Performance Benchmarks
//!
//! This benchmark suite tests the performance of Beardog security components:
//! - Authentication operations
//! - Authorization checks
//! - Token validation and management
//! - Security metadata processing
//! - Credential handling

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

use squirrel::protocol::types::AuthCredentials;
use universal_patterns::config::ConfigBuilder;
use universal_patterns::security::BeardogIntegration;
use universal_patterns::traits::{AuthResult, Credentials, Principal};

/// Setup test security integration
fn setup_test_security() -> BeardogIntegration {
    let config = ConfigBuilder::new()
        .beardog()
        .with_endpoint("http://localhost:8443")
        .with_environment("test")
        .build()
        .unwrap();

    BeardogIntegration::new(config)
}

/// Benchmark authentication operations
fn bench_authentication(c: &mut Criterion) {
    let mut group = c.benchmark_group("authentication");
    let rt = Runtime::new().unwrap();

    // Benchmark basic authentication
    group.bench_function("basic_authentication", |b| {
        let security = setup_test_security();

        let credentials = Credentials {
            username: "benchmark_user".to_string(),
            password: "benchmark_pass".to_string(),
            token: Some("benchmark_token".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        b.to_async(&rt)
            .iter(|| async { black_box(security.authenticate(credentials.clone()).await.unwrap()) })
    });

    // Benchmark authentication with metadata
    group.bench_function("authentication_with_metadata", |b| {
        let security = setup_test_security();

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("client_id".to_string(), "benchmark_client".to_string());
        metadata.insert("ip_address".to_string(), "127.0.0.1".to_string());
        metadata.insert("user_agent".to_string(), "benchmark_agent".to_string());

        let credentials = Credentials {
            username: "benchmark_user".to_string(),
            password: "benchmark_pass".to_string(),
            token: Some("benchmark_token".to_string()),
            metadata,
        };

        b.to_async(&rt)
            .iter(|| async { black_box(security.authenticate(credentials.clone()).await.unwrap()) })
    });

    group.finish();
}

/// Benchmark authorization operations
fn bench_authorization(c: &mut Criterion) {
    let mut group = c.benchmark_group("authorization");
    let rt = Runtime::new().unwrap();

    // Benchmark basic authorization
    group.bench_function("basic_authorization", |b| {
        let security = setup_test_security();

        let principal = Principal {
            id: "benchmark_user".to_string(),
            name: "Benchmark User".to_string(),
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string()],
            metadata: std::collections::HashMap::new(),
        };

        b.to_async(&rt).iter(|| async {
            black_box(security.authorize(principal.clone(), "read").await.unwrap())
        })
    });

    // Benchmark authorization with multiple roles
    group.bench_function("authorization_multiple_roles", |b| {
        let security = setup_test_security();

        let principal = Principal {
            id: "benchmark_admin".to_string(),
            name: "Benchmark Admin".to_string(),
            roles: vec![
                "user".to_string(),
                "admin".to_string(),
                "moderator".to_string(),
                "developer".to_string(),
            ],
            permissions: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "admin".to_string(),
            ],
            metadata: std::collections::HashMap::new(),
        };

        b.to_async(&rt).iter(|| async {
            black_box(
                security
                    .authorize(principal.clone(), "admin")
                    .await
                    .unwrap(),
            )
        })
    });

    group.finish();
}

/// Benchmark token operations
fn bench_token_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_operations");
    let rt = Runtime::new().unwrap();

    // Benchmark token validation
    group.bench_function("token_validation", |b| {
        let security = setup_test_security();

        b.to_async(&rt)
            .iter(|| async { black_box(security.validate_token("benchmark_token").await.unwrap()) })
    });

    // Benchmark token refresh
    group.bench_function("token_refresh", |b| {
        let security = setup_test_security();

        b.to_async(&rt)
            .iter(|| async { black_box(security.refresh_token("benchmark_token").await.unwrap()) })
    });

    // Benchmark token revocation
    group.bench_function("token_revocation", |b| {
        let security = setup_test_security();

        b.to_async(&rt)
            .iter(|| async { black_box(security.revoke_token("benchmark_token").await.unwrap()) })
    });

    group.finish();
}

/// Benchmark credential processing
fn bench_credential_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("credential_processing");

    // Benchmark credential creation
    group.bench_function("credential_creation", |b| {
        b.iter(|| {
            let credentials = AuthCredentials {
                username: "benchmark_user".to_string(),
                password: "benchmark_pass".to_string(),
                token: Some("benchmark_token".to_string()),
                metadata: std::collections::HashMap::new(),
            };
            black_box(credentials)
        })
    });

    // Benchmark credential validation
    group.bench_function("credential_validation", |b| {
        let credentials = AuthCredentials {
            username: "benchmark_user".to_string(),
            password: "benchmark_pass".to_string(),
            token: Some("benchmark_token".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        b.iter(|| {
            let is_valid = !credentials.username.is_empty()
                && !credentials.password.is_empty()
                && credentials.token.is_some();
            black_box(is_valid)
        })
    });

    // Benchmark credential conversion
    group.bench_function("credential_conversion", |b| {
        let auth_credentials = AuthCredentials {
            username: "benchmark_user".to_string(),
            password: "benchmark_pass".to_string(),
            token: Some("benchmark_token".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        b.iter(|| {
            let credentials = Credentials {
                username: auth_credentials.username.clone(),
                password: auth_credentials.password.clone(),
                token: auth_credentials.token.clone(),
                metadata: auth_credentials.metadata.clone(),
            };
            black_box(credentials)
        })
    });

    group.finish();
}

/// Benchmark concurrent security operations
fn bench_concurrent_security(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_security");
    let rt = Runtime::new().unwrap();

    // Benchmark concurrent authentication
    group.bench_function("concurrent_authentication", |b| {
        let security = setup_test_security();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..10 {
                let security_clone = security.clone();
                let credentials = Credentials {
                    username: format!("user_{}", i),
                    password: "password".to_string(),
                    token: Some(format!("token_{}", i)),
                    metadata: std::collections::HashMap::new(),
                };

                handles.push(tokio::spawn(async move {
                    security_clone.authenticate(credentials).await
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    // Benchmark concurrent authorization
    group.bench_function("concurrent_authorization", |b| {
        let security = setup_test_security();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..10 {
                let security_clone = security.clone();
                let principal = Principal {
                    id: format!("user_{}", i),
                    name: format!("User {}", i),
                    roles: vec!["user".to_string()],
                    permissions: vec!["read".to_string()],
                    metadata: std::collections::HashMap::new(),
                };

                handles.push(tokio::spawn(async move {
                    security_clone.authorize(principal, "read").await
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    // Benchmark concurrent token validation
    group.bench_function("concurrent_token_validation", |b| {
        let security = setup_test_security();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..15 {
                let security_clone = security.clone();
                let token = format!("token_{}", i);

                handles.push(tokio::spawn(async move {
                    security_clone.validate_token(&token).await
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    group.finish();
}

/// Benchmark security load testing
fn bench_security_load_testing(c: &mut Criterion) {
    let mut group = c.benchmark_group("security_load_testing");
    let rt = Runtime::new().unwrap();

    // Benchmark high-volume authentication
    group.bench_function("high_volume_authentication", |b| {
        let security = setup_test_security();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..100 {
                let security_clone = security.clone();
                let credentials = Credentials {
                    username: format!("load_test_user_{}", i),
                    password: "load_test_pass".to_string(),
                    token: Some(format!("load_test_token_{}", i)),
                    metadata: std::collections::HashMap::new(),
                };

                handles.push(tokio::spawn(async move {
                    security_clone.authenticate(credentials).await
                }));
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    // Benchmark mixed security operations
    group.bench_function("mixed_security_operations", |b| {
        let security = setup_test_security();

        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();

            for i in 0..50 {
                let security_clone = security.clone();

                match i % 3 {
                    0 => {
                        // Authentication
                        let credentials = Credentials {
                            username: format!("mixed_user_{}", i),
                            password: "mixed_pass".to_string(),
                            token: Some(format!("mixed_token_{}", i)),
                            metadata: std::collections::HashMap::new(),
                        };

                        handles.push(tokio::spawn(async move {
                            security_clone.authenticate(credentials).await.map(|_| ())
                        }));
                    }
                    1 => {
                        // Authorization
                        let principal = Principal {
                            id: format!("mixed_user_{}", i),
                            name: format!("Mixed User {}", i),
                            roles: vec!["user".to_string()],
                            permissions: vec!["read".to_string()],
                            metadata: std::collections::HashMap::new(),
                        };

                        handles.push(tokio::spawn(async move {
                            security_clone
                                .authorize(principal, "read")
                                .await
                                .map(|_| ())
                        }));
                    }
                    2 => {
                        // Token validation
                        let token = format!("mixed_token_{}", i);

                        handles.push(tokio::spawn(async move {
                            security_clone.validate_token(&token).await.map(|_| ())
                        }));
                    }
                    _ => unreachable!(),
                }
            }

            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        })
    });

    group.finish();
}

/// Benchmark memory usage
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // Benchmark large credential creation
    group.bench_function("large_credential_creation", |b| {
        b.iter(|| {
            let mut metadata = std::collections::HashMap::new();
            for i in 0..1000 {
                metadata.insert(format!("meta_key_{}", i), format!("meta_value_{}", i));
            }

            let credentials = Credentials {
                username: "large_credential_user".to_string(),
                password: "large_credential_pass".to_string(),
                token: Some("large_credential_token_with_lots_of_content".to_string()),
                metadata,
            };

            black_box(credentials)
        })
    });

    // Benchmark principal with many roles
    group.bench_function("principal_many_roles", |b| {
        b.iter(|| {
            let mut roles = Vec::new();
            let mut permissions = Vec::new();

            for i in 0..100 {
                roles.push(format!("role_{}", i));
                permissions.push(format!("permission_{}", i));
            }

            let principal = Principal {
                id: "complex_user".to_string(),
                name: "Complex User".to_string(),
                roles,
                permissions,
                metadata: std::collections::HashMap::new(),
            };

            black_box(principal)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_authentication,
    bench_authorization,
    bench_token_operations,
    bench_credential_processing,
    bench_concurrent_security,
    bench_security_load_testing,
    bench_memory_usage
);

criterion_main!(benches);
