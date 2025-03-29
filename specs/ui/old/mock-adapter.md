# MockMonitoringAdapter Specification

## Overview

The `MockMonitoringAdapter` provides a test implementation of the `MonitoringAdapter` trait. It generates realistic test data for metrics, health checks, alerts, and protocol status that can be used during development and testing without connecting to a real monitoring system.

## Current Issues

1. **Trait Mismatch**: 
   - The `MockMonitoringAdapter` currently implements methods that don't match the `MonitoringAdapter` trait in `mod.rs`.
   - It has async methods using the `async_trait` macro while the trait doesn't use async.

2. **Data Structure Discrepancies**:
   - Uses `generate_metrics_snapshot()` to create a `MetricsSnapshot` while the trait expects `Metrics`.
   - The structure of `Metrics` in the implementation doesn't match the expected structure.
   - Issues with field types and names for `DiskMetrics`, `CpuMetrics`, etc.

3. **Protocol Type Handling**:
   - Treats `Protocol` and `ProtocolStatus` as structs rather than as enums.

## Required Changes

### 1. Update Trait Implementation

```rust
// Current (incorrect) implementation
#[async_trait]
impl MonitoringAdapter for MockMonitoringAdapter {
    async fn get_system_metrics(&self) -> Result<MetricsSnapshot, String> {
        // ...
    }
    // ...other async methods
}

// Required (correct) implementation
impl MonitoringAdapter for MockMonitoringAdapter {
    fn get_metrics(&self) -> Metrics {
        // ...
    }
    
    fn health_checks(&self) -> Vec<HealthCheck> {
        // ...
    }
    
    fn alerts(&self) -> Vec<Alert> {
        // ...
    }
    
    fn protocol_status(&self) -> Option<ProtocolData> {
        // ...
    }
}
```

### 2. Fix Data Structures

- Update `generate_metrics_snapshot()` to return the correct `Metrics` type.
- Ensure `CpuMetrics`, `MemoryMetrics`, `DiskMetrics`, and `NetworkMetrics` match the dashboard-core definitions.
- Update field names and types to match the expected structures.

### 3. Handle Enums Correctly

- Update the code to treat `Protocol` and `ProtocolStatus` as enums rather than structs.
- Use the appropriate enum variants rather than creating struct instances.

## Implementation Plan

1. Redefine the `MockMonitoringAdapter` struct with the correct field types.
2. Implement the correct methods for the `MonitoringAdapter` trait.
3. Update the data generation methods to create structures matching the dashboard-core definitions.
4. Replace any references to async methods with synchronous implementations.
5. Update usage of `Protocol` and `ProtocolStatus` to use enum variants.

## Testing

The implementation should be tested by:

1. Running `cargo check` to verify compilation.
2. Creating a test that instantiates a `MockMonitoringAdapter` and calls each method.
3. Validating that the returned data matches the expected structures.
4. Ensuring that the `TuiDashboard` can use the mock adapter correctly.

## Integration

Once fixed, the `MockMonitoringAdapter` will be used in:

1. The `TuiDashboard::new_with_monitoring_adapter()` method.
2. Unit tests for the dashboard UI.
3. Development environments where a real monitoring system is not available. 