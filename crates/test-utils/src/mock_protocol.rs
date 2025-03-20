use std::sync::Arc;
use tokio::sync::RwLock;
use squirrel_core::error::Result;
use uuid::Uuid;

/// Mock message for protocol testing
#[derive(Debug, Clone)]
pub struct MockMessage {
    /// Message ID
    pub id: String,
    /// Message type
    pub msg_type: String,
    /// Message payload
    pub payload: String,
}

impl MockMessage {
    /// Create a new mock message
    pub fn new(msg_type: &str, payload: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            msg_type: msg_type.to_string(),
            payload: payload.to_string(),
        }
    }
}

/// Mock protocol implementation for testing
#[derive(Debug, Default)]
pub struct MockProtocol {
    /// Whether the mock is initialized
    pub initialized: bool,
    /// Messages sent through the mock
    pub messages: Vec<MockMessage>,
}

impl MockProtocol {
    /// Create a new mock protocol
    pub fn new() -> Self {
        Self {
            initialized: false,
            messages: Vec::new(),
        }
    }
    
    /// Initialize the mock protocol
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }
    
    /// Check if the mock protocol is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Send a message through the mock protocol
    pub fn send_message(&mut self, msg_type: &str, payload: &str) -> Result<String> {
        let message = MockMessage::new(msg_type, payload);
        let id = message.id.clone();
        self.messages.push(message);
        Ok(id)
    }
    
    /// Get all messages of a specific type
    pub fn get_messages_by_type(&self, msg_type: &str) -> Vec<MockMessage> {
        self.messages.iter()
            .filter(|msg| msg.msg_type == msg_type)
            .cloned()
            .collect()
    }
}

/// Create a new mock protocol with shared ownership
pub fn create_mock_protocol() -> Arc<RwLock<MockProtocol>> {
    Arc::new(RwLock::new(MockProtocol::new()))
}

/// Create an initialized mock protocol with shared ownership
pub fn create_initialized_mock_protocol() -> Result<Arc<RwLock<MockProtocol>>> {
    let protocol = Arc::new(RwLock::new(MockProtocol::new()));
    {
        let mut proto = protocol.try_write().unwrap();
        proto.initialize()?;
    }
    Ok(protocol)
} 