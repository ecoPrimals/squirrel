// crates/ui-terminal/src/widgets/chart.rs
// Basic implementation for a time series chart widget

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{
        Axis,
        Block,
        Borders,
        Chart,
        Dataset,
        GraphType,
    },
    Frame,
};
use std::collections::VecDeque;
use chrono::{DateTime, Duration, Utc};

// Define a reasonable Y-axis range, e.g., 0-100 for percentages
const Y_AXIS_BOUNDS: [f64; 2] = [0.0, 100.0]; 
// Max labels on axes
const MAX_AXIS_LABELS: usize = 5;

pub fn render<B: Backend>(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    data: &VecDeque<(DateTime<Utc>, f64)>,
) {
    if data.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(block, area);
        return;
    }

    // Find time bounds (first and last timestamp)
    let start_time = data.front().map(|(ts, _)| *ts).unwrap_or_else(Utc::now);
    let end_time = data.back().map(|(ts, _)| *ts).unwrap_or_else(Utc::now);
    let time_span_seconds = (end_time - start_time).num_seconds().max(1) as f64; // Avoid division by zero

    // Convert data to Vec<(f64, f64)> where x is seconds from start_time
    let chart_data: Vec<(f64, f64)> = data
        .iter()
        .map(|(ts, val)| {
            let x = (*ts - start_time).num_milliseconds() as f64 / 1000.0;
            (x, *val)
        })
        .collect();

    // Create dataset
    let dataset = Dataset::default()
        .name(title) // Use title for dataset name for legend (if shown)
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&chart_data);

    // Create X Axis (Time)
    let x_axis_labels: Vec<Span> = (0..=MAX_AXIS_LABELS)
        .map(|i| {
            let time_offset = Duration::seconds((time_span_seconds / MAX_AXIS_LABELS as f64 * i as f64) as i64);
            let label_time = start_time + time_offset;
            Span::raw(label_time.format("%H:%M:%S").to_string())
        })
        .collect();
    
    let x_axis = Axis::default()
        .title("Time")
        .style(Style::default().fg(Color::Gray))
        .bounds([0.0, time_span_seconds]) // Seconds from start
        .labels(x_axis_labels);

    // Create Y Axis (Value - assuming percentage 0-100)
    // TODO: Make Y-axis bounds dynamic or configurable?
    let y_axis_labels: Vec<Span> = (0..=MAX_AXIS_LABELS)
        .map(|i| {
            let val = Y_AXIS_BOUNDS[0] + (Y_AXIS_BOUNDS[1] - Y_AXIS_BOUNDS[0]) / MAX_AXIS_LABELS as f64 * i as f64;
            Span::raw(format!("{:.0}%", val))
        })
        .collect();

    let y_axis = Axis::default()
        .title("Usage (%)")
        .style(Style::default().fg(Color::Gray))
        .bounds(Y_AXIS_BOUNDS)
        .labels(y_axis_labels);

    // Create the chart
    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    frame.render_widget(chart, area);
} 