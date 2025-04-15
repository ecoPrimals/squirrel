/// Input handling utilities for the terminal UI
/// 
/// This module provides functionality for handling input events
/// in the terminal UI.

pub use crossterm::event::{self, Event, KeyEvent, KeyCode, KeyModifiers};

/// Poll for terminal events with a timeout
pub fn poll(timeout: std::time::Duration) -> Result<bool, std::io::Error> {
    crossterm::event::poll(timeout)
}

/// Read a terminal event
pub fn read() -> Result<Event, std::io::Error> {
    crossterm::event::read()
} 