use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Text},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Row, Table},
    Frame,
};

use dashboard_core::data::Metrics;

/// Widget for displaying system metrics
pub struct MetricsWidget<'a> {
    metrics: &'a Metrics,
    title: &'a str,
}

impl<'a> MetricsWidget<'a> {
    /// Create a new metrics widget
    pub fn new(metrics: &'a Metrics, title: &'a str) -> Self {
        Self { metrics, title }
    }
    
    /// Render the widget
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Create main block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        
        // Render main block
        f.render_widget(block.clone(), area);
        
        // Create inner layout
        let inner_area = block.inner(area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // CPU and Memory
                Constraint::Percentage(30), // Network
                Constraint::Percentage(40), // Charts
            ])
            .split(inner_area);
        
        // Create top row
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // CPU
                Constraint::Percentage(50), // Memory
            ])
            .split(chunks[0]);
        
        // Render CPU metrics
        self.render_cpu_metrics(f, top_chunks[0]);
        
        // Render memory metrics
        self.render_memory_metrics(f, top_chunks[1]);
        
        // Render network metrics
        self.render_network_metrics(f, chunks[1]);
        
        // Render charts
        self.render_charts(f, chunks[2]);
    }
    
    fn render_cpu_metrics<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Create block
        let block = Block::default()
            .borders(Borders::ALL)
            .title("CPU");
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Create content
        let inner_area = block.inner(area);
        
        let cpu_text = vec![
            Text::from(vec![
                Span::raw("Usage: "),
                Span::styled(
                    format!("{:.1}%", self.metrics.cpu.usage),
                    Style::default().fg(get_usage_color(self.metrics.cpu.usage)),
                ),
            ]),
            Text::from(vec![
                Span::raw("Cores: "),
                Span::raw(format!("{}", self.metrics.cpu.cores.len())),
            ]),
        ];
        
        let temp_text = if let Some(temp) = self.metrics.cpu.temperature {
            format!("Temperature: {:.1}°C", temp)
        } else {
            "Temperature: N/A".to_string()
        };
        
        cpu_text.iter().enumerate().for_each(|(i, text)| {
            let paragraph = Paragraph::new(text.clone())
                .style(Style::default());
            
            let y_offset = (i as u16) * 2;
            let paragraph_area = Rect::new(
                inner_area.x,
                inner_area.y + y_offset,
                inner_area.width,
                2,
            );
            
            f.render_widget(paragraph, paragraph_area);
        });
        
        // Add load averages
        let load_text = format!(
            "Load avg: {:.2}, {:.2}, {:.2}",
            self.metrics.cpu.load[0],
            self.metrics.cpu.load[1],
            self.metrics.cpu.load[2]
        );
        
        let paragraph = Paragraph::new(load_text)
            .style(Style::default());
        
        let paragraph_area = Rect::new(
            inner_area.x,
            inner_area.y + 4,
            inner_area.width,
            2,
        );
        
        f.render_widget(paragraph, paragraph_area);
        
        // Add temperature
        let paragraph = Paragraph::new(temp_text)
            .style(Style::default());
        
        let paragraph_area = Rect::new(
            inner_area.x,
            inner_area.y + 6,
            inner_area.width,
            2,
        );
        
        f.render_widget(paragraph, paragraph_area);
    }
    
    fn render_memory_metrics<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Create block
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Memory");
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Create content
        let inner_area = block.inner(area);
        
        let used_percentage = self.metrics.memory.used as f64 / self.metrics.memory.total as f64 * 100.0;
        let swap_percentage = if self.metrics.memory.swap_total > 0 {
            self.metrics.memory.swap_used as f64 / self.metrics.memory.swap_total as f64 * 100.0
        } else {
            0.0
        };
        
        let mem_text = vec![
            Text::from(vec![
                Span::raw("Usage: "),
                Span::styled(
                    format!("{:.1}%", used_percentage),
                    Style::default().fg(get_usage_color(used_percentage)),
                ),
            ]),
            Text::from(vec![
                Span::raw("Used: "),
                Span::raw(format!("{}", format_bytes(self.metrics.memory.used))),
                Span::raw(" / "),
                Span::raw(format!("{}", format_bytes(self.metrics.memory.total))),
            ]),
            Text::from(vec![
                Span::raw("Free: "),
                Span::raw(format!("{}", format_bytes(self.metrics.memory.free))),
            ]),
            Text::from(vec![
                Span::raw("Swap: "),
                Span::styled(
                    format!("{:.1}%", swap_percentage),
                    Style::default().fg(get_usage_color(swap_percentage)),
                ),
                Span::raw(format!(" ({}/{})",
                    format_bytes(self.metrics.memory.swap_used),
                    format_bytes(self.metrics.memory.swap_total)
                )),
            ]),
        ];
        
        mem_text.iter().enumerate().for_each(|(i, text)| {
            let paragraph = Paragraph::new(text.clone())
                .style(Style::default());
            
            let y_offset = (i as u16) * 2;
            let paragraph_area = Rect::new(
                inner_area.x,
                inner_area.y + y_offset,
                inner_area.width,
                2,
            );
            
            f.render_widget(paragraph, paragraph_area);
        });
    }
    
    fn render_network_metrics<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Create block
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Network");
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Create content
        let inner_area = block.inner(area);
        
        let net_text = vec![
            Text::from(vec![
                Span::raw("RX: "),
                Span::raw(format!("{}/s", format_bytes(self.metrics.network.rx_per_sec as u64))),
                Span::raw(" (total: "),
                Span::raw(format!("{})", format_bytes(self.metrics.network.rx_total))),
            ]),
            Text::from(vec![
                Span::raw("TX: "),
                Span::raw(format!("{}/s", format_bytes(self.metrics.network.tx_per_sec as u64))),
                Span::raw(" (total: "),
                Span::raw(format!("{})", format_bytes(self.metrics.network.tx_total))),
            ]),
            Text::from(vec![
                Span::raw("Interfaces: "),
                Span::raw(format!("{}", self.metrics.network.interfaces.len())),
            ]),
        ];
        
        net_text.iter().enumerate().for_each(|(i, text)| {
            let paragraph = Paragraph::new(text.clone())
                .style(Style::default());
            
            let y_offset = (i as u16) * 2;
            let paragraph_area = Rect::new(
                inner_area.x,
                inner_area.y + y_offset,
                inner_area.width,
                2,
            );
            
            f.render_widget(paragraph, paragraph_area);
        });
    }
    
    fn render_charts<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Create block
        let block = Block::default()
            .borders(Borders::ALL)
            .title("History");
        
        // Create layout for charts
        let inner_area = block.inner(area);
        let charts_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // CPU chart
                Constraint::Percentage(50), // Memory chart
            ])
            .split(inner_area);
        
        // Check if we have history data
        if self.metrics.history.timestamps.is_empty() {
            // No history data yet
            let message = Paragraph::new("Collecting history data...")
                .style(Style::default());
            
            f.render_widget(block, area);
            f.render_widget(message, inner_area);
            return;
        }
        
        // Prepare CPU history data
        let cpu_data: Vec<(f64, f64)> = self.metrics.history.timestamps
            .iter()
            .enumerate()
            .map(|(i, ts)| {
                let timestamp = ts.timestamp() as f64;
                let usage = self.metrics.history.cpu_usage[i];
                (timestamp, usage)
            })
            .collect();
        
        // Prepare memory history data
        let mem_data: Vec<(f64, f64)> = self.metrics.history.timestamps
            .iter()
            .enumerate()
            .map(|(i, ts)| {
                let timestamp = ts.timestamp() as f64;
                let usage = self.metrics.history.memory_usage[i];
                (timestamp, usage)
            })
            .collect();
        
        // Find min and max for time axis
        let min_time = self.metrics.history.timestamps.first()
            .map(|t| t.timestamp() as f64)
            .unwrap_or(0.0);
        
        let max_time = self.metrics.history.timestamps.last()
            .map(|t| t.timestamp() as f64)
            .unwrap_or(0.0);
        
        // Render CPU chart
        let cpu_dataset = Dataset::default()
            .name("CPU (%)")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&cpu_data);
        
        let cpu_chart = Chart::new(vec![cpu_dataset])
            .block(Block::default().title("CPU Usage").borders(Borders::ALL))
            .x_axis(Axis::default()
                .bounds([min_time, max_time])
                .labels(vec![
                    Span::styled("", Style::default().fg(Color::Gray)),
                    Span::styled("Time", Style::default().fg(Color::Gray)),
                ]))
            .y_axis(Axis::default()
                .bounds([0.0, 100.0])
                .labels(vec![
                    Span::styled("0%", Style::default().fg(Color::Gray)),
                    Span::styled("50%", Style::default().fg(Color::Gray)),
                    Span::styled("100%", Style::default().fg(Color::Gray)),
                ]));
        
        // Render memory chart
        let mem_dataset = Dataset::default()
            .name("Memory (%)")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&mem_data);
        
        let mem_chart = Chart::new(vec![mem_dataset])
            .block(Block::default().title("Memory Usage").borders(Borders::ALL))
            .x_axis(Axis::default()
                .bounds([min_time, max_time])
                .labels(vec![
                    Span::styled("", Style::default().fg(Color::Gray)),
                    Span::styled("Time", Style::default().fg(Color::Gray)),
                ]))
            .y_axis(Axis::default()
                .bounds([0.0, 100.0])
                .labels(vec![
                    Span::styled("0%", Style::default().fg(Color::Gray)),
                    Span::styled("50%", Style::default().fg(Color::Gray)),
                    Span::styled("100%", Style::default().fg(Color::Gray)),
                ]));
        
        // Render charts
        f.render_widget(block, area);
        f.render_widget(cpu_chart, charts_layout[0]);
        f.render_widget(mem_chart, charts_layout[1]);
    }
}

/// Get color based on usage percentage
fn get_usage_color(usage: f64) -> Color {
    if usage < 50.0 {
        Color::Green
    } else if usage < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Format bytes to human readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
} 