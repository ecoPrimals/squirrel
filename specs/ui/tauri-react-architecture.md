---
title: Squirrel Tauri + React Architecture
version: 1.0.0
date: 2024-04-09
status: active
---

# Squirrel Tauri + React Architecture

## Overview

This document outlines the architecture for implementing Squirrel's unified UI system using Tauri and React. This approach consolidates the previously separate web and desktop UI strategies into a cohesive system that serves both platforms effectively while building on the foundation established in the Terminal UI.

## Core Architecture Principles

1. **Unified Codebase**: One React codebase that serves both web and desktop interfaces
2. **Native Integration**: Leverage Tauri for OS-level integration and performance
3. **Component Reusability**: Build component library mirroring Terminal UI widgets
4. **Data Consistency**: Use the same data models and interfaces across all UIs
5. **Dashboard Core Integration**: Maintain compatibility with DashboardService
6. **Progressive Enhancement**: Build with browser compatibility while enhancing desktop experience

## Technology Stack

### Core Technologies
- **Tauri**: Rust-based framework for building lightweight, secure desktop applications
- **React**: Frontend library for building user interfaces
- **TypeScript**: Type-safe JavaScript
- **Vite**: Modern frontend build tool
- **TailwindCSS**: Utility-first CSS framework

### Rust Backend
- **Tauri API**: Native capabilities (file system, notifications, system tray)
- **DashboardService**: Integration with existing Squirrel dashboard core
- **WebView**: Rendering the React UI in Tauri's WebView
- **IPC**: Inter-process communication between Rust and JavaScript

### React Frontend
- **React Router**: Navigation and routing
- **React Query**: Data fetching and caching
- **Zustand**: State management
- **React Hook Form**: Form handling
- **Vitest**: Testing framework
- **Storybook**: Component documentation and testing

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    React Frontend                       │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Shared    │  │  Platform   │  │    View     │     │
│  │ Components  │  │  Adapters   │  │  Templates  │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │    State    │  │     API     │  │     UI      │     │
│  │  Management │  │    Client   │  │  Utilities  │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└───────────────────────│─│─────────────────────────────┘
                        │ │
┌───────────────────────│─│─────────────────────────────┐
│                       │ │                              │
│  ┌─────────────────┐  │ │  ┌─────────────────────┐    │
│  │   Tauri Core    │◄─┘ └─►│  DashboardService   │    │
│  └─────────────────┘       └─────────────────────┘    │
│                                                        │
│  ┌─────────────────┐       ┌─────────────────────┐    │
│  │  Native APIs    │       │  Squirrel Core      │    │
│  └─────────────────┘       └─────────────────────┘    │
│                                                        │
│                  Rust Backend                          │
└────────────────────────────────────────────────────────┘
```

## Component Architecture

The React component architecture mirrors the Terminal UI structure while adapting for web and desktop interfaces:

1. **Layout Components**:
   - `AppShell`: Main application layout
   - `TabNavigation`: Primary navigation
   - `SidePanel`: Contextual information and controls
   - `StatusBar`: System status and metadata

2. **Dashboard Widgets** (mapping to Terminal UI):
   - `HealthWidget`: System health indicators
   - `MetricsWidget`: System metrics display
   - `ChartWidget`: Time-series data visualization
   - `NetworkWidget`: Network status and performance
   - `AlertsWidget`: System alerts and notifications
   - `ProtocolWidget`: Protocol status and details
   - `SystemWidget`: System information and process list

3. **Platform-Specific Components**:
   - `Notifications`: Desktop notifications wrapper
   - `SystemTray`: System tray integration
   - `NativeDialog`: OS-native dialog wrapper
   - `DragDrop`: File drag-and-drop handler

## Data Flow Architecture

```
┌─────────────────────┐         ┌───────────────────┐
│                     │         │                   │
│   React Components  │         │  API Client       │
│                     │         │                   │
└─────────┬───────────┘         └────────┬──────────┘
          │                              │
          │ Props                        │ Requests
          ▼                              ▼
┌─────────────────────┐         ┌───────────────────┐
│                     │         │                   │
│   State Management  │◄────────┤  Tauri Commands   │
│                     │         │                   │
└─────────────────────┘         └────────┬──────────┘
                                         │
                                         │ IPC
                                         ▼
                               ┌───────────────────┐
                               │                   │
                               │  Rust Backend     │
                               │                   │
                               └────────┬──────────┘
                                        │
                                        │
                                        ▼
                               ┌───────────────────┐
                               │                   │
                               │ DashboardService  │
                               │                   │
                               └───────────────────┘
```

## Integration with DashboardService

The Tauri + React implementation integrates with the same DashboardService as the Terminal UI through these layers:

1. **Rust Backend (Tauri Commands)**:
   - Exposes Tauri commands that wrap the DashboardService
   - Handles periodic data fetching in the background
   - Manages subscription to real-time updates

2. **JS/TS Bridge**:
   - API client in TypeScript for calling Tauri commands
   - Type definitions matching Rust structures

3. **React Hooks**:
   - Custom hooks to fetch and manage dashboard data
   - useQuery/useMutation patterns for data operations
   - State management with Zustand

## Project Structure

```
squirrel-ui/
├── src-tauri/                  # Rust backend code
│   ├── src/
│   │   ├── main.rs             # Tauri application entry
│   │   ├── commands/           # Tauri command definitions
│   │   ├── dashboard/          # DashboardService integration
│   │   ├── models/             # Data models
│   │   └── utils/              # Utility functions
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
├── src/                        # React frontend code
│   ├── main.tsx                # Application entry
│   ├── App.tsx                 # Root component
│   ├── components/             # UI components
│   │   ├── layout/             # Layout components
│   │   ├── widgets/            # Dashboard widgets
│   │   └── common/             # Common UI elements
│   ├── hooks/                  # Custom React hooks
│   ├── api/                    # API client for Tauri commands
│   ├── stores/                 # State management
│   ├── types/                  # TypeScript type definitions
│   └── utils/                  # Utility functions
├── public/                     # Static assets
└── package.json                # JS dependencies
```

## Implementation Phases

### Phase 1: Foundation
- Setup Tauri + React + TypeScript project
- Implement basic layout and navigation
- Create core widgets (Health, Metrics, Charts)
- Establish DashboardService integration

### Phase 2: Feature Parity
- Implement remaining widgets (Network, Alerts, Protocol, System)
- Add data visualization components
- Complete state management
- Add basic customization options

### Phase 3: Desktop Enhancements
- Add system tray integration
- Implement native notifications
- Add file system integration
- Create cross-platform installers

### Phase 4: Web Optimization
- Optimize for web deployment
- Implement responsive design adaptations
- Add progressive web app capabilities
- Optimize loading performance

### Phase 5: Advanced Features
- Add theming and customization
- Implement advanced data visualization
- Add keyboard shortcuts and accessibility
- Create user preferences system

## Cross-Platform Considerations

### Desktop-Specific Features
- Native file dialogs
- System tray integration
- Global keyboard shortcuts
- Background processing
- Auto-updates

### Web-Specific Features
- Offline support (PWA)
- Responsive layouts for mobile
- Deep linking
- SEO considerations
- Web-specific authentication

## Accessibility Standards

The Tauri + React UI will adhere to these accessibility standards:

- WCAG 2.1 AA compliance
- Keyboard navigation support
- Screen reader compatibility
- High contrast mode support
- Reduced motion support
- Focus management

## Performance Targets

- Initial load time: < 2 seconds
- Time to interactive: < 3 seconds
- Memory usage: < 200MB
- CPU usage: < 5% at idle
- Animation frame rate: 60fps

## Security Considerations

- CSP for web deployment
- Minimized Tauri privileges
- Input validation
- Data sanitization
- Secure IPC communication
- Regular dependency audits

## Testing Strategy

- Unit tests for React components
- Integration tests for component combinations
- E2E tests for critical user flows
- Tauri command testing
- Cross-platform testing (Windows, macOS, Linux)
- Performance testing and monitoring

## References

- [Squirrel Terminal UI Specifications](./README.md)
- [Dashboard Integration](./dashboard_integration.md)
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [React Documentation](https://reactjs.org/docs/getting-started.html) 