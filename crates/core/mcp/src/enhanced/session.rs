// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Enhanced Session Management
//!
//! Advanced session management with persistent context, intelligent session
//! restoration, and multi-client support.

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;

use tokio::sync::{RwLock, Mutex};
use tokio::time::{interval, Instant};
use tracing::{info, error, warn, debug, instrument};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::{Result, types::MCPError};
use crate::protocol::types::MCPMessage;
use super::{SessionConfig, MCPEvent, ClientInfo, UserPreferences, MCPContext};

/// Enhanced Session Manager - Advanced session orchestration
#[derive(Debug)]
pub struct EnhancedSessionManager {
    /// Configuration
    config: Arc<SessionConfig>,
    
    /// Active sessions
    active_sessions: Arc<RwLock<HashMap<String, Arc<MCPSession>>>>,
    
    /// Session storage
    storage: Arc<dyn SessionStorage>,
    
    /// Context manager
    context_manager: Arc<ContextManager>,
    
    /// Session monitor
    monitor: Arc<SessionMonitor>,
    
    /// Cleanup manager
    cleanup_manager: Arc<CleanupManager>,
    
    /// Metrics
    metrics: Arc<Mutex<SessionMetrics>>,
}

/// MCP Session - Enhanced session with rich context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPSession {
    /// Session ID
    pub session_id: String,
    
    /// Client information
    pub client_info: ClientInfo,
    
    /// Session context
    pub context: MCPContext,
    
    /// Session state
    pub state: SessionState,
    
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
    
    /// Session metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Configuration
    pub config: SessionConfiguration,
    
    /// Statistics
    pub stats: SessionStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfiguration {
    /// Auto-save enabled
    pub auto_save: bool,
    
    /// Auto-save interval (seconds)
    pub auto_save_interval: u64,
    
    /// Context history limit
    pub context_limit: usize,
    
    /// Idle timeout (seconds)
    pub idle_timeout: u64,
    
    /// Persistence settings
    pub persistence: PersistenceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Enable persistence
    pub enabled: bool,
    
    /// Storage backend
    pub backend: StorageBackend,
    
    /// Compression enabled
    pub compression: bool,
    
    /// Encryption enabled
    pub encryption: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    Memory,
    FileSystem { path: String },
    Database { url: String },
    Redis { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    /// Total messages
    pub total_messages: u64,
    
    /// Tool executions
    pub tool_executions: u64,
    
    /// AI interactions
    pub ai_interactions: u64,
    
    /// Context switches
    pub context_switches: u64,
    
    /// Session duration (seconds)
    pub duration_seconds: u64,
    
    /// Data transferred (bytes)
    pub data_transferred: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    Initializing,
    Active,
    Idle,
    Suspended,
    Restoring,
    Terminating,
    Terminated,
}

impl EnhancedSessionManager {
    /// Create a new Enhanced Session Manager
    #[instrument]
    pub async fn new(config: SessionConfig) -> Result<Self> {
        info!("Initializing Enhanced Session Manager");
        
        let config = Arc::new(config);
        
        // Initialize storage
        let storage: Arc<dyn SessionStorage> = Arc::new(
            MemorySessionStorage::new().await?
        );
        
        // Initialize context manager
        let context_manager = Arc::new(ContextManager::new(config.clone()).await?);
        
        // Initialize monitor
        let monitor = Arc::new(SessionMonitor::new(config.clone()).await?);
        
        // Initialize cleanup manager
        let cleanup_manager = Arc::new(CleanupManager::new(config.clone()).await?);
        
        let manager = Self {
            config: config.clone(),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            storage,
            context_manager,
            monitor,
            cleanup_manager,
            metrics: Arc::new(Mutex::new(SessionMetrics::default())),
        };
        
        info!("Enhanced Session Manager initialized successfully");
        Ok(manager)
    }
    
    /// Start the session manager
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting Enhanced Session Manager");
        
        // Start monitor
        self.monitor.start().await?;
        
        // Start cleanup manager
        self.cleanup_manager.start().await?;
        
        // Start context manager
        self.context_manager.start().await?;
        
        // Start periodic tasks
        self.start_periodic_tasks().await?;
        
        info!("Enhanced Session Manager started successfully");
        Ok(())
    }
    
    /// Stop the session manager
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Enhanced Session Manager");
        
        // Save all active sessions
        self.save_all_sessions().await?;
        
        // Stop components
        self.cleanup_manager.stop().await?;
        self.monitor.stop().await?;
        self.context_manager.stop().await?;
        
        info!("Enhanced Session Manager stopped successfully");
        Ok(())
    }
    
    /// Create a new session
    #[instrument(skip(self, client_info))]
    pub async fn create_session(&self, client_info: ClientInfo) -> Result<MCPSession> {
        debug!("Creating new session for client: {}", client_info.id);
        
        // Check session limits
        self.check_session_limits().await?;
        
        // Create session
        let session = MCPSession {
            session_id: Uuid::new_v4().to_string(),
            client_info,
            context: MCPContext::new(),
            state: SessionState::Initializing,
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            metadata: HashMap::new(),
            config: SessionConfiguration::default(),
            stats: SessionStatistics::default(),
        };
        
        // Store session
        let session_arc = Arc::new(session.clone());
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session.session_id.clone(), session_arc.clone());
        }
        
        // Initialize context
        self.context_manager.initialize_context(&session.session_id).await?;
        
        // Start monitoring
        self.monitor.start_monitoring(&session.session_id).await?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.total_sessions += 1;
            metrics.active_sessions += 1;
        }
        
        info!("Created session: {}", session.session_id);
        Ok(session)
    }
    
    /// Get session by ID
    #[instrument(skip(self))]
    pub async fn get_session(&self, session_id: &str) -> Result<Arc<MCPSession>> {
        debug!("Getting session: {}", session_id);
        
        // Check active sessions first
        {
            let sessions = self.active_sessions.read().await;
            if let Some(session) = sessions.get(session_id) {
                return Ok(session.clone());
            }
        }
        
        // Try to restore from storage
        if let Some(session) = self.storage.load_session(session_id).await? {
            return self.restore_session(session).await;
        }
        
        Err(MCPError::NotFound(format!("Session not found: {}", session_id)))
    }
    
    /// Update session activity
    #[instrument(skip(self))]
    pub async fn update_activity(&self, session_id: &str) -> Result<()> {
        debug!("Updating activity for session: {}", session_id);
        
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            let mut session_mut = Arc::make_mut(session);
            session_mut.last_activity = chrono::Utc::now();
            session_mut.state = SessionState::Active;
            
            // Update metrics
            {
                let mut metrics = self.metrics.lock().await;
                metrics.total_activity_updates += 1;
            }
        }
        
        Ok(())
    }
    
    /// Add message to session context
    #[instrument(skip(self, message))]
    pub async fn add_message(&self, session_id: &str, message: MCPMessage) -> Result<()> {
        debug!("Adding message to session: {}", session_id);
        
        // Update context
        self.context_manager.add_message(session_id, message).await?;
        
        // Update session statistics
        self.update_session_stats(session_id, |stats| {
            stats.total_messages += 1;
        }).await?;
        
        // Update activity
        self.update_activity(session_id).await?;
        
        Ok(())
    }
    
    /// Get session context
    #[instrument(skip(self))]
    pub async fn get_context(&self, session_id: &str) -> Result<MCPContext> {
        debug!("Getting context for session: {}", session_id);
        
        let context = self.context_manager.get_context(session_id).await?;
        Ok(context)
    }
    
    /// Save session
    #[instrument(skip(self))]
    pub async fn save_session(&self, session_id: &str) -> Result<()> {
        debug!("Saving session: {}", session_id);
        
        let session = self.get_session(session_id).await?;
        self.storage.save_session(session.as_ref()).await?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.sessions_saved += 1;
        }
        
        Ok(())
    }
    
    /// Restore session
    #[instrument(skip(self, session))]
    async fn restore_session(&self, session: MCPSession) -> Result<Arc<MCPSession>> {
        debug!("Restoring session: {}", session.session_id);
        
        // Update state
        let mut restored_session = session.clone();
        restored_session.state = SessionState::Restoring;
        
        // Store in active sessions
        let session_arc = Arc::new(restored_session);
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session.session_id.clone(), session_arc.clone());
        }
        
        // Restore context
        self.context_manager.restore_context(&session.session_id, &session.context).await?;
        
        // Update state to active
        {
            let mut sessions = self.active_sessions.write().await;
            if let Some(session) = sessions.get_mut(&session.session_id) {
                let mut session_mut = Arc::make_mut(session);
                session_mut.state = SessionState::Active;
                session_mut.last_activity = chrono::Utc::now();
            }
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.sessions_restored += 1;
            metrics.active_sessions += 1;
        }
        
        info!("Restored session: {}", session.session_id);
        Ok(session_arc)
    }
    
    /// Terminate session
    #[instrument(skip(self))]
    pub async fn terminate_session(&self, session_id: &str) -> Result<()> {
        info!("Terminating session: {}", session_id);
        
        // Save session before termination
        self.save_session(session_id).await?;
        
        // Update state
        {
            let mut sessions = self.active_sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                let mut session_mut = Arc::make_mut(session);
                session_mut.state = SessionState::Terminating;
            }
        }
        
        // Stop monitoring
        self.monitor.stop_monitoring(session_id).await?;
        
        // Clean up context
        self.context_manager.cleanup_context(session_id).await?;
        
        // Remove from active sessions
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.remove(session_id);
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().await;
            metrics.sessions_terminated += 1;
            metrics.active_sessions = metrics.active_sessions.saturating_sub(1);
        }
        
        info!("Terminated session: {}", session_id);
        Ok(())
    }
    
    /// List active sessions
    #[instrument(skip(self))]
    pub async fn list_active_sessions(&self) -> Result<Vec<String>> {
        let sessions = self.active_sessions.read().await;
        let session_ids: Vec<String> = sessions.keys().cloned().collect();
        Ok(session_ids)
    }
    
    /// Get session metrics
    pub async fn get_metrics(&self) -> SessionMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }
    
    // Private methods
    
    async fn check_session_limits(&self) -> Result<()> {
        let sessions = self.active_sessions.read().await;
        if sessions.len() >= self.config.max_sessions {
            return Err(MCPError::ResourceLimitExceeded("Maximum sessions reached".to_string()));
        }
        Ok(())
    }
    
    async fn start_periodic_tasks(&self) -> Result<()> {
        // Start auto-save task
        let active_sessions = self.active_sessions.clone();
        let storage = self.storage.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.auto_save_interval));
            
            loop {
                interval.tick().await;
                
                let sessions = active_sessions.read().await;
                for session in sessions.values() {
                    if let Err(e) = storage.save_session(session.as_ref()).await {
                        error!("Failed to auto-save session {}: {}", session.session_id, e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    async fn save_all_sessions(&self) -> Result<()> {
        let sessions = self.active_sessions.read().await;
        
        for session in sessions.values() {
            if let Err(e) = self.storage.save_session(session.as_ref()).await {
                error!("Failed to save session {}: {}", session.session_id, e);
            }
        }
        
        Ok(())
    }
    
    async fn update_session_stats<F>(&self, session_id: &str, updater: F) -> Result<()>
    where
        F: FnOnce(&mut SessionStatistics),
    {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            let mut session_mut = Arc::make_mut(session);
            updater(&mut session_mut.stats);
        }
        Ok(())
    }
}

// Supporting types and implementations

#[async_trait::async_trait]
pub trait SessionStorage: Send + Sync {
    async fn save_session(&self, session: &MCPSession) -> Result<()>;
    async fn load_session(&self, session_id: &str) -> Result<Option<MCPSession>>;
    async fn delete_session(&self, session_id: &str) -> Result<()>;
    async fn list_sessions(&self) -> Result<Vec<String>>;
}

#[derive(Debug)]
pub struct MemorySessionStorage {
    sessions: Arc<RwLock<HashMap<String, MCPSession>>>,
}

impl MemorySessionStorage {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

#[async_trait::async_trait]
impl SessionStorage for MemorySessionStorage {
    async fn save_session(&self, session: &MCPSession) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session.clone());
        Ok(())
    }
    
    async fn load_session(&self, session_id: &str) -> Result<Option<MCPSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }
    
    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }
    
    async fn list_sessions(&self) -> Result<Vec<String>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.keys().cloned().collect())
    }
}

#[derive(Debug)]
pub struct ContextManager {
    config: Arc<SessionConfig>,
    contexts: Arc<RwLock<HashMap<String, MCPContext>>>,
}

impl ContextManager {
    pub async fn new(config: Arc<SessionConfig>) -> Result<Self> {
        Ok(Self {
            config,
            contexts: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    pub async fn initialize_context(&self, session_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts.insert(session_id.to_string(), MCPContext::new());
        Ok(())
    }
    
    pub async fn get_context(&self, session_id: &str) -> Result<MCPContext> {
        let contexts = self.contexts.read().await;
        contexts.get(session_id)
            .cloned()
            .ok_or_else(|| MCPError::NotFound(format!("Context not found for session: {}", session_id)))
    }
    
    pub async fn add_message(&self, session_id: &str, message: MCPMessage) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(session_id) {
            context.conversation_history.push(message);
            
            // Trim history if it exceeds the limit
            if context.conversation_history.len() > self.config.context_history_limit {
                context.conversation_history.remove(0);
            }
        }
        Ok(())
    }
    
    pub async fn restore_context(&self, session_id: &str, context: &MCPContext) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts.insert(session_id.to_string(), context.clone());
        Ok(())
    }
    
    pub async fn cleanup_context(&self, session_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts.remove(session_id);
        Ok(())
    }
}

#[derive(Debug)]
pub struct SessionMonitor {
    config: Arc<SessionConfig>,
}

impl SessionMonitor {
    pub async fn new(config: Arc<SessionConfig>) -> Result<Self> {
        Ok(Self { config })
    }
    
    pub async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    pub async fn start_monitoring(&self, _session_id: &str) -> Result<()> {
        Ok(())
    }
    
    pub async fn stop_monitoring(&self, _session_id: &str) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct CleanupManager {
    config: Arc<SessionConfig>,
}

impl CleanupManager {
    pub async fn new(config: Arc<SessionConfig>) -> Result<Self> {
        Ok(Self { config })
    }
    
    pub async fn start(&self) -> Result<()> {
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct SessionMetrics {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub sessions_saved: u64,
    pub sessions_restored: u64,
    pub sessions_terminated: u64,
    pub total_activity_updates: u64,
    pub average_session_duration: u64,
}

impl MCPContext {
    pub fn new() -> Self {
        Self {
            conversation_history: Vec::new(),
            tool_results: HashMap::new(),
            user_preferences: UserPreferences::default(),
            plugin_states: HashMap::new(),
            ai_model_state: crate::enhanced::server::AIModelState {
                current_model: None,
                context_length: 4096,
                temperature: 0.7,
                max_tokens: None,
            },
        }
    }
}

impl Default for SessionConfiguration {
    fn default() -> Self {
        Self {
            auto_save: true,
            auto_save_interval: 30,
            context_limit: 1000,
            idle_timeout: 3600,
            persistence: PersistenceConfig {
                enabled: true,
                backend: StorageBackend::Memory,
                compression: false,
                encryption: false,
            },
        }
    }
}

impl Default for SessionStatistics {
    fn default() -> Self {
        Self {
            total_messages: 0,
            tool_executions: 0,
            ai_interactions: 0,
            context_switches: 0,
            duration_seconds: 0,
            data_transferred: 0,
        }
    }
} 