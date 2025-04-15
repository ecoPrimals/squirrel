// use crate::app::App;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::{Duration, Instant};

use crate::error::Result;

/// Terminal events that the application can handle.
pub enum Event {
    /// Terminal tick for regular updates
    Tick,
    /// Key press event
    Key(KeyEvent),
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
    pub fn next(&mut self) -> Result<Event> {
        // Wait for event or tick
        if event::poll(self.tick_rate.saturating_sub(self.last_tick.elapsed()))? {
            // Process terminal events
            match event::read()? {
                CrosstermEvent::Key(key) => {
                    return Ok(Event::Key(key));
                }
                CrosstermEvent::Mouse(mouse) => {
                    return Ok(Event::Mouse(mouse));
                }
                CrosstermEvent::Resize(width, height) => {
                    return Ok(Event::Resize(width, height));
                }
                _ => {}
            }
        }

        // If we've reached the tick rate, return a tick event
        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
            return Ok(Event::Tick);
        }
        
        // Otherwise keep polling
        self.next()
    }
} 