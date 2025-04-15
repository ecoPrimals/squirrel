// crates/ui-terminal/src/widgets/metrics.rs
// Implementation for MetricsWidget

use ratatui::{
    prelude::{Backend, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use dashboard_core::service::DashboardService;
use crate::app::App;

/// Helper function to format bytes to a human-readable format (KB, MB, GB)
pub fn format_bytes(bytes: u64) -> String {
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

pub fn render<B: Backend, S: DashboardService + Send + Sync + 'static + ?Sized>(
    frame: &mut Frame<'_>,
    app: &App<S>,
    area: Rect
) {
    let block = Block::default().borders(Borders::ALL).title("Key Metrics");

    let metrics_data = match &app.state.metrics {
        Some(metrics) => {
            // Extract relevant metrics
            let cpu_line = Line::from(vec![
                Span::styled("CPU Usage:", Style::default().fg(Color::Cyan)),
                Span::raw(format!(" {:.1}%", metrics.cpu.usage)),
            ]);

            let mem_line = Line::from(vec![
                Span::styled("Memory:   ", Style::default().fg(Color::Cyan)),
                Span::raw(format!(" {} / {} ({:.1}%)",
                    format_bytes(metrics.memory.used),
                    format_bytes(metrics.memory.total),
                    if metrics.memory.total > 0 {
                        (metrics.memory.used as f64 / metrics.memory.total as f64) * 100.0
                    } else {
                        0.0
                    }
                )),
            ]);
            
            // TODO: Add more metrics as needed (e.g., Disk IO, Network totals)
            // let disk_line = Line::from(vec![...]);
            // let network_line = Line::from(vec![...]);

            vec![cpu_line, mem_line]
            // vec![cpu_line, mem_line, disk_line, network_line]
        }
        None => {
            vec![Line::from(Span::styled("No metrics data available.", Style::default().fg(Color::DarkGray)))]
        }
    };

    let paragraph = Paragraph::new(metrics_data)
        .block(block);

    frame.render_widget(paragraph, area);
} 