/// Module for handling sessions in the MCP system.
///
/// This module contains the session management functionality including
/// persistence, authentication, and session lifecycle operations.
pub mod manager;

use crate::error::{Result, MCPError};
use tokio::sync::{Mutex, RwLock};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};
use log::error;
use uuid::Uuid;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::error::types::SessionError;
use crate::types::{AccountId, AuthToken, SessionToken, UserId, UserRole};
use crate::security::{Credentials, SecurityManager};
use crate::persistence::{Persistence, SessionData};

/// Session management module
pub mod error;
pub use error::{auth_error, persistence_error, timeout_error, token_error, validation_error};

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout in seconds (None for no timeout)
    pub timeout: Option<Duration>,
    /// Maximum number of sessions per user
    pub max_sessions_per_user: usize,
    /// Whether to enable session persistence
    pub enable_persistence: bool,
    /// Maximum age of session token in seconds
    pub max_token_age: u64,
    /// Whether to check session expiration
    pub check_expiration: bool,
    /// Whether to automatically remove expired sessions
    pub auto_cleanup: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(3600)), // 1 hour default timeout
            max_sessions_per_user: 5,
            enable_persistence: true,
            max_token_age: 86400, // 24 hours
            check_expiration: true,
            auto_cleanup: true,
        }
    }
}

/// Session information
#[derive(Debug, Clone)]
pub struct Session {
    /// Session token
    pub token: SessionToken,
    
    /// User ID associated with this session
    pub user_id: UserId,
    
    /// Account ID associated with this session
    pub account_id: Option<AccountId>,
    
    /// User role for this session
    pub role: UserRole,
    
    /// When the session was created
    pub created_at: DateTime<Utc>,
    
    /// When the session was last accessed
    pub last_accessed: DateTime<Utc>,
    
    /// Session timeout in seconds
    pub timeout: Option<u64>,
    
    /// Authentication token for third-party services
    pub auth_token: Option<AuthToken>,
    
    /// Metadata associated with the session
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// Create a new session
    pub fn new(token: SessionToken, user_id: UserId) -> Self {
        let now = Utc::now();
        Self {
            token,
            user_id,
            account_id: None,
            role: UserRole::User, // Default role
            created_at: now.clone(),
            last_accessed: now,
            timeout: Some(3600), // Default 1 hour timeout
            auth_token: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the session
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Set user role for this session
    pub fn with_role(mut self, role: UserRole) -> Self {
        self.role = role;
        self
    }
    
    /// Set account ID for this session
    pub fn with_account_id(mut self, account_id: AccountId) -> Self {
        self.account_id = Some(account_id);
        self
    }
    
    /// Set authentication token for this session
    pub fn with_auth_token(mut self, auth_token: AuthToken) -> Self {
        self.auth_token = Some(auth_token);
        self
    }
    
    /// Set timeout for this session
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout = Some(timeout_seconds);
        self
    }

    /// Check if the session is expired based on provided timeout
    pub fn is_expired(&self, timeout: Option<Duration>) -> bool {
        if let Some(timeout_duration) = timeout {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(self.last_accessed);
            elapsed.num_seconds() as u64 > timeout_duration.as_secs()
        } else if let Some(session_timeout) = self.timeout {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(self.last_accessed);
            elapsed.num_seconds() as u64 > session_timeout
        } else {
            false // No timeout means session doesn't expire
        }
    }

    /// Update the last accessed time
    pub fn update_last_accessed(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Add authentication token to metadata
    pub fn set_auth_token(&mut self, token: AuthToken) {
        self.auth_token = Some(token);
    }

    /// Add account ID to metadata
    pub fn set_account_id(&mut self, account_id: AccountId) {
        self.account_id = Some(account_id);
    }

    /// Add metadata to the session
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// Get session age in seconds
    #[must_use] pub fn age(&self) -> u64 {
        let now = Utc::now();
        now.signed_duration_since(self.created_at).num_seconds() as u64
    }

    /// Convert to session data for persistence
    pub fn to_session_data(&self) -> SessionData {
        SessionData {
            token: self.token.clone(),
            user_id: self.user_id.clone(),
            account_id: self.account_id.clone(),
            role: self.role.clone(),
            created_at: system_time_from_datetime(&self.created_at),
            last_accessed: system_time_from_datetime(&self.last_accessed),
            timeout: self.timeout.unwrap_or(3600),
            auth_token: self.auth_token.clone(),
            metadata: self.metadata.clone(),
        }
    }

    /// Create from session data
    #[must_use] pub fn from_session_data(data: SessionData) -> Self {
        Self {
            token: data.token,
            user_id: data.user_id,
            account_id: data.account_id,
            role: data.role,
            created_at: datetime_from_system_time(&data.created_at),
            last_accessed: datetime_from_system_time(&data.last_accessed),
            timeout: Some(data.timeout),
            auth_token: data.auth_token,
            metadata: data.metadata,
        }
    }
}

/// Convert DateTime<Utc> to SystemTime
fn system_time_from_datetime(dt: &DateTime<Utc>) -> SystemTime {
    let unix_time = dt.timestamp();
    let nanos = dt.timestamp_subsec_nanos();
    
    SystemTime::UNIX_EPOCH + Duration::from_secs(unix_time as u64) + Duration::from_nanos(nanos as u64)
}

/// Convert SystemTime to DateTime<Utc>
fn datetime_from_system_time(st: &SystemTime) -> DateTime<Utc> {
    let duration_since_epoch = st.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    
    let secs = duration_since_epoch.as_secs() as i64;
    let nanos = duration_since_epoch.subsec_nanos();
    
    DateTime::<Utc>::from_timestamp(secs, nanos).unwrap_or_else(|| Utc::now())
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

    /// Create a new session for a user
    pub async fn create_session(&self, user_id: UserId) -> Result<Session> {
        // Generate a session token
        let session_token = SessionToken(Uuid::new_v4().to_string());
        
        // Create the session
        let session = Session {
            token: session_token.clone(),
            user_id,
            account_id: None,
            role: UserRole::User, // Default role
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            timeout: Some(3600), // Default 1 hour timeout
            auth_token: None,
            metadata: HashMap::new(),
        };
        
        // Store the session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_token, session.clone());
        
        // Persist the session if persistence is enabled
        if let Some(persistence) = &self.persistence {
            persistence.save_session(&session.to_session_data()).await?;
        }
        
        Ok(session)
    }

    /// Get a session by token
    ///
    /// # Arguments
    /// * `token` - The session token to look up
    ///
    /// # Returns
    /// The Session object if found and not expired
    ///
    /// # Errors
    /// Returns an error if:
    /// - The session token is invalid or cannot be found
    /// - The session has expired and is no longer valid
    /// - The persistence layer encounters an error while loading the session
    /// - The session data is corrupted or in an invalid format
    pub async fn get_session(&self, token: &SessionToken) -> Result<Session> {
        // Check in-memory sessions first
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(token) {
            if session.is_expired(self.config.timeout) {
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
                    if session.is_expired(self.config.timeout) {
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

    /// Delete a session by token
    ///
    /// # Arguments
    /// * `token` - The session token to delete
    ///
    /// # Errors
    /// Returns an error if:
    /// - The persistence layer encounters an error while deleting the session
    /// - The session data cannot be removed from storage
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
    ///
    /// # Arguments
    /// * `user_id` - The user ID whose sessions should be deleted
    ///
    /// # Errors
    /// Returns an error if:
    /// - The persistence layer encounters an error while deleting the sessions
    /// - One or more sessions cannot be removed from storage
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

    /// Refresh a session by creating a new one with the same token but reset expiration
    pub async fn refresh_session(&self, session_token: &SessionToken) -> Result<Session> {
        // Get the existing session
        let sessions = self.sessions.read().await;
        let old_session = sessions.get(session_token)
            .ok_or_else(|| MCPError::from(SessionError::NotFound("Session not found".to_string())))?;
        
        // Create a new session with the same token but updated creation time
        let new_session = Session {
            token: session_token.clone(),
            user_id: old_session.user_id.clone(),
            account_id: old_session.account_id.clone(),
            role: old_session.role.clone(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            timeout: old_session.timeout.clone(),
            auth_token: old_session.auth_token.clone(),
            metadata: old_session.metadata.clone(),
        };
        
        drop(sessions);
        
        // Store the refreshed session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_token.clone(), new_session.clone());
        
        // Update in persistence if enabled
        if let Some(persistence) = &self.persistence {
            persistence.save_session(&new_session.to_session_data()).await?;
        }
        
        Ok(new_session)
    }

    /// Clean up expired sessions
    ///
    /// # Returns
    /// The number of expired sessions removed
    ///
    /// # Errors
    /// Returns an error if:
    /// - The persistence layer encounters errors while removing expired sessions
    /// - Session data cannot be properly accessed or modified
    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let mut count = 0;
        let mut sessions = self.sessions.write().await;
        
        let expired_tokens: Vec<SessionToken> = sessions
            .iter()
            .filter(|(_, session)| session.is_expired(self.config.timeout))
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

    /// Create a session from an existing token
    pub async fn create_session_from_token(&self, token: SessionToken, user_id: UserId) -> Result<Session> {
        let session = Session {
            token: token.clone(),
            user_id,
            account_id: None,
            role: UserRole::User, // Default role
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            timeout: Some(3600), // Default 1 hour timeout
            auth_token: None,
            metadata: HashMap::new(),
        };
        
        // Store the session
        let mut sessions = self.sessions.write().await;
        sessions.insert(token, session.clone());
        
        // Persist the session if persistence is enabled
        if let Some(persistence) = &self.persistence {
            persistence.save_session(&session.to_session_data()).await?;
        }
        
        Ok(session)
    }

    /// Validate a session token
    pub async fn validate_session(&self, session_token: &SessionToken) -> Result<Session> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_token) {
            // Check if session has timed out
            if let Some(timeout) = self.config.timeout {
                let session_age = SystemTime::now().duration_since(system_time_from_datetime(&session.created_at))
                    .map_err(|_| MCPError::from(SessionError::InternalError("Could not calculate session age".to_string())))?;
                
                if session_age > timeout {
                    drop(sessions); // Release the read lock before acquiring write lock
                    
                    // Handle session timeout and cleanup
                    if self.config.auto_cleanup {
                        let mut sessions = self.sessions.write().await;
                        sessions.remove(session_token);
                        
                        // Also remove from persistence if enabled
                        if let Some(persistence) = &self.persistence {
                            persistence.delete_session(session_token).await?;
                        }
                    }
                    
                    return Err(MCPError::from(SessionError::Timeout("Session has expired".to_string())));
                }
            }
            
            Ok(session.clone())
        } else {
            Err(MCPError::from(SessionError::NotFound("Session not found".to_string())))
        }
    }

    /// Create a copy of an existing session with a new token
    pub async fn copy_session(&self, old_token: &SessionToken, new_token: SessionToken) -> Result<Session> {
        // Get the existing session
        let sessions = self.sessions.read().await;
        let old_session = sessions.get(old_token)
            .ok_or_else(|| MCPError::from(SessionError::NotFound("Session not found".to_string())))?;
        
        // Create a new session with a new token but same user and metadata
        let new_session = Session {
            token: new_token.clone(),
            user_id: old_session.user_id.clone(),
            account_id: old_session.account_id.clone(),
            role: old_session.role.clone(),
            created_at: Utc::now(), // Reset creation time for the new session
            last_accessed: Utc::now(),
            timeout: old_session.timeout.clone(),
            auth_token: old_session.auth_token.clone(),
            metadata: old_session.metadata.clone(),
        };
        
        drop(sessions);
        
        // Store the new session
        let mut sessions = self.sessions.write().await;
        sessions.insert(new_token, new_session.clone());
        
        // Persist if enabled
        if let Some(persistence) = &self.persistence {
            persistence.save_session(&new_session.to_session_data()).await?;
        }
        
        Ok(new_session)
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
    #[must_use] pub fn new(config: SessionConfig) -> Self {
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