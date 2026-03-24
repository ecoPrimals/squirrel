// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)] // Invariant or startup failure: unwrap/expect after validation
//! # Observability Framework Demo
//!
//! This example demonstrates the comprehensive observability capabilities
//! including structured logging, correlation IDs, and performance metrics.

use squirrel::observability::{CorrelationId, OperationContext, utils};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_target(false)
        .with_thread_ids(true)
        .init();

    info!("🚀 Starting Observability Framework Demo");

    // Demo 1: Basic operation tracking
    demo_basic_operation().await?;

    // Demo 2: Service call with correlation
    demo_service_call_with_correlation().await?;

    // Demo 3: API operation with metadata
    demo_api_operation_with_metadata().await?;

    // Demo 4: Failed operation tracking
    demo_failed_operation().await?;

    info!("✅ Observability Framework Demo completed successfully");
    Ok(())
}

/// Demo basic operation tracking
async fn demo_basic_operation() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = OperationContext::new("demo_basic_operation")
        .with_metadata("demo_type", "basic")
        .with_metadata("version", "1.0");

    ctx.log_start();

    // Simulate some work
    sleep(Duration::from_millis(100)).await;
    ctx.metrics
        .record_phase("initialization", Duration::from_millis(50));

    sleep(Duration::from_millis(200)).await;
    ctx.metrics
        .record_phase("processing", Duration::from_millis(200));

    // Complete successfully
    let result = ctx.complete_success();

    info!(
        "Demo 1 completed: {} in {:?} with {} phases",
        result.operation,
        result.duration(),
        result.metrics.phase_durations.len()
    );

    Ok(())
}

/// Demo service call with correlation ID propagation
async fn demo_service_call_with_correlation() -> Result<(), Box<dyn std::error::Error>> {
    let correlation_id = CorrelationId::new();

    let mut ctx =
        OperationContext::with_correlation_id("service_call_demo", correlation_id.clone())
            .with_metadata_map(utils::service_call_metadata(
                "songbird_orchestrator",
                "http://localhost:8080/api/v1/orchestrate",
                "POST",
            ));

    ctx.log_start();

    // Simulate multiple attempts (retry pattern)
    for attempt in 1..=3 {
        ctx.metrics.increment_attempts();
        ctx.log_attempt(attempt, 3);

        sleep(Duration::from_millis(50)).await;

        if attempt == 3 {
            // Success on final attempt
            break;
        }
        // Simulate retry
        let delay = Duration::from_millis(1000);
        ctx.log_retry(attempt, delay, "connection_timeout");
        sleep(Duration::from_millis(10)).await; // Shortened for demo
    }

    let result = ctx.complete_success();

    info!(
        "Demo 2 completed: {} attempts, correlation_id: {}, duration: {:?}",
        result.metrics.attempts,
        result.correlation_id,
        result.duration()
    );

    Ok(())
}

/// Demo API operation with rich metadata
async fn demo_api_operation_with_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = OperationContext::new("api_operation_demo")
        .with_metadata_map(utils::api_operation_metadata(
            "create_user",
            "users",
            Some("v2"),
        ))
        .with_metadata("request_size", "1.2KB")
        .with_metadata("client_type", "web_app");

    ctx.log_start();

    // Simulate API phases
    sleep(Duration::from_millis(30)).await;
    ctx.metrics
        .record_phase("validation", Duration::from_millis(30));

    sleep(Duration::from_millis(80)).await;
    ctx.metrics
        .record_phase("database_write", Duration::from_millis(80));

    sleep(Duration::from_millis(20)).await;
    ctx.metrics
        .record_phase("response_serialization", Duration::from_millis(20));

    let result = ctx.complete_success();

    info!(
        "Demo 3 completed: API operation with {} metadata entries and {} phases",
        result.metadata.len(),
        result.metrics.phase_durations.len()
    );

    Ok(())
}

/// Demo failed operation handling
async fn demo_failed_operation() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = OperationContext::new("failing_operation_demo")
        .with_metadata("expected_outcome", "failure")
        .with_metadata("error_type", "demonstration");

    ctx.log_start();

    // Simulate work before failure
    sleep(Duration::from_millis(75)).await;
    ctx.metrics.record_phase("setup", Duration::from_millis(75));

    // Simulate failure
    sleep(Duration::from_millis(25)).await;
    let result = ctx.complete_failure("Simulated failure for demonstration purposes");

    info!(
        "Demo 4 completed: Failed operation tracked with error: {:?}",
        result.error_info()
    );

    Ok(())
}
