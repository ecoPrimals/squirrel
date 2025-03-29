use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::util; // For format_bytes
use dashboard_core::service::DashboardService;

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
        Line::from(format!(" Total: {}", util::format_bytes(memory.total))),
        Line::from(format!("  Used: {} ({:.1}%)", util::format_bytes(memory.used), 
                          if memory.total > 0 { (memory.used as f64 / memory.total as f64) * 100.0 } else { 0.0 })),
        Line::from(format!("  Free: {}", util::format_bytes(memory.free))),
        Line::from(format!(" Avail: {}", util::format_bytes(memory.available))),
        Line::from(""),
        Line::from(Span::styled("Swap:", Style::default().bold())),
        Line::from(format!(" Total: {}", util::format_bytes(memory.swap_total))),
        Line::from(format!("  Used: {} ({:.1}%)", util::format_bytes(memory.swap_used),
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
                                            util::format_bytes(usage.used),
                                            util::format_bytes(usage.total),
                                            usage.used_percentage
                                           )));
        }
    }
    disk_text.push(Line::from(""));
    disk_text.push(Line::from(Span::styled("Total I/O:", Style::default().bold())));
    disk_text.push(Line::from(format!(" Reads: {} ({})", disk.total_reads, util::format_bytes(disk.read_bytes))));
    disk_text.push(Line::from(format!(" Writes: {} ({})", disk.total_writes, util::format_bytes(disk.written_bytes))));

    let disk_paragraph = Paragraph::new(disk_text)
        .block(Block::default().borders(Borders::ALL).title("Disk Details"));
    frame.render_widget(disk_paragraph, chunks[2]);
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
        // Define a larger area to accommodate the 3 columns
        let backend = TestBackend::new(120, 12); // Width = 120, Height = 12
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = Some(create_basic_metrics());
        let area = Rect::new(0, 0, 120, 12);

        terminal.draw(|f| {
            render_system_widgets::<TestBackend, _>(f, &app, area);
        }).unwrap();

        // Expected buffer needs to be constructed carefully for 3 columns
        // This is an approximate representation. Actual check relies on assert_buffer.
        let expected_lines = vec![
            "┌CPU Details───────────────────────────┐┌Memory Details─────────────────────────┐┌Disk Details──────────────────────────┐",
            "│Usage: 25.5 %                         ││RAM:                                   ││Usage per Mount:                      │",
            "│                                      ││ Total: 16.00 GiB                      ││ /: 50.00 GiB / 100.00 GiB (50.0%)   │",
            "│Per Core:                             ││  Used: 8.00 GiB (50.0%)             ││                                      │",
            "│10.0% | 20.0% | 30.0% | 40.0%         ││  Free: 8.00 GiB                       ││Total I/O:                            │",
            "│                                      ││ Avail: 7.00 GiB                       ││ Reads: 10000 (1.00 GiB)              │",
            "│Load Avg:                             ││                                       ││ Writes: 5000 (512.00 MiB)          │",
            "│1m: 1.10, 5m: 1.20, 15m: 1.30         ││Swap:                                  ││                                      │",
            "│                                      ││ Total: 4.00 GiB                       ││                                      │",
            "│Temperature:                          ││                                       ││                                      │",
            "│55.2 °C                               ││                                       ││                                      │",
            "└──────────────────────────────────────┘└───────────────────────────────────────┘└──────────────────────────────────────┘",
        ];
        let mut expected = Buffer::with_lines(expected_lines);
        
        // Add styles (Example for CPU Usage)
        expected.set_style(Rect::new(1, 1, 13, 1), Style::default().fg(Color::Green).bold()); // CPU Usage: green bold
        // ... other styles would be added here for a precise check

        terminal.backend().assert_buffer(&expected);
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
        let backend = TestBackend::new(120, 14); // Adjusted height for more disk mounts
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.metrics = Some(create_edge_case_metrics());
        let area = Rect::new(0, 0, 120, 14);

        terminal.draw(|f| {
            render_system_widgets::<TestBackend, _>(f, &app, area);
        }).unwrap();

        let expected_lines = vec![
            "┌CPU Details───────────────────────────┐┌Memory Details─────────────────────────┐┌Disk Details──────────────────────────┐",
            "│Usage: 5.0 %                          ││RAM:                                   ││Usage per Mount:                      │", // CPU Usage
            "│                                      ││ Total: 0 B                            ││ /: 10.00 GiB / 100.00 GiB (10.0%)   │", // Zero RAM, Disk /
            "│Per Core:                             ││  Used: 0 B (0.0%)                   ││ /data: 400.00 GiB / 500.00 GiB (80.0%│", // Zero RAM %, Disk /data
            "│2.0% | 8.0%                           ││  Free: 0 B                            ││ /home: 150.00 GiB / 200.00 GiB (75.0%│", // Disk /home (sorted)
            "│                                      ││ Avail: 0 B                            ││                                      │", // Zero RAM Avail
            "│Load Avg:                             ││                                       ││Total I/O:                            │",
            "│1m: 0.10, 5m: 0.20, 15m: 0.30         ││Swap:                                  ││ Reads: 50 (10.00 KiB)                │", // Reads
            "│                                      ││ Total: 0 B                            ││ Writes: 25 (5.00 KiB)                │", // Zero Swap, Writes
            "│Temperature:                          ││  Used: 0 B (0.0%)                   ││                                      │", // Temp N/A, Zero Swap %
            "│N/A                                   ││                                       ││                                      │",
            "│                                      ││                                       ││                                      │",
            "│                                      ││                                       ││                                      │",
            "└──────────────────────────────────────┘└───────────────────────────────────────┘└──────────────────────────────────────┘",
        ];
        let expected = Buffer::with_lines(expected_lines);
        // NOTE: Styles are omitted for brevity, but would be checked in a real scenario

        terminal.backend().assert_buffer(&expected);
    }
} 