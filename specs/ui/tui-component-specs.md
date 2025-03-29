# TUI (ui-terminal) Component Specifications

This document provides detailed specifications for the core components of the `ui-terminal` rebuild, complementing the `tui-rebuild-plan.md`.

## 1. Main Application State (`app.rs::App`)

The `App` struct will hold the overall state of the TUI.

```rust
// Tentative structure in app.rs
use dashboard_core::adapter::McpMetricsProviderTrait;
use dashboard_core::data::{DashboardData, McpMetrics, Alert}; // Add Alert
use crate::adapter::{ConnectionHealth, ConnectionStatus}; // Assuming these are in the *new* crate::adapter or similar
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque}; // Use VecDeque for history
use std::sync::Arc;
use tokio::sync::Mutex; // If provider is mutex-guarded

pub enum ActiveTab {
    Overview,
    System,
    Network,
    Protocol,
    Alerts,
}

pub struct AppState {
    // Data fetched from provider
    pub metrics: Option<McpMetrics>,
    pub connection_health: Option<ConnectionHealth>,
    pub connection_status: ConnectionStatus, // Assume a default like Disconnected
    pub alerts: Vec<Alert>,
    pub protocol_metrics: HashMap<String, f64>, // Or appropriate type
    pub recent_errors: Vec<String>, // Errors from the provider

    // Internal UI State
    pub active_tab: ActiveTab,
    pub show_help: bool,
    pub should_quit: bool,
    pub last_update: Option<DateTime<Utc>>,

    // Time series data for charts (Map Metric Key -> History)
    // Example key: "cpu_usage", "mem_usage", "net_rx_eth0", "net_tx_eth0"
    pub time_series: HashMap<String, VecDeque<(f64, f64)>>, // (timestamp, value)
    pub time_window_secs: u64, // Duration of history to keep/display
    pub max_history_points: usize, // Max data points per series
}

pub struct App {
    pub state: AppState,
    // Keep the provider trait object
    // Ensure the provider is Send + Sync if used across async tasks/threads
    pub provider: Arc<dyn McpMetricsProviderTrait + Send + Sync>,
}

impl App {
    // pub fn new(provider: Arc<dyn McpMetricsProviderTrait + Send + Sync>) -> Self { ... }
    // pub async fn update(&mut self) { /* Fetch data from provider */ }
    // pub fn on_key(&mut self, key: KeyEvent) { /* Handle input */ }
    // pub fn on_tick(&mut self) { /* Update internal state like time series */ }
}
```

## 2. Widget Data Requirements (`widgets/*.rs`)

Widgets should be designed to render based on immutable references to data derived from `AppState`.

*   **`HealthWidget`:** Needs `&Vec<HealthCheck>` (or similar derived structure from `AppState.connection_health`, `AppState.metrics`, etc.). We'll need to define `HealthCheck` or reuse/adapt the old one.
*   **`MetricsWidget`:** Needs `Option<&McpMetrics>`. Displays CPU %, Mem %, basic counts.
*   **`AlertsWidget`:** Needs `&[Alert]`. Displays recent alerts.
*   **`NetworkWidget`:** Needs `Option<&NetworkMetrics>` (likely nested within `McpMetrics`). Displays interface list, IPs, status, basic RX/TX totals.
*   **`ChartWidget`:** Needs `&VecDeque<(f64, f64)>` for data points, title, potentially axis bounds/labels. Should be generic enough for CPU, Mem, Network IO.
*   **`ConnectionHealthWidget`:** Needs `Option<&ConnectionHealth>` and `&ConnectionStatus`. Displays status, latency, etc.

## 3. Error Handling

*   Errors from the `McpMetricsProviderTrait` during the update cycle should be:
    *   Logged using the `log` crate.
    *   Stored in `AppState.recent_errors`.
    *   Potentially displayed in a dedicated status area (e.g., footer or a specific widget).
*   UI rendering errors (`ratatui` errors) should ideally be logged, and the application might need to attempt recovery or exit gracefully. Panics should be avoided.
*   Define a specific `Error` enum in `error.rs` for UI-specific failures.

## 4. Data Fetching Loop (`app.rs` or main loop in `lib.rs`)

*   An async task/loop should run periodically (e.g., every 1-2 seconds).
*   Inside the loop:
    1.  Call `provider.get_metrics()`.
    2.  Call `provider.get_connection_health()`.
    3.  Call `provider.get_connection_status()`.
    4.  Call `provider.get_recent_errors()`.
    5.  (Optional) Call `provider.get_protocol_metrics()`.
    6.  Update the fields in `AppState` with the results. Handle potential errors from the provider calls.
    7.  Update `AppState.last_update`.
    8.  Trigger a UI redraw if necessary.

## 5. Main Entry Point (`lib.rs`)

*   Define a public async function to start the UI.
*   It should take the `McpMetricsProviderTrait` as an argument.
*   It needs to initialize the terminal, create the `App` instance, run the main event loop (handling input and ticks), and restore the terminal on exit.

```rust
// Tentative structure in lib.rs
use dashboard_core::adapter::McpMetricsProviderTrait;
use std::sync::Arc;
use std::time::Duration;
use crate::{app::App, event::EventHandler, ui}; // Assuming these modules exist
use ratatui::backend::Backend;
use ratatui::Terminal;

// Define a Result type for the run function
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn run_ui<B: Backend>(
    
    terminal: &mut Terminal<B>,
    provider: Arc<dyn McpMetricsProviderTrait + Send + Sync>,
    tick_rate: Duration,      // e.g., Duration::from_millis(250)
    update_rate: Duration,    // e.g., Duration::from_secs(1)
) -> Result<()> {
    // 1. Setup logging, terminal raw mode, alternate screen

    // 2. Create App instance
    let app = App::new(provider);

    // 3. Create EventHandler (handles input and ticks)
    // let event_handler = EventHandler::new(tick_rate);

    // 4. Run main loop
    // loop {
        // handle events (input, tick) -> update app state
        // draw ui
        // check app.state.should_quit
    // }

    // 5. Restore terminal
    Ok(())
}
``` 