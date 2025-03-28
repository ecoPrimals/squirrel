use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, Gauge},
    Frame,
};

use dashboard_core::data::Metrics;
use crate::util::format_bytes;

/// Widget for displaying system metrics
pub struct MetricsWidget<'a> {
    /// System metrics to display
    metrics: &'a Metrics,
    /// Widget title
    title: &'a str,
}

impl<'a> MetricsWidget<'a> {
    /// Create a new metrics widget
    pub fn new(metrics: &'a Metrics, title: &'a str) -> Self {
        Self { metrics, title }
    }
    
    /// Render the widget
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create base block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),  // CPU & Memory
                Constraint::Length(3),  // Disk
                Constraint::Min(0),     // Other metrics
            ])
            .split(inner_area);
        
        // Render CPU & Memory
        self.render_cpu_memory(f, chunks[0]);
        
        // Render Disk
        self.render_disk(f, chunks[1]);
        
        // Render other metrics
        self.render_other_metrics(f, chunks[2]);
    }
    
    /// Render CPU and Memory gauges
    fn render_cpu_memory(&self, f: &mut Frame, area: Rect) {
        // Split area into two for CPU and Memory
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);
        
        // Render CPU gauge
        self.render_cpu_gauge(f, chunks[0]);
        
        // Render Memory gauge
        self.render_memory_gauge(f, chunks[1]);
    }
    
    /// Render CPU gauge
    fn render_cpu_gauge(&self, f: &mut Frame, area: Rect) {
        let cpu_usage = self.metrics.cpu.usage;
        
        // Get color based on CPU usage
        let color = if cpu_usage > 90.0 {
            Color::Red
        } else if cpu_usage > 70.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        
        // Create gauge
        let gauge = Gauge::default()
            .block(Block::default().title(format!("CPU Usage: {:.1}%", cpu_usage)))
            .gauge_style(Style::default().fg(color))
            .percent(cpu_usage as u16);
        
        f.render_widget(gauge, area);
    }
    
    /// Render Memory gauge
    fn render_memory_gauge(&self, f: &mut Frame, area: Rect) {
        let memory_used = self.metrics.memory.used;
        let memory_total = self.metrics.memory.total;
        let memory_percentage = if memory_total > 0 {
            (memory_used as f64 / memory_total as f64 * 100.0) as u16
        } else {
            0
        };
        
        // Get color based on memory usage
        let color = if memory_percentage > 90 {
            Color::Red
        } else if memory_percentage > 70 {
            Color::Yellow
        } else {
            Color::Green
        };
        
        // Create gauge
        let memory_label = format!(
            "Memory: {}/{} ({:.1}%)",
            format_bytes(memory_used),
            format_bytes(memory_total),
            memory_percentage
        );
        
        let gauge = Gauge::default()
            .block(Block::default().title(memory_label))
            .gauge_style(Style::default().fg(color))
            .percent(memory_percentage);
        
        f.render_widget(gauge, area);
    }
    
    /// Render Disk usage
    fn render_disk(&self, f: &mut Frame, area: Rect) {
        // Create disk usage table
        let mut rows = Vec::new();
        
        // Add disk usage for each mount point
        for (mount, disk_info) in &self.metrics.disk.usage {
            let used_gb = disk_info.used as f64 / 1024.0 / 1024.0 / 1024.0;
            let total_gb = disk_info.total as f64 / 1024.0 / 1024.0 / 1024.0;
            let percent = disk_info.used_percentage;
            
            // Format values
            let mount_text = if mount.len() > 20 {
                format!("{}...", &mount[..17])
            } else {
                mount.clone()
            };
            
            let usage_text = format!("{:.1} / {:.1} GB", used_gb, total_gb);
            let percent_text = format!("{:.1}%", percent);
            
            // Create row
            rows.push(Row::new(vec![
                Cell::from(mount_text),
                Cell::from(usage_text),
                Cell::from(percent_text),
            ]));
        }
        
        // Add header
        let header = Row::new(vec![
            Cell::from(Span::styled("Mount", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("Usage", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("Percent", Style::default().add_modifier(Modifier::BOLD))),
        ]);
        
        // Create table with header
        let header_and_rows = std::iter::once(header).chain(rows).collect::<Vec<_>>();
        
        // Create table
        let table = Table::new(
            header_and_rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
            ]
        )
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Disk Usage"))
            .widths(&[
                Constraint::Percentage(40),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
            ])
            .column_spacing(1);
        
        // Render table
        f.render_widget(table, area);
    }
    
    /// Render other system metrics
    fn render_other_metrics(&self, f: &mut Frame, area: Rect) {
        // Create table rows
        let mut rows = Vec::new();
        
        // Add load averages
        rows.push(Row::new(vec![
            Cell::from("Load Average (1m)"),
            Cell::from(format!("{:.2}", self.metrics.cpu.load[0])),
        ]));
        
        rows.push(Row::new(vec![
            Cell::from("Load Average (5m)"),
            Cell::from(format!("{:.2}", self.metrics.cpu.load[1])),
        ]));
        
        rows.push(Row::new(vec![
            Cell::from("Load Average (15m)"),
            Cell::from(format!("{:.2}", self.metrics.cpu.load[2])),
        ]));
        
        // Add network I/O
        rows.push(Row::new(vec![
            Cell::from("Network RX"),
            Cell::from(format!("{}", format_bytes(self.metrics.network.total_rx_bytes))),
        ]));
        
        rows.push(Row::new(vec![
            Cell::from("Network TX"),
            Cell::from(format!("{}", format_bytes(self.metrics.network.total_tx_bytes))),
        ]));
        
        // Add number of CPU cores if available
        if !self.metrics.cpu.cores.is_empty() {
            rows.push(Row::new(vec![
                Cell::from("CPU Cores"),
                Cell::from(format!("{}", self.metrics.cpu.cores.len())),
            ]));
        }
        
        // Create table
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
        )
            .block(Block::default().title("System Information"))
            .widths(&[
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .column_spacing(1);
        
        f.render_widget(table, area);
    }
} 