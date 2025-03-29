// crates/ui-terminal/src/widgets/network.rs
// Placeholder for NetworkWidget implementation

use ratatui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use crate::app::App; // App is no longer generic
use dashboard_core::data::NetworkInterface; // Assuming this is the correct type
use dashboard_core::service::DashboardService;

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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Network Interfaces"),
        )
        .column_spacing(1);

    frame.render_widget(table, area);
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
        let backend = TestBackend::new(50, 5); // Adjusted width slightly
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = Some(create_metrics_empty_network()); // Metrics with empty network interfaces
        let area = Rect::new(0, 0, 50, 5);

        terminal.draw(|f| {
            render_network_widget::<TestBackend, _>(f, &app, area);
        }).unwrap();

        let mut expected = Buffer::with_lines(vec![
            "┌Network Interfaces───────────────────────────────┐",
            "│Interface         RX Bytes          TX Bytes     │", // Header row
            "│                                                 │",
            "│                                                 │",
            "└─────────────────────────────────────────────────┘",
        ]);
        // Style for the header row
        expected.set_style(Rect::new(1, 1, 48, 1), Style::default().blue().bold());

        terminal.backend().assert_buffer(&expected);
    }

    // Helper function to create metrics with sample network interface data
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
                ip_addresses: vec!["192.168.1.100".to_string()],
                mac_address: Some("00:11:22:33:44:55".to_string()),
                is_up: true,
                is_loopback: false,
            },
            NetworkInterface {
                name: "lo".to_string(),
                rx_bytes: 1024,
                tx_bytes: 1024,
                rx_packets: 10,
                tx_packets: 10,
                rx_errors: 0,
                tx_errors: 0,
                ip_addresses: vec!["127.0.0.1".to_string()],
                mac_address: None,
                is_up: true,
                is_loopback: true,
            },
        ];
        metrics
    }

    #[test]
    fn test_render_network_widget_with_data() {
        let backend = TestBackend::new(50, 6); // Increased height for data rows
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = Some(create_metrics_with_interfaces());
        let area = Rect::new(0, 0, 50, 6);

        terminal.draw(|f| {
            render_network_widget::<TestBackend, _>(f, &app, area);
        }).unwrap();

        let mut expected = Buffer::with_lines(vec![
            "┌Network Interfaces───────────────────────────────┐",
            "│Interface         RX Bytes          TX Bytes     │", // Header row
            "│                                                 │", // Bottom margin space
            "│eth0              1024000           512000       │", // Data row 1
            "│lo                1024              1024         │", // Data row 2
            "└─────────────────────────────────────────────────┘",
        ]);
        // Style for the header row
        expected.set_style(Rect::new(1, 1, 48, 1), Style::default().blue().bold());

        terminal.backend().assert_buffer(&expected);
    }

} 