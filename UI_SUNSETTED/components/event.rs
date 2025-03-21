use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Receiver, Sender};
use crate::ui::error::UiError;
use std::fmt;

/// A unique identifier for an event handler
pub type HandlerId = usize;

/// A trait for events that can be sent between components
pub trait Event: Any + Send + 'static {}

/// A handler for UI component events.
///
/// Event handlers are used to respond to various UI events such as key presses,
/// mouse clicks, or custom events. They can be registered with components to
/// provide interactive behavior.
pub struct EventHandler<T> {
    /// The type of event this handler responds to.
    pub event_type: String,
    /// The function that handles the event.
    pub handler: Box<dyn Fn(&T) -> bool + Send + Sync>,
}

impl<T> EventHandler<T> {
    /// Creates a new event handler with the specified event type and handler function.
    ///
    /// # Arguments
    /// * `event_type` - The type of event to handle
    /// * `handler` - The function to call when the event occurs
    pub fn new<F>(event_type: impl Into<String>, handler: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        Self {
            event_type: event_type.into(),
            handler: Box::new(handler),
        }
    }

    /// Handles an event by calling the handler function with the event data.
    ///
    /// Returns true if the event was handled, false otherwise.
    pub fn handle(&self, event: &T) -> bool {
        (self.handler)(event)
    }
}

impl<T> Default for EventHandler<T> {
    fn default() -> Self {
        Self {
            event_type: String::new(),
            handler: Box::new(|_: &T| false),
        }
    }
}

/// A collection of event handlers for a UI component.
pub struct EventHandlerRegistry<T> {
    /// The handlers registered for different event types.
    handlers: HashMap<String, Vec<EventHandler<T>>>,
}

impl<T> EventHandlerRegistry<T> {
    /// Creates a new empty event handler registry.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Registers a new event handler.
    ///
    /// # Arguments
    /// * `handler` - The event handler to register
    pub fn register(&mut self, handler: EventHandler<T>) {
        self.handlers
            .entry(handler.event_type.clone())
            .or_default()
            .push(handler);
    }

    /// Handles an event by calling all registered handlers for the event type.
    ///
    /// # Arguments
    /// * `event_type` - The type of event to handle
    /// * `event` - The event data
    ///
    /// Returns true if any handler processed the event, false otherwise.
    pub fn handle(&self, event_type: &str, event: &T) -> bool {
        if let Some(handlers) = self.handlers.get(event_type) {
            handlers.iter().any(|h| h.handle(event))
        } else {
            false
        }
    }

    /// Removes all handlers for the specified event type.
    ///
    /// # Arguments
    /// * `event_type` - The type of event handlers to remove
    pub fn unregister(&mut self, event_type: &str) {
        self.handlers.remove(event_type);
    }

    /// Removes all event handlers.
    pub fn clear(&mut self) {
        self.handlers.clear();
    }
}

impl<T> Default for EventHandlerRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A function type for handling events that can return a Result
type EventHandlerFn = Box<dyn Fn(&dyn Any) -> Result<(), UiError> + Send + Sync>;

/// A tuple containing a handler ID and its associated handler function
type HandlerEntry = (HandlerId, EventHandlerFn);

/// A map of event type IDs to their registered handlers
type HandlersMap = HashMap<TypeId, Vec<HandlerEntry>>;

/// The event dispatcher manages event subscriptions and delivery
pub struct EventDispatcher {
    /// Thread-safe map of event type IDs to their registered handlers
    handlers: Arc<Mutex<HandlersMap>>,
    /// Thread-safe counter for generating unique handler IDs
    next_id: Arc<Mutex<HandlerId>>,
    /// Channel sender for dispatching events
    sender: Sender<Box<dyn Any + Send>>,
    /// Channel receiver for processing events
    receiver: Mutex<Receiver<Box<dyn Any + Send>>>,
}

impl EventDispatcher {
    /// Creates a new event dispatcher with default settings.
    ///
    /// The dispatcher is initialized with an empty set of handlers and a channel
    /// for event communication.
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
            sender,
            receiver: Mutex::new(receiver),
        }
    }

    /// Subscribes to events of a specific type
    pub fn subscribe<T: 'static>(&self, handler: impl Fn(&T) -> Result<(), UiError> + Send + Sync + 'static) -> HandlerId {
        let type_id = TypeId::of::<T>();
        let mut handlers = self.handlers.lock().unwrap();
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            *next_id += 1;
            *next_id
        };

        let event_handler: EventHandlerFn = Box::new(move |event: &dyn Any| {
            if let Some(event) = event.downcast_ref::<T>() {
                handler(event)
            } else {
                Ok(())
            }
        });

        handlers
            .entry(type_id)
            .or_default()
            .push((id, event_handler));

        id
    }

    /// Unsubscribes a handler by its ID
    pub fn unsubscribe(&self, handler_id: &HandlerId) -> Result<(), UiError> {
        let mut handlers = self.handlers.lock()
            .map_err(|_| UiError::Component("Failed to lock handlers".to_string()))?;

        for handlers_vec in handlers.values_mut() {
            if let Some(pos) = handlers_vec.iter().position(|(id, _)| id == handler_id) {
                let _removed = handlers_vec.remove(pos);
                return Ok(());
            }
        }
        Ok(())
    }

    /// Dispatches an event to all registered handlers
    pub fn dispatch<E: Event>(&self, event: E) -> Result<(), UiError> {
        self.sender.send(Box::new(event))
            .map_err(|_| UiError::Component("Failed to send event".to_string()))?;
        Ok(())
    }

    /// Processes all pending events
    pub fn process_events(&self) -> Result<(), UiError> {
        let receiver = self.receiver.lock()
            .map_err(|_| UiError::Component("Failed to lock receiver".to_string()))?;

        let handlers = self.handlers.lock()
            .map_err(|_| UiError::Component("Failed to lock handlers".to_string()))?;

        while let Ok(event) = receiver.try_recv() {
            let type_id = (*event).type_id();
            if let Some(handlers_vec) = handlers.get(&type_id) {
                for (_, handler) in handlers_vec {
                    handler(&*event)?;
                }
            }
        }
        Ok(())
    }

    /// Clears all event handlers
    pub fn clear(&self) -> Result<(), UiError> {
        let mut handlers = self.handlers.lock()
            .map_err(|_| UiError::Component("Failed to lock handlers".to_string()))?;
        handlers.clear();
        Ok(())
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for EventDispatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventDispatcher")
            .field("handlers", &"<event handlers>")
            .finish()
    }
}

/// Event representing a component gaining or losing focus.
pub struct FocusEvent {
    /// The unique identifier of the component.
    pub component_id: String,
    /// Whether the component gained focus (true) or lost focus (false).
    pub focused: bool,
}

impl Event for FocusEvent {}

/// Event representing a mouse click on a component.
pub struct ClickEvent {
    /// The unique identifier of the component.
    pub component_id: String,
    /// The x-coordinate of the click position.
    pub x: u16,
    /// The y-coordinate of the click position.
    pub y: u16,
}

impl Event for ClickEvent {}

/// Event representing a keyboard input on a component.
pub struct KeyEvent {
    /// The unique identifier of the component.
    pub component_id: String,
    /// The character that was typed.
    pub key: char,
    /// The modifier keys that were held during the key press.
    pub modifiers: KeyModifiers,
}

/// Represents the state of modifier keys during a key event.
pub struct KeyModifiers {
    /// Whether the shift key was held.
    pub shift: bool,
    /// Whether the control key was held.
    pub ctrl: bool,
    /// Whether the alt key was held.
    pub alt: bool,
}

impl Event for KeyEvent {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Debug)]
    struct TestEvent {
        value: i32,
    }

    impl Event for TestEvent {}

    #[test]
    fn test_event_subscription() {
        let dispatcher = EventDispatcher::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let handler_id = dispatcher
            .subscribe(move |event: &TestEvent| {
                assert_eq!(event.value, 42);
                called_clone.store(true, Ordering::SeqCst);
                Ok(())
            });

        dispatcher.dispatch(TestEvent { value: 42 }).unwrap();
        dispatcher.process_events().unwrap();

        assert!(called.load(Ordering::SeqCst));

        // Test unsubscribe
        dispatcher.unsubscribe(&handler_id).unwrap();
        called.store(false, Ordering::SeqCst);

        dispatcher.dispatch(TestEvent { value: 42 }).unwrap();
        dispatcher.process_events().unwrap();

        assert!(!called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_multiple_handlers() {
        let dispatcher = EventDispatcher::new();
        let count = Arc::new(AtomicBool::new(false));
        let count_clone = count.clone();

        dispatcher
            .subscribe(move |_: &TestEvent| {
                count_clone.store(true, Ordering::SeqCst);
                Ok(())
            });

        let count_clone = count.clone();
        dispatcher
            .subscribe(move |_: &TestEvent| {
                assert!(count_clone.load(Ordering::SeqCst));
                Ok(())
            });

        dispatcher.dispatch(TestEvent { value: 42 }).unwrap();
        dispatcher.process_events().unwrap();
    }

    #[test]
    fn test_focus_event() {
        let dispatcher = EventDispatcher::new();
        let focused = Arc::new(AtomicBool::new(false));
        let focused_clone = focused.clone();

        dispatcher
            .subscribe(move |event: &FocusEvent| {
                assert_eq!(event.component_id, "test");
                focused_clone.store(event.focused, Ordering::SeqCst);
                Ok(())
            });

        dispatcher
            .dispatch(FocusEvent {
                component_id: "test".to_string(),
                focused: true,
            })
            .unwrap();
        dispatcher.process_events().unwrap();

        assert!(focused.load(Ordering::SeqCst));
    }
} 