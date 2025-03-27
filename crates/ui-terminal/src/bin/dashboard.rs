use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use dashboard_core::data::{Alert, DashboardData, ProtocolData};
use dashboard_core::mcp::MockMcpClient;
use dashboard_core::mcp::McpClient;
use rand::Rng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::Terminal;
use tracing::info;
use ui_terminal::adapter::{McpAdapter, MonitoringToDashboardAdapter};
use ui_terminal::app::{App, Tab};
use ui_terminal::config::{ConfigError, DashboardConfig};
use ui_terminal::events::{Event as AppEvent, EventHandler};
use ui_terminal::widgets::alerts::AlertsWidget;
use ui_terminal::widgets::metrics::MetricsWidget;
use ui_terminal::widgets::protocol::ProtocolWidget;

/// Command line arguments
#[derive(Debug, Parser)]
#[clap(
    name = "MCP Dashboard",
    about = "A terminal dashboard for MCP metrics"
)]
struct Args {
    /// Update interval in milliseconds
    #[clap(short, long)]
    interval: Option<u64>,
    
    /// Maximum history points to keep
    #[clap(short, long)]
    history_points: Option<usize>,
    
    /// Use real MCP client
    #[clap(long, conflicts_with = "mock_mcp")]
    real_mcp: bool,
    
    /// Use mock MCP client
    #[clap(long, conflicts_with = "real_mcp")]
    mock_mcp: bool,
    
    /// Test error scenarios
    #[clap(long)]
    test_errors: bool,
    
    /// UI theme
    #[clap(short, long)]
    theme: Option<String>,
    
    /// Display help
    #[clap(short, long)]
    help: bool,
    
    /// Save configuration
    #[clap(long)]
    save_config: bool,
}

/// Dashboard application entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let result = DashboardConfig::load();
    let mut config = match result {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            info!("Using default configuration: {}", e);
            DashboardConfig::default()
        }
    };
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Override config with command line args
    if let Some(interval) = args.interval {
        config.update_interval = Duration::from_millis(interval);
    }
    
    if let Some(points) = args.history_points {
        config.max_history_points = points;
    }
    
    if args.real_mcp {
        config.use_real_mcp = true;
    } else if args.mock_mcp {
        config.use_real_mcp = false;
    }
    
    if args.test_errors {
        config.simulate_errors = true;
    }
    
    if let Some(theme) = args.theme {
        config.theme = theme;
    }
    
    // Save configuration if requested
    if args.save_config {
        if let Err(e) = config.save() {
            eprintln!("Failed to save configuration: {}", e);
        } else {
            println!("Configuration saved successfully");
        }
    }
    
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    
    // Create dashboard data
    let dashboard_data = Arc::new(Mutex::new(DashboardData::default()));
    
    // Create MCP client
    let mcp_client: Arc<Mutex<dyn McpClient + Send>> = if config.use_real_mcp {
        // TODO: Implement real MCP client
        eprintln!("Real MCP client not implemented yet, using mock client");
        Arc::new(Mutex::new(MockMcpClient::new()))
    } else {
        Arc::new(Mutex::new(MockMcpClient::new()))
    };
    
    // Create adapter
    let adapter = Arc::new(McpAdapter::new(
        mcp_client.clone(),
        config.max_history_points,
    ));
    
    // Create app with initial state
    let mut app = App::new("MCP Dashboard");
    app.set_help_visibility(args.help);
    
    // Create tabs
    let tabs = vec![
        Tab::new("Metrics", "1"),
        Tab::new("Protocol", "2"),
        Tab::new("Alerts", "3"),
    ];
    app.set_tabs(tabs);
    
    // Create event handler
    let event_handler = EventHandler::new(Duration::from_millis(250));
    
    // Track last update time
    let mut last_update = Instant::now();
    
    // Setup error simulation variables
    let mut simulate_error_counter = 0;
    let error_frequency = 10; // Every 10 updates
    
    // Generate some sample alerts
    let mut sample_alerts = vec![
        Alert {
            id: "alert-001".to_string(),
            severity: "critical".to_string(),
            message: "CPU usage above threshold".to_string(),
            details: Some("System CPU usage has exceeded 90% for more than 5 minutes".to_string()),
            timestamp: chrono::Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        },
        Alert {
            id: "alert-002".to_string(),
            severity: "warning".to_string(),
            message: "Memory usage increasing".to_string(),
            details: Some("System memory usage trend shows consistent increase over last hour".to_string()),
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(30),
            acknowledged: true,
            acknowledged_by: Some("system".to_string()),
            acknowledged_at: Some(chrono::Utc::now() - chrono::Duration::minutes(15)),
        },
        Alert {
            id: "alert-003".to_string(),
            severity: "info".to_string(),
            message: "Scheduled maintenance".to_string(),
            details: Some("System scheduled for maintenance in 24 hours".to_string()),
            timestamp: chrono::Utc::now() - chrono::Duration::hours(2),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        },
    ];
    
    // Run the event loop
    loop {
        // Draw the UI
        terminal.draw(|f| {
            // Create layout
            let size = f.size();
            
            // Create help text if enabled
            let chunks = if app.is_help_visible() {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // Tabs row
                        Constraint::Min(5),     // Content area
                        Constraint::Length(3),  // Help area
                    ])
                    .split(size)
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // Tabs row
                        Constraint::Min(0),     // Content area
                    ])
                    .split(size)
            };
            
            // Render tabs
            let titles = app.tabs().iter().map(|t| {
                let (name, key) = (t.name(), t.key());
                Span::styled(
                    format!("{} [{}] ", name, key),
                    Style::default().fg(Color::White),
                )
            });
            
            let tabs = Tabs::new(titles.collect())
                .block(Block::default().borders(Borders::ALL).title("Dashboard"))
                .select(app.selected_tab_index())
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            
            f.render_widget(tabs, chunks[0]);
            
            // Lock dashboard data for reading
            let dashboard_data = dashboard_data.blocking_lock();
            
            // Render content based on selected tab
            match app.selected_tab_index() {
                0 => {
                    // Metrics tab
                    let metrics_widget = MetricsWidget::new(&dashboard_data.metrics, "System Metrics");
                    f.render_widget(metrics_widget, chunks[1]);
                }
                1 => {
                    // Protocol tab
                    let protocol_widget = ProtocolWidget::new(&dashboard_data.protocol, "Protocol Status");
                    f.render_widget(protocol_widget, chunks[1]);
                }
                2 => {
                    // Alerts tab
                    let alerts_widget = AlertsWidget::new(&sample_alerts, "System Alerts");
                    f.render_widget(alerts_widget, chunks[1]);
                }
                _ => {}
            }
            
            // Render help if enabled
            if app.is_help_visible() {
                let help_text = vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().fg(Color::Yellow)),
                    Span::raw(" to exit, "),
                    Span::styled("h", Style::default().fg(Color::Yellow)),
                    Span::raw(" to toggle help, "),
                    Span::styled("1-3", Style::default().fg(Color::Yellow)),
                    Span::raw(" to switch tabs"),
                ];
                
                let help = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(help_text))
                    .block(Block::default().borders(Borders::ALL).title("Help"));
                
                f.render_widget(help, chunks[2]);
            }
        })?;
        
        // Check if it's time to update data
        if last_update.elapsed() >= config.update_interval {
            // Update data from MCP client
            let mut client = mcp_client.lock().await;
            
            // Simulate errors if enabled
            let should_simulate_error = config.simulate_errors && 
                simulate_error_counter % error_frequency == 0 &&
                simulate_error_counter > 0;
                
            if should_simulate_error {
                info!("Simulating connection error");
                client.set_should_fail(true);
                
                // Update protocol data to show error state
                let mut protocol = dashboard_data.lock().await.protocol.clone();
                protocol.connected = false;
                protocol.status = "Error".to_string();
                protocol.error = Some("Simulated connection error".to_string());
                protocol.retry_count += 1;
                dashboard_data.lock().await.protocol = protocol;
            } else {
                client.set_should_fail(false);
            }
            
            // Update dashboard data
            let result = adapter.update_dashboard_data(&mut dashboard_data.lock().await).await;
            
            if let Err(e) = result {
                info!("Error updating dashboard: {}", e);
            }
            
            // Occasionally add a new alert
            if rand::thread_rng().gen_ratio(1, 20) {
                let severity = match rand::thread_rng().gen_range(0..3) {
                    0 => "info",
                    1 => "warning",
                    _ => "critical",
                };
                
                sample_alerts.push(Alert {
                    id: format!("alert-{:03}", sample_alerts.len() + 1),
                    severity: severity.to_string(),
                    message: format!("Dynamic alert #{}", sample_alerts.len() + 1),
                    details: Some(format!("This is a dynamically generated {} alert", severity)),
                    timestamp: chrono::Utc::now(),
                    acknowledged: false,
                    acknowledged_by: None,
                    acknowledged_at: None,
                });
                
                // Limit number of alerts to 20
                if sample_alerts.len() > 20 {
                    sample_alerts.remove(0);
                }
            }
            
            last_update = Instant::now();
            simulate_error_counter += 1;
        }
        
        // Handle events
        if let AppEvent::Key(key) = event_handler.next()? {
            match key.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Char('h') => {
                    app.set_help_visibility(!app.is_help_visible());
                }
                KeyCode::Char(c) if c >= '1' && c <= '3' => {
                    let index = c as usize - '1' as usize;
                    app.select_tab(index);
                }
                KeyCode::Tab => {
                    let index = (app.selected_tab_index() + 1) % app.tabs().len();
                    app.select_tab(index);
                }
                KeyCode::BackTab => {
                    let index = if app.selected_tab_index() == 0 {
                        app.tabs().len() - 1
                    } else {
                        app.selected_tab_index() - 1
                    };
                    app.select_tab(index);
                }
                _ => {}
            }
        }
    }
    
    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    
    Ok(())
} 