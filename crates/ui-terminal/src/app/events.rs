use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEvent};
use std::time::{Duration, Instant};

use crate::error::Result;
use dashboard_core::service::DashboardService;
use crate::app::App;
use crate::app::AppTab;

/// Terminal events that the application can handle.
pub enum AppEvent {
    /// Terminal tick for regular updates
    Tick,
    /// Key press event
    Key(KeyCode, KeyModifiers),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
}

/// Event handler for terminal events.
pub struct EventHandler {
    /// The tick rate for the terminal events.
    tick_rate: Duration,
    /// The last time a tick was processed.
    last_tick: Instant,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate.
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            tick_rate,
            last_tick: Instant::now(),
        }
    }

    /// Wait for the next event and return it.
    pub fn next(&mut self) -> Result<AppEvent> {
        // Wait for event or tick
        if event::poll(self.tick_rate.saturating_sub(self.last_tick.elapsed()))? {
            // Process terminal events
            match event::read()? {
                Event::Key(key) => {
                    return Ok(AppEvent::Key(key.code, key.modifiers));
                }
                Event::Mouse(mouse) => {
                    return Ok(AppEvent::Mouse(mouse));
                }
                Event::Resize(width, height) => {
                    return Ok(AppEvent::Resize(width, height));
                }
                _ => {}
            }
        }

        // If we've reached the tick rate, return a tick event
        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
            return Ok(AppEvent::Tick);
        }
        
        // Otherwise keep polling
        self.next()
    }
}

/// Handles the key events for the application.
pub struct KeyEventHandler;

impl KeyEventHandler {
    /// Handle key events for the application
    pub fn on_key<S>(app: &mut App<S>, key: event::KeyEvent) 
    where 
        S: DashboardService + Send + Sync + 'static + ?Sized,
    {
        match key.code {
            // Navigation
            KeyCode::Tab => app.next_tab(),
            KeyCode::BackTab => app.previous_tab(),
            
            // Specific tabs
            KeyCode::Char('1') => {
                app.state.active_tab = AppTab::Overview;
            },
            KeyCode::Char('2') => {
                app.state.active_tab = AppTab::System;
            },
            KeyCode::Char('3') => {
                app.state.active_tab = AppTab::Network;
            },
            KeyCode::Char('4') => {
                app.state.active_tab = AppTab::Protocol;
            },
            KeyCode::Char('5') => {
                app.state.active_tab = AppTab::Alerts;
            },
            
            // Help
            KeyCode::Char('?') => {
                app.state.show_help = !app.state.show_help;
            },
            
            // Quit
            KeyCode::Char('q') => {
                app.state.should_quit = true;
            },
            
            // Ignore other keys
            _ => {}
        }
    }
} 