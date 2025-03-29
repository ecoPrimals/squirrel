use std::collections::VecDeque;
use chrono::{DateTime, Utc, TimeZone};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Block;

use crate::adapter::{ConnectionHealth, ConnectionEvent, ConnectionEventType, ConnectionStatus};
use crate::widgets::ConnectionHealthWidget;

/// Helper function to create a test connection health object
fn create_test_connection_health() -> ConnectionHealth {
    let now = Utc::now();
    
    ConnectionHealth {
        latency_ms: 45.6,
        packet_loss: 0.5,
        stability: 98.7,
        signal_strength: 92.3,
        health_score: 0.95,
        status: ConnectionStatus::Connected,
        connected_since: Some(now - chrono::Duration::minutes(30)),
        last_status_change: Some(now - chrono::Duration::minutes(30)),
        last_checked: now,
    }
}

/// Helper function to create connection events for testing
fn create_test_connection_events() -> Vec<ConnectionEvent> {
    let now = Utc::now();
    
    vec![
        ConnectionEvent {
            event_type: ConnectionEventType::Connected,
            details: "Initial connection established".to_string(),
            timestamp: now - chrono::Duration::minutes(30),
        },
        ConnectionEvent {
            event_type: ConnectionEventType::Disconnected,
            details: "Connection lost".to_string(),
            timestamp: now - chrono::Duration::minutes(25),
        },
        ConnectionEvent {
            event_type: ConnectionEventType::Reconnecting,
            details: "Attempting to reconnect".to_string(),
            timestamp: now - chrono::Duration::minutes(24),
        },
        ConnectionEvent {
            event_type: ConnectionEventType::ReconnectFailure,
            details: "Failed to reconnect".to_string(),
            timestamp: now - chrono::Duration::minutes(23),
        },
        ConnectionEvent {
            event_type: ConnectionEventType::Reconnecting,
            details: "Attempting to reconnect again".to_string(),
            timestamp: now - chrono::Duration::minutes(22),
        },
        ConnectionEvent {
            event_type: ConnectionEventType::ReconnectSuccess,
            details: "Successfully reconnected".to_string(),
            timestamp: now - chrono::Duration::minutes(21),
        },
        ConnectionEvent {
            event_type: ConnectionEventType::Error,
            details: "Error processing message".to_string(),
            timestamp: now - chrono::Duration::minutes(15),
        },
        ConnectionEvent {
            event_type: ConnectionEventType::Connected,
            details: "New connection established".to_string(),
            timestamp: now - chrono::Duration::minutes(10),
        },
    ]
}

/// Helper function to create connection health history metrics
fn create_test_history_metrics() -> Vec<(DateTime<Utc>, f64)> {
    let now = Utc::now();
    
    (0..10).map(|i| {
        let time = now - chrono::Duration::minutes(i);
        let score = 0.8 + (i as f64 * 0.02);
        (time, score)
    }).collect()
}

#[test]
fn test_connection_health_widget_new() {
    let widget = ConnectionHealthWidget::new("Connection Health");
    
    // Assert title is set correctly
    assert_eq!(widget.title, "Connection Health");
    
    // Check that all options are None initially
    assert!(widget.connection_health.is_none());
    assert!(widget.connection_history.is_none());
    assert!(widget.history_metrics.is_none());
    assert!(widget.last_update.is_none());
}

#[test]
fn test_connection_health_widget_with_health() {
    let health = create_test_connection_health();
    let widget = ConnectionHealthWidget::new("Connection Health")
        .with_health(&health);
    
    // Check that health data is set
    assert!(widget.connection_health.is_some());
    assert!(widget.last_update.is_some());
    
    // Check that history has been updated with at least one data point
    assert!(!widget.health_score_history.is_empty());
}

#[test]
fn test_connection_health_widget_with_history() {
    let events = create_test_connection_events();
    let widget = ConnectionHealthWidget::new("Connection Health")
        .with_history(&events);
    
    // Check that history data is set
    assert!(widget.connection_history.is_some());
    assert_eq!(widget.connection_history.unwrap().len(), events.len());
}

#[test]
fn test_connection_health_widget_with_history_metrics() {
    let metrics = create_test_history_metrics();
    let widget = ConnectionHealthWidget::new("Connection Health")
        .with_history_metrics(&metrics);
    
    // Check that metrics data is set
    assert!(widget.history_metrics.is_some());
    assert_eq!(widget.history_metrics.unwrap().len(), metrics.len());
}

#[test]
fn test_connection_health_widget_with_all_data() {
    let health = create_test_connection_health();
    let events = create_test_connection_events();
    let metrics = create_test_history_metrics();
    
    let widget = ConnectionHealthWidget::new("Connection Health")
        .with_health(&health)
        .with_history(&events)
        .with_history_metrics(&metrics);
    
    // Check that all data is set
    assert!(widget.connection_health.is_some());
    assert!(widget.connection_history.is_some());
    assert!(widget.history_metrics.is_some());
    assert!(widget.last_update.is_some());
}

#[test]
fn test_connection_health_score_color() {
    use crate::widgets::connection_health::ConnectionHealthWidget;
    
    // Test color mapping for different health scores
    assert_eq!(
        ConnectionHealthWidget::health_score_color(0.9),
        ratatui::style::Color::Green
    );
    
    assert_eq!(
        ConnectionHealthWidget::health_score_color(0.7),
        ratatui::style::Color::Yellow
    );
    
    assert_eq!(
        ConnectionHealthWidget::health_score_color(0.5),
        ratatui::style::Color::Rgb(255, 165, 0) // Orange
    );
    
    assert_eq!(
        ConnectionHealthWidget::health_score_color(0.3),
        ratatui::style::Color::Red
    );
}

#[test]
fn test_connection_status_color() {
    use crate::widgets::connection_health::ConnectionHealthWidget;
    
    // Test color mapping for different connection statuses
    assert_eq!(
        ConnectionHealthWidget::connection_status_color(&ConnectionStatus::Connected),
        ratatui::style::Color::Green
    );
    
    assert_eq!(
        ConnectionHealthWidget::connection_status_color(&ConnectionStatus::Connecting),
        ratatui::style::Color::Yellow
    );
    
    assert_eq!(
        ConnectionHealthWidget::connection_status_color(&ConnectionStatus::Disconnected),
        ratatui::style::Color::Red
    );
    
    assert_eq!(
        ConnectionHealthWidget::connection_status_color(&ConnectionStatus::Error("test".to_string())),
        ratatui::style::Color::Red
    );
}

#[test]
fn test_connection_event_color() {
    use crate::widgets::connection_health::ConnectionHealthWidget;
    
    // Test color mapping for different connection event types
    assert_eq!(
        ConnectionHealthWidget::connection_event_color(&ConnectionEventType::Connected),
        ratatui::style::Color::Green
    );
    
    assert_eq!(
        ConnectionHealthWidget::connection_event_color(&ConnectionEventType::ReconnectSuccess),
        ratatui::style::Color::Green
    );
    
    assert_eq!(
        ConnectionHealthWidget::connection_event_color(&ConnectionEventType::Reconnecting),
        ratatui::style::Color::Yellow
    );
    
    assert_eq!(
        ConnectionHealthWidget::connection_event_color(&ConnectionEventType::Disconnected),
        ratatui::style::Color::Red
    );
    
    assert_eq!(
        ConnectionHealthWidget::connection_event_color(&ConnectionEventType::ReconnectFailure),
        ratatui::style::Color::Red
    );
    
    assert_eq!(
        ConnectionHealthWidget::connection_event_color(&ConnectionEventType::Error),
        ratatui::style::Color::Red
    );
}

#[test]
fn test_format_duration() {
    use crate::widgets::connection_health::ConnectionHealthWidget;
    use std::time::Duration;
    
    // Test duration formatting
    assert_eq!(
        ConnectionHealthWidget::format_duration(Duration::from_secs(30)),
        "30s"
    );
    
    assert_eq!(
        ConnectionHealthWidget::format_duration(Duration::from_secs(90)),
        "1m 30s"
    );
    
    assert_eq!(
        ConnectionHealthWidget::format_duration(Duration::from_secs(3661)),
        "1h 1m"
    );
}

#[test]
fn test_connection_health_widget_with_disconnected_status() {
    let mut health = create_test_connection_health();
    health.status = ConnectionStatus::Disconnected;
    health.connected_since = None;
    health.last_status_change = Some(Utc::now() - chrono::Duration::minutes(5));
    
    let widget = ConnectionHealthWidget::new("Connection Health")
        .with_health(&health);
    
    // Check that health data is set with disconnected status
    assert!(widget.connection_health.is_some());
    assert_eq!(widget.connection_health.unwrap().status, ConnectionStatus::Disconnected);
}

#[test]
fn test_connection_health_widget_with_error_status() {
    let mut health = create_test_connection_health();
    health.status = ConnectionStatus::Error("Connection refused".to_string());
    health.connected_since = None;
    health.health_score = 0.1;
    
    let widget = ConnectionHealthWidget::new("Connection Health")
        .with_health(&health);
    
    // Check that health data is set with error status
    assert!(widget.connection_health.is_some());
    if let ConnectionStatus::Error(msg) = &widget.connection_health.unwrap().status {
        assert_eq!(msg, "Connection refused");
    } else {
        panic!("Expected Error status");
    }
}

#[test]
fn test_connection_health_widget_with_empty_history() {
    let events: Vec<ConnectionEvent> = Vec::new();
    let widget = ConnectionHealthWidget::new("Connection Health")
        .with_history(&events);
    
    // Check that history data is set but empty
    assert!(widget.connection_history.is_some());
    assert_eq!(widget.connection_history.unwrap().len(), 0);
}

#[test]
fn test_connection_health_widget_with_long_history() {
    // Create a very long history (more than what would be displayed)
    let now = Utc::now();
    let events: Vec<ConnectionEvent> = (0..50).map(|i| {
        ConnectionEvent {
            event_type: if i % 2 == 0 { 
                ConnectionEventType::Connected 
            } else { 
                ConnectionEventType::Disconnected 
            },
            details: format!("Event {}", i),
            timestamp: now - chrono::Duration::minutes(i as i64),
        }
    }).collect();
    
    let widget = ConnectionHealthWidget::new("Connection Health")
        .with_history(&events);
    
    // Check that all history data is set even though not all will be displayed
    assert!(widget.connection_history.is_some());
    assert_eq!(widget.connection_history.unwrap().len(), 50);
} 