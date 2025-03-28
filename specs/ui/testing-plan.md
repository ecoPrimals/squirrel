---
title: Terminal UI Testing Plan
author: DataScienceBioLab
version: 1.0.0
date: 2024-08-30
status: active
---

# Terminal UI Testing Plan

## Overview

This document outlines a comprehensive testing strategy for the Terminal UI components, with a focus on the Protocol Widget, McpMetricsProvider integration, and the connection health/history tracking features implemented in Phase 2 of the MCP Integration.

## Test Categories

### 1. Unit Tests

#### Protocol Widget Tests
- **Tab Navigation**
  - Test tab switching functionality
  - Verify correct rendering of each tab
  - Test keyboard shortcuts for tab navigation

- **Connection Health Display**
  - Test rendering with various connection health states
  - Verify color-coding based on connection status
  - Test display of connection metrics (latency, failure count)

- **Connection History Display**
  - Test rendering of connection event history
  - Verify timestamp formatting
  - Test empty history handling

- **Metrics Chart Rendering**
  - Test chart generation with various datasets
  - Verify axis scaling and labeling
  - Test empty metrics handling

#### McpMetricsProvider Tests
- **Connection Health Methods**
  - Test `connection_health()` functionality
  - Verify correct error handling
  - Test state transitions

- **Connection History Methods**
  - Test `connection_history()` functionality
  - Verify event recording
  - Test history limit enforcement

- **Reconnection Logic**
  - Test `reconnect()` functionality
  - Verify success/failure handling
  - Test retry logic

### 2. Integration Tests

#### Dashboard Integration
- **Widget Integration**
  - Test widget integration with dashboard layout
  - Verify widget resizing behavior
  - Test focus handling

- **Data Flow**
  - Test data flow from provider to widget
  - Verify real-time updates
  - Test subscription mechanism

#### MCP Service Integration
- **Real Service Connection**
  - Test connection to actual MCP service
  - Verify metrics collection
  - Test reconnection with real service

- **Error Handling**
  - Test behavior during service outages
  - Verify error recovery
  - Test degraded mode operation

### 3. Performance Tests

- **Rendering Performance**
  - Measure rendering time with large datasets
  - Test scrolling performance
  - Identify rendering bottlenecks

- **Memory Usage**
  - Monitor memory usage during extended operation
  - Test history storage efficiency
  - Identify memory leaks

- **CPU Usage**
  - Monitor CPU usage during updates
  - Test impact of polling frequency
  - Optimize high-CPU operations

## Mock Implementation

For testing purposes, we've implemented a `MockMcpMetricsProvider` with the following capabilities:

```rust
impl MockMcpMetricsProvider {
    pub fn new(config: McpMetricsConfig) -> Self {
        Self {
            config,
            should_fail: false,
            connection_health: ConnectionHealth {
                status: ConnectionStatus::Connected,
                last_successful: Some(Utc::now()),
                failure_count: 0,
                latency_ms: Some(5),
                error_details: None,
            },
            connection_history: vec![
                ConnectionEvent {
                    timestamp: Utc::now() - chrono::Duration::minutes(5),
                    event_type: ConnectionEventType::Connected,
                    details: None,
                }
            ],
            last_reconnect: None,
        }
    }

    // Toggle failure mode for testing error handling
    pub fn set_should_fail(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
        if should_fail {
            self.connection_health.status = ConnectionStatus::Disconnected;
            self.connection_health.failure_count += 1;
            self.connection_health.error_details = Some("Simulated failure".to_string());
            
            // Record disconnection event
            self.connection_history.push(ConnectionEvent {
                timestamp: Utc::now(),
                event_type: ConnectionEventType::Disconnected,
                details: Some("Simulated failure".to_string()),
            });
        }
    }

    // Simulate reconnection attempts
    pub fn simulate_reconnect(&mut self, success: bool) {
        self.last_reconnect = Some(Utc::now());
        
        if success {
            self.connection_health.status = ConnectionStatus::Connected;
            self.connection_health.last_successful = Some(Utc::now());
            self.connection_health.error_details = None;
            
            // Record reconnection success event
            self.connection_history.push(ConnectionEvent {
                timestamp: Utc::now(),
                event_type: ConnectionEventType::ReconnectSuccess,
                details: None,
            });
        } else {
            self.connection_health.failure_count += 1;
            
            // Record reconnection failure event
            self.connection_history.push(ConnectionEvent {
                timestamp: Utc::now(),
                event_type: ConnectionEventType::ReconnectFailure,
                details: Some(format!("Failed attempt #{}", self.connection_health.failure_count)),
            });
        }
    }
}
```

## Test Data Generation

For consistent testing, we use the following test data generators:

```rust
// Generate random connection history
fn generate_connection_history(count: usize) -> Vec<ConnectionEvent> {
    let mut history = Vec::with_capacity(count);
    let mut rng = rand::thread_rng();
    let now = Utc::now();
    
    for i in 0..count {
        let minutes_ago = count - i;
        let event_type = match i % 5 {
            0 => ConnectionEventType::Connected,
            1 => ConnectionEventType::Disconnected,
            2 => ConnectionEventType::Reconnecting,
            3 => ConnectionEventType::ReconnectSuccess,
            _ => ConnectionEventType::ReconnectFailure,
        };
        
        history.push(ConnectionEvent {
            timestamp: now - chrono::Duration::minutes(minutes_ago as i64),
            event_type,
            details: if i % 2 == 0 { 
                Some(format!("Test event {}", i)) 
            } else { 
                None 
            },
        });
    }
    
    history
}

// Generate random metrics history
fn generate_metrics_history(metrics: &[&str], points: usize) -> HashMap<String, Vec<(DateTime<Utc>, f64)>> {
    let mut history = HashMap::new();
    let mut rng = rand::thread_rng();
    let now = Utc::now();
    
    for &metric in metrics {
        let mut series = Vec::with_capacity(points);
        let base_value = rng.gen_range(10.0..100.0);
        
        for i in 0..points {
            let minutes_ago = points - i;
            let variation = rng.gen_range(-5.0..5.0);
            let value = base_value + variation;
            
            series.push((
                now - chrono::Duration::minutes(minutes_ago as i64),
                value
            ));
        }
        
        history.insert(metric.to_string(), series);
    }
    
    history
}
```

## Test Implementation Plan

### Phase 1: Unit Tests (In Progress)
- Implement unit tests for `ProtocolWidget` tab navigation
- Implement unit tests for `McpMetricsProvider` connection health
- Implement unit tests for metrics rendering
- Implement unit tests for `CompressedTimeSeries` functionality:
  - Base point tracking
  - Multiple resampling strategies 
  - Statistics calculation
  - Time range analysis
  - Empty series handling

### Phase 2: Integration Tests (Planned)
- Create test harness for dashboard integration
- Implement integration tests for data flow
- Test with simulated MCP service

### Phase 3: Performance Tests (Planned)
- Develop performance benchmarks
- Implement memory usage tests
- Create CPU usage profiling

## Continuous Integration

All tests will be integrated into the CI pipeline with the following requirements:
- Unit tests must pass for all PRs
- Integration tests must pass for release candidates
- Performance tests must not show regression for releases

## Test Coverage Goals

- **Unit Tests**: >90% coverage for UI components
- **Integration Tests**: >85% coverage for end-to-end flows
- **Performance Tests**: Cover all critical rendering paths

## CompressedTimeSeries Tests

### Basic Functionality Tests
- **Base Point Tracking**
  - Test correct base point setting on first point add
  - Verify subsequent points are stored as deltas
  - Test clear() method resets base point tracking
  - Test proper handling of has_base_point field

- **Point Addition and Retrieval**
  - Test adding multiple points
  - Verify correct order of retrieved points
  - Test point retrieval after clearing

- **Capacity Management**
  - Test behavior when reaching max capacity
  - Verify oldest points are removed first
  - Test with different capacity settings

### Resampling Strategy Tests
- **EvenlySpaced Strategy**
  - Test downsampling with various target sizes
  - Verify points are distributed evenly
  - Test edge cases (target size = 1, target size > points)

- **LargestValues Strategy**
  - Test retrieval of largest values
  - Verify correct number of points returned
  - Test with negative values and edge cases

- **SignificantChanges Strategy**
  - Test extraction of points with significant changes
  - Verify threshold parameter works correctly
  - Test with various threshold values

### Statistics Tests
- **Basic Statistics**
  - Test min/max/avg calculation
  - Verify correct point count
  - Test with empty series

- **Time Range Statistics**
  - Test statistics within specified time ranges
  - Verify proper handling of ranges outside data
  - Test partial overlapping ranges

### Time Range Analysis Tests
- **Range Calculation**
  - Test time_range() method
  - Verify correct min/max timestamps
  - Test with empty series

### Edge Case Tests
- **Empty Series**
  - Test behavior with no points
  - Verify all operations handle empty series gracefully
  - Test adding points after clearing

- **Negative Values**
  - Test with negative value deltas
  - Verify correct reconstruction of original values
  - Test statistics with negative values

- **Large Datasets**
  - Test performance with large number of points
  - Verify memory usage scales appropriately
  - Test downsampling large datasets

## Test Data Generation for CompressedTimeSeries

```rust
// Generate time series test data
fn generate_time_series_data<T>(
    count: usize,
    start_time: DateTime<Utc>,
    value_gen: impl Fn(usize) -> T
) -> Vec<(DateTime<Utc>, T)>
where
    T: Copy + std::ops::Sub<Output = T> + std::ops::Add<Output = T> + Default
{
    let mut data = Vec::with_capacity(count);
    
    for i in 0..count {
        let timestamp = start_time + chrono::Duration::seconds(i as i64 * 60);
        let value = value_gen(i);
        data.push((timestamp, value));
    }
    
    data
}

// Test different resampling strategies
fn test_resampling_strategies<T>(
    data: &[(DateTime<Utc>, T)],
    target_size: usize
) -> HashMap<&'static str, Vec<(DateTime<Utc>, T)>>
where
    T: Copy + std::ops::Sub<Output = T> + std::ops::Add<Output = T> + Default + PartialOrd
{
    let mut series = CompressedTimeSeries::new(data.len());
    
    // Add all data points
    for &(timestamp, value) in data {
        series.add(timestamp, value);
    }
    
    // Test different strategies
    let mut results = HashMap::new();
    
    // Evenly spaced strategy (default)
    results.insert("evenly_spaced", series.downsample(target_size));
    
    // Largest values strategy
    let largest_strategy = ResampleStrategy::LargestValues;
    results.insert("largest_values", series.downsample_with_strategy(target_size, largest_strategy));
    
    // Significant changes strategy with threshold
    let significant_threshold = Default::default(); // Use appropriate threshold for type T
    let significant_strategy = ResampleStrategy::SignificantChanges(significant_threshold);
    results.insert("significant_changes", 
                  series.downsample_with_strategy(target_size, significant_strategy));
    
    results
}
```

## Conclusion

This testing plan provides a comprehensive approach to verifying the functionality, reliability, and performance of the Terminal UI components, with a special focus on the MCP Integration Phase 2 features. By implementing these tests, we can ensure a high-quality user experience and stable operation of the Terminal UI. 