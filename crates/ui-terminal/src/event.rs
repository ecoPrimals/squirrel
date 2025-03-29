// use crate::app::App;
use crate::error::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent, KeyCode};
use std::time::{Duration, Instant};

/// Represents events handled by the application
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

/// Handles terminal events (input, ticks)
#[derive(Debug)]
pub struct EventHandler {
    tick_rate: Duration,
    last_tick: Instant,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            tick_rate,
            last_tick: Instant::now(),
        }
    }

    /// Get the next event from the terminal
    /// Blocks until an event occurs or the tick timeout is reached
    pub fn next(&mut self) -> Result<Event> {
        let timeout = self.tick_rate
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                CrosstermEvent::Key(key) => {
                    // Example: Quit on Ctrl+C
                    if key.code == KeyCode::Char('c') && key.modifiers == event::KeyModifiers::CONTROL {
                        // Or potentially signal App to quit gracefully
                    }
                    Ok(Event::Key(key))
                }
                CrosstermEvent::Mouse(mouse) => Ok(Event::Mouse(mouse)),
                CrosstermEvent::Resize(width, height) => Ok(Event::Resize(width, height)),
                _ => unimplemented!(), // Handle other event types if necessary
            }
        } else {
            self.last_tick = Instant::now();
            Ok(Event::Tick)
        }
    }
} 