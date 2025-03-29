# TUI (ui-terminal) Rebuild Plan

## 1. Goals

*   Create a stable and maintainable terminal UI based on `ratatui`.
*   Correctly integrate with the `McpMetricsProviderTrait` for data display.
*   Resolve existing dependency conflicts and type mismatches.
*   Establish a clear, modular structure for UI components (widgets).
*   Provide essential views: Overview, Network, Protocol, Alerts.

## 2. Core Components & Views

*   **Main Application Structure:** Handles tabs, input, lifecycle.
*   **Tabs:** Overview, System (initially basic), Network, Protocol, Alerts.
*   **Widgets:**
    *   `HealthWidget`: Displays overall system health statuses.
    *   `MetricsWidget`: Shows key system metrics (CPU, Memory).
    *   `NetworkWidget`: Lists network interfaces and basic stats.
    *   `ConnectionHealthWidget`: Shows MCP connection status/health.
    *   `AlertsWidget`: Displays active alerts.
    *   `ChartWidget`: Reusable component for time-series data (CPU, Mem, Network IO).
*   **Help Screen:** Simple overlay showing keybinds.

## 3. Data Flow

*   The main application loop will periodically fetch data using the `McpMetricsProviderTrait` provided by the `core` or `adapter` layer.
    *   Methods likely needed: `get_metrics()`, `get_connection_health()`, `get_connection_status()`, `get_recent_errors()`.
    *   Consider subscribing to connection status events (`start_connection_check()`).
*   Fetched data will be stored in the main application state (`App` struct).
*   Widgets will receive relevant slices of the application state as immutable references during the `draw` cycle.
*   Time-series data for charts will be maintained within the `App` state, updated periodically from fetched metrics.

## 4. Key Interactions

*   Switch tabs (e.g., using number keys 1-5).
*   Toggle help screen (`h`).
*   Quit application (`q`).

## 5. Dependencies

*   `ratatui`: Latest compatible version (currently 0.26.3 from workspace).
*   `dashboard-core`: Accessing data structures (`DashboardData`, `Metrics`, `Alert`, etc.) and the `McpMetricsProviderTrait`.
*   `tokio`: For async operations in the main loop/data fetching.
*   `chrono`: For timestamps.
*   `log`, `thiserror`: Standard utilities.

## 6. Simplification & Phasing

*   **Phase 1 (MVP):**
    *   Basic `App` structure with tab switching and data fetching loop.
    *   Implement `Overview` tab with `HealthWidget` and basic `MetricsWidget`.
    *   Implement `Alerts` tab with `AlertsWidget`.
    *   Basic `ConnectionHealthWidget` (perhaps integrated into footer/header initially).
    *   Focus on correct data display and integration.
*   **Phase 2:**
    *   Implement `Network` tab with `NetworkWidget` and basic RX/TX charts.
    *   Implement `Protocol` tab (details TBD based on `ProtocolData` structure).
    *   Refine charting (`ChartWidget`).
*   **Phase 3:**
    *   Implement `System` tab with more detailed metrics (CPU cores, disk IO, etc.).
    *   Refine layout and styling.
    *   Add Help screen.

## 7. Proposed Directory Structure (`crates/ui-terminal/src`)

*   `lib.rs`: Main library entry point, exports `run_ui`.
*   `app.rs`: Defines the main `App` struct, state management, and update logic.
*   `config.rs`: UI-specific configuration (if any).
*   `event.rs`: Handles input events (keyboard, mouse).
*   `ui.rs`: Contains the main `draw` function and layout logic.
*   `widgets/`: Directory for individual widget implementations.
    *   `mod.rs`: Exports widgets.
    *   `health.rs`
    *   `metrics.rs`
    *   `network.rs`
    *   `alerts.rs`
    *   `chart.rs`
    *   `connection_health.rs`
    *   `base.rs`: (Optional) Define a common `Widget` trait if needed.
*   `util.rs`: Helper functions (e.g., `centered_rect`, formatting).
*   `error.rs`: Defines UI-specific errors.

## 8. Progress Log

*   **[Current Date - Auto-filled if possible, otherwise YYYY-MM-DD]:**
    *   Resolved initial build errors from workspace integration.
    *   Corrected `ratatui::Frame<'_, B>` to `ratatui::Frame<'_>` across relevant files (`ui.rs`, `widgets/*.rs`).
    *   Refactored `app::update` function to use the correct `DashboardService::get_dashboard_data` method from `dashboard-core`.
    *   Inferred `connection_status` and `connection_health` from `DashboardData::protocol` fields.
    *   Fixed various compiler errors (type inference, argument order, unused code) in `ui-terminal` crate.
    *   Verified `ui-terminal` compiles cleanly (`cargo check`).
    *   Verified the entire workspace builds successfully (`cargo build`).

### [YYYY-MM-DD] - Initial Review & Plan
- Reviewed existing `ui-terminal` crate.
- Identified need for better modularity (widgets), error handling, and state management.
- Created initial plan for rebuild focusing on core components: App state, Event handling, Basic UI rendering, Widget system.

### [YYYY-MM-DD] - Type Mismatch Resolution
- Addressed `E0308` mismatched types error in `basic_run.rs` by attempting to align `run_ui` function signature and call site.
- Explored using direct ownership vs. `Arc` for the `DashboardService` provider.
- Iteratively adjusted function signatures and call sites in `app.rs`, `lib.rs`, and `basic_run.rs`.

### [YYYY-MM-DD] - Generics & Trait Bound Refactoring
- Encountered persistent `E0277` trait bound errors (`Arc<T>: Trait` not satisfied) and type inference issues (`E0283`, `E0107`) related to generic propagation (`<P: DashboardService>`).
- Refactored `App`, `run_ui`, `ui::render`, and widget render functions (`metrics`, `alerts`, `connection_health`, `network`) to remove the generic parameter `P`.
- Adopted `Arc<dyn DashboardService + Send + Sync + 'static>` (trait object) for the provider throughout `ui-terminal`.
- Corrected `DefaultDashboardService::default()` usage which already returned an `Arc`, removing redundant `Arc::new()` wrapper.
- Successfully compiled and ran `basic_run` example after refactoring.

## Next Steps

1.  **Address Warnings:** Run `cargo fix` to resolve remaining unused imports and other warnings.
2.  **Implement Remaining Widgets:** Develop render functions for other tabs/widgets (System, Network, Protocol, Alerts).
3.  **Refine Error Handling:** Improve error propagation and display within the UI.
4.  **Enhance State Management:** Add more sophisticated state updates and transitions.
5.  **Testing:** Add unit and integration tests for UI components and logic. 