---
title: Squirrel UI Specifications
version: 2.0.0
date: 2024-03-29
status: active
---

# Squirrel UI Specifications

## Overview

This directory contains specifications and documentation for the user interfaces of the Squirrel system. The primary focus currently is the Terminal UI.

1.  **Terminal UI (`ui-terminal`)**: A responsive, efficient terminal UI using the Ratatui framework.
2.  **Web UI (`ui-web`)**: A browser-based interface (status/plans may need review).
3.  **Desktop UI (`ui-desktop`)**: A future native GUI (status/plans may need review).

## Current Status (Terminal UI - Post-Rollback)

The Terminal UI implementation was recently **rolled back** to a simpler, foundational state.

- ✅ Core Terminal UI features implemented with Ratatui 0.24.0+.
- ✅ Integrates with `dashboard-core` via the `DashboardService` trait.
- ✅ Basic Overview tab with Health, Metrics, and CPU/Memory charts is functional.
- ✅ Dashboard binary compiles and runs correctly.
- ✅ Primary warnings and errors resolved in the core structure.
- 🔄 Development is now focused on incrementally adding core features (Network, Alerts, Protocol, System tabs) and improving test coverage based on the revised plan. **Advanced MCP integration and performance optimizations are NOT currently implemented.**

## Documentation Structure

This directory aims to contain the **active** specifications for the UI. Outdated or historical documents are moved to the `old/` subdirectory.

### Active Specifications

| File                       | Description                                        | Status  |
| :------------------------- | :------------------------------------------------- | :------ |
| `README.md`                | This overview document                             | Updated |
| `IMPLEMENTATION_PROGRESS.md` | Detailed progress tracking for Terminal UI        | Active  |
| `TERMINAL_UI_TASKS.md`     | Task checklist for Terminal UI implementation       | Active  |
| `tui-component-specs.md`   | Specifications for core TUI widgets and state     | Active  |
| `dashboard_integration.md` | How Terminal UI integrates with `DashboardService` | Active  |
| `05-dashboard.md`          | Core dashboard specification (Needs Review)        | Review  |
| `terminal-ui-strategy.md`  | High-level strategy for Terminal UI (Needs Review) | Review  |
| `web-ui-strategy.md`       | Strategy for Web UI (Needs Review)                 | Review  |
| `desktop-ui-strategy.md`   | Strategy for Desktop UI (Needs Review)             | Review  |
| `TERMINAL_UI_SUMMARY.md`   | Summary of Terminal UI (Needs Review)              | Review  |
| `old/`                     | Directory containing archived/outdated specs       | Archive |

*(File list reflects expected state after moving recommended files)*

### Archived / Outdated Specifications (`old/`)

The `old/` directory contains specifications that are no longer relevant due to implementation changes, completion, or strategic shifts (e.g., the recent rollback). This includes previous testing plans, roadmaps, status reports, integration plans related to the more complex pre-rollback implementation, and completed migration guides.

## Key Architecture Concepts (Current ui-terminal)

The `ui-terminal` uses Ratatui and follows a basic structure:

1.  **App State (`app.rs`)**: Holds UI state, fetched data (`DashboardData`), and update logic.
2.  **UI Rendering (`ui.rs`)**: Handles main layout, tab rendering, and delegates to widgets.
3.  **Widgets (`widgets/*.rs`)**: Reusable components responsible for rendering specific data slices (Health, Metrics, Charts, etc.).
4.  **Event Handling (`event.rs`, `lib.rs`)**: Manages input and tick events.
5.  **Main Loop (`lib.rs`)**: Orchestrates rendering, event handling, and periodic data updates via `DashboardService`.

## Next Steps (Terminal UI)

Development will follow the revised plan outlined in `IMPLEMENTATION_PROGRESS.md` and `TERMINAL_UI_TASKS.md`, focusing on:

1.  **Implement Missing Widgets/Tabs**: Network, Alerts, Protocol, System.
2.  **Improve Testing**: Add unit tests for existing and new widgets, implement basic E2E tests.
3.  **Refine Core Features**: Improve connection status logic, enhance charts.
4.  **Code Quality**: Address warnings, add documentation.

*(Advanced MCP integration and performance optimization are lower priority / future considerations).*

## Getting Started

For developers working on the `ui-terminal`:

1.  Review `IMPLEMENTATION_PROGRESS.md` for current status and roadmap.
2.  Check `TERMINAL_UI_TASKS.md` for pending tasks.
3.  Refer to `tui-component-specs.md` for widget/state details.
4.  Refer to `dashboard_integration.md` for data flow details.

## References

- [Ratatui Documentation](https://docs.rs/ratatui/latest/ratatui/)
- [Crossterm Documentation](https://docs.rs/crossterm/latest/crossterm/)
- [Squirrel Core Specifications](../README.md) (Verify Link/Relevance)
- [Dashboard Core Documentation](../../crates/dashboard-core/README.md) (Verify Link)
- [Terminal UI Source Code](../../crates/ui-terminal) (Verify Link) 