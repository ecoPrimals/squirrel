use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};

/// Terminal events that can be processed
pub enum Event {
    /// Key press
    Key(KeyEvent),
    /// Mouse activity
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Timer tick for animations
    Tick,
}

/// Terminal event handler
pub struct Events {
    /// Event receiver channel
    rx: mpsc::Receiver<Event>,
    
    /// Event sender channel (kept to prevent premature drop)
    #[allow(dead_code)]
    tx: mpsc::Sender<Event>,
    
    /// Event handling thread
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
    
    /// Last tick time
    last_tick: Instant,
    
    /// Tick rate for animations
    tick_rate: Duration,
}

impl Events {
    /// Create a new event handler with the given tick rate
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let handler = {
            let tx = tx.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    // Timeout for polling
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or_else(|| Duration::from_secs(0));
                    
                    // Poll for events
                    if event::poll(timeout).unwrap() {
                        match event::read().unwrap() {
                            CrosstermEvent::Key(key) => {
                                if let Err(err) = tx.send(Event::Key(key)) {
                                    eprintln!("Error sending key event: {}", err);
                                    return;
                                }
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                if let Err(err) = tx.send(Event::Mouse(mouse)) {
                                    eprintln!("Error sending mouse event: {}", err);
                                    return;
                                }
                            }
                            CrosstermEvent::Resize(width, height) => {
                                if let Err(err) = tx.send(Event::Resize(width, height)) {
                                    eprintln!("Error sending resize event: {}", err);
                                    return;
                                }
                            }
                            _ => {}
                        }
                    }
                    
                    // Check if we need to send a tick
                    if last_tick.elapsed() >= tick_rate {
                        if let Err(err) = tx.send(Event::Tick) {
                            eprintln!("Error sending tick event: {}", err);
                            return;
                        }
                        last_tick = Instant::now();
                    }
                }
            })
        };
        
        Self {
            rx,
            tx,
            handler,
            last_tick: Instant::now(),
            tick_rate,
        }
    }
    
    /// Get the next event
    pub fn next(&self) -> io::Result<Option<Event>> {
        Ok(self.rx.try_recv().ok())
    }
    
    /// Check if it's time for a tick
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= self.tick_rate {
            self.last_tick = now;
            true
        } else {
            false
        }
    }
} 