use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::{Span, Line as Spans},
    widgets::{Axis, Block, Borders, Chart as RatatuiChart, Dataset, GraphType, Widget},
};

use chrono::{DateTime, Utc};

/// Chart widget for displaying time-series data
pub struct ChartWidget<'a> {
    /// Data points [(time, value)]
    data: &'a [(DateTime<Utc>, f64)],
    
    /// Widget title
    title: &'a str,
    
    /// Chart type
    chart_type: ChartType,
    
    /// Y-axis label
    y_label: &'a str,
    
    /// X-axis range in seconds
    time_range: u64,
    
    /// Max data points to display
    max_points: usize,
}

/// Chart types
pub enum ChartType {
    /// Line chart
    Line,
    /// Bar chart
    Bar,
}

impl<'a> ChartWidget<'a> {
    /// Create a new chart widget
    pub fn new(data: &'a [(DateTime<Utc>, f64)], title: &'a str) -> Self {
        Self {
            data,
            title,
            chart_type: ChartType::Line,
            y_label: "",
            time_range: 300, // Default 5 minutes
            max_points: 100,
        }
    }
    
    /// Set the chart type
    pub fn chart_type(mut self, chart_type: ChartType) -> Self {
        self.chart_type = chart_type;
        self
    }
    
    /// Set the y-axis label
    pub fn y_label(mut self, label: &'a str) -> Self {
        self.y_label = label;
        self
    }
    
    /// Set the time range in seconds
    pub fn time_range(mut self, seconds: u64) -> Self {
        self.time_range = seconds;
        self
    }
    
    /// Set the maximum number of data points to display
    pub fn max_points(mut self, points: usize) -> Self {
        self.max_points = points;
        self
    }
}

impl<'a> Widget for ChartWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create base block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        
        // Render block
        block.clone().render(area, buf);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Handle empty data
        if self.data.is_empty() {
            let no_data = Spans::from(vec![
                Span::styled("No data available", Style::default().fg(Color::Gray))
            ]);
            
            // Calculate center position
            let x = inner_area.x + inner_area.width / 2 - 8;
            let y = inner_area.y + inner_area.height / 2;
            
            buf.set_line(x, y, &no_data, inner_area.width);
            return;
        }
        
        // Filter data by time range
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(self.time_range as i64);
        
        // Convert to chart data format
        let mut chart_data: Vec<(f64, f64)> = self.data
            .iter()
            .filter(|(time, _)| *time >= cutoff)
            .map(|(time, value)| {
                let time_diff = now.signed_duration_since(*time).num_seconds() as f64;
                (self.time_range as f64 - time_diff, *value)
            })
            .collect();
        
        // If there are too many points, sample them
        if chart_data.len() > self.max_points && self.max_points > 0 {
            chart_data = sample_data(&chart_data, self.max_points);
        }
        
        // Find min/max values for y-axis scaling
        let min_y = chart_data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
        let max_y = chart_data.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);
        
        // Ensure there's some range even with flat data
        let y_range = if (max_y - min_y).abs() < 0.001 {
            (max_y - 1.0, max_y + 1.0)
        } else {
            let padding = (max_y - min_y) * 0.1;
            (min_y - padding, max_y + padding)
        };
        
        // Create dataset
        let dataset = match self.chart_type {
            ChartType::Line => Dataset::default()
                .name(self.y_label)
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&chart_data),
                
            ChartType::Bar => Dataset::default()
                .name(self.y_label)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(Color::Cyan))
                .data(&chart_data),
        };
        
        // Create x-axis labels
        let x_labels = vec![
            Span::styled("now", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("-{}s", self.time_range / 2),
                Style::default().fg(Color::Gray)
            ),
            Span::styled(
                format!("-{}s", self.time_range),
                Style::default().fg(Color::Gray)
            ),
        ];
        
        // Create chart
        let chart = RatatuiChart::new(vec![dataset])
            .x_axis(
                Axis::default()
                    .title(Spans::from(vec![
                        Span::styled("Time", Style::default().fg(Color::Gray))
                    ]))
                    .bounds([0.0, self.time_range as f64])
                    .labels(x_labels)
            )
            .y_axis(
                Axis::default()
                    .title(Spans::from(vec![
                        Span::styled(self.y_label, Style::default().fg(Color::Gray))
                    ]))
                    .bounds([y_range.0, y_range.1])
                    .labels(vec![
                        Span::styled(format!("{:.1}", y_range.0), Style::default().fg(Color::Gray)),
                        Span::styled(format!("{:.1}", (y_range.0 + y_range.1) / 2.0), Style::default().fg(Color::Gray)),
                        Span::styled(format!("{:.1}", y_range.1), Style::default().fg(Color::Gray)),
                    ])
            );
        
        chart.render(inner_area, buf);
    }
}

/// Sample data to reduce the number of points
fn sample_data(data: &[(f64, f64)], max_points: usize) -> Vec<(f64, f64)> {
    if data.len() <= max_points {
        return data.to_vec();
    }
    
    let mut result = Vec::with_capacity(max_points);
    let step = data.len() as f64 / max_points as f64;
    
    for i in 0..max_points {
        let idx = (i as f64 * step) as usize;
        if idx < data.len() {
            result.push(data[idx]);
        }
    }
    
    result
} 