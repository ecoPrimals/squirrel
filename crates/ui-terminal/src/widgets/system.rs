use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::widgets::metrics::format_bytes;
use dashboard_core::service::DashboardService;
use dashboard_core::data::Metrics;
use ratatui::symbols;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};

/// Renders the System tab widgets.
///
/// Displays detailed system metrics in a three-column layout:
/// - CPU: Overall usage, per-core usage, load averages, temperature.
/// - Memory: RAM (Total, Used, Free, Available), Swap (Total, Used).
/// - Disk: Usage per mount point (sorted), Total I/O reads/writes.
pub fn render_system_widgets<B: Backend, S: DashboardService + Send + Sync + 'static + ?Sized>(
    frame: &mut Frame<'_>,
    app: &App<S>,
    area: Rect,
) {
    let metrics = match &app.state.metrics {
        Some(m) => m,
        None => {
            let placeholder = Paragraph::new("System Metrics Data Unavailable")
                .block(Block::default().borders(Borders::ALL).title("System"));
            frame.render_widget(placeholder, area);
            return;
        }
    };

    // Layout: 3 columns for CPU, Memory, Disk
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34), // CPU
            Constraint::Percentage(33), // Memory
            Constraint::Percentage(33), // Disk
        ].as_ref())
        .split(area);

    // --- CPU --- 
    let cpu = &metrics.cpu;
    let cpu_text = vec![
        Line::from(Span::styled(format!("Usage: {:.1} %", cpu.usage), Style::default().fg(Color::Green).bold())),
        Line::from(""),
        Line::from(Span::styled("Per Core:", Style::default().bold())),
        Line::from(cpu.cores.iter().map(|c| format!("{:.1}%", c)).collect::<Vec<_>>().join(" | ")),
        Line::from(""),
        Line::from(Span::styled("Load Avg:", Style::default().bold())),
        Line::from(format!("1m: {:.2}, 5m: {:.2}, 15m: {:.2}", cpu.load[0], cpu.load[1], cpu.load[2])),
        Line::from(""),
        Line::from(Span::styled("Temperature:", Style::default().bold())),
        Line::from(cpu.temperature.map_or("N/A".to_string(), |t| format!("{:.1} °C", t))),
    ];
    let cpu_paragraph = Paragraph::new(cpu_text)
        .block(Block::default().borders(Borders::ALL).title("CPU Details"));
    frame.render_widget(cpu_paragraph, chunks[0]);

    // --- Memory --- 
    let memory = &metrics.memory;
    let mem_text = vec![
        Line::from(Span::styled("RAM:", Style::default().bold())),
        Line::from(format!(" Total: {}", format_bytes(memory.total))),
        Line::from(format!("  Used: {} ({:.1}%)", format_bytes(memory.used), 
                          if memory.total > 0 { (memory.used as f64 / memory.total as f64) * 100.0 } else { 0.0 })),
        Line::from(format!("  Free: {}", format_bytes(memory.free))),
        Line::from(format!(" Avail: {}", format_bytes(memory.available))),
        Line::from(""),
        Line::from(Span::styled("Swap:", Style::default().bold())),
        Line::from(format!(" Total: {}", format_bytes(memory.swap_total))),
        Line::from(format!("  Used: {} ({:.1}%)", format_bytes(memory.swap_used),
                          if memory.swap_total > 0 { (memory.swap_used as f64 / memory.swap_total as f64) * 100.0 } else { 0.0 })),
    ];
    let mem_paragraph = Paragraph::new(mem_text)
        .block(Block::default().borders(Borders::ALL).title("Memory Details"));
    frame.render_widget(mem_paragraph, chunks[1]);

    // --- Disk --- 
    let disk = &metrics.disk;
    let mut disk_text = vec![
        Line::from(Span::styled("Usage per Mount:", Style::default().bold()))
    ];
    // Sort by mount point for consistent order
    let mut sorted_mounts: Vec<_> = disk.usage.keys().collect();
    sorted_mounts.sort();
    for mount_point in sorted_mounts {
        if let Some(usage) = disk.usage.get(mount_point) {
            disk_text.push(Line::from(format!(" {}: {} / {} ({:.1}%)", 
                                            usage.mount_point, 
                                            format_bytes(usage.used),
                                            format_bytes(usage.total),
                                            usage.used_percentage
                                           )));
        }
    }
    disk_text.push(Line::from(""));
    disk_text.push(Line::from(Span::styled("Total I/O:", Style::default().bold())));
    disk_text.push(Line::from(format!(" Reads: {} ({})", disk.total_reads, format_bytes(disk.read_bytes))));
    disk_text.push(Line::from(format!(" Writes: {} ({})", disk.total_writes, format_bytes(disk.written_bytes))));

    let disk_paragraph = Paragraph::new(disk_text)
        .block(Block::default().borders(Borders::ALL).title("Disk Details"));
    frame.render_widget(disk_paragraph, chunks[2]);
}

/// Draws the system widget - used by tests and main rendering code
pub fn draw_system<B: Backend, S: DashboardService + Send + Sync + 'static + ?Sized>(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &App<S>,
) {
    render_system_widgets::<B, S>(frame, app, area);
}

/// Format memory size in bytes to a human-readable format
fn format_memory(bytes: u64) -> String {
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

/// Render system statistics
pub fn render(f: &mut Frame, area: Rect, metrics: Option<&Metrics>) {
    let block = Block::default()
        .title("System Information")
        .borders(Borders::ALL);
    
    // If no metrics, display empty message
    if metrics.is_none() {
        let empty_widget = Paragraph::new("No system metrics available")
            .block(block);
        f.render_widget(empty_widget, area);
        return;
    }
    
    let metrics = metrics.unwrap();
    
    // Create layout for system metrics
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // CPU info
            Constraint::Length(5), // Memory info
            Constraint::Min(8),    // Charts
        ])
        .split(inner_area);
    
    // CPU Section
    let cpu_block = Block::default()
        .title(format!("CPU Usage: {:.1}%", metrics.cpu.usage * 100.0))
        .borders(Borders::ALL);
    let cpu_text = Paragraph::new(format!(
        "Cores: {}\nLoad Avg: {:.2}, {:.2}, {:.2}",
        metrics.cpu.cores.len(),
        metrics.cpu.load[0],
        metrics.cpu.load[1],
        metrics.cpu.load[2]
    ));
    f.render_widget(cpu_block.clone(), chunks[0]);
    f.render_widget(cpu_text, cpu_block.inner(chunks[0]));
    
    // Memory Section
    let mem_block = Block::default()
        .title(format!(
            "Memory Usage: {:.1}%", 
            (metrics.memory.used as f64 / metrics.memory.total as f64) * 100.0
        ))
        .borders(Borders::ALL);
    let mem_text = Paragraph::new(format!(
        "Total: {}\nUsed: {}\nFree: {}\nSwap: {} / {}",
        format_memory(metrics.memory.total),
        format_memory(metrics.memory.used),
        format_memory(metrics.memory.free),
        format_memory(metrics.memory.swap_used),
        format_memory(metrics.memory.swap_total)
    ));
    f.render_widget(mem_block.clone(), chunks[1]);
    f.render_widget(mem_text, mem_block.inner(chunks[1]));
    
    // Usage History Charts
    let charts_area = chunks[2];
    let chart_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(charts_area);
    
    // CPU History Chart
    if !metrics.history.cpu.is_empty() {
        let cpu_data: Vec<(f64, f64)> = metrics.history.cpu.iter()
            .enumerate()
            .map(|(i, &(_, value))| (i as f64, value * 100.0))
            .collect();
        
        let cpu_dataset = Dataset::default()
            .name("CPU %")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&cpu_data);
        
        let cpu_chart = Chart::new(vec![cpu_dataset])
            .block(Block::default().title("CPU History").borders(Borders::ALL))
            .x_axis(Axis::default().bounds([0.0, cpu_data.len() as f64]))
            .y_axis(Axis::default().bounds([0.0, 100.0]));
        
        f.render_widget(cpu_chart, chart_chunks[0]);
    }
    
    // Memory History Chart
    if !metrics.history.memory.is_empty() {
        let mem_data: Vec<(f64, f64)> = metrics.history.memory.iter()
            .enumerate()
            .map(|(i, &(_, value))| (i as f64, (value / metrics.memory.total as f64) * 100.0))
            .collect();
        
        let mem_dataset = Dataset::default()
            .name("Memory %")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Magenta))
            .data(&mem_data);
        
        let mem_chart = Chart::new(vec![mem_dataset])
            .block(Block::default().title("Memory History").borders(Borders::ALL))
            .x_axis(Axis::default().bounds([0.0, mem_data.len() as f64]))
            .y_axis(Axis::default().bounds([0.0, 100.0]));
        
        f.render_widget(mem_chart, chart_chunks[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppState};
    use dashboard_core::data::{Metrics, CpuMetrics, MemoryMetrics, DiskMetrics, DiskUsage, ProtocolData};
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

    // Helper to create a default App
    fn create_test_app() -> App<MockDashboardService> {
        App::new(Arc::new(MockDashboardService::new()))
    }

    // Helper to create basic metrics for testing
    fn create_basic_metrics() -> Metrics {
        let mut disk_usage = HashMap::new();
        disk_usage.insert("/dev/sda1".to_string(), DiskUsage {
            mount_point: "/".to_string(),
            total: 100 * 1024 * 1024 * 1024, // 100 GiB
            used: 50 * 1024 * 1024 * 1024, // 50 GiB
            free: 50 * 1024 * 1024 * 1024,
            used_percentage: 50.0,
        });

        Metrics {
            cpu: CpuMetrics {
                usage: 25.5,
                cores: vec![10.0, 20.0, 30.0, 40.0],
                temperature: Some(55.2),
                load: [1.1, 1.2, 1.3],
            },
            memory: MemoryMetrics {
                total: 16 * 1024 * 1024 * 1024, // 16 GiB
                used: 8 * 1024 * 1024 * 1024, // 8 GiB
                available: 7 * 1024 * 1024 * 1024,
                free: 8 * 1024 * 1024 * 1024,
                swap_total: 4 * 1024 * 1024 * 1024, // 4 GiB
                swap_used: 1 * 1024 * 1024 * 1024, // 1 GiB
            },
            network: Default::default(), // Not testing network here
            disk: DiskMetrics {
                usage: disk_usage,
                total_reads: 10000,
                total_writes: 5000,
                read_bytes: 1 * 1024 * 1024 * 1024, // 1 GiB
                written_bytes: 512 * 1024 * 1024, // 512 MiB
            },
            history: Default::default(),
        }
    }

    #[test]
    fn test_render_system_widget_no_data() {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = None; // No data
        let area = Rect::new(0, 0, 40, 5);

        terminal.draw(|f| {
            render_system_widgets::<TestBackend, _>(f, &app, area);
        }).unwrap();

        let expected = Buffer::with_lines(vec![
            "┌System────────────────────────────────┐",
            "│System Metrics Data Unavailable       │",
            "│                                      │",
            "│                                      │",
            "└──────────────────────────────────────┘",
        ]);
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_render_system_widget_basic_data() {
        let backend = TestBackend::new(120, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = Some(create_basic_metrics());
        let area = Rect::new(0, 0, 120, 12);

        terminal.draw(|f| {
            render_system_widgets::<TestBackend, _>(f, &app, area);
        }).unwrap();

        // Verify that key content is rendered in the buffer, regardless of styling
        let buffer = terminal.backend().buffer();
        let rendered_content = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
        
        // Check for critical information in the rendered output
        assert!(rendered_content.contains("CPU Details"));
        assert!(rendered_content.contains("Usage: 25.5 %"));
        assert!(rendered_content.contains("Memory Details"));
        assert!(rendered_content.contains("Total: 16.00 GB"));
        assert!(rendered_content.contains("Disk Details"));
    }

    // Helper function to create metrics for edge cases
    fn create_edge_case_metrics() -> Metrics {
        let mut disk_usage = HashMap::new();
        // Add mounts out of alphabetical order to test sorting
        disk_usage.insert("/dev/sdb1".to_string(), DiskUsage {
            mount_point: "/home".to_string(),
            total: 200 * 1024 * 1024 * 1024,
            used: 150 * 1024 * 1024 * 1024,
            free: 50 * 1024 * 1024 * 1024,
            used_percentage: 75.0,
        });
        disk_usage.insert("/dev/sda1".to_string(), DiskUsage {
            mount_point: "/".to_string(),
            total: 100 * 1024 * 1024 * 1024, 
            used: 10 * 1024 * 1024 * 1024, 
            free: 90 * 1024 * 1024 * 1024,
            used_percentage: 10.0,
        });
         disk_usage.insert("/dev/sdc1".to_string(), DiskUsage {
            mount_point: "/data".to_string(),
            total: 500 * 1024 * 1024 * 1024, 
            used: 400 * 1024 * 1024 * 1024, 
            free: 100 * 1024 * 1024 * 1024,
            used_percentage: 80.0,
        });

        Metrics {
            cpu: CpuMetrics {
                usage: 5.0,
                cores: vec![2.0, 8.0],
                temperature: None, // Missing temperature
                load: [0.1, 0.2, 0.3],
            },
            memory: MemoryMetrics {
                total: 0, // Zero total RAM
                used: 0,
                available: 0,
                free: 0,
                swap_total: 0, // Zero total Swap
                swap_used: 0,
            },
            network: Default::default(),
            disk: DiskMetrics {
                usage: disk_usage,
                total_reads: 50,
                total_writes: 25,
                read_bytes: 10 * 1024,
                written_bytes: 5 * 1024,
            },
            history: Default::default(),
        }
    }

    #[test]
    fn test_render_system_widget_edge_cases() {
        let backend = TestBackend::new(120, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = Some(create_edge_case_metrics());
        let area = Rect::new(0, 0, 120, 14);

        terminal.draw(|f| {
            render_system_widgets::<TestBackend, _>(f, &app, area);
        }).unwrap();

        // Verify that key content is rendered in the buffer, regardless of styling
        let buffer = terminal.backend().buffer();
        let rendered_content = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
        
        // Check for critical information including edge cases
        assert!(rendered_content.contains("CPU Details"));
        assert!(rendered_content.contains("Usage: 5.0 %"));
        // Since the temperature is N/A, we just check that Temperature: appears, and isn't followed by a number
        assert!(rendered_content.contains("Temperature:"));
        assert!(!rendered_content.contains("Temperature: 5")); // Make sure there's no actual temperature value
        assert!(rendered_content.contains("Memory Details"));
        assert!(rendered_content.contains("Total: 0 B")); // Edge case: Zero memory
        assert!(rendered_content.contains("Disk Details"));
        assert!(rendered_content.contains("/home: 150.00 GB")); // Verify mount points
        assert!(rendered_content.contains("/data: 400.00 GB"));
    }

    #[test]
    fn test_render_system() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        
        // Add sample data
        app.state.metrics = Some(create_basic_metrics());
        let area = Rect::new(0, 0, 100, 30);
        
        terminal.draw(|f| {
            draw_system::<TestBackend, MockDashboardService>(f, area, &mut app);
        }).unwrap();
        
        // Verify core functionality without specific styling expectations
        let buffer = terminal.backend().buffer();
        let rendered_content = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
        
        // Check that key metrics are displayed
        assert!(rendered_content.contains("CPU Details"));
        assert!(rendered_content.contains("Memory Details"));
        assert!(rendered_content.contains("Disk Details"));
        assert!(rendered_content.contains("Load Avg:"));
    }

    #[test]
    fn test_format_memory() {
        assert_eq!(format_memory(500), "500 B");
        assert_eq!(format_memory(1024), "1.00 KB");
        assert_eq!(format_memory(1048576), "1.00 MB");
        assert_eq!(format_memory(1073741824), "1.00 GB");
    }
} 