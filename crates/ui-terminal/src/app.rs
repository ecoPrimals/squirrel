use dashboard_core::data::{Alert, Metrics, AlertSeverity, ProtocolData};
use dashboard_core::service::DashboardService;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent};
use log::{error, warn};
use crate::widgets::health::{HealthCheck, HealthStatus};
use crate::error::Error as UiError; // Rename UiError to Error, alias locally if needed
use dashboard_core::health::{HealthStatus as CoreHealthStatus}; // Use alias for core status

// TODO: Define these properly, placeholders for now
type ConnectionHealth = String;
type ConnectionStatus = CoreHealthStatus;

const MAX_PROVIDER_ERRORS: usize = 10; // Keep last 10 errors from the provider
const MAX_HISTORY_POINTS: usize = 100; // Keep last 100 data points for charts

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveTab {
    Overview,
    System,
    Network,
    Protocol,
    Alerts,
}

impl Default for ActiveTab {
    fn default() -> Self {
        ActiveTab::Overview
    }
}


#[derive(Debug)]
pub struct AppState {
    // Data fetched from provider
    pub metrics: Option<Metrics>,
    pub protocol_data: Option<ProtocolData>,
    pub connection_health: Option<ConnectionHealth>,
    pub connection_status: ConnectionStatus,
    pub alerts: Vec<Alert>,
    pub recent_errors: Vec<UiError>,

    // Internal UI State
    pub active_tab: ActiveTab,
    pub show_help: bool,
    pub should_quit: bool,
    pub last_update: Option<DateTime<Utc>>,

    // Time series data for charts
    pub time_series: HashMap<String, VecDeque<(f64, f64)>>,
    pub time_window_secs: u64,
    pub max_history_points: usize,

    // New time series data
    pub cpu_history: VecDeque<(DateTime<Utc>, f64)>,
    pub memory_history: VecDeque<(DateTime<Utc>, f64)>,
}

impl Default for AppState {
     fn default() -> Self {
         Self {
             metrics: None,
             protocol_data: None,
             connection_health: None,
             connection_status: ConnectionStatus::Unknown,
             alerts: Vec::new(),
             recent_errors: Vec::new(),
             active_tab: ActiveTab::default(),
             show_help: false,
             should_quit: false,
             last_update: None,
             time_series: HashMap::new(),
             time_window_secs: 60, // Default to 1 minute window
             max_history_points: 100, // Default max points
             cpu_history: VecDeque::with_capacity(MAX_HISTORY_POINTS),
             memory_history: VecDeque::with_capacity(MAX_HISTORY_POINTS),
         }
     }
}


#[derive(Debug)]
pub struct App<S: DashboardService + Send + Sync + 'static + ?Sized> {
    pub state: AppState,
    pub provider: Arc<S>,
}

impl<S: DashboardService + Send + Sync + 'static + ?Sized> App<S> {
    /// Constructs a new instance of `App`.
    pub fn new(provider: Arc<S>) -> Self {
        let state = AppState::default();
        Self {
            state,
            provider,
        }
    }

    /// Infers the CoreHealthStatus based on ProtocolData.
    ///
    /// Maps the `connected` status and the `status` string from `ProtocolData`
    /// to a `CoreHealthStatus` enum variant (Ok, Warning, Critical, Unknown).
    fn infer_connection_status(protocol: &ProtocolData) -> CoreHealthStatus {
        if !protocol.connected {
            return CoreHealthStatus::Critical; // Not connected is critical
        }

        let status_lower = protocol.status.to_lowercase();
        if status_lower.is_empty() {
            return CoreHealthStatus::Unknown; // Connected, but empty status is unknown
        }

        if status_lower.contains("ok") || status_lower.contains("connected") {
            CoreHealthStatus::Ok
        } else if status_lower.contains("warn") || status_lower.contains("degraded") {
            CoreHealthStatus::Warning
        } else if status_lower.contains("error") || status_lower.contains("failed") || status_lower.contains("critical") {
            CoreHealthStatus::Critical // Explicitly map error states to Critical
        } else {
            CoreHealthStatus::Unknown // Connected, but status string is unrecognized
        }
    }

    /// Placeholder for handling key input events
    pub fn on_key(&mut self, key: KeyEvent) {
         match key.code {
             KeyCode::Char('q') => self.state.should_quit = true,
             KeyCode::Char('h') => self.state.show_help = !self.state.show_help,
             KeyCode::Char('1') => self.state.active_tab = ActiveTab::Overview,
             KeyCode::Char('2') => self.state.active_tab = ActiveTab::Network,
             KeyCode::Char('3') => self.state.active_tab = ActiveTab::System,
             KeyCode::Char('4') => self.state.active_tab = ActiveTab::Protocol,
             KeyCode::Char('5') => self.state.active_tab = ActiveTab::Alerts,
             _ => {}
         }
    }

    /// Placeholder for actions performed on each UI tick (e.g., animation, cleanup)
    pub fn on_tick(&mut self) {
        // Example: Trim old data from time_series if needed
        // self.trim_time_series();
    }

    /// Fetches data from the provider and updates the application state.
    pub async fn update(&mut self) {
        let now = Utc::now();
        self.state.last_update = Some(now);

        match self.provider.get_dashboard_data().await {
            Ok(dashboard_data) => {
                // --- Update Metrics --- 
                let metrics = dashboard_data.metrics;
                let protocol = dashboard_data.protocol;
                self.state.metrics = Some(metrics.clone()); // Clone metrics if needed elsewhere

                // --- Update Protocol Data --- 
                self.state.protocol_data = Some(protocol.clone());

                // --- Update Alerts --- 
                self.state.alerts = dashboard_data.alerts;

                // --- Update Connection Status (Infer from ProtocolData) ---
                self.state.connection_status = Self::infer_connection_status(&protocol);
                
                // --- Update Connection Health String --- 
                self.state.connection_health = Some(protocol.status);

                // --- Update Time Series --- 
                let cpu_usage = metrics.cpu.usage as f64;
                let mem_percentage = if metrics.memory.total > 0 {
                    (metrics.memory.used as f64 / metrics.memory.total as f64) * 100.0
                } else {
                    0.0
                };

                if self.state.cpu_history.len() >= MAX_HISTORY_POINTS {
                    self.state.cpu_history.pop_front();
                }
                self.state.cpu_history.push_back((now, cpu_usage));

                if self.state.memory_history.len() >= MAX_HISTORY_POINTS {
                    self.state.memory_history.pop_front();
                }
                self.state.memory_history.push_back((now, mem_percentage));
                // --- End Time Series Update --- 
                
                // Check for protocol errors
                if let Some(protocol_error) = protocol.error {
                     let err_msg = format!("Protocol error reported: {}", protocol_error);
                     warn!("{}", err_msg);
                     self.state.add_error(UiError::ProviderSpecificError(err_msg));
                }

            }
            Err(e) => {
                let err_msg = format!("Failed to fetch dashboard data: {}", e);
                error!("{}", err_msg);
                self.state.add_error(UiError::DataProvider(err_msg));

                // Reset state or set to error indicators
                self.state.metrics = None;
                self.state.protocol_data = None;
                self.state.alerts.clear();
                self.state.connection_status = CoreHealthStatus::Unknown; // Or Critical?
                self.state.connection_health = None;
                // Clear time series as well?
                // self.state.cpu_history.clear();
                // self.state.memory_history.clear();
            }
        }
    }

    /// Generates a vector of HealthCheck items based on the current AppState.
    pub fn get_health_checks(&self) -> Vec<HealthCheck> {
        let mut checks = Vec::new();
        let state = &self.state;

        // 1. Connection Status
        // Match directly on the variants
        let (conn_status_local, conn_msg) = match state.connection_status {
            CoreHealthStatus::Ok => (HealthStatus::Healthy, "Healthy"),
            CoreHealthStatus::Warning => (HealthStatus::Warning, "Warning"), // Use generic message or fetch details elsewhere if needed
            CoreHealthStatus::Critical => (HealthStatus::Critical, "Critical"), // Use generic message
            CoreHealthStatus::Unknown => (HealthStatus::Unknown, "Unknown"),
        };
        checks.push(HealthCheck::new("Connection", conn_status_local).with_message(conn_msg));

        // 2. System Metrics (CPU, Memory)
        if let Some(metrics) = &state.metrics {
            // CPU Check
            let cpu_usage = metrics.cpu.usage;
            let cpu_status = if cpu_usage < 70.0 {
                HealthStatus::Healthy
            } else if cpu_usage < 90.0 {
                HealthStatus::Warning
            } else {
                HealthStatus::Critical
            };
            checks.push(
                HealthCheck::new("CPU", cpu_status)
                    .with_message(format!("{:.1}%", cpu_usage))
                    .with_percentage(cpu_usage)
            );

            // Memory Check
            let mem_used = metrics.memory.used;
            let mem_total = metrics.memory.total;
            let mem_percent = if mem_total > 0 {
                (mem_used as f64 / mem_total as f64) * 100.0
            } else {
                0.0
            };
            let mem_status = if mem_percent < 70.0 {
                HealthStatus::Healthy
            } else if mem_percent < 90.0 {
                HealthStatus::Warning
            } else {
                HealthStatus::Critical
            };
            checks.push(
                HealthCheck::new("Memory", mem_status)
                    .with_message(format!(
                        "{} / {}",
                        crate::util::format_bytes(mem_used),
                        crate::util::format_bytes(mem_total)
                    ))
                    .with_percentage(mem_percent)
            );
        } else {
            checks.push(HealthCheck::new("CPU", HealthStatus::Unknown).with_message("No data"));
            checks.push(HealthCheck::new("Memory", HealthStatus::Unknown).with_message("No data"));
        }

        // 3. Connection Health
        if let Some(health_str) = &state.connection_health {
            // Simple check based on string content - adjust logic as needed
            let status = if health_str.to_lowercase().contains("ok") || health_str.is_empty() {
                HealthStatus::Healthy
            } else if health_str.to_lowercase().contains("warn") || health_str.to_lowercase().contains("degraded") {
                HealthStatus::Warning
            } else {
                HealthStatus::Critical // Assume critical if not OK/Warning
            };
            checks.push(HealthCheck::new("Link Quality", status).with_message(health_str));
        } else {
            checks.push(HealthCheck::new("Link Quality", HealthStatus::Unknown).with_message("No data"));
        }

        // 4. Alerts Status
        let critical_alerts = state.alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Critical)).count();
        let warning_alerts = state.alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Warning)).count();
        let alert_status = if critical_alerts > 0 {
            HealthStatus::Critical
        } else if warning_alerts > 0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
         checks.push(
             HealthCheck::new("Alerts", alert_status)
                .with_message(format!("{} Crit, {} Warn", critical_alerts, warning_alerts))
         );

        checks
    }

    // Placeholder for handling key events (e.g., tab switching)
    pub async fn handle_key_event(&mut self, _key: KeyEvent) {
        // TODO: Implement key handling logic, especially for tab switching
        // For example:
        // match key.code {
        //     KeyCode::Char('1') => self.state.active_tab = ActiveTab::Overview,
        //     KeyCode::Char('2') => self.state.active_tab = ActiveTab::System,
        //     ...
        //     KeyCode::Char('q') => self.state.should_quit = true,
        //     _ => {}
        // }
    }
}

impl AppState {
    /// Adds an error to the recent errors list, maintaining max size.
    fn add_error(&mut self, error: UiError) {
        if self.recent_errors.len() >= MAX_PROVIDER_ERRORS {
            self.recent_errors.remove(0); // Remove the oldest error
        }
        self.recent_errors.push(error);
    }

    // TODO: Add methods for managing time series data
    // fn trim_time_series(&mut self) { ... }
} 