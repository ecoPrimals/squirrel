---
title: Squirrel UI Specifications
version: 3.0.0
date: 2024-04-09
status: active
---

# Squirrel UI Specifications

## Overview

This directory contains specifications and documentation for the user interfaces of the Squirrel system. The system now features multiple UI implementations:

1. **Terminal UI (`ui-terminal`)**: A responsive, efficient terminal UI using the Ratatui framework.
2. **Web + Desktop UI**: A unified Tauri + React implementation serving both web browser access and desktop application needs.

## Current Status

### Terminal UI (`ui-terminal`)
The Terminal UI implementation has been simplified to a foundational state.
- ✅ Core Terminal UI features implemented with Ratatui 0.24.0+.
- ✅ Integrates with `dashboard-core` via the `DashboardService` trait.
- ✅ Basic Overview tab with Health, Metrics, and CPU/Memory charts is functional.
- ✅ Dashboard binary compiles and runs correctly.
- ✅ Primary warnings and errors resolved in the core structure.
- 🔄 Development continues to incrementally add core features and improve test coverage.

### Tauri + React UI (Web & Desktop)
The Tauri + React UI implementation is in planning phase.
- 🔄 Architecture and specifications defined
- 🔄 Integration with `DashboardService` planned
- 🔄 Component specifications defined
- 🔄 Implementation roadmap created
- ⏹️ Development not yet started

## Documentation Structure

This directory contains the active specifications for the UI. Outdated or historical documents are in the `old/` subdirectory.

### Active Specifications

| File                         | Description                                        | Status  |
| :--------------------------- | :------------------------------------------------- | :------ |
| `README.md`                  | This overview document                             | Updated |
| `tauri-react-architecture.md`| Architecture for unified Tauri + React UI          | New     |
| `react-component-specs.md`   | React component specifications                     | New     |
| `IMPLEMENTATION_PROGRESS_TAURI_REACT.md` | Implementation plan for Tauri + React UI | New     |
| `web/web-ui-strategy.md`     | Updated strategy for Web UI using React            | Updated |
| `desktop/desktop-ui-strategy.md` | Updated strategy for Desktop UI using Tauri     | Updated |
| `IMPLEMENTATION_PROGRESS.md` | Progress tracking for Terminal UI                  | Active  |
| `TERMINAL_UI_TASKS.md`       | Task checklist for Terminal UI implementation      | Active  |
| `tui-component-specs.md`     | Specifications for core TUI widgets and state      | Active  |
| `dashboard_integration.md`   | How UI integrates with `DashboardService`          | Active  |
| `05-dashboard.md`            | Core dashboard specification                       | Active  |
| `terminal-ui-strategy.md`    | High-level strategy for Terminal UI                | Active  |
| `TERMINAL_UI_SUMMARY.md`     | Summary of Terminal UI                             | Active  |
| `old/`                       | Directory containing archived/outdated specs       | Archive |

### Terminal UI Architecture

The `ui-terminal` uses Ratatui and follows this structure:

1. **App State (`app.rs`)**: Holds UI state, fetched data (`DashboardData`), and update logic.
2. **UI Rendering (`ui.rs`)**: Handles main layout, tab rendering, and delegates to widgets.
3. **Widgets (`widgets/*.rs`)**: Reusable components for rendering specific data.
4. **Event Handling (`event.rs`, `lib.rs`)**: Manages input and tick events.
5. **Main Loop (`lib.rs`)**: Orchestrates rendering, events, and data updates.

### Tauri + React Architecture

The unified Tauri + React implementation follows this structure:

1. **Tauri Backend**: Rust code that integrates with `DashboardService` and provides native OS capabilities.
2. **React Frontend**: TypeScript/React components that display dashboard data and handle user interaction.
3. **Shared Component Library**: React components that implement the same functionality as Terminal UI widgets.
4. **State Management**: Zustand stores that manage application state similar to Terminal UI's AppState.
5. **Platform Adaptations**: Feature detection and enhancements for web vs. desktop environments.

## Next Steps

### Terminal UI
- Continue implementation of missing widgets and tabs
- Improve testing coverage
- Refine core features and error handling

### Tauri + React UI
- Begin implementation following the phases defined in `IMPLEMENTATION_PROGRESS_TAURI_REACT.md`
- Start with foundational project setup and core components
- Build dashboard integration with DashboardService
- Implement shared component library

## Getting Started

For developers working on the UI:

1. **Terminal UI**:
   - Review `IMPLEMENTATION_PROGRESS.md` for current status
   - Check `TERMINAL_UI_TASKS.md` for pending tasks
   - Refer to `tui-component-specs.md` for widget details

2. **Tauri + React UI**:
   - Review `tauri-react-architecture.md` for overall architecture
   - Check `IMPLEMENTATION_PROGRESS_TAURI_REACT.md` for implementation plan
   - Refer to `react-component-specs.md` for component details
   - Review integration patterns in `dashboard_integration.md`

## References

- [Ratatui Documentation](https://docs.rs/ratatui/latest/ratatui/)
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [React Documentation](https://reactjs.org/docs/getting-started.html)
- [Dashboard Core Documentation](../../crates/dashboard-core/README.md)
- [Terminal UI Source Code](../../crates/ui-terminal) 