# Unified UI Integration

**Version**: 1.0.0
**Date**: 2024-07-20
**Status**: Draft

## Overview

This document outlines the integration strategy for the unified UI approach in Squirrel, combining the Tauri + React implementation with the existing codebase. The integration focuses on maintaining compatibility while leveraging the strengths of both the web-based React UI and the native capabilities provided by Tauri.

## Integration Principles

1. **Unified Codebase**: Maintain a single source of truth for UI components and logic
2. **Backend API Consistency**: Ensure consistent interaction patterns between UI and core services
3. **Progressive Enhancement**: Add platform-specific features without breaking cross-platform compatibility
4. **Shared Data Models**: Use consistent data structures across all UI implementations
5. **Decoupled Architecture**: Keep UI implementations loosely coupled from core business logic

## Architecture Integration

### Component Structure

The unified UI integrates with the core Squirrel system as follows:

```
squirrel/
├── crates/
│   ├── dashboard-core/       # Core dashboard functionality (shared)
│   ├── interfaces/           # Shared data models and traits
│   ├── ui-tauri-react/       # Tauri + React implementation
│   │   ├── src/              # React frontend code
│   │   └── src-tauri/        # Tauri backend code
│   └── ui-terminal/          # Terminal UI implementation
└── specs/
    └── ui/                   # UI specifications
```

### Technology Stack Integration

| Layer | Technology | Integration Point |
|-------|------------|-------------------|
| Frontend | React, TypeScript, TailwindCSS | Consumes Tauri API, renders UI |
| Backend Bridge | Tauri | Connects React UI with Rust services |
| Business Logic | Rust (dashboard-core) | Provides core functionality |
| Data Access | Rust (interfaces) | Defines data models and APIs |

## Service Integration

### DashboardService Integration

The DashboardService is the primary integration point between the UI and the core business logic:

1. **Initialization**: The Tauri backend initializes the DashboardService on application startup
2. **Command Exposure**: Core functionality is exposed via Tauri commands:
   - `get_dashboard_data`: Fetches current dashboard state
   - `get_metric_history`: Retrieves historical metrics
   - `acknowledge_alert`: Handles alert acknowledgment
   - `update_config`: Updates dashboard configuration
   - `trigger_data_refresh`: Triggers data refresh
3. **Real-time Updates**: Dashboard updates are streamed to the UI via Tauri events

### Event System

The event system provides real-time communication between backend services and the UI:

1. **Event Emission**: Backend services emit events when data changes
2. **Event Listening**: UI components subscribe to relevant events
3. **Event Types**:
   - `dashboard-update`: Dashboard data changes
   - `alert-new`: New alerts
   - `config-change`: Configuration updates

## Data Flow

1. User interacts with the React UI
2. UI dispatches Tauri commands to the backend
3. Backend processes commands through the DashboardService
4. DashboardService returns data or emits events
5. UI updates based on returned data or received events

## Cross-Platform Considerations

### Desktop-Specific Integration

1. **System Tray**: Integration with OS tray for background operation
2. **Native Notifications**: OS-level notifications for alerts
3. **File System Access**: Direct access to local file system
4. **Keyboard Shortcuts**: Global shortcuts for quick actions

### Web-Specific Integration

1. **Progressive Web App**: Installable web application
2. **Service Workers**: Offline capability and caching
3. **Web Notifications**: Browser notification API integration
4. **Responsive Design**: Adaptive layouts for various devices

## Implementation Status

The current implementation status of the unified UI integration:

1. **Core Integration**: Complete - DashboardService is fully integrated with Tauri backend
2. **Command API**: Complete - All necessary commands are implemented
3. **Event System**: Complete - Real-time updates working via Tauri events
4. **Desktop Features**: Partial - Basic features implemented, advanced features pending
5. **Web Optimization**: Pending - Optimization for web deployment in progress

## Future Enhancements

1. **Shared Component Library**: Extract common UI components into a shared library
2. **Plugin Integration**: Support for UI plugins across platforms
3. **Sync Services**: User preference synchronization between instances
4. **Performance Optimization**: Further optimizations for both web and desktop

## References

- [Tauri + React Architecture](./tauri-react-architecture.md)
- [Implementation Progress](./IMPLEMENTATION_PROGRESS_TAURI_REACT.md)
- [Web UI Strategy](./web/web-ui-strategy.md)
- [Desktop UI Strategy](./desktop/desktop-ui-strategy.md)

---

Last Updated: 2024-07-20 