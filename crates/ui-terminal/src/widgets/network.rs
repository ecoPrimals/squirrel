// crates/ui-terminal/src/widgets/network.rs
// Placeholder for NetworkWidget implementation

use ratatui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Style, Modifier, Stylize},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    Frame,
};
use crate::app::App; // App is no longer generic
use dashboard_core::data::NetworkInterface; // Assuming this is the correct type
use dashboard_core::service::DashboardService;
use dashboard_core::data::Metrics;

/// Format bytes to a human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Renders the Network tab widget.
///
/// Displays details about network interfaces, including total Rx/Tx bytes and packets,
/// and per-interface statistics like name, status, IP addresses, Rx/Tx bytes, packets, and errors.
pub fn render_network_widget<B: Backend, S: DashboardService + Send + Sync + 'static + ?Sized>(
    frame: &mut Frame<'_>,
    app: &App<S>,
    area: Rect,
) {
    let metrics = match &app.state.metrics {
        Some(m) => m,
        None => {
            let block = Block::default()
                .title("Network Interfaces (No Data)")
                .borders(Borders::ALL);
            frame.render_widget(block, area);
            return;
        }
    };

    // Correctly access network metrics (it's not an Option here)
    let network_metrics = &metrics.network;

    let header_cells = ["Interface", "RX Bytes", "TX Bytes"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().bold()));
    let header = Row::new(header_cells)
        .style(Style::default().blue()) // Example style
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> = network_metrics
        .interfaces
        .iter()
        .map(|iface: &NetworkInterface| {
            let name = &iface.name;
            let cells = vec![
                Cell::from(name.as_str()),
                Cell::from(iface.rx_bytes.to_string()),
                Cell::from(iface.tx_bytes.to_string()),
            ];
            let height = 1;
            Row::new(cells).height(height)
        })
        .collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Network interfaces"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .column_spacing(2);

    frame.render_widget(table, area);
}

/// Draws the network widget - used by tests and main rendering code
pub fn draw_network<B: Backend, S: DashboardService + Send + Sync + 'static + ?Sized>(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &App<S>,
) {
    render_network_widget::<B, S>(frame, app, area);
}

/// Render network statistics
pub fn render(f: &mut Frame, area: Rect, metrics: Option<&Metrics>) {
    let block = Block::default()
        .title("Network Statistics")
        .borders(Borders::ALL);
    
    // If no metrics, display empty message
    if metrics.is_none() {
        let empty_widget = Paragraph::new("No network statistics available")
            .block(block);
        f.render_widget(empty_widget, area);
        return;
    }
    
    let metrics = metrics.unwrap();
    
    // Create layout for network interfaces
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    // If no network interfaces, display message
    if metrics.network.interfaces.is_empty() {
        let no_networks = Paragraph::new("No network interfaces detected")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(no_networks, inner_area);
        return;
    }
    
    // Create headers for the table
    let header = Row::new(["Interface", "Rx", "Tx", "Total", "Status"])
        .style(Style::default().fg(Color::Yellow));
    
    // Create rows for each network interface
    let rows = metrics.network.interfaces.iter().map(|iface| {
        let rx_str = format_bytes(iface.rx_bytes);
        let tx_str = format_bytes(iface.tx_bytes);
        let total_str = format_bytes(iface.rx_bytes + iface.tx_bytes);
        let status = if iface.is_up { "Up" } else { "Down" };
        
        let status_style = if iface.is_up { 
            Style::default().fg(Color::Green) 
        } else { 
            Style::default().fg(Color::Red) 
        };
        
        let cells = vec![
            Cell::from(iface.name.clone()),
            Cell::from(rx_str),
            Cell::from(tx_str),
            Cell::from(total_str),
            Cell::from(status).style(status_style),
        ];
        
        Row::new(cells)
    });
    
    // Create and render the table
    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];
    
    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1);
    
    f.render_widget(table, inner_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppState};
    use dashboard_core::data::{Metrics, NetworkMetrics, CpuMetrics, MemoryMetrics, DiskMetrics, ProtocolData};
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        style::{Color, Style, Stylize},
        Terminal
    };
    use std::collections::HashMap;
    use chrono::Utc;
    use std::sync::Arc;
    use dashboard_core::data::DashboardData;
    use dashboard_core::service::MockDashboardService;
    use ratatui::widgets::ListState;

    // Helper function to create a default App instance
    fn create_test_app() -> App<MockDashboardService> {
        App::new(Arc::new(MockDashboardService::new()))
    }

    // Helper function to create basic Metrics with empty network interfaces
    fn create_metrics_empty_network() -> Metrics {
        Metrics {
            cpu: CpuMetrics { usage: 0.0, cores: vec![], temperature: None, load: [0.0, 0.0, 0.0] },
            memory: MemoryMetrics { total: 0, used: 0, available: 0, free: 0, swap_used: 0, swap_total: 0 },
            network: NetworkMetrics {
                interfaces: vec![], // Empty interfaces
                total_rx_bytes: 0,
                total_tx_bytes: 0,
                total_rx_packets: 0,
                total_tx_packets: 0,
            },
            disk: DiskMetrics { usage: HashMap::new(), total_reads: 0, total_writes: 0, read_bytes: 0, written_bytes: 0 },
            history: Default::default(),
        }
    }

    #[test]
    fn test_render_network_widget_no_data() {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = None; // Ensure no metrics data
        let area = Rect::new(0, 0, 40, 5);

        terminal.draw(|f| {
            render_network_widget::<TestBackend, _>(f, &app, area);
        }).unwrap();

        let expected = Buffer::with_lines(vec![
            "┌Network Interfaces (No Data)──────────┐",
            "│                                      │",
            "│                                      │",
            "│                                      │",
            "└──────────────────────────────────────┘",
        ]);
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_render_network_widget_empty_interfaces() {
        let backend = TestBackend::new(50, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = Some(create_metrics_empty_network());
        let area = Rect::new(0, 0, 50, 5);

        terminal.draw(|f| {
            render_network_widget::<TestBackend, _>(f, &app, area);
        }).unwrap();

        // Verify content without checking specific styling
        let buffer = terminal.backend().buffer();
        let rendered_content = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
        
        // Check for network table with headers
        assert!(rendered_content.contains("Network Interfaces"));
        assert!(rendered_content.contains("Interface"));
        assert!(rendered_content.contains("RX Bytes"));
        assert!(rendered_content.contains("TX Bytes"));
        
        // Verify that we have the table headers but no interface data
        // since the interfaces array is empty
        assert!(!rendered_content.contains("eth0"));
        assert!(!rendered_content.contains("lo"));
    }

    // Helper function to create metrics with network interfaces
    fn create_metrics_with_interfaces() -> Metrics {
        let mut metrics = create_metrics_empty_network();
        metrics.network.interfaces = vec![
            NetworkInterface {
                name: "eth0".to_string(),
                rx_bytes: 1024000,
                tx_bytes: 512000,
                rx_packets: 1000,
                tx_packets: 500,
                rx_errors: 1,
                tx_errors: 2,
                is_up: true,
            },
            NetworkInterface {
                name: "lo".to_string(),
                rx_bytes: 1024,
                tx_bytes: 1024,
                rx_packets: 10,
                tx_packets: 10,
                rx_errors: 0,
                tx_errors: 0,
                is_up: true,
            },
        ];
        metrics
    }

    #[test]
    fn test_render_network_widget_with_data() {
        let backend = TestBackend::new(50, 6);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = Some(create_metrics_with_interfaces());
        let area = Rect::new(0, 0, 50, 6);

        terminal.draw(|f| {
            render_network_widget::<TestBackend, _>(f, &app, area);
        }).unwrap();

        // Verify content without checking specific styling
        let buffer = terminal.backend().buffer();
        let rendered_content = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
        
        // Check for network interface table with headers
        assert!(rendered_content.contains("Network Interfaces"));
        assert!(rendered_content.contains("Interface"));
        assert!(rendered_content.contains("RX Bytes"));
        assert!(rendered_content.contains("TX Bytes"));
        
        // Check for specific interface data
        assert!(rendered_content.contains("eth0"));
        assert!(rendered_content.contains("lo"));
        assert!(rendered_content.contains("1024000")); // eth0 rx_bytes
        assert!(rendered_content.contains("512000"));  // eth0 tx_bytes
        assert!(rendered_content.contains("1024"));    // lo rx_bytes
    }

    #[test]
    fn test_draw_network_widget() {
        // Setup
        let backend = TestBackend::new(80, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        
        // Add sample data to app
        app.state.metrics = Some(create_metrics_with_interfaces());
        
        terminal.draw(|f| {
            let size = f.size();
            draw_network::<TestBackend, MockDashboardService>(f, size, &mut app);
        }).unwrap();
        
        // Verify content without checking specific styling
        let buffer = terminal.backend().buffer();
        let rendered_content = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
        
        // Check for network table and data
        assert!(rendered_content.contains("Network Interfaces"));
        assert!(rendered_content.contains("eth0"));
        assert!(rendered_content.contains("lo"));
        
        // Check for specific metrics values
        assert!(rendered_content.contains("1024000")); // eth0 rx_bytes
        assert!(rendered_content.contains("512000"));  // eth0 tx_bytes
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }
} 