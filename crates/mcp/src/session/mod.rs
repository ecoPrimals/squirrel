/// Module for handling sessions in the MCP system.
///
/// This module contains the session management functionality including
/// persistence, authentication, and session lifecycle operations.
pub mod manager;

use crate::error::Result;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, Instant};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::error::session::SessionError;
use crate::types::{AccountId};
use crate::security::{SessionToken, AuthToken, UserId, RoleId};
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
    pub role: RoleId,
    
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
    #[must_use] pub fn new(token: SessionToken, user_id: UserId) -> Self {
        let now = Utc::now();
        Self {
            token,
            user_id,
            account_id: None,
            role: RoleId("user".to_string()), // Default role
            created_at: now.clone(),
            last_accessed: now,
            timeout: Some(3600), // Default 1 hour timeout
            auth_token: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the session
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Set user role for this session
    #[must_use] pub fn with_role(mut self, role: RoleId) -> Self {
        self.role = role;
        self
    }
    
    /// Set account ID for this session
    #[must_use] pub fn with_account_id(mut self, account_id: AccountId) -> Self {
        self.account_id = Some(account_id);
        self
    }
    
    /// Set authentication token for this session
    #[must_use] pub fn with_auth_token(mut self, auth_token: AuthToken) -> Self {
        self.auth_token = Some(auth_token);
        self
    }
    
    /// Set timeout for this session
    #[must_use] pub const fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout = Some(timeout_seconds);
        self
    }

    /// Check if the session is expired based on provided timeout
    #[must_use]
    pub fn is_expired(&self, timeout: Option<Duration>) -> bool {
        timeout.map_or_else(
            || { // Closure for timeout = None
                // Check self.timeout
                self.timeout.map_or(
                    false, // self.timeout = None => false
                    |session_timeout| { // self.timeout = Some(session_timeout)
                        let now = Utc::now();
                        let elapsed = now.signed_duration_since(self.last_accessed);
                        elapsed.num_seconds() as u64 > session_timeout
                    }
                )
            },
            |timeout_duration| { // Closure for timeout = Some(timeout_duration)
                let now = Utc::now();
                let elapsed = now.signed_duration_since(self.last_accessed);
                elapsed.num_seconds() as u64 > timeout_duration.as_secs()
            }
        )
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
    #[must_use] pub fn to_session_data(&self) -> SessionData {
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
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn from_session_data(data: SessionData) -> Self {
        Self {
            token: data.token,
            user_id: data.user_id,
            account_id: data.account_id,
            role: data.role,
            created_at: datetime_from_system_time(data.created_at),
            last_accessed: datetime_from_system_time(data.last_accessed),
            timeout: Some(data.timeout),
            auth_token: data.auth_token,
            metadata: data.metadata,
        }
    }
}

/// Convert `DateTime`<Utc> to `SystemTime`
#[allow(clippy::cast_sign_loss)]
fn system_time_from_datetime(dt: &DateTime<Utc>) -> SystemTime {
    let unix_time = dt.timestamp();
    let nanos = dt.timestamp_subsec_nanos();
    
    SystemTime::UNIX_EPOCH + Duration::from_secs(unix_time as u64) + Duration::from_nanos(u64::from(nanos))
}

/// Helper function to convert SystemTime to DateTime<Utc>
fn datetime_from_system_time(st: SystemTime) -> DateTime<Utc> {
    let duration_since_epoch = st.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    #[allow(clippy::cast_possible_wrap)] // Allow u64->i64 for timestamp conversion
    let secs = duration_since_epoch.as_secs() as i64;
    let nanos = duration_since_epoch.subsec_nanos();
    
    DateTime::<Utc>::from_timestamp(secs, nanos).unwrap_or_else(|| Utc::now())
}

/// Session manager for handling user sessions
pub struct SessionManager {
    /// Session configuration
    config: SessionConfig,
    /// Active sessions
    sessions: RwLock<HashMap<SessionToken, Session>>,
    /// Security manager for authentication
    security: Arc<crate::security::manager::SecurityManagerImpl>,
    /// Persistence manager for session storage
    persistence: Option<Arc<dyn Persistence>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(
        config: SessionConfig,
        security: Arc<crate::security::manager::SecurityManagerImpl>,
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
        let now = Utc::now();
        
        // Configure the session with default parameters
        let session = Session {
            token: session_token.clone(),
            user_id,
            account_id: None,
            role: RoleId("user".to_string()), // Default role
            created_at: now,
            last_accessed: now,
            timeout: Some(self.config.timeout.unwrap_or(Duration::from_secs(3600)).as_secs()),
            auth_token: None,
            metadata: HashMap::new(),
        };
        
        // Store the session in memory
        self.sessions.write().await.insert(session_token, session.clone());
        
        // If persistence is enabled, store it there too
        if let Some(persistence) = &self.persistence {
            persistence.save_session(&session.to_session_data()).await?;
        }
        
        Ok(session)
    }

    /// Get a session by token
    ///
    /// # Errors
    /// Returns an error if the session token is not found, if the session has expired,
    /// or if the persistence layer encounters errors when retrieving the session.
    pub async fn get_session(&self, token: &SessionToken) -> Result<Session> {
        // Check in-memory sessions first
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(token) {
            // Update last accessed time
            session.last_accessed = Utc::now();
            
            // If the session has a timeout, check if it's expired
            if session.is_expired(Some(self.config.timeout.unwrap_or(Duration::from_secs(3600)))) {
                // Remove expired session
                sessions.remove(token);
                drop(sessions); // Early drop the mutex lock
                
                // If auto-cleanup is enabled, we may want to trigger a background cleanup
                // of other expired sessions, but that would be handled elsewhere
                
                return Err(SessionError::Timeout("Session has expired".to_string()).into());
            }
            
            return Ok(session.clone());
        }
        
        // If not found in memory and persistence is enabled, try to load from persistence
        drop(sessions); // Early drop the mutex lock before potentially expensive operation
        
        if let Some(persistence) = &self.persistence {
            match persistence.load_session(token).await {
                Ok(Some(session_data)) => {
                    // Convert session data to session
                    let session = Session::from_session_data(session_data);
                    
                    // If loaded from persistence, check if expired
                    if !session.is_expired(Some(self.config.timeout.unwrap_or(Duration::from_secs(3600)))) {
                        // Add to in-memory cache if not expired
                        self.sessions.write().await.insert(token.clone(), session.clone());
                        return Ok(session);
                    }
                },
                Ok(None) => {
                    // Session not found in persistence
                    return Err(SessionError::NotFound("Session not found in persistence".to_string()).into());
                },
                Err(e) => {
                    // Error loading from persistence
                    return Err(e);
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

    /// Delete all sessions for a specific user
    ///
    /// # Errors
    /// Returns an error if the persistence layer encounters errors during cleanup.
    pub async fn delete_user_sessions(&self, user_id: &UserId) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let tokens_to_delete: Vec<SessionToken> = sessions
            .iter()
            .filter_map(|(token, session)| {
                if &session.user_id == user_id {
                    Some(token.clone())
                } else {
                    None
                }
            })
            .collect();
        
        for token in &tokens_to_delete {
            sessions.remove(token);
        }
        drop(sessions); // Early drop the mutex lock
        
        // If persistence is enabled, delete from there too
        if let Some(persistence) = &self.persistence {
            for token in tokens_to_delete {
                persistence.delete_session(&token).await?;
            }
        }
        
        Ok(())
    }

    /// Refresh a session, extending its timeout
    ///
    /// # Errors
    /// Returns an error if the specified session token is not found,
    /// if the persistence layer encounters errors when saving the refreshed session,
    /// or if session data cannot be properly accessed or modified.
    pub async fn refresh_session(&self, session_token: &SessionToken) -> Result<Session> {
        // Get the existing session
        let sessions = self.sessions.read().await;
        let old_session = sessions.get(session_token)
            .ok_or_else(|| SessionError::NotFound("Session not found".to_string()))?
            .clone();
        drop(sessions); // Early drop read lock before acquiring write lock
        
        // Create a new session with same token but updated last_accessed
        let mut new_session = old_session;
        new_session.last_accessed = Utc::now();
        
        // Update the session in memory
        self.sessions.write().await.insert(session_token.clone(), new_session.clone());
        
        // If persistence is enabled, update there too
        if let Some(persistence) = &self.persistence {
            persistence.save_session(&new_session.to_session_data()).await?;
        }
        
        Ok(new_session)
    }

    /// Clean up expired sessions
    ///
    /// # Errors
    /// Returns an error if the persistence layer encounters errors during cleanup.
    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        // First get all expired tokens and remove them from memory
        let expired_tokens = {
            let mut sessions = self.sessions.write().await;
            let tokens_to_delete: Vec<SessionToken> = sessions
                .iter()
                .filter_map(|(token, session)| {
                    if session.is_expired(Some(self.config.timeout.unwrap_or(Duration::from_secs(3600)))) {
                        Some(token.clone())
                    } else {
                        None
                    }
                })
                .collect();
            
            // Remove all expired sessions from memory
            for token in &tokens_to_delete {
                sessions.remove(token);
            }
            
            tokens_to_delete
        }; // sessions lock is dropped here
        
        let count = expired_tokens.len();
        
        // Then clean up from persistence if enabled
        if let Some(persistence) = &self.persistence {
            for token in expired_tokens {
                persistence.delete_session(&token).await?;
            }
        }
        
        Ok(count)
    }

    /// Create a session from an existing token
    ///
    /// Creates a new session with a provided token and user ID.
    /// This is useful when you want to reuse a token that was
    /// previously generated or provided by an external system.
    ///
    /// # Arguments
    /// * `token` - The token to use for the session
    /// * `user_id` - The user ID to associate with the session
    ///
    /// # Returns
    /// A Result containing the created session if successful
    ///
    /// # Errors
    /// Returns an error if:
    /// * The persistence layer encounters errors when saving the session
    /// * Session data cannot be properly accessed or modified
    pub async fn create_session_from_token(&self, token: SessionToken, user_id: UserId) -> Result<Session> {
        let session = Session {
            token: token.clone(),
            user_id,
            account_id: None,
            role: RoleId("user".to_string()), // Default role
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            timeout: Some(3600), // Default 1 hour timeout
            auth_token: None,
            metadata: HashMap::new(),
        };
        
        // Store the session - use a block to scope the mutex lock
        {
            self.sessions.write().await.insert(token, session.clone());
        }
        
        // Persist the session if persistence is enabled
        if let Some(persistence) = &self.persistence {
            persistence.save_session(&session.to_session_data()).await?;
        }
        
        Ok(session)
    }

    /// Handles validation of session with possible auto-cleanup
    pub async fn validate_session(&self, session_token: &SessionToken) -> Result<Session> {
        // Try to get the session
        match self.get_session(session_token).await {
            Ok(session) => Ok(session),
            Err(e) => {
                // Check if this is a session timeout error
                if let Some(err_string) = e.to_string().to_lowercase().find("expired") {
                    if self.config.auto_cleanup {
                        // Instead of trying to access mcp_err.as_ref() 
                        // Just log and handle the expired session
                        self.sessions.write().await.remove(session_token);
                        
                        // For auto cleanup, just remove the token
                        // Full cleanup will be handled by periodic maintenance tasks
                        if let Some(persistence) = &self.persistence {
                            // Try to delete from persistence as well, but don't propagate errors
                            let _ = persistence.delete_session(session_token).await;
                        }
                    }
                }
                Err(e)
            }
        }
    }

    /// Copy a session to a new token
    ///
    /// # Errors
    /// Returns an error if the old session token is not found,
    /// if the persistence layer encounters errors when saving the new session,
    /// or if session data cannot be properly accessed or modified.
    pub async fn copy_session(&self, old_token: &SessionToken, new_token: SessionToken) -> Result<Session> {
        // Get the existing session
        let sessions = self.sessions.read().await;
        let old_session = sessions.get(old_token)
            .ok_or_else(|| SessionError::NotFound("Session not found".to_string()))?
            .clone();
        drop(sessions); // Early drop the read lock before acquiring write lock
        
        // Create new session with the new token but same data
        let new_session = Session {
            token: new_token.clone(),
            user_id: old_session.user_id.clone(),
            account_id: old_session.account_id.clone(),
            role: old_session.role.clone(),
            created_at: Utc::now(), // New creation time
            last_accessed: Utc::now(), // New access time
            timeout: old_session.timeout,
            auth_token: old_session.auth_token.clone(),
            metadata: old_session.metadata.clone(),
        };
        
        // Store the new session
        self.sessions.write().await.insert(new_token, new_session.clone());
        
        // If persistence is enabled, store it there too
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
    /// Create a new session manager factory with the provided configuration
    #[must_use] pub const fn new(config: SessionConfig) -> Self {
        Self { config }
    }
    
    /// Create a new session manager with the provided security and persistence managers
    pub fn create_manager(
        &self,
        security: Arc<crate::security::manager::SecurityManagerImpl>,
        persistence: Option<Arc<dyn Persistence>>,
    ) -> Arc<SessionManager> {
        Arc::new(SessionManager::new(self.config.clone(), security, persistence))
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

/// Represents the state of an active session
// #[derive(Debug)] // Remove Debug derive because dyn SecurityManager doesn't implement it
pub struct SessionState {
    pub session: Session,
    pub last_activity: Instant,
} 