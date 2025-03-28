use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};
use chrono::{DateTime, Utc, Duration};
use dashboard_core::data::{MetricsHistory as DashboardMetricsHistory};

/// Chart types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    /// Line chart
    Line,
    /// Scatter plot
    Scatter,
}

/// Network data type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkDataType {
    /// Receive (RX)
    Rx,
    /// Transmit (TX)
    Tx,
}

/// Chart widget for visualizing time series data
pub struct ChartWidget<'a> {
    /// Data points to display
    data: &'a [(DateTime<Utc>, f64)],
    /// Network data points (optional, used for network charts)
    network_data: Option<(&'a [(DateTime<Utc>, (u64, u64))], NetworkDataType)>,
    /// Chart title
    title: &'a str,
    /// Chart type
    chart_type: ChartType,
    /// Y-axis label
    y_label: &'a str,
    /// Time range in seconds to display
    time_range: u64,
    /// Minimum y value
    min_y: Option<f64>,
    /// Maximum y value
    max_y: Option<f64>,
}

impl<'a> ChartWidget<'a> {
    /// Create a new chart widget
    pub fn new(data: &'a [(DateTime<Utc>, f64)], title: &'a str) -> Self {
        Self {
            data,
            network_data: None,
            title,
            chart_type: ChartType::Line,
            y_label: "",
            time_range: 300, // Default 5 minutes
            min_y: None,
            max_y: None,
        }
    }
    
    /// Create a new chart widget for network data
    pub fn new_network(data: &'a [(DateTime<Utc>, (u64, u64))], data_type: NetworkDataType, title: &'a str) -> Self {
        Self {
            data: &[], // Empty standard data
            network_data: Some((data, data_type)),
            title,
            chart_type: ChartType::Line,
            y_label: match data_type {
                NetworkDataType::Rx => "RX Bytes",
                NetworkDataType::Tx => "TX Bytes",
            },
            time_range: 300, // Default 5 minutes
            min_y: None,
            max_y: None,
        }
    }
    
    /// Create a new chart widget from dashboard metrics history
    pub fn from_dashboard_cpu(history: &'a DashboardMetricsHistory, title: &'a str) -> Self {
        Self::new(&history.cpu, title)
    }
    
    /// Create a new chart widget from dashboard metrics history for memory
    pub fn from_dashboard_memory(history: &'a DashboardMetricsHistory, title: &'a str) -> Self {
        Self::new(&history.memory, title)
    }
    
    /// Create a new chart widget from dashboard metrics history for network
    pub fn from_dashboard_network(
        history: &'a DashboardMetricsHistory, 
        data_type: NetworkDataType,
        title: &'a str
    ) -> Self {
        Self::new_network(&history.network, data_type, title)
    }
    
    /// Set chart type
    pub fn chart_type(mut self, chart_type: ChartType) -> Self {
        self.chart_type = chart_type;
        self
    }
    
    /// Set y-axis label
    pub fn y_label(mut self, y_label: &'a str) -> Self {
        self.y_label = y_label;
        self
    }
    
    /// Set time range in seconds
    pub fn time_range(mut self, time_range: u64) -> Self {
        self.time_range = time_range;
        self
    }
    
    /// Set minimum y value
    pub fn min_y(mut self, min_y: f64) -> Self {
        self.min_y = Some(min_y);
        self
    }
    
    /// Set maximum y value
    pub fn max_y(mut self, max_y: f64) -> Self {
        self.max_y = Some(max_y);
        self
    }
    
    /// Render the chart
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create the data points
        let data_points = self.prepare_data_points();
        
        if data_points.is_empty() {
            self.render_no_data(f, area);
            return;
        }
        
        // Calculate min and max values
        let (min_y, max_y) = self.calculate_y_range();
        
        // Create the dataset
        let datasets = self.create_datasets(&data_points);
        
        // Create the chart
        let chart = Chart::new(datasets)
            .block(Block::default().borders(Borders::ALL).title(self.title))
            .x_axis(
                Axis::default()
                    .title(Span::styled("Time", Style::default().fg(Color::White)))
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, data_points.len() as f64])
                    .labels(self.create_time_labels(&data_points)),
            )
            .y_axis(
                Axis::default()
                    .title(Span::styled(self.y_label, Style::default().fg(Color::White)))
                    .style(Style::default().fg(Color::White))
                    .bounds([min_y, max_y])
                    .labels(self.create_value_labels(min_y, max_y)),
            );
        
        f.render_widget(chart, area);
    }
    
    /// Render a message when no data is available
    fn render_no_data(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);
        
        f.render_widget(block.clone(), area);
        
        let message = Span::styled(
            "No data available", 
            Style::default().fg(Color::Gray)
        );
        
        let paragraph = ratatui::widgets::Paragraph::new(Line::from(vec![message]))
            .alignment(ratatui::layout::Alignment::Center);
        
        let inner_area = block.inner(area);
        f.render_widget(paragraph, inner_area);
    }
    
    /// Prepare data points by filtering based on time range and normalizing x values
    fn prepare_data_points(&self) -> Vec<(f64, f64)> {
        // If we have network data, use that
        if let Some((network_data, data_type)) = &self.network_data {
            if network_data.is_empty() {
                return vec![];
            }
            
            // Get the current time
            let now = Utc::now();
            
            // Filter data points within the time range
            let filtered_data: Vec<_> = network_data
                .iter()
                .filter(|(time, _)| {
                    // Keep data points within the time range
                    now.signed_duration_since(*time) < Duration::seconds(self.time_range as i64)
                })
                .collect();
            
            if filtered_data.is_empty() {
                return vec![];
            }
            
            // Convert to (index, value) pairs based on data type
            return filtered_data
                .iter()
                .enumerate()
                .map(|(i, (_, value))| {
                    let y_value = match data_type {
                        NetworkDataType::Rx => value.0 as f64,
                        NetworkDataType::Tx => value.1 as f64,
                    };
                    (i as f64, y_value)
                })
                .collect();
        }
        
        // Otherwise use standard data
        if self.data.is_empty() {
            return vec![];
        }
        
        // Get the current time
        let now = Utc::now();
        
        // Filter data points within the time range
        let filtered_data: Vec<_> = self.data
            .iter()
            .filter(|(time, _)| {
                // Keep data points within the time range
                now.signed_duration_since(*time) < Duration::seconds(self.time_range as i64)
            })
            .collect();
        
        if filtered_data.is_empty() {
            return vec![];
        }
        
        // Convert to (index, value) pairs
        filtered_data
            .iter()
            .enumerate()
            .map(|(i, (_, value))| (i as f64, *value))
            .collect()
    }
    
    /// Calculate min and max y values
    fn calculate_y_range(&self) -> (f64, f64) {
        // If we have network data, calculate based on that
        if let Some((network_data, data_type)) = &self.network_data {
            if network_data.is_empty() {
                return (0.0, 100.0); // Default range
            }
            
            // Get min and max from data if not explicitly set
            let min_y = self.min_y.unwrap_or_else(|| {
                let min_data = network_data
                    .iter()
                    .map(|(_, value)| match data_type {
                        NetworkDataType::Rx => value.0 as f64,
                        NetworkDataType::Tx => value.1 as f64,
                    })
                    .fold(f64::INFINITY, f64::min);
                
                if min_data.is_finite() {
                    // Round down to create some padding
                    (min_data * 0.9).max(0.0)
                } else {
                    0.0
                }
            });
            
            let max_y = self.max_y.unwrap_or_else(|| {
                let max_data = network_data
                    .iter()
                    .map(|(_, value)| match data_type {
                        NetworkDataType::Rx => value.0 as f64,
                        NetworkDataType::Tx => value.1 as f64,
                    })
                    .fold(f64::NEG_INFINITY, f64::max);
                
                if max_data.is_finite() {
                    // Round up to create some padding
                    (max_data * 1.1).max(min_y + 1.0)
                } else {
                    100.0
                }
            });
            
            return (min_y, max_y);
        }
        
        // Otherwise calculate based on standard data
        if self.data.is_empty() {
            return (0.0, 100.0); // Default range
        }
        
        // Get min and max from data if not explicitly set
        let min_y = self.min_y.unwrap_or_else(|| {
            let min_data = self.data
                .iter()
                .map(|(_, value)| *value)
                .fold(f64::INFINITY, f64::min);
            
            if min_data.is_finite() {
                // Round down to create some padding
                (min_data * 0.9).max(0.0)
            } else {
                0.0
            }
        });
        
        let max_y = self.max_y.unwrap_or_else(|| {
            let max_data = self.data
                .iter()
                .map(|(_, value)| *value)
                .fold(f64::NEG_INFINITY, f64::max);
            
            if max_data.is_finite() {
                // Round up to create some padding
                (max_data * 1.1).max(min_y + 1.0)
            } else {
                100.0
            }
        });
        
        (min_y, max_y)
    }
    
    /// Create datasets based on chart type
    fn create_datasets<'b>(&self, data_points: &'b [(f64, f64)]) -> Vec<Dataset<'b>> {
        let graph_type = match self.chart_type {
            ChartType::Line => GraphType::Line,
            ChartType::Scatter => GraphType::Scatter,
        };
        
        let color = if let Some((_, data_type)) = &self.network_data {
            match data_type {
                NetworkDataType::Rx => Color::Blue,
                NetworkDataType::Tx => Color::Green,
            }
        } else {
            match self.chart_type {
                ChartType::Line => Color::Cyan,
                ChartType::Scatter => Color::Green,
            }
        };
        
        vec![Dataset::default()
            .name("Data")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .graph_type(graph_type)
            .data(data_points)]
    }
    
    /// Create time labels for x-axis
    fn create_time_labels<'b>(&self, data_points: &[(f64, f64)]) -> Vec<Span<'b>> {
        if (self.data.is_empty() && self.network_data.is_none()) || data_points.is_empty() {
            return vec![Span::raw("")];
        }
        
        // Get the time range
        let now = Utc::now();
        let oldest_time = now - Duration::seconds(self.time_range as i64);
        
        // Create time labels
        vec![
            Span::styled(
                oldest_time.format("%H:%M").to_string(),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                "Now",
                Style::default().fg(Color::White),
            ),
        ]
    }
    
    /// Create value labels for y-axis
    fn create_value_labels(&self, min_y: f64, max_y: f64) -> Vec<Span> {
        // Create value labels
        vec![
            Span::from(format!("{:.1}", min_y)),
            Span::from(format!("{:.1}", (min_y + max_y) / 2.0)),
            Span::from(format!("{:.1}", max_y)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    
    #[test]
    fn test_chart_widget_new() {
        let data: Vec<(DateTime<Utc>, f64)> = vec![];
        let widget = ChartWidget::new(&data, "Test Chart");
        
        // The widget should be created with default values
        assert_eq!(widget.title, "Test Chart");
        assert!(matches!(widget.chart_type, ChartType::Line));
        assert_eq!(widget.y_label, "");
        assert_eq!(widget.time_range, 300);
        assert!(widget.min_y.is_none());
        assert!(widget.max_y.is_none());
    }
    
    #[test]
    fn test_chart_widget_new_network() {
        let data: Vec<(DateTime<Utc>, (u64, u64))> = vec![];
        let widget = ChartWidget::new_network(&data, NetworkDataType::Rx, "Network RX");
        
        // The widget should be created with correct values
        assert_eq!(widget.title, "Network RX");
        assert!(matches!(widget.chart_type, ChartType::Line));
        assert_eq!(widget.y_label, "RX Bytes");
        assert_eq!(widget.time_range, 300);
        assert!(widget.min_y.is_none());
        assert!(widget.max_y.is_none());
    }
    
    #[test]
    fn test_chart_widget_from_dashboard_cpu() {
        let now = Utc::now();
        let cpu_history: Vec<(DateTime<Utc>, f64)> = (0..5).map(|i| {
            (now - chrono::Duration::seconds(i * 5), 40.0 + (i as f64 * 0.5))
        }).collect();
        
        let memory_history: Vec<(DateTime<Utc>, f64)> = (0..5).map(|i| {
            (now - chrono::Duration::seconds(i * 5), 25.0 + (i as f64 * 0.3))
        }).collect();
        
        let network_history: Vec<(DateTime<Utc>, (u64, u64))> = (0..5).map(|i| {
            (now - chrono::Duration::seconds(i * 5), (1_000_000 - (i * 10000) as u64, 400_000 - (i * 5000) as u64))
        }).collect();
        
        let dashboard_history = DashboardMetricsHistory {
            cpu: cpu_history.clone(),
            memory: memory_history.clone(),
            network: network_history.clone(),
            custom: HashMap::new(),
        };
        
        // Test CPU history widget
        let cpu_widget = ChartWidget::from_dashboard_cpu(&dashboard_history, "CPU Usage");
        assert_eq!(cpu_widget.title, "CPU Usage");
        assert_eq!(cpu_widget.data, &dashboard_history.cpu);
        assert!(cpu_widget.network_data.is_none());
        
        // Test memory history widget
        let memory_widget = ChartWidget::from_dashboard_memory(&dashboard_history, "Memory Usage");
        assert_eq!(memory_widget.title, "Memory Usage");
        assert_eq!(memory_widget.data, &dashboard_history.memory);
        assert!(memory_widget.network_data.is_none());
        
        // Test network RX history widget
        let network_rx_widget = ChartWidget::from_dashboard_network(&dashboard_history, NetworkDataType::Rx, "Network RX");
        assert_eq!(network_rx_widget.title, "Network RX");
        assert!(network_rx_widget.data.is_empty());
        assert!(network_rx_widget.network_data.is_some());
        if let Some((data, data_type)) = network_rx_widget.network_data {
            assert_eq!(data, &dashboard_history.network);
            assert!(matches!(data_type, NetworkDataType::Rx));
        }
        
        // Test network TX history widget
        let network_tx_widget = ChartWidget::from_dashboard_network(&dashboard_history, NetworkDataType::Tx, "Network TX");
        assert_eq!(network_tx_widget.title, "Network TX");
        assert!(network_tx_widget.data.is_empty());
        assert!(network_tx_widget.network_data.is_some());
        if let Some((data, data_type)) = network_tx_widget.network_data {
            assert_eq!(data, &dashboard_history.network);
            assert!(matches!(data_type, NetworkDataType::Tx));
        }
    }
    
    #[test]
    fn test_chart_widget_calculate_y_range() {
        let now = Utc::now();
        let data: Vec<(DateTime<Utc>, f64)> = vec![
            (now, 10.0),
            (now - chrono::Duration::seconds(5), 20.0),
            (now - chrono::Duration::seconds(10), 30.0),
            (now - chrono::Duration::seconds(15), 15.0),
        ];
        
        // Test with no min/max values
        let widget = ChartWidget::new(&data, "Test Chart");
        let (min, max) = widget.calculate_y_range();
        assert!(min >= 0.0 && min <= 10.0); // Min should be at most the minimum value with some padding
        assert!(max >= 30.0); // Max should be at least the maximum value with some padding
        
        // Test with explicit min/max values
        let widget = ChartWidget::new(&data, "Test Chart")
            .min_y(5.0)
            .max_y(50.0);
        let (min, max) = widget.calculate_y_range();
        assert_eq!(min, 5.0);
        assert_eq!(max, 50.0);
    }
    
    #[test]
    fn test_chart_widget_network_data_y_range() {
        let now = Utc::now();
        let network_data: Vec<(DateTime<Utc>, (u64, u64))> = vec![
            (now, (100_000, 50_000)),
            (now - chrono::Duration::seconds(5), (150_000, 75_000)),
            (now - chrono::Duration::seconds(10), (200_000, 100_000)),
            (now - chrono::Duration::seconds(15), (120_000, 60_000)),
        ];
        
        // Test RX data range
        let widget = ChartWidget::new_network(&network_data, NetworkDataType::Rx, "Network RX");
        let (min, max) = widget.calculate_y_range();
        assert!(min >= 0.0 && min <= 100_000.0); // Min should be at most the minimum RX value with some padding
        assert!(max >= 200_000.0); // Max should be at least the maximum RX value with some padding
        
        // Test TX data range
        let widget = ChartWidget::new_network(&network_data, NetworkDataType::Tx, "Network TX");
        let (min, max) = widget.calculate_y_range();
        assert!(min >= 0.0 && min <= 50_000.0); // Min should be at most the minimum TX value with some padding
        assert!(max >= 100_000.0); // Max should be at least the maximum TX value with some padding
    }
    
    #[test]
    fn test_chart_widget_prepare_data_points() {
        let now = Utc::now();
        let data: Vec<(DateTime<Utc>, f64)> = vec![
            (now, 10.0),
            (now - chrono::Duration::seconds(5), 20.0),
            (now - chrono::Duration::seconds(10), 30.0),
            (now - chrono::Duration::seconds(15), 15.0),
        ];
        
        let widget = ChartWidget::new(&data, "Test Chart");
        let points = widget.prepare_data_points();
        
        assert_eq!(points.len(), 4); // All points should be included as they're within the default time range
        
        // Test with shorter time range
        let widget = ChartWidget::new(&data, "Test Chart").time_range(7);
        let points = widget.prepare_data_points();
        
        assert_eq!(points.len(), 2); // Only the two most recent points should be included
    }
} 