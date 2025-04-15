use std::io;
use std::time::Duration;
use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use clap::Parser;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Include necessary types directly in the example
// First, define the connection status types similar to those in the ui-terminal crate

/// Status of a connection
#[derive(Debug, Clone, PartialEq)]
enum ConnectionStatus {
    /// Connected state
    Connected,
    /// Disconnected state
    Disconnected,
    /// Connecting state
    Connecting,
    /// Error state with a message
    Error(String),
    /// Degraded quality state
    Degraded,
    /// Unknown state
    Unknown,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "Connected"),
            ConnectionStatus::Disconnected => write!(f, "Disconnected"),
            ConnectionStatus::Connecting => write!(f, "Connecting"),
            ConnectionStatus::Error(msg) => write!(f, "Error: {}", msg),
            ConnectionStatus::Degraded => write!(f, "Degraded"),
            ConnectionStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Type of connection event
#[derive(Debug, Clone, PartialEq)]
enum ConnectionEventType {
    /// Connection established
    Connected,
    /// Connection lost
    Disconnected,
    /// Attempting to reconnect
    Reconnecting,
    /// Successfully reconnected
    ReconnectSuccess,
    /// Failed to reconnect
    ReconnectFailure,
    /// Error occurred
    Error,
}

/// Connection event
#[derive(Debug, Clone)]
struct ConnectionEvent {
    /// Event type
    event_type: ConnectionEventType,
    /// Additional details
    details: String,
    /// When the event occurred
    timestamp: DateTime<Utc>,
}

/// Health information about a connection
#[derive(Debug, Clone)]
struct ConnectionHealth {
    /// Latency in milliseconds
    latency_ms: f64,
    /// Packet loss percentage (0-100)
    packet_loss: f64,
    /// Connection stability percentage (0-100)
    stability: f64,
    /// Signal strength percentage (0-100)
    signal_strength: f64,
    /// Overall health score (0.0-1.0)
    health_score: f64,
    /// Current connection status
    status: ConnectionStatus,
    /// When the connection was established
    connected_since: Option<DateTime<Utc>>,
    /// When the last status change occurred
    last_status_change: Option<DateTime<Utc>>,
    /// When the health was last checked
    last_checked: DateTime<Utc>,
}

/// Connection Health Widget to display connection health information
struct ConnectionHealthWidget<'a> {
    /// Widget title
    title: &'a str,
    /// Connection health data
    health: Option<&'a ConnectionHealth>,
    /// Connection history data
    history: Option<&'a [ConnectionEvent]>,
    /// History metrics for visualization
    history_metrics: Option<&'a [(DateTime<Utc>, f64)]>,
    /// Health score history
    health_score_history: VecDeque<u64>,
}

impl<'a> ConnectionHealthWidget<'a> {
    /// Create a new widget with title
    fn new(title: &'a str) -> Self {
        Self {
            title,
            health: None,
            history: None,
            history_metrics: None,
            health_score_history: VecDeque::with_capacity(50),
        }
    }
    
    /// Set the connection health data
    fn with_health(mut self, health: &'a ConnectionHealth) -> Self {
        // Convert health score to percentage (0-100)
        let score = (health.health_score * 100.0) as u64;
        
        // Add to history, maintaining max size
        self.health_score_history.push_back(score);
        if self.health_score_history.len() > 50 {
            self.health_score_history.pop_front();
        }
        
        self.health = Some(health);
        self
    }
    
    /// Set the connection history data
    fn with_history(mut self, history: &'a [ConnectionEvent]) -> Self {
        self.history = Some(history);
        self
    }
    
    /// Set the history metrics data
    fn with_history_metrics(mut self, metrics: &'a [(DateTime<Utc>, f64)]) -> Self {
        self.history_metrics = Some(metrics);
        self
    }
    
    /// Get color for health score
    fn health_score_color(score: f64) -> Color {
        if score >= 0.8 {
            Color::Green
        } else if score >= 0.6 {
            Color::Yellow
        } else if score >= 0.4 {
            Color::Rgb(255, 165, 0) // Orange
        } else {
            Color::Red
        }
    }
    
    /// Get color for connection status
    fn connection_status_color(status: &ConnectionStatus) -> Color {
        match status {
            ConnectionStatus::Connected => Color::Green,
            ConnectionStatus::Connecting => Color::Yellow,
            ConnectionStatus::Disconnected => Color::Red,
            ConnectionStatus::Error(_) => Color::Red,
            ConnectionStatus::Degraded => Color::Yellow,
            ConnectionStatus::Unknown => Color::Gray,
        }
    }
    
    /// Get color for a connection event type
    fn connection_event_color(event_type: &ConnectionEventType) -> Color {
        match event_type {
            ConnectionEventType::Connected => Color::Green,
            ConnectionEventType::ReconnectSuccess => Color::Green,
            ConnectionEventType::Reconnecting => Color::Yellow,
            ConnectionEventType::Disconnected => Color::Red,
            ConnectionEventType::ReconnectFailure => Color::Red,
            ConnectionEventType::Error => Color::Red,
        }
    }
    
    /// Format duration as a human-readable string
    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        
        if total_secs < 60 {
            format!("{}s", total_secs)
        } else if total_secs < 3600 {
            format!("{}m {}s", total_secs / 60, total_secs % 60)
        } else {
            format!("{}h {}m", total_secs / 3600, (total_secs % 3600) / 60)
        }
    }
    
    /// Render the connection status section
    fn render_status(&self, f: &mut Frame, area: Rect) {
        if let Some(health) = self.health {
            // Create block
            let block = Block::default()
                .title("Connection Status")
                .borders(Borders::ALL);
            
            // Render block
            f.render_widget(block.clone(), area);
            
            // Calculate inner area
            let inner_area = block.inner(area);
            
            // Create vertical layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(1), // Status
                    Constraint::Length(1), // Since
                    Constraint::Length(1), // Health score
                    Constraint::Length(1), // Sparkline
                    Constraint::Length(1), // Latency
                    Constraint::Length(1), // Stability
                ])
                .split(inner_area);
            
            // Status text
            let status_text = health.status.to_string();
            
            let status_line = Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    status_text, 
                    Style::default()
                        .fg(Self::connection_status_color(&health.status))
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            
            // Since/uptime text
            let since_text = if let Some(since) = health.connected_since {
                let now = Utc::now();
                let duration = now.signed_duration_since(since);
                
                // Format duration
                let hours = duration.num_hours();
                let mins = duration.num_minutes() % 60;
                let secs = duration.num_seconds() % 60;
                
                if hours > 0 {
                    format!("{}h {}m {}s", hours, mins, secs)
                } else if mins > 0 {
                    format!("{}m {}s", mins, secs)
                } else {
                    format!("{}s", secs)
                }
            } else {
                match health.status {
                    ConnectionStatus::Connected => "Just connected".to_string(),
                    ConnectionStatus::Connecting => "Connecting...".to_string(),
                    ConnectionStatus::Disconnected => {
                        if let Some(last) = health.last_status_change {
                            let now = Utc::now();
                            let duration = now.signed_duration_since(last);
                            
                            // Format duration for how long disconnected
                            let hours = duration.num_hours();
                            let mins = duration.num_minutes() % 60;
                            let secs = duration.num_seconds() % 60;
                            
                            if hours > 0 {
                                format!("Down for {}h {}m {}s", hours, mins, secs)
                            } else if mins > 0 {
                                format!("Down for {}m {}s", mins, secs)
                            } else {
                                format!("Down for {}s", secs)
                            }
                        } else {
                            "Disconnected".to_string()
                        }
                    }
                    ConnectionStatus::Error(_) => "Error".to_string(),
                    ConnectionStatus::Degraded => "Degraded".to_string(),
                    ConnectionStatus::Unknown => "Unknown Status".to_string(),
                }
            };
            
            let since_line = Line::from(vec![
                Span::raw("Uptime: "),
                Span::styled(
                    since_text, 
                    Style::default().fg(Color::White),
                ),
            ]);
            
            // Health score
            let health_score = health.health_score * 100.0;
            let health_color = Self::health_score_color(health.health_score);
            
            let health_gauge = Gauge::default()
                .block(Block::default().title("Health Score"))
                .gauge_style(Style::default().fg(health_color))
                .use_unicode(true)
                .ratio(health.health_score)
                .label(format!("{:.1}%", health_score));
            
            // Convert history data to vector
            let history_data: Vec<u64> = self.health_score_history.iter().copied().collect();
            
            // Health score history sparkline
            let sparkline = Sparkline::default()
                .block(Block::default().title("Health Score History"))
                .style(Style::default().fg(Color::Blue))
                .data(&history_data);
            
            // Latency
            let latency_text = format!("{:.2} ms", health.latency_ms);
            let latency_line = Line::from(vec![
                Span::raw("Latency: "),
                Span::styled(
                    latency_text, 
                    Style::default().fg(Color::White),
                ),
            ]);
            
            // Stability
            let stability_text = format!("{:.1}%", health.stability);
            let stability_color = if health.stability >= 90.0 {
                Color::Green
            } else if health.stability >= 70.0 {
                Color::Yellow
            } else {
                Color::Red
            };
            
            let stability_line = Line::from(vec![
                Span::raw("Stability: "),
                Span::styled(
                    stability_text, 
                    Style::default().fg(stability_color),
                ),
            ]);
            
            // Render everything
            f.render_widget(Paragraph::new(status_line), chunks[0]);
            f.render_widget(Paragraph::new(since_line), chunks[1]);
            f.render_widget(health_gauge, chunks[2]);
            f.render_widget(sparkline, chunks[3]);
            f.render_widget(Paragraph::new(latency_line), chunks[4]);
            f.render_widget(Paragraph::new(stability_line), chunks[5]);
        } else {
            // No health data available
            let block = Block::default()
                .title("Connection Status")
                .borders(Borders::ALL);
            
            f.render_widget(block.clone(), area);
            
            let inner_area = block.inner(area);
            
            let text = Paragraph::new(Line::from(vec![
                Span::styled(
                    "No connection health data available", 
                    Style::default().fg(Color::Gray),
                ),
            ]))
            .alignment(Alignment::Center);
            
            f.render_widget(text, inner_area);
        }
    }
    
    /// Render connection history
    fn render_history(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Connection History")
            .borders(Borders::ALL);
        
        f.render_widget(block.clone(), area);
        
        let inner_area = block.inner(area);
        
        if let Some(history) = self.history {
            if history.is_empty() {
                // No history data
                let text = Paragraph::new(Line::from(vec![
                    Span::styled(
                        "No connection history data available", 
                        Style::default().fg(Color::Gray),
                    ),
                ]))
                .alignment(Alignment::Center);
                
                f.render_widget(text, inner_area);
                return;
            }
            
            // Get the most recent events first (reversed)
            let recent_events: Vec<_> = history.iter().rev().take(10).collect();
            
            // Create lines for each event
            let mut event_lines = Vec::new();
            
            for event in recent_events {
                let event_type_text = match event.event_type {
                    ConnectionEventType::Connected => "Connected",
                    ConnectionEventType::Disconnected => "Disconnected",
                    ConnectionEventType::Reconnecting => "Reconnecting",
                    ConnectionEventType::ReconnectSuccess => "Reconnect Success",
                    ConnectionEventType::ReconnectFailure => "Reconnect Failed",
                    ConnectionEventType::Error => "Error",
                };
                
                let time_text = event.timestamp.format("%H:%M:%S").to_string();
                
                let mut spans = vec![
                    Span::styled(
                        format!("[{}] ", time_text), 
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(
                        event_type_text, 
                        Style::default()
                            .fg(Self::connection_event_color(&event.event_type))
                            .add_modifier(Modifier::BOLD),
                    ),
                ];
                
                // Add details if available
                if !event.details.is_empty() {
                    spans.push(Span::raw(" - "));
                    spans.push(Span::raw(event.details.clone()));
                }
                
                event_lines.push(Line::from(spans));
            }
            
            // Render as paragraph
            let events_paragraph = Paragraph::new(event_lines);
            
            f.render_widget(events_paragraph, inner_area);
        } else {
            // No history data
            let text = Paragraph::new(Line::from(vec![
                Span::styled(
                    "No connection history data available", 
                    Style::default().fg(Color::Gray),
                ),
            ]))
            .alignment(Alignment::Center);
            
            f.render_widget(text, inner_area);
        }
    }
    
    /// Render the widget
    fn render(&self, f: &mut Frame, area: Rect) {
        // Create block
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Calculate inner area
        let inner_area = block.inner(area);
        
        // Create horizontal layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(inner_area);
        
        // Render the two sections
        self.render_status(f, chunks[0]);
        self.render_history(f, chunks[1]);
    }
}

/// Command-line arguments for the connection health monitor example
#[derive(Parser, Debug)]
#[clap(author, version, about = "Connection Health Monitor Example")]
struct Args {
    /// Connection status to simulate (connected, disconnected, connecting, error)
    #[clap(short, long, default_value = "connected")]
    status: String,
    
    /// Simulate random connection events
    #[clap(short, long)]
    simulate_events: bool,
    
    /// Update interval in milliseconds
    #[clap(short, long, default_value = "1000")]
    interval: u64,
}

/// Application state
struct App {
    /// Connection health data
    health: ConnectionHealth,
    /// Connection history events
    events: Vec<ConnectionEvent>,
    /// Health score history metrics
    metrics: Vec<(DateTime<Utc>, f64)>,
    /// Whether the application is running
    running: bool,
    /// Random event simulation active
    simulate_events: bool,
    /// Iteration counter
    counter: usize,
}

impl App {
    /// Create a new application with the specified initial status
    fn new(status: ConnectionStatus, simulate_events: bool) -> Self {
        let now = Utc::now();
        
        // Create initial health data
        let health = ConnectionHealth {
            latency_ms: 25.0,
            packet_loss: 0.5,
            stability: 98.5,
            signal_strength: 95.0,
            health_score: 0.95,
            status: status.clone(),
            connected_since: if let ConnectionStatus::Connected = status {
                Some(now - chrono::Duration::minutes(5))
            } else {
                None
            },
            last_status_change: Some(now),
            last_checked: now,
        };
        
        // Create initial events
        let events = vec![
            ConnectionEvent {
                event_type: ConnectionEventType::Connected,
                details: "Initial connection established".to_string(),
                timestamp: now - chrono::Duration::minutes(5),
            },
        ];
        
        // Create initial metrics
        let metrics: Vec<(DateTime<Utc>, f64)> = (0..10).map(|i| {
            let time = now - chrono::Duration::seconds(i * 30);
            let score = 0.9 + (rand::random::<f64>() * 0.1 - 0.05);
            (time, score)
        }).collect();
        
        Self {
            health,
            events,
            metrics,
            running: true,
            simulate_events,
            counter: 0,
        }
    }
    
    /// Update the application state
    fn update(&mut self) {
        self.counter += 1;
        let now = Utc::now();
        
        // Update health data
        self.health.last_checked = now;
        
        // Simulate latency variations
        self.health.latency_ms = 20.0 + (rand::random::<f64>() * 10.0);
        
        // Simulate varying signal strength
        self.health.signal_strength = 92.0 + (rand::random::<f64>() * 6.0);
        
        // Sometimes update health score
        if self.counter % 3 == 0 {
            // Slight variations to health score
            self.health.health_score = (self.health.health_score * 0.95) + 
                (rand::random::<f64>() * 0.05);
            
            // Ensure between 0.0 and 1.0
            self.health.health_score = self.health.health_score.max(0.0).min(1.0);
        }
        
        // Add a new metric point
        self.metrics.push((now, self.health.health_score));
        if self.metrics.len() > 20 {
            self.metrics.remove(0);
        }
        
        // Simulate random connection events if enabled
        if self.simulate_events && self.counter % 5 == 0 {
            // 20% chance of a connection event
            if rand::random::<f64>() < 0.2 {
                let event_types = [
                    ConnectionEventType::Connected,
                    ConnectionEventType::Disconnected,
                    ConnectionEventType::Reconnecting,
                    ConnectionEventType::ReconnectSuccess,
                    ConnectionEventType::ReconnectFailure,
                    ConnectionEventType::Error,
                ];
                
                let event_type = event_types[rand::random::<usize>() % event_types.len()].clone();
                
                // Update connection status based on event
                match event_type {
                    ConnectionEventType::Connected | ConnectionEventType::ReconnectSuccess => {
                        self.health.status = ConnectionStatus::Connected;
                        self.health.connected_since = Some(now);
                    },
                    ConnectionEventType::Disconnected => {
                        self.health.status = ConnectionStatus::Disconnected;
                        self.health.connected_since = None;
                    },
                    ConnectionEventType::Reconnecting => {
                        self.health.status = ConnectionStatus::Connecting;
                        self.health.connected_since = None;
                    },
                    ConnectionEventType::ReconnectFailure | ConnectionEventType::Error => {
                        self.health.status = ConnectionStatus::Error("Connection failed".to_string());
                        self.health.connected_since = None;
                    },
                }
                
                self.health.last_status_change = Some(now);
                
                // Create event details
                let details = match event_type {
                    ConnectionEventType::Connected => "Connection established".to_string(),
                    ConnectionEventType::Disconnected => "Connection lost".to_string(),
                    ConnectionEventType::Reconnecting => "Attempting to reconnect".to_string(),
                    ConnectionEventType::ReconnectSuccess => "Successfully reconnected".to_string(),
                    ConnectionEventType::ReconnectFailure => "Failed to reconnect".to_string(),
                    ConnectionEventType::Error => "Connection error occurred".to_string(),
                };
                
                // Add the event
                self.events.push(ConnectionEvent {
                    event_type,
                    details,
                    timestamp: now,
                });
                
                // Keep a limited history
                if self.events.len() > 50 {
                    self.events.remove(0);
                }
            }
        }
    }
    
    /// Render the application UI
    fn render(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        terminal.draw(|f| {
            // Create main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Min(0),     // Content
                    Constraint::Length(3),  // Instructions
                ])
                .split(f.size());
            
            // Create title
            let title = Paragraph::new("Connection Health Monitor Example")
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);
            
            f.render_widget(title, chunks[0]);
            
            // Create connection health widget
            let connection_widget = ConnectionHealthWidget::new("Connection Health Status")
                .with_health(&self.health)
                .with_history(&self.events)
                .with_history_metrics(&self.metrics);
            
            // Render the widget
            connection_widget.render(f, chunks[1]);
            
            // Create instructions
            let instructions = Paragraph::new("Press 'q' to quit, 'c' to connect, 'd' to disconnect, 'r' to reconnect, 'e' to generate error")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            
            f.render_widget(instructions, chunks[2]);
        })?;
        
        Ok(())
    }
    
    /// Handle user input
    fn handle_input(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => self.running = false,
                    KeyCode::Char('c') => {
                        // Connect
                        self.health.status = ConnectionStatus::Connected;
                        self.health.connected_since = Some(Utc::now());
                        self.health.last_status_change = Some(Utc::now());
                        self.events.push(ConnectionEvent {
                            event_type: ConnectionEventType::Connected,
                            details: "Manual connection".to_string(),
                            timestamp: Utc::now(),
                        });
                    },
                    KeyCode::Char('d') => {
                        // Disconnect
                        self.health.status = ConnectionStatus::Disconnected;
                        self.health.connected_since = None;
                        self.health.last_status_change = Some(Utc::now());
                        self.events.push(ConnectionEvent {
                            event_type: ConnectionEventType::Disconnected,
                            details: "Manual disconnection".to_string(),
                            timestamp: Utc::now(),
                        });
                    },
                    KeyCode::Char('r') => {
                        // Reconnect
                        self.health.status = ConnectionStatus::Connecting;
                        self.health.connected_since = None;
                        self.health.last_status_change = Some(Utc::now());
                        self.events.push(ConnectionEvent {
                            event_type: ConnectionEventType::Reconnecting,
                            details: "Manual reconnection".to_string(),
                            timestamp: Utc::now(),
                        });
                        
                        // Simulate reconnection process
                        if rand::random::<f64>() > 0.3 {
                            // 70% chance of success
                            self.health.status = ConnectionStatus::Connected;
                            self.health.connected_since = Some(Utc::now());
                            self.events.push(ConnectionEvent {
                                event_type: ConnectionEventType::ReconnectSuccess,
                                details: "Reconnection successful".to_string(),
                                timestamp: Utc::now(),
                            });
                        } else {
                            // 30% chance of failure
                            self.health.status = ConnectionStatus::Error("Reconnection failed".to_string());
                            self.events.push(ConnectionEvent {
                                event_type: ConnectionEventType::ReconnectFailure,
                                details: "Reconnection failed".to_string(),
                                timestamp: Utc::now(),
                            });
                        }
                    },
                    KeyCode::Char('e') => {
                        // Generate error
                        self.health.status = ConnectionStatus::Error("Simulated error condition".to_string());
                        self.health.connected_since = None;
                        self.health.last_status_change = Some(Utc::now());
                        self.events.push(ConnectionEvent {
                            event_type: ConnectionEventType::Error,
                            details: "Manual error simulation".to_string(),
                            timestamp: Utc::now(),
                        });
                    },
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
}

/// Main function
fn main() -> io::Result<()> {
    // Parse command-line arguments
    let args = Args::parse();
    
    // Parse connection status from arguments
    let status = match args.status.to_lowercase().as_str() {
        "connected" => ConnectionStatus::Connected,
        "disconnected" => ConnectionStatus::Disconnected,
        "connecting" => ConnectionStatus::Connecting,
        "error" => ConnectionStatus::Error("Initial error state".to_string()),
        _ => ConnectionStatus::Connected,
    };
    
    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create application
    let mut app = App::new(status, args.simulate_events);
    
    // Main loop
    let tick_rate = Duration::from_millis(args.interval);
    let mut last_tick = std::time::Instant::now();
    
    while app.running {
        // Render the UI
        app.render(&mut terminal)?;
        
        // Handle input
        app.handle_input()?;
        
        // Update at the specified tick rate
        let now = std::time::Instant::now();
        if now.duration_since(last_tick) >= tick_rate {
            app.update();
            last_tick = now;
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    println!("Thanks for using the Connection Health Monitor Example!");
    
    Ok(())
} 