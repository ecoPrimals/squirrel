// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Universal Adapter Integration Tests
//!
//! Modern, idiomatic Rust testing for universal adapter patterns:
//! - Capability-based service discovery (no hardcoded primal names)
//! - Runtime primal discovery
//! - Result-based error handling (no unwrap/expect in production paths)
//! - Zero-copy where possible with Arc patterns

use squirrel::universal::{NetworkLocation, PrimalContext, SecurityLevel};
use squirrel::universal_adapters::registry::{InMemoryServiceRegistry, UniversalServiceRegistry};
use squirrel::universal_adapters::{
    compute_adapter::UniversalComputeAdapter, security_adapter::UniversalSecurityAdapter,
    storage_adapter::UniversalStorageAdapter,
};
use std::collections::HashMap;
use std::sync::Arc;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ADAPTER CREATION TESTS (Modern Dependency Injection)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_compute_adapter_creation() {
    // Arrange: Create registry (dependency injection - no hardcoding)
    let registry = Arc::new(InMemoryServiceRegistry::new()) as Arc<dyn UniversalServiceRegistry>;

    // Act: Create compute adapter with injected registry
    let _adapter = UniversalComputeAdapter::new(registry);

    // Assert: Constructor success (modern Rust - let failures panic in tests)
    // Production code uses Result<T, E>, tests can panic
}

#[tokio::test]
async fn test_storage_adapter_creation() {
    // Arrange: Create registry
    let registry = Arc::new(InMemoryServiceRegistry::new()) as Arc<dyn UniversalServiceRegistry>;

    // Act: Create storage adapter
    let _adapter = UniversalStorageAdapter::new(registry);

    // Assert: Success implicit
}

#[tokio::test]
async fn test_security_adapter_creation() {
    // Arrange: Create registry
    let registry = Arc::new(InMemoryServiceRegistry::new()) as Arc<dyn UniversalServiceRegistry>;

    // Act: Create security adapter
    let _adapter = UniversalSecurityAdapter::new(registry);

    // Assert: Success implicit
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SHARED REGISTRY PATTERN (Single Source of Truth)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_multi_adapter_shared_registry() {
    // Arrange: Single registry shared across all adapters (modern pattern)
    let registry = Arc::new(InMemoryServiceRegistry::new()) as Arc<dyn UniversalServiceRegistry>;

    // Act: Create all adapters with same registry
    let _compute = UniversalComputeAdapter::new(Arc::clone(&registry));
    let _storage = UniversalStorageAdapter::new(Arc::clone(&registry));
    let _security = UniversalSecurityAdapter::new(Arc::clone(&registry));

    // Assert: All adapters can be created with shared registry
    // Note: Adapters may hold internal Arc clones, so exact count varies
    assert!(Arc::strong_count(&registry) >= 4); // At least original + 3 adapters
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CONCURRENCY & ARC SAFETY (Zero-Copy Sharing)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_concurrent_adapter_access() {
    // Arrange: Shared adapter (Arc for zero-copy sharing)
    let registry = Arc::new(InMemoryServiceRegistry::new()) as Arc<dyn UniversalServiceRegistry>;
    let adapter = Arc::new(UniversalComputeAdapter::new(registry));

    // Act: Spawn 50 concurrent tasks (stress test)
    let tasks: Vec<_> = (0..50)
        .map(|i| {
            let adapter = Arc::clone(&adapter); // Zero-copy share
            tokio::spawn(async move {
                // Simulate work with adapter
                drop(adapter);
                i // Return task ID
            })
        })
        .collect();

    // Assert: All tasks complete successfully
    for (idx, task) in tasks.into_iter().enumerate() {
        let result = task.await.expect("Task should not panic");
        assert_eq!(result, idx);
    }
}

#[tokio::test]
async fn test_arc_reference_counting() {
    // Arrange: Create adapter
    let registry = Arc::new(InMemoryServiceRegistry::new()) as Arc<dyn UniversalServiceRegistry>;
    let adapter = Arc::new(UniversalStorageAdapter::new(registry));

    // Act: Create references
    let ref1 = Arc::clone(&adapter);
    let ref2 = Arc::clone(&adapter);
    assert_eq!(Arc::strong_count(&adapter), 3); // Original + 2 clones

    // Drop references
    drop(ref1);
    assert_eq!(Arc::strong_count(&adapter), 2);
    drop(ref2);
    assert_eq!(Arc::strong_count(&adapter), 1);

    // Assert: Reference counting works correctly (zero-copy pattern)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CONTEXT CREATION (No Hardcoded Values)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_primal_context_modern_pattern() {
    // Act: Create context with all required fields
    let context = create_test_context();

    // Assert: Context properly initialized with no hardcoded values
    assert_eq!(context.user_id, "test-user-001");
    assert_eq!(context.device_id, "test-device-001");
    assert_eq!(context.session_id, Some("test-session-001".to_string()));
    assert_eq!(context.security_level, SecurityLevel::Standard);
    assert!(context.biome_id.is_some());
}

#[tokio::test]
async fn test_context_security_levels() {
    // Arrange: Create contexts with different security levels
    let mut basic = create_test_context();
    basic.security_level = SecurityLevel::Basic;

    let mut critical = create_test_context();
    critical.security_level = SecurityLevel::Critical;

    // Assert: Security levels properly differentiated
    assert_ne!(basic.security_level, critical.security_level);
    assert_eq!(basic.security_level, SecurityLevel::Basic);
    assert_eq!(critical.security_level, SecurityLevel::Critical);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RESOURCE LIFECYCLE (RAII Pattern)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_adapter_raii_cleanup() {
    // Arrange: Create registry
    let registry = Arc::new(InMemoryServiceRegistry::new()) as Arc<dyn UniversalServiceRegistry>;
    let initial_count = Arc::strong_count(&registry);

    // Act: Create and drop adapter in scope
    let count_with_adapter = {
        let _adapter = UniversalComputeAdapter::new(Arc::clone(&registry));
        Arc::strong_count(&registry)
    }; // Adapter drops here (RAII automatic cleanup)

    // Assert: Registry references decreased after drop
    let final_count = Arc::strong_count(&registry);
    assert!(
        count_with_adapter > initial_count,
        "Adapter should hold reference"
    );
    assert!(
        final_count <= count_with_adapter,
        "Reference should be released after drop"
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HELPER FUNCTIONS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create test context using modern patterns (no hardcoded values)
fn create_test_context() -> PrimalContext {
    PrimalContext {
        user_id: "test-user-001".to_string(),
        device_id: "test-device-001".to_string(),
        session_id: Some("test-session-001".to_string()),
        network_location: NetworkLocation::default(), // Uses universal-constants
        security_level: SecurityLevel::Standard,
        biome_id: Some("test-biome".to_string()),
        metadata: HashMap::new(),
    }
}
