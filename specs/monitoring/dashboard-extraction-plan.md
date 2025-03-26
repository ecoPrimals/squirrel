# Dashboard Extraction Plan

**Version**: 1.1.0  
**Last Updated**: 2024-06-22  
**Status**: Proposed  
**Priority**: High

## Overview

This document outlines the plan for extracting the dashboard functionality from the monitoring crate into a dedicated dashboard architecture. This will allow for multiple UI implementations (Terminal, Web, Desktop) while maintaining a consistent core.

## Current State

Currently, the dashboard functionality is tightly coupled with the monitoring system. This makes it difficult to:

1. Support multiple UI implementations (Terminal, Web, Desktop)
2. Develop UI features independently from monitoring features
3. Test UI components in isolation
4. Maintain clear boundaries between monitoring and visualization

## Target State

After the extraction, we will have:

1. A core dashboard module providing UI-agnostic functionality
2. Multiple UI implementations sharing the core functionality:
   - Terminal UI for command-line usage
   - Web UI for browser access
   - Desktop UI for native applications (future)
3. A clear interface between monitoring data collection and visualization
4. Improved testability of both dashboard and monitoring components

## Implementation Plan

### Phase 1: Core Extraction

1. Create a `squirrel-dashboard-core` crate:
   - Implement data models for metrics, alerts, health checks
   - Define service interfaces for collecting data
   - Create configuration system for dashboard settings
   - Implement error handling and logging
   - Add real-time update mechanism

2. Define clean interfaces between monitoring and dashboard:
   - Monitoring service interface for data collection
   - Metrics database interface for historical data
   - Update channel for real-time notifications

3. Implement the core service:
   - Default dashboard service implementation
   - In-memory metrics database implementation
   - Mock implementations for testing

### Phase 2: UI-Specific Implementations

1. Implement Terminal UI in `crates/ui-terminal`:
   - Use ratatui for terminal UI rendering
   - Implement dashboard widgets for metrics, alerts, health, etc.
   - Create tab-based navigation
   - Add keyboard shortcuts
   - Provide command-line options

2. Enhance Web UI in `crates/ui-web`:
   - Integrate dashboard core with existing web UI
   - Implement real-time updates using WebSockets
   - Create responsive dashboard components
   - Add user authentication and permissions
   - Support customizable layouts

3. Implement Desktop UI in `crates/ui-desktop`:
   - Use a native GUI toolkit (like Iced)
   - Implement desktop-specific features
   - Support system tray integration
   - Add notifications and alerts
   - Provide customizable layouts

### Phase 3: Integration and Testing

1. Integrate with existing monitoring system:
   - Implement monitoring service adapter
   - Connect metrics database to monitoring data
   - Ensure backward compatibility

2. Comprehensive testing:
   - Unit tests for each component
   - Integration tests for the full system
   - Performance testing for UI responsiveness
   - Cross-platform testing

3. Documentation and examples:
   - API documentation for dashboard core
   - User guides for each UI implementation
   - Examples of extending the dashboard

## Dependencies

- ratatui for terminal UI
- Web framework for browser UI
- GUI toolkit for desktop UI
- Existing monitoring system

## Timeline

- Phase 1 (Core Extraction): 2 weeks
- Phase 2 (UI Implementations): 3 weeks
- Phase 3 (Integration and Testing): 2 weeks
- Total: 7 weeks

## Success Criteria

- Dashboard functionality works independently from monitoring
- Multiple UI implementations share the same core functionality
- Performance is equal to or better than the current implementation
- All existing dashboard features are preserved
- New UI implementations can be added with minimal effort

## Risks and Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Breaking changes to monitoring API | High | Medium | Design backward-compatible interfaces |
| Performance degradation | Medium | Low | Performance testing throughout development |
| Feature parity gaps | Medium | Medium | Comprehensive testing of all features |
| Integration challenges | High | Medium | Incremental approach with frequent integration testing |

## Next Steps

1. Create the `squirrel-dashboard-core` crate
2. Define interfaces between monitoring and dashboard
3. Implement core data models and services
4. Set up the Terminal UI implementation in `crates/ui-terminal`
5. Begin integration testing with existing monitoring system

## Appendix

### Crate Structure

```
crates/
├── squirrel-dashboard-core/    (Core dashboard functionality)
├── ui-terminal/               (Terminal UI implementation)
├── ui-web/                    (Web UI implementation)
└── ui-desktop/                (Future desktop UI implementation)
```

### Key Interfaces

```rust
/// Dashboard service interface
pub trait DashboardService {
    /// Get current dashboard data
    async fn get_dashboard_data(&self) -> Result<DashboardData>;
    
    /// Get historical metrics
    async fn get_metric_history(&self, metric: &str, range: TimeRange) -> Result<MetricHistory>;
    
    /// Subscribe to real-time updates
    async fn subscribe(&self) -> Result<Receiver<DashboardUpdate>>;
}

/// Monitoring service interface for dashboard
pub trait MonitoringService {
    /// Get system metrics
    async fn get_system_metrics(&self) -> Result<SystemMetrics>;
    
    /// Get alerts
    async fn get_alerts(&self) -> Result<Vec<Alert>>;
    
    /// Get health checks
    async fn get_health_checks(&self) -> Result<Vec<HealthCheck>>;
}
``` 