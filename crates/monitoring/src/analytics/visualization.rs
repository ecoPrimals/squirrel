//! Visualization module for the analytics system.
//!
//! This module provides functionality for visualizing analytics data,
//! including time series, trends, and predictions.

use serde::{Serialize, Deserialize};

use crate::analytics::time_series::{DataPoint, TimeWindow};
use crate::analytics::trend_detection::Trend;
use crate::analytics::predictive::Prediction;

/// Type of chart for data visualization
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ChartType {
    /// Line chart for time series data
    Line,
    
    /// Bar chart for categorical data
    Bar,
    
    /// Area chart for time series data with emphasis on volume
    Area,
    
    /// Scatter plot for showing relationships between data points
    Scatter,
    
    /// Heatmap for showing density or frequency
    Heatmap,
    
    /// Pie chart for showing proportions
    Pie,
    
    /// Candlestick chart for financial data
    Candlestick,
    
    /// Box plot for showing distribution
    BoxPlot,
    
    /// Histogram for showing distribution
    Histogram,
}

impl Default for ChartType {
    fn default() -> Self {
        Self::Line
    }
}

/// Configuration for data visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Default chart type
    pub default_chart_type: ChartType,
    
    /// Whether to include trend lines
    pub include_trend_lines: bool,
    
    /// Whether to include prediction intervals
    pub include_prediction_intervals: bool,
    
    /// Whether to include anomaly markers
    pub include_anomaly_markers: bool,
    
    /// Maximum number of data points to display
    pub max_data_points: usize,
    
    /// Whether to use logarithmic scale for Y-axis
    pub use_log_scale: bool,
    
    /// Custom color scheme
    pub color_scheme: Option<Vec<String>>,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            default_chart_type: ChartType::default(),
            include_trend_lines: true,
            include_prediction_intervals: true,
            include_anomaly_markers: true,
            max_data_points: 500,
            use_log_scale: false,
            color_scheme: None,
        }
    }
}

/// Data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    /// The component ID
    pub component_id: String,
    
    /// The metric name
    pub metric_name: String,
    
    /// The type of chart to use
    pub chart_type: ChartType,
    
    /// The time series data
    pub time_series_data: Vec<DataPoint>,
    
    /// The trends detected in the data
    pub trends: Vec<Trend>,
    
    /// The prediction for future values
    pub prediction: Option<Prediction>,
}

/// Additional options for customizing a visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationOptions {
    /// Chart title
    pub title: Option<String>,
    
    /// X-axis label
    pub x_axis_label: Option<String>,
    
    /// Y-axis label
    pub y_axis_label: Option<String>,
    
    /// Minimum value for Y-axis
    pub y_axis_min: Option<f64>,
    
    /// Maximum value for Y-axis
    pub y_axis_max: Option<f64>,
    
    /// Whether to show the legend
    pub show_legend: bool,
    
    /// Whether to show grid lines
    pub show_grid: bool,
    
    /// Whether to animate transitions
    pub animate: bool,
    
    /// Custom colors for data series
    pub colors: Option<Vec<String>>,
    
    /// Width of the chart in pixels
    pub width: Option<u32>,
    
    /// Height of the chart in pixels
    pub height: Option<u32>,
    
    /// Whether to make the chart responsive
    pub responsive: bool,
    
    /// Type of interpolation to use for line charts
    pub interpolation: Option<String>,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            title: None,
            x_axis_label: None,
            y_axis_label: None,
            y_axis_min: None,
            y_axis_max: None,
            show_legend: true,
            show_grid: true,
            animate: true,
            colors: None,
            width: None,
            height: None,
            responsive: true,
            interpolation: None,
        }
    }
}

/// A data series for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSeries {
    /// The name of the series
    pub name: String,
    
    /// The data points in the series
    pub data: Vec<(i64, f64)>,
    
    /// The color of the series
    pub color: Option<String>,
    
    /// The type of the series
    pub series_type: DataSeriesType,
}

/// Type of data series
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DataSeriesType {
    /// Actual data
    Actual,
    
    /// Predicted data
    Predicted,
    
    /// Lower prediction interval
    LowerBound,
    
    /// Upper prediction interval
    UpperBound,
    
    /// Trend line
    Trend,
    
    /// Anomaly markers
    Anomaly,
}

/// A complete visualization including data, options, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visualization {
    /// The data for the visualization
    pub data: VisualizationData,
    
    /// The options for the visualization
    pub options: VisualizationOptions,
    
    /// The processed data series for rendering
    pub series: Vec<DataSeries>,
    
    /// The timestamp when the visualization was created
    pub timestamp: i64,
}

/// Generate a visualization from data with default options
pub fn generate_visualization(data: VisualizationData) -> Visualization {
    generate_visualization_with_options(data, VisualizationOptions::default())
}

/// Generate a visualization from data with custom options
pub fn generate_visualization_with_options(data: VisualizationData, options: VisualizationOptions) -> Visualization {
    let mut series = Vec::new();
    
    // Add the actual data series
    series.push(DataSeries {
        name: format!("{} (Actual)", data.metric_name),
        data: data.time_series_data.iter().map(|dp| (dp.timestamp, dp.value)).collect(),
        color: None,
        series_type: DataSeriesType::Actual,
    });
    
    // Add prediction series if available
    if let Some(prediction) = &data.prediction {
        // Add the prediction series
        series.push(DataSeries {
            name: format!("{} (Predicted)", data.metric_name),
            data: prediction.values.iter().map(|v| (v.timestamp, v.value)).collect(),
            color: None,
            series_type: DataSeriesType::Predicted,
        });
        
        // Add prediction interval series
        series.push(DataSeries {
            name: format!("{} (Lower Bound)", data.metric_name),
            data: prediction.values.iter().map(|v| (v.timestamp, v.lower_bound)).collect(),
            color: None,
            series_type: DataSeriesType::LowerBound,
        });
        
        series.push(DataSeries {
            name: format!("{} (Upper Bound)", data.metric_name),
            data: prediction.values.iter().map(|v| (v.timestamp, v.upper_bound)).collect(),
            color: None,
            series_type: DataSeriesType::UpperBound,
        });
    }
    
    // Add trend series for each trend
    for (i, trend) in data.trends.iter().enumerate() {
        let trend_data = vec![
            (trend.start_timestamp, data.time_series_data.iter().find(|dp| dp.timestamp == trend.start_timestamp).map_or(0.0, |dp| dp.value)),
            (trend.end_timestamp, data.time_series_data.iter().find(|dp| dp.timestamp == trend.end_timestamp).map_or(0.0, |dp| dp.value)),
        ];
        
        series.push(DataSeries {
            name: format!("Trend {}: {:?}", i + 1, trend.trend_type),
            data: trend_data,
            color: None,
            series_type: DataSeriesType::Trend,
        });
    }
    
    Visualization {
        data,
        options,
        series,
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}

/// Convert a visualization to JSON
pub fn visualization_to_json(visualization: &Visualization) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(visualization)
}

/// Convert a visualization to a simple HTML string
pub fn visualization_to_html(visualization: &Visualization) -> String {
    let title = visualization.options.title.clone().unwrap_or_else(|| 
        format!("{} - {}", visualization.data.component_id, visualization.data.metric_name)
    );
    
    let width = visualization.options.width.unwrap_or(800);
    let height = visualization.options.height.unwrap_or(400);
    
    let mut html = format!(r#"
<!DOCTYPE html>
<html>
<head>
  <title>{}</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    body {{ font-family: Arial, sans-serif; margin: 20px; }}
    .chart-container {{ width: {}px; height: {}px; margin: 0 auto; }}
  </style>
</head>
<body>
  <h1>{}</h1>
  <div class="chart-container">
    <canvas id="chart"></canvas>
  </div>
  <script>
    const data = {};
    
    const ctx = document.getElementById('chart').getContext('2d');
    const chart = new Chart(ctx, {{
      type: '{}',
      data: {{
        datasets: [
"#, 
        title, width, height, title, 
        serde_json::to_string(visualization).unwrap_or_else(|_| "{}".to_string()),
        match visualization.data.chart_type {
            ChartType::Line => "line",
            ChartType::Bar => "bar",
            ChartType::Area => "line", // Use line with fill: true
            ChartType::Scatter => "scatter",
            ChartType::Pie => "pie",
            ChartType::Heatmap => "scatter", // Custom setup needed for heatmap
            ChartType::Candlestick => "bar", // Custom setup needed for candlestick
            ChartType::BoxPlot => "bar", // Custom setup needed for boxplot
            ChartType::Histogram => "bar", // Custom setup needed for histogram
        }
    );
    
    // Add datasets
    for (i, series) in visualization.series.iter().enumerate() {
        let color = series.color.clone().unwrap_or_else(|| {
            // Default colors
            match i % 6 {
                0 => "#4285F4".to_string(), // Google Blue
                1 => "#EA4335".to_string(), // Google Red
                2 => "#FBBC05".to_string(), // Google Yellow
                3 => "#34A853".to_string(), // Google Green
                4 => "#8F00FF".to_string(), // Violet
                _ => "#00FFFF".to_string(), // Cyan
            }
        });
        
        let fill = match series.series_type {
            DataSeriesType::Actual => "false",
            DataSeriesType::Predicted => "false",
            DataSeriesType::LowerBound => "false",
            DataSeriesType::UpperBound => "false",
            DataSeriesType::Trend => "false",
            DataSeriesType::Anomaly => "false",
        };
        
        let border_dash = match series.series_type {
            DataSeriesType::Predicted => "[5, 5]",
            DataSeriesType::LowerBound => "[5, 5]",
            DataSeriesType::UpperBound => "[5, 5]",
            DataSeriesType::Trend => "[10, 5]",
            _ => "[]",
        };
        
        let point_radius = match series.series_type {
            DataSeriesType::Anomaly => "6",
            _ => "2",
        };
        
        html.push_str(&format!(r#"
          {{
            label: "{}",
            data: [
"#, 
            series.name
        ));
        
        // Add data points
        for (timestamp, value) in &series.data {
            let date_str = format!("new Date({})", timestamp);
            html.push_str(&format!(r#"              {{ x: {}, y: {} }},
"#, 
                date_str, value
            ));
        }
        
        html.push_str(&format!(r#"
            ],
            backgroundColor: "{}",
            borderColor: "{}",
            borderWidth: 2,
            pointRadius: {},
            borderDash: {},
            fill: {}
          }},
"#, 
            color, color, point_radius, border_dash, fill
        ));
    }
    
    html.push_str(r#"
        ]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          x: {
            type: 'time',
            time: {
              unit: 'minute',
              tooltipFormat: 'MMM dd, HH:mm:ss'
            },
            title: {
              display: true,
              text: 'Time'
            }
          },
          y: {
            title: {
              display: true,
              text: 'Value'
            }
          }
        },
        plugins: {
          legend: {
            position: 'top',
          },
          tooltip: {
            mode: 'index',
            intersect: false,
          }
        }
      }
    });
  </script>
</body>
</html>
"#);
    
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::time_series::DataPoint;
    
    #[test]
    fn test_generate_visualization() {
        // Create test data
        let data_points = vec![
            DataPoint {
                component_id: "test".to_string(),
                metric_name: "cpu_usage".to_string(),
                value: 50.0,
                timestamp: 1653472092000,
            },
            DataPoint {
                component_id: "test".to_string(),
                metric_name: "cpu_usage".to_string(),
                value: 60.0,
                timestamp: 1653472152000,
            },
            DataPoint {
                component_id: "test".to_string(),
                metric_name: "cpu_usage".to_string(),
                value: 55.0,
                timestamp: 1653472212000,
            },
        ];
        
        let viz_data = VisualizationData {
            component_id: "test".to_string(),
            metric_name: "cpu_usage".to_string(),
            chart_type: ChartType::Line,
            time_series_data: data_points,
            trends: Vec::new(),
            prediction: None,
        };
        
        let visualization = generate_visualization(viz_data);
        
        assert_eq!(visualization.series.len(), 1);
        assert_eq!(visualization.series[0].data.len(), 3);
    }
}

/// Generator for creating visualizations from analytics data
#[derive(Debug)]
pub struct VisualizationGenerator {
    /// Default configuration for visualizations
    config: VisualizationConfig,
}

impl VisualizationGenerator {
    /// Create a new visualization generator with the default configuration
    pub fn new() -> Self {
        Self {
            config: VisualizationConfig::default(),
        }
    }
    
    /// Create a new visualization generator with a custom configuration
    pub fn with_config(config: VisualizationConfig) -> Self {
        Self { config }
    }
    
    /// Generate a visualization for the provided data
    pub async fn generate_visualizations(&self, data: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
        // Extract component_id and metric_name from the data
        let component_id = data["component_id"].as_str().ok_or_else(|| 
            anyhow::anyhow!("Missing component_id in visualization request"))?;
            
        let metric_name = data["metric_name"].as_str().ok_or_else(|| 
            anyhow::anyhow!("Missing metric_name in visualization request"))?;
        
        // Extract optional parameters
        let chart_type = match data["chart_type"].as_str() {
            Some("line") => ChartType::Line,
            Some("bar") => ChartType::Bar,
            Some("area") => ChartType::Area,
            Some("scatter") => ChartType::Scatter,
            Some("heatmap") => ChartType::Heatmap,
            Some("pie") => ChartType::Pie,
            Some("candlestick") => ChartType::Candlestick,
            Some("boxplot") => ChartType::BoxPlot,
            Some("histogram") => ChartType::Histogram,
            _ => self.config.default_chart_type,
        };
        
        // Create a mock VisualizationData instance
        let visualization_data = VisualizationData {
            component_id: component_id.to_string(),
            metric_name: metric_name.to_string(),
            chart_type,
            time_series_data: Vec::new(), // In a real implementation, would fetch data
            trends: Vec::new(),           // In a real implementation, would detect trends
            prediction: None,             // In a real implementation, would generate prediction
        };
        
        // Extract visualization options
        let options = VisualizationOptions {
            title: data["options"]["title"].as_str().map(|s| s.to_string()),
            x_axis_label: data["options"]["x_axis_label"].as_str().map(|s| s.to_string()),
            y_axis_label: data["options"]["y_axis_label"].as_str().map(|s| s.to_string()),
            y_axis_min: data["options"]["y_axis_min"].as_f64(),
            y_axis_max: data["options"]["y_axis_max"].as_f64(),
            show_legend: data["options"]["show_legend"].as_bool().unwrap_or(true),
            show_grid: data["options"]["show_grid"].as_bool().unwrap_or(true),
            animate: data["options"]["animate"].as_bool().unwrap_or(true),
            colors: None, // In a real implementation, would extract colors
            width: data["options"]["width"].as_u64().map(|w| w as u32),
            height: data["options"]["height"].as_u64().map(|h| h as u32),
            responsive: data["options"]["responsive"].as_bool().unwrap_or(true),
            interpolation: data["options"]["interpolation"].as_str().map(|s| s.to_string()),
        };
        
        // Generate the visualization
        let visualization = generate_visualization_with_options(visualization_data, options);
        
        // Convert to JSON
        Ok(serde_json::to_value(visualization)?)
    }
}

impl Default for VisualizationGenerator {
    fn default() -> Self {
        Self::new()
    }
} 