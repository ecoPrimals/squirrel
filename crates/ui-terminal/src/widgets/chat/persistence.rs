//! Chat history persistence module
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use super::message::ChatMessage;

/// Structure for saving chat history to a file
#[derive(Serialize, Deserialize)]
pub struct SavedChatHistory {
    pub messages: Vec<ChatMessage>,
    pub timestamp: u64,
}

/// Get the default history file path
pub fn default_history_file() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("squirrel").join("chat_history.json"))
}

/// Load chat history from a file
pub fn load_history(file_path: &Path) -> std::io::Result<Vec<ChatMessage>> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        create_dir_all(parent)?;
    }

    // If the file exists, try to load it
    if file_path.exists() {
        log::debug!("Loading chat history from {:?}", file_path);
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        match serde_json::from_str::<SavedChatHistory>(&contents) {
            Ok(history) => {
                log::debug!("Loaded {} messages from history", history.messages.len());
                Ok(history.messages)
            },
            Err(e) => {
                log::error!("Failed to parse chat history: {}", e);
                Ok(Vec::new()) // Continue without history rather than failing
            }
        }
    } else {
        log::debug!("No existing chat history at {:?}", file_path);
        Ok(Vec::new())
    }
}

/// Save chat history to a file
pub fn save_history(file_path: &Path, messages: &[ChatMessage]) -> std::io::Result<()> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        create_dir_all(parent)?;
    }
    
    log::debug!("Saving {} messages to history file {:?}", messages.len(), file_path);
    
    let history = SavedChatHistory {
        messages: messages.to_vec(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };
    
    let json = serde_json::to_string_pretty(&history)?;
    
    let mut file = File::create(file_path)?;
    file.write_all(json.as_bytes())?;
    log::debug!("Chat history saved successfully");
    Ok(())
} 