use std::{fmt, io};
use crossbeam_channel::{Receiver, Sender};
use ratatui::event::{Event as TuiEvent, KeyEvent, MouseEvent};
use crossterm::event::{KeyEvent, MouseEvent};
use std::fmt::Debug;

pub trait Event: Debug + Send + 'static {
    fn event_type(&self) -> EventType;
}

#[derive(Debug, Clone)]
pub enum EventType {
    Key,
    Mouse,
    Resize,
    Focus,
    Custom(String),
}

impl Event for EventType {
    fn event_type(&self) -> EventType {
        self.clone()
    }
}

impl From<KeyEvent> for EventType {
    fn from(event: KeyEvent) -> Self {
        EventType::Key
    }
}

impl From<MouseEvent> for EventType {
    fn from(event: MouseEvent) -> Self {
        EventType::Mouse
    }
}

#[derive(Debug, Clone)]
pub struct ResizeEvent {
    pub width: u16,
    pub height: u16,
}

impl Event for ResizeEvent {
    fn event_type(&self) -> EventType {
        EventType::Resize
    }
}

#[derive(Debug, Clone)]
pub struct FocusEvent {
    pub focused: bool,
}

impl Event for FocusEvent {
    fn event_type(&self) -> EventType {
        EventType::Focus
    }
}

#[derive(Debug, Clone)]
pub struct CustomEvent {
    pub payload: String,
}

impl Event for CustomEvent {
    fn event_type(&self) -> EventType {
        EventType::Custom(self.payload.clone())
    }
}

pub struct EventBus {
    sender: Sender<Box<dyn Event>>,
    receiver: Receiver<Box<dyn Event>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }

    pub fn sender(&self) -> Sender<Box<dyn Event>> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> Receiver<Box<dyn Event>> {
        self.receiver.clone()
    }
}

pub struct EventHandler {
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    tick_rate: u64,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> io::Result<Self> {
        let (sender, receiver) = crossbeam_channel::bounded(100);
        Ok(Self {
            sender,
            receiver,
            tick_rate,
        })
    }

    pub fn sender(&self) -> Sender<Event> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> Receiver<Event> {
        self.receiver.clone()
    }

    pub fn tick_rate(&self) -> u64 {
        self.tick_rate
    }

    pub fn start(&self) -> io::Result<()> {
        let sender = self.sender.clone();
        let tick_rate = self.tick_rate;

        std::thread::spawn(move || {
            let mut last_tick = std::time::Instant::now();
            loop {
                let timeout = std::time::Duration::from_millis(tick_rate)
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| std::time::Duration::from_secs(0));

                if crossbeam_channel::select! {
                    recv(crossbeam_channel::after(timeout)) -> _ => {
                        sender.send(Event::Tick).unwrap();
                        last_tick = std::time::Instant::now();
                        false
                    }
                    default => false
                } {
                    break;
                }
            }
        });

        Ok(())
    }
} 