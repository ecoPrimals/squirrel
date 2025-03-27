---
title: Protocol Widget Upgrade Example
version: 1.0.0
date: 2024-05-01
status: active
---

# Protocol Widget Upgrade Example

This document provides a step-by-step guide for upgrading the `ProtocolWidget` to be compatible with Ratatui 0.24.0. The `ProtocolWidget` is one of the more complex widgets in our codebase and serves as a good example for the upgrade process.

## Original Implementation

The original implementation of the `ProtocolWidget` uses the generic Backend parameter and older Ratatui APIs:

```rust
// crates/ui-terminal/src/widgets/protocol.rs
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use chrono::DateTime;
use chrono::Utc;
use dashboard_core::data::ProtocolData;
use crate::widgets::ChartWidget;

/// Widget for displaying protocol metrics
pub struct ProtocolWidget<'a> {
    protocol: &'a ProtocolData,
    title: &'a str,
}

impl<'a> ProtocolWidget<'a> {
    /// Create a new protocol widget
    pub fn new(protocol: &'a ProtocolData, title: &'a str) -> Self {
        Self { protocol, title }
    }

    /// Render the widget
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Implementation details...
    }

    // Helper methods with Backend parameters
    fn render_connection_status<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Implementation details...
    }

    fn render_protocol_info<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Implementation details...
    }

    fn render_protocol_metrics<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Implementation details...
    }
}
```

## Step-by-Step Upgrade Guide

### Step 1: Update Imports

Update the imports to match the latest API structure:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};
```

Note that we've removed the `backend::Backend` import since it's no longer needed in the render methods.

### Step 2: Update the Main Render Method

Remove the generic Backend parameter from the `render` method:

```rust
// Old implementation
pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
    // Implementation...
}

// New implementation
pub fn render(&self, f: &mut Frame, area: Rect) {
    // Implementation...
}
```

### Step 3: Update Helper Methods

Update all helper methods to remove the Backend parameter:

```rust
// Old implementation
fn render_connection_status<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
    // Implementation...
}

// New implementation
fn render_connection_status(&self, f: &mut Frame, area: Rect) {
    // Implementation...
}

// Apply the same change to other helper methods
fn render_protocol_info(&self, f: &mut Frame, area: Rect) {
    // Implementation...
}

fn render_protocol_metrics(&self, f: &mut Frame, area: Rect) {
    // Implementation...
}
```

### Step 4: Update Style Implementation

Update any style methods to match the new API:

```rust
// Old implementation
Cell::from(status_text).style(Style::default().fg(status_color))

// New implementation (if needed)
Cell::from(status_text).style(Style::default().fg(status_color))
```

Note: Basic style methods like this might not need to change if they don't use the shorthand methods on strings directly.

### Step 5: Update Text Components

Replace any usage of `Spans` with `Line`:

```rust
// Old implementation (if present)
Spans::from(vec![
    Span::raw("Status: "),
    Span::styled(status_text, Style::default().fg(status_color)),
])

// New implementation
Line::from(vec![
    "Status: ".into(),
    status_text.fg(status_color),
])
```

### Step 6: Update Any Table Implementations

Review and update Table implementations:

```rust
// Old implementation
let table = Table::new(rows)
    .block(Block::default().borders(Borders::ALL).title("Connection Status"))
    .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
    .column_spacing(1);

// New implementation (may be the same, but check for any API changes)
let table = Table::new(rows)
    .block(Block::default().borders(Borders::ALL).title("Connection Status"))
    .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
    .column_spacing(1);
```

### Step 7: Update UI Calls to the Protocol Widget

Update any code that calls the Protocol widget:

```rust
// Old implementation
protocol_widget.render::<ratatui::backend::CrosstermBackend<std::io::Stdout>>(f, area);

// New implementation
protocol_widget.render(f, area);
```

## Complete Example

Here's a simplified example of the updated Protocol widget:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use chrono::DateTime;
use chrono::Utc;
use dashboard_core::data::ProtocolData;
use crate::widgets::ChartWidget;

/// Widget for displaying protocol metrics
pub struct ProtocolWidget<'a> {
    protocol: &'a ProtocolData,
    title: &'a str,
}

impl<'a> ProtocolWidget<'a> {
    /// Create a new protocol widget
    pub fn new(protocol: &'a ProtocolData, title: &'a str) -> Self {
        Self { protocol, title }
    }

    /// Render the widget
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create a layout with sections for different protocol metrics
        let _chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25), // Message stats
                Constraint::Percentage(25), // Transaction stats
                Constraint::Percentage(50), // Latency & Error stats
            ])
            .split(area);

        // Draw title block around the whole widget with data quality indicator
        let title = format!("{}{}", self.title, self.get_data_quality_indicator());
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title);
        f.render_widget(block, area);

        // Apply inner margins for content
        let inner_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(25), // Connection status
                Constraint::Percentage(25), // Protocol info
                Constraint::Percentage(50), // Protocol metrics
            ])
            .split(area);

        // Render connection status
        self.render_connection_status(f, inner_area[0]);
        
        // Render protocol info
        self.render_protocol_info(f, inner_area[1]);
        
        // Render protocol metrics
        self.render_protocol_metrics(f, inner_area[2]);
    }

    /// Get data quality indicator for the title
    fn get_data_quality_indicator(&self) -> &str {
        if self.is_simulated_data() {
            " [Simulated]"
        } else if self.is_stale_data() {
            " [Stale]"
        } else {
            ""
        }
    }

    /// Check if the data is simulated
    fn is_simulated_data(&self) -> bool {
        self.protocol.metrics.get("simulated")
            .map_or(false, |v| v == "true")
    }

    /// Check if the data is stale (cached)
    fn is_stale_data(&self) -> bool {
        match self.protocol.metrics.get("last_real_data") {
            Some(timestamp_str) => {
                // Try to parse the timestamp
                if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                    let now = Utc::now();
                    let time_diff = now.signed_duration_since(timestamp.with_timezone(&Utc));
                    // Consider data stale if it's more than 5 minutes old
                    time_diff.num_minutes() > 5
                } else {
                    false
                }
            },
            None => false,
        }
    }

    /// Render connection status
    fn render_connection_status(&self, f: &mut Frame, area: Rect) {
        let status_color = if self.protocol.connected {
            Color::Green
        } else {
            Color::Red
        };
        
        let status_text = if self.protocol.connected {
            "Connected"
        } else {
            "Disconnected"
        };
        
        let status_since = if let Some(last_connected) = self.protocol.last_connected {
            let now = Utc::now();
            let duration = now.signed_duration_since(last_connected);
            
            if duration.num_seconds() < 60 {
                format!("since {} seconds ago", duration.num_seconds())
            } else if duration.num_minutes() < 60 {
                format!("since {} minutes ago", duration.num_minutes())
            } else {
                format!("since {}", last_connected.format("%Y-%m-%d %H:%M:%S"))
            }
        } else {
            "".to_string()
        };
        
        let error_text = if let Some(error) = &self.protocol.error {
            format!("Error: {}", error)
        } else {
            "No errors".to_string()
        };
        
        let rows = vec![
            Row::new(vec![
                Cell::from("Status:"),
                Cell::from(status_text).style(Style::default().fg(status_color)),
            ]),
            Row::new(vec![
                Cell::from("Connection:"),
                Cell::from(status_since),
            ]),
            Row::new(vec![
                Cell::from("Retries:"),
                Cell::from(format!("{}", self.protocol.retry_count)),
            ]),
            Row::new(vec![
                Cell::from("Error:"),
                Cell::from(error_text).style(Style::default().fg(if self.protocol.error.is_some() { Color::Red } else { Color::Green })),
            ]),
        ];
        
        let table = Table::new(rows)
            .block(Block::default().borders(Borders::ALL).title("Connection Status"))
            .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
            .column_spacing(1);
        
        f.render_widget(table, area);
    }

    // Similar changes for other helper methods...
}
```

## Testing the Changes

After updating the ProtocolWidget, you should test it thoroughly:

1. Verify that it renders correctly in all states (connected, disconnected, with errors, etc.)
2. Check that all tables, blocks, and other widgets are positioned and styled correctly
3. Test with different terminal sizes to ensure responsive layout
4. Verify that data quality indicators are displayed correctly

## Next Steps

After updating the ProtocolWidget, follow a similar process for other widgets:

1. Update the MetricsWidget
2. Update the AlertsWidget
3. Update the NetworkWidget
4. Update the ChartWidget

Once all widgets are updated, update the main UI code in `ui.rs` to use the updated widget implementations.

## Common Issues and Solutions

| Issue | Solution |
|-------|----------|
| Compiler errors about incompatible types | Check for changes in Ratatui API and update accordingly |
| Visual differences in rendered UI | Verify block positions and styles, adjust layout constraints if needed |
| Style methods not behaving as expected | Check for changes in style API and update the style code |
| Performance issues with large layouts | Consider adjusting Layout cache size with `Layout::init_cache()` |

## References

1. [Ratatui 0.24.0 Highlights](https://forum.ratatui.rs/t/ratatui-0-24-0-highlights/18)
2. [Ratatui Breaking Changes](https://github.com/ratatui-org/ratatui/blob/main/BREAKING-CHANGES.md)
3. [Ratatui Documentation](https://docs.rs/ratatui/) 