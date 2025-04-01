use squirrel_mcp::error::Result;
use squirrel_mcp::protocol::{MessageType, MCPMessage};
use serde_json::json;
use tokio::main;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// A simple message journal for MCP messages
struct MessageJournal {
    _name: String,
    max_entries: usize,
    messages: Arc<Mutex<VecDeque<JournalEntry>>>,
}

/// Journal entry with timestamp and message
struct JournalEntry {
    timestamp: u64,
    message: MCPMessage,
}

impl MessageJournal {
    fn new(name: &str, max_entries: usize) -> Self {
        Self {
            _name: name.to_string(),
            max_entries,
            messages: Arc::new(Mutex::new(VecDeque::with_capacity(max_entries))),
        }
    }
    
    fn record_message(&self, message: MCPMessage) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let entry = JournalEntry {
            timestamp,
            message,
        };
        
        let mut messages = self.messages.lock().unwrap();
        
        // If we're at capacity, remove oldest
        if messages.len() >= self.max_entries {
            messages.pop_front();
        }
        
        messages.push_back(entry);
        
        Ok(())
    }
    
    fn get_entries(&self) -> Vec<(u64, MCPMessage)> {
        let messages = self.messages.lock().unwrap();
        
        messages.iter()
            .map(|entry| (entry.timestamp, entry.message.clone()))
            .collect()
    }
    
    fn get_entries_by_type(&self, message_type: MessageType) -> Vec<(u64, MCPMessage)> {
        let messages = self.messages.lock().unwrap();
        
        messages.iter()
            .filter(|entry| entry.message.type_ == message_type)
            .map(|entry| (entry.timestamp, entry.message.clone()))
            .collect()
    }
    
    fn clear(&self) {
        let mut messages = self.messages.lock().unwrap();
        messages.clear();
    }
}

/// Example demonstrating a simple message journal
#[main]
async fn main() -> Result<()> {
    // Create a new journal
    let journal = MessageJournal::new("TestJournal", 5);
    
    println!("Creating a message journal with capacity of 5 entries");
    
    // Record some messages
    println!("\nRecording messages...");
    
    // Command messages
    journal.record_message(MCPMessage::new(
        MessageType::Command,
        json!({
            "action": "get_user",
            "id": "user123"
        })
    ))?;
    
    journal.record_message(MCPMessage::new(
        MessageType::Command,
        json!({
            "action": "update_user",
            "id": "user123",
            "data": {
                "email": "user@example.com"
            }
        })
    ))?;
    
    // Response messages
    journal.record_message(MCPMessage::new(
        MessageType::Response,
        json!({
            "status": "success",
            "data": {
                "id": "user123",
                "name": "Test User"
            }
        })
    ))?;
    
    // Event messages
    journal.record_message(MCPMessage::new(
        MessageType::Event,
        json!({
            "type": "user_updated",
            "user_id": "user123"
        })
    ))?;
    
    journal.record_message(MCPMessage::new(
        MessageType::Event,
        json!({
            "type": "session_created",
            "session_id": "sess_123456"
        })
    ))?;
    
    // Display all entries
    println!("\nAll journal entries:");
    for (timestamp, message) in journal.get_entries() {
        println!("[{}] Type: {:?}, ID: {:?}", 
            timestamp, 
            message.type_,
            message.id
        );
    }
    
    // Display only events
    println!("\nEvent entries only:");
    for (timestamp, message) in journal.get_entries_by_type(MessageType::Event) {
        println!("[{}] ID: {:?}, Event: {}", 
            timestamp,
            message.id,
            message.payload.get("type").unwrap_or(&json!("unknown")).as_str().unwrap_or("unknown")
        );
    }
    
    // Display only commands
    println!("\nCommand entries only:");
    for (timestamp, message) in journal.get_entries_by_type(MessageType::Command) {
        println!("[{}] ID: {:?}, Action: {}", 
            timestamp,
            message.id,
            message.payload.get("action").unwrap_or(&json!("unknown")).as_str().unwrap_or("unknown")
        );
    }
    
    // Add one more message to test capacity
    println!("\nAdding one more message (should drop oldest)...");
    journal.record_message(MCPMessage::new(
        MessageType::Command,
        json!({
            "action": "delete_user",
            "id": "user123"
        })
    ))?;
    
    // Display all entries again
    println!("\nUpdated journal entries (oldest should be removed):");
    for (timestamp, message) in journal.get_entries() {
        println!("[{}] Type: {:?}, ID: {:?}", 
            timestamp, 
            message.type_,
            message.id
        );
    }
    
    // Clear the journal
    println!("\nClearing journal...");
    journal.clear();
    println!("Journal entries after clearing: {}", journal.get_entries().len());
    
    println!("\nSimple journal example completed!");
    Ok(())
} 