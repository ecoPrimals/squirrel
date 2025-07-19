use crate::AuthError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub active: bool,
}

impl Session {
    pub fn new(user_id: Uuid, username: String, expires_at: DateTime<Utc>) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            user_id,
            username,
            expires_at,
            created_at: now,
            last_accessed: now,
            active: true,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        self.active && !self.is_expired()
    }

    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }

    pub fn invalidate(&mut self) {
        self.active = false;
    }
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    user_sessions: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        username: String,
        expires_at: DateTime<Utc>,
    ) -> Result<Session, AuthError> {
        let session = Session::new(user_id, username, expires_at);
        let session_id = session.id;

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session.clone());
        }

        // Track user sessions
        {
            let mut user_sessions = self.user_sessions.write().await;
            user_sessions.entry(user_id).or_default().push(session_id);
        }

        Ok(session)
    }

    pub async fn get_session(&self, session_id: &Uuid) -> Result<Option<Session>, AuthError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    pub async fn update_session_access(&self, session_id: &Uuid) -> Result<(), AuthError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.touch();
        }
        Ok(())
    }

    pub async fn invalidate_session(&self, session_id: &Uuid) -> Result<(), AuthError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.invalidate();
        }
        Ok(())
    }

    pub async fn invalidate_user_sessions(&self, user_id: &Uuid) -> Result<(), AuthError> {
        let user_sessions = self.user_sessions.read().await;
        if let Some(session_ids) = user_sessions.get(user_id) {
            let mut sessions = self.sessions.write().await;
            for session_id in session_ids {
                if let Some(session) = sessions.get_mut(session_id) {
                    session.invalidate();
                }
            }
        }
        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u32, AuthError> {
        let mut removed_count = 0;
        let now = Utc::now();

        // Clean up expired sessions
        {
            let mut sessions = self.sessions.write().await;
            sessions.retain(|_, session| {
                if session.expires_at <= now {
                    removed_count += 1;
                    false
                } else {
                    true
                }
            });
        }

        // Clean up user session references
        {
            let mut user_sessions = self.user_sessions.write().await;
            let sessions = self.sessions.read().await;

            for (_, session_ids) in user_sessions.iter_mut() {
                session_ids.retain(|session_id| sessions.contains_key(session_id));
            }

            // Remove empty user session lists
            user_sessions.retain(|_, session_ids| !session_ids.is_empty());
        }

        Ok(removed_count)
    }

    pub async fn get_user_sessions(&self, user_id: &Uuid) -> Result<Vec<Session>, AuthError> {
        let user_sessions = self.user_sessions.read().await;
        let sessions = self.sessions.read().await;

        let mut result = Vec::new();
        if let Some(session_ids) = user_sessions.get(user_id) {
            for session_id in session_ids {
                if let Some(session) = sessions.get(session_id) {
                    if session.is_valid() {
                        result.push(session.clone());
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn get_active_session_count(&self) -> Result<usize, AuthError> {
        let sessions = self.sessions.read().await;
        let count = sessions.values().filter(|s| s.is_valid()).count();
        Ok(count)
    }

    pub async fn get_session_stats(&self) -> Result<SessionStats, AuthError> {
        let sessions = self.sessions.read().await;
        let now = Utc::now();

        let total_sessions = sessions.len();
        let active_sessions = sessions.values().filter(|s| s.is_valid()).count();
        let expired_sessions = sessions.values().filter(|s| s.is_expired()).count();
        let inactive_sessions = sessions.values().filter(|s| !s.active).count();

        Ok(SessionStats {
            total_sessions,
            active_sessions,
            expired_sessions,
            inactive_sessions,
            timestamp: now,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub expired_sessions: usize,
    pub inactive_sessions: usize,
    pub timestamp: DateTime<Utc>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
