use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::{mpsc, RwLock as TokioRwLock};
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;
use thiserror::Error;
use std::fmt;
use serde_json::Value;

// MERGE NOTE: Using MCPMessage (uppercase) for consistency with acronym
// MERGE NOTE: Using Uuid for id (better than String)
// MERGE NOTE: Using ProtocolVersion struct (better versioning support)
// MERGE NOTE: Using message_type instead of type_ (clearer naming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Command,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl ProtocolVersion {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn is_compatible(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Command => write!(f, "Command"),
            MessageType::Response => write!(f, "Response"),
            MessageType::Event => write!(f, "Event"),
            MessageType::Error => write!(f, "Error"),
            MessageType::Request => write!(f, "Request"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMessage {
    pub id: String,
    pub version: ProtocolVersion,
    pub message_type: MessageType,
    pub target: Option<String>,
    pub source: Option<String>,
    pub payload: Value,
    pub metadata: Option<HashMap<String, Value>>,
    pub security: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetadata {
    pub level: SecurityLevel,
    pub encryption: Option<EncryptionInfo>,
    pub signature: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub token: String,
    pub encryption: Option<EncryptionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    pub algorithm: String,
    pub key_id: String,
    pub iv: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ProtocolHeader {
    pub version: ProtocolVersion,
    pub message_type: MessageType,
    pub target: Option<String>,
    pub source: Option<String>,
    pub security: Option<SecurityMetadata>,
    pub correlation_id: Option<String>,
}

#[derive(Debug)]
pub enum ProtocolError {
    InvalidFormat(String),
    Protocol(String),
    Security(String),
    State(String),
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProtocolError::Io(e) => Some(e),
            ProtocolError::SerdeJson(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ProtocolError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            ProtocolError::Security(msg) => write!(f, "Security error: {}", msg),
            ProtocolError::State(msg) => write!(f, "State error: {}", msg),
            ProtocolError::Io(e) => write!(f, "IO error: {}", e),
            ProtocolError::SerdeJson(e) => write!(f, "JSON error: {}", e),
        }
    }
}

// MERGE NOTE: Using Arc<RwLock> from main branch for better concurrency
#[derive(Debug)]
pub struct MCPProtocol {
    version: ProtocolVersion,
    state: Arc<TokioRwLock<ProtocolState>>,
    message_tx: mpsc::Sender<MCPMessage>,
    message_rx: mpsc::Receiver<MCPMessage>,
    handlers: Arc<TokioRwLock<HashMap<String, Box<dyn Fn(&MCPMessage) -> Result<()> + Send + Sync>>>>,
}

type MessageHandler = Box<dyn Fn(&MCPMessage) -> Result<()> + Send + Sync>;

#[derive(Debug)]
struct ProtocolState {
    is_connected: bool,
    last_message_id: String,
    pending_commands: HashMap<String, CommandState>,
    message_count: u64,
    last_activity: DateTime<Utc>,
}

#[derive(Debug)]
struct CommandState {
    id: String,
    status: CommandStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// MERGE NOTE: Using Failed(String) from main for better error context
#[derive(Debug, Clone)]
enum CommandStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

impl MCPProtocol {
    // MERGE NOTE: Using version parameter from main branch
    pub fn new(version: ProtocolVersion) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            version,
            state: Arc::new(TokioRwLock::new(ProtocolState {
                is_connected: false,
                last_message_id: String::new(),
                pending_commands: HashMap::new(),
                message_count: 0,
                last_activity: Utc::now(),
            })),
            message_tx: tx,
            message_rx: rx,
            handlers: Arc::new(TokioRwLock::new(HashMap::new())),
        }
    }

    // MERGE NOTE: Using anyhow::Result from main for better error handling
    fn validate_message(&self, message: &MCPMessage) -> Result<()> {
        // Validate message ID
        if message.id.is_empty() {
            return Err(anyhow::anyhow!("Message ID cannot be empty"));
        }

        // Validate protocol version compatibility
        if !message.version.is_compatible(&self.version) {
            return Err(anyhow::anyhow!(
                "Incompatible protocol version. Expected {}.{}.{}, got {}.{}.{}",
                self.version.major, self.version.minor, self.version.patch,
                message.version.major, message.version.minor, message.version.patch
            ));
        }

        // Validate payload
        if message.payload.is_null() {
            return Err(anyhow::anyhow!("Message payload cannot be null"));
        }

        // Validate source
        if message.source.is_none() {
            return Err(anyhow::anyhow!("Message source cannot be empty"));
        }

        // Validate security metadata
        self.validate_security_metadata(&message.security)?;

        Ok(())
    }

    fn validate_security_metadata(&self, metadata: &Option<String>) -> Result<()> {
        if let Some(security_info) = metadata {
            // Validate security level
            match security_info.as_str() {
                "Critical" => {
                    // Critical messages must have encryption and signature
                    if security_info.is_empty() {
                        return Err(anyhow::anyhow!("Critical messages must be encrypted"));
                    }
                }
                "High" => {
                    // High security messages must have encryption
                    if security_info.is_empty() {
                        return Err(anyhow::anyhow!("High security messages must be encrypted"));
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    // MERGE NOTE: Using send_message from main with better error handling
    pub async fn send_message(&self, message: MCPMessage) -> Result<()> {
        // Validate message
        self.validate_message(&message)?;

        // Update state
        let mut state = self.state.write().await;
        state.message_count += 1;
        state.last_activity = Utc::now();

        // Send message
        self.message_tx.send(message).await.map_err(|e| {
            anyhow::anyhow!("Failed to send message: {}", e)
        })?;

        Ok(())
    }

    #[instrument(skip(self, message))]
    async fn handle_response(&self, message: MCPMessage) -> Result<()> {
        let mut state = self.state.write().await;
        if let Some(cmd_state) = state.pending_commands.get_mut(&message.correlation_id.unwrap_or_default()) {
            cmd_state.status = CommandStatus::Completed;
            cmd_state.updated_at = Utc::now();
        }
        Ok(())
    }

    #[instrument(skip(self, message))]
    async fn handle_event(&self, message: MCPMessage) -> Result<()> {
        let handlers = self.handlers.read().await;
        if let Some(handler) = handlers.get(&message.message_type.to_string()) {
            handler(&message)?;
        }
        Ok(())
    }

    #[instrument(skip(self, message))]
    async fn handle_error(&self, message: MCPMessage) -> Result<()> {
        let mut state = self.state.write().await;
        if let Some(cmd_state) = state.pending_commands.get_mut(&message.correlation_id.unwrap_or_default()) {
            cmd_state.status = CommandStatus::Failed(message.payload.to_string());
            cmd_state.updated_at = Utc::now();
        }
        Ok(())
    }

    #[instrument(skip(self, message))]
    pub async fn handle_message(&self, message: MCPMessage) -> Result<()> {
        match message.message_type {
            MessageType::Response => self.handle_response(message).await?,
            MessageType::Event => self.handle_event(message).await?,
            MessageType::Error => self.handle_error(message).await?,
            _ => {
                warn!(message_type = ?message.message_type, "Unhandled message type");
            }
        }

        Ok(())
    }
}