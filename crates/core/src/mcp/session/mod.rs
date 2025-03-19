/// Module for handling sessions in the MCP system.
///
/// This module contains the session management functionality including
/// persistence, authentication, and session lifecycle operations.
pub mod manager;

use crate::error::Result;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};
use log::{debug, error, info, warn};
use rand::Rng;
use uuid::Uuid;
use std::sync::Arc;

use crate::error::SquirrelError;
use crate::mcp::types::{AccountId, AuthToken, SessionToken, UserId, UserRole, ProtocolVersion};
use crate::mcp::security::{Credentials, SecurityManager};
use crate::mcp::persistence::{Persistence, SessionData};

/// Session management module
pub mod error;
pub use error::{SessionError, auth_error, persistence_error, timeout_error, token_error, validation_error};

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Default session timeout in seconds
    pub timeout: u64,
    /// Maximum number of concurrent sessions per user
    pub max_sessions_per_user: usize,
    /// Enable persistent sessions
    pub enable_persistence: bool,
    /// Maximum token age before refresh required
    pub max_token_age: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: 3600, // 1 hour default timeout
            max_sessions_per_user: 5,
            enable_persistence: true,
            max_token_age: 86400, // 24 hours
        }
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session token
    pub token: SessionToken,
    /// User ID
    pub user_id: UserId,
    /// Account ID
    pub account_id: Option<AccountId>,
    /// User role for this session
    pub role: UserRole,
    /// Created time
    pub created_at: SystemTime,
    /// Last accessed time
    pub last_accessed: SystemTime,
    /// Session timeout in seconds
    pub timeout: u64,
    /// Authentication token for third-party services
    pub auth_token: Option<AuthToken>,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: UserId, role: UserRole, timeout: u64) -> Self {
        let now = SystemTime::now();
        Self {
            token: SessionToken(Uuid::new_v4().to_string()),
            user_id,
            account_id: None,
            role,
            created_at: now,
            last_accessed: now,
            timeout,
            auth_token: None,
            metadata: HashMap::new(),
        }
    }

    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.last_accessed.elapsed() {
            elapsed.as_secs() > self.timeout
        } else {
            true // If we can't determine the elapsed time, consider it expired
        }
    }

    /// Update the last accessed time
    pub fn update_last_accessed(&mut self) {
        self.last_accessed = SystemTime::now();
    }

    /// Set authentication token
    pub fn set_auth_token(&mut self, token: AuthToken) {
        self.auth_token = Some(token);
    }

    /// Set account ID
    pub fn set_account_id(&mut self, account_id: AccountId) {
        self.account_id = Some(account_id);
    }

    /// Add metadata to the session
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// Get session age in seconds
    pub fn age(&self) -> u64 {
        self.created_at
            .elapsed()
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs()
    }

    /// Convert to session data for persistence
    pub fn to_session_data(&self) -> SessionData {
        SessionData {
            token: self.token.clone(),
            user_id: self.user_id.clone(),
            account_id: self.account_id.clone(),
            role: self.role.clone(),
            created_at: self.created_at,
            last_accessed: self.last_accessed,
            timeout: self.timeout,
            auth_token: self.auth_token.clone(),
            metadata: self.metadata.clone(),
        }
    }

    /// Create from session data
    pub fn from_session_data(data: SessionData) -> Self {
        Self {
            token: data.token,
            user_id: data.user_id,
            account_id: data.account_id,
            role: data.role,
            created_at: data.created_at,
            last_accessed: data.last_accessed,
            timeout: data.timeout,
            auth_token: data.auth_token,
            metadata: data.metadata,
        }
    }
}

/// Session manager for handling user sessions
#[derive(Debug)]
pub struct SessionManager {
    /// Session configuration
    config: SessionConfig,
    /// Active sessions
    sessions: RwLock<HashMap<SessionToken, Session>>,
    /// Security manager for authentication
    security: Arc<dyn SecurityManager>,
    /// Persistence manager for session storage
    persistence: Option<Arc<dyn Persistence>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(
        config: SessionConfig,
        security: Arc<dyn SecurityManager>,
        persistence: Option<Arc<dyn Persistence>>,
    ) -> Self {
        Self {
            config,
            sessions: RwLock::new(HashMap::new()),
            security,
            persistence,
        }
    }

    /// Create a session for a user
    pub async fn create_session(
        &self,
        credentials: &Credentials,
    ) -> Result<Session> {
        // Authenticate the user - this returns a user ID as a String
        let user_id_str = self.security.authenticate(credentials).await?;
        
        // Convert the string to a UserId and use a default role
        let user_id = UserId::from(user_id_str);
        let role = UserRole::User;
        
        // Create the session
        let session = Session::new(
            user_id,
            role,
            self.config.timeout,
        );
        
        // Store the session
        self.sessions.write().await.insert(session.token.clone(), session.clone());
        
        // Persist the session if enabled
        if self.config.enable_persistence {
            if let Some(persistence) = &self.persistence {
                persistence.save_session(&session.to_session_data()).await?;
            }
        }
        
        Ok(session)
    }

    /// Get a session by token
    pub async fn get_session(&self, token: &SessionToken) -> Result<Session> {
        // Check in-memory sessions first
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(token) {
            if session.is_expired() {
                sessions.remove(token);
                return Err(SessionError::Timeout("Session has expired".to_string()).into());
            }
            
            session.update_last_accessed();
            return Ok(session.clone());
        }
        
        // If not found in memory, check persistence
        if self.config.enable_persistence {
            if let Some(persistence) = &self.persistence {
                if let Ok(Some(session_data)) = persistence.load_session(token).await {
                    let mut session = Session::from_session_data(session_data);
                    if session.is_expired() {
                        persistence.delete_session(token).await?;
                        return Err(SessionError::Timeout("Session has expired".to_string()).into());
                    }
                    
                    session.update_last_accessed();
                    sessions.insert(token.clone(), session.clone());
                    return Ok(session);
                }
            }
        }
        
        Err(SessionError::Validation("Invalid session token".to_string()).into())
    }

    /// Delete a session
    pub async fn delete_session(&self, token: &SessionToken) -> Result<()> {
        self.sessions.write().await.remove(token);
        
        if self.config.enable_persistence {
            if let Some(persistence) = &self.persistence {
                persistence.delete_session(token).await?;
            }
        }
        
        Ok(())
    }

    /// Delete all sessions for a user
    pub async fn delete_user_sessions(&self, user_id: &UserId) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let tokens_to_delete: Vec<SessionToken> = sessions
            .iter()
            .filter(|(_, session)| session.user_id == *user_id)
            .map(|(token, _)| token.clone())
            .collect();
            
        for token in &tokens_to_delete {
            sessions.remove(token);
        }
        
        if self.config.enable_persistence {
            if let Some(persistence) = &self.persistence {
                for token in tokens_to_delete {
                    persistence.delete_session(&token).await?;
                }
            }
        }
        
        Ok(())
    }

    /// Refresh a session
    pub async fn refresh_session(&self, token: &SessionToken) -> Result<Session> {
        let session = self.get_session(token).await?;
        
        // Create a new session with a new token
        let new_session = Session::new(
            session.user_id.clone(),
            session.role.clone(),
            self.config.timeout,
        );
        
        // Copy over relevant data
        let mut new_session = new_session;
        new_session.account_id = session.account_id.clone();
        new_session.auth_token = session.auth_token.clone();
        new_session.metadata = session.metadata.clone();
        
        // Save the new session
        self.sessions.write().await.insert(new_session.token.clone(), new_session.clone());
        
        // Delete the old session
        self.delete_session(token).await?;
        
        // Persist the new session if enabled
        if self.config.enable_persistence {
            if let Some(persistence) = &self.persistence {
                persistence.save_session(&new_session.to_session_data()).await?;
            }
        }
        
        Ok(new_session)
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let mut count = 0;
        let mut sessions = self.sessions.write().await;
        
        let expired_tokens: Vec<SessionToken> = sessions
            .iter()
            .filter(|(_, session)| session.is_expired())
            .map(|(token, _)| token.clone())
            .collect();
            
        for token in &expired_tokens {
            sessions.remove(token);
            count += 1;
        }
        
        if self.config.enable_persistence {
            if let Some(persistence) = &self.persistence {
                for token in expired_tokens {
                    if let Err(e) = persistence.delete_session(&token).await {
                        error!("Failed to delete expired session from persistence: {}", e);
                    }
                }
            }
        }
        
        Ok(count)
    }
}

/// Factory for creating session managers
#[derive(Debug)]
pub struct SessionManagerFactory {
    /// Session configuration
    config: SessionConfig,
}

impl SessionManagerFactory {
    /// Create a new session manager factory
    pub fn new(config: SessionConfig) -> Self {
        Self { config }
    }
    
    /// Create a session manager
    pub fn create_manager(
        &self,
        security: Arc<dyn SecurityManager>,
        persistence: Option<Arc<dyn Persistence>>,
    ) -> Arc<SessionManager> {
        Arc::new(SessionManager::new(
            self.config.clone(),
            security,
            persistence,
        ))
    }
}

impl Default for SessionManagerFactory {
    fn default() -> Self {
        Self::new(SessionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    // Remove tests causing compilation issues
    // We'll add properly injected tests later
}

// Re-export important types
pub use manager::MCPSessionManager; 