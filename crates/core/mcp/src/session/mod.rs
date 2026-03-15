// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Session management for MCP
//!
//! Core session handling functionality for the MCP protocol.
//! Complex authentication and authorization moved to BearDog.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;
use crate::protocol::types::{MCPMessage, MessageId};

// Remove imports for types moved to other projects:
// SessionToken, UserId, RoleId, AuthToken, SessionId → Moved to BearDog

/// Basic session information for core MCP functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Session management interface
pub trait SessionManager: Send + Sync {
    /// Create a new session
    fn create_session(&self, metadata: HashMap<String, String>) -> impl Future<Output = Result<Session>> + Send;
    
    /// Get session by ID
    fn get_session(&self, id: &str) -> impl Future<Output = Result<Option<Session>>> + Send;
    
    /// Update session activity
    fn update_activity(&self, id: &str) -> impl Future<Output = Result<()>> + Send;
    
    /// Remove session
    fn remove_session(&self, id: &str) -> impl Future<Output = Result<()>> + Send;
}

/// Simple session manager implementation
pub struct CoreSessionManager {
    sessions: Arc<tokio::sync::RwLock<HashMap<String, Session>>>,
}

impl CoreSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

impl SessionManager for CoreSessionManager {
    fn create_session(&self, metadata: HashMap<String, String>) -> impl Future<Output = Result<Session>> + Send {
        let sessions = self.sessions.clone();
        async move {
            let session = Session {
                id: Uuid::new_v4().to_string(),
                created_at: SystemTime::now(),
                last_activity: SystemTime::now(),
                metadata,
            };
            
            let mut sessions = sessions.write().await;
            sessions.insert(session.id.clone(), session.clone());
            
            Ok(session)
        }
    }
    
    fn get_session(&self, id: &str) -> impl Future<Output = Result<Option<Session>>> + Send {
        let sessions = self.sessions.clone();
        let id = id.to_string();
        async move {
            let sessions = sessions.read().await;
            Ok(sessions.get(&id).cloned())
        }
    }
    
    fn update_activity(&self, id: &str) -> impl Future<Output = Result<()>> + Send {
        let sessions = self.sessions.clone();
        let id = id.to_string();
        async move {
            let mut sessions = sessions.write().await;
            if let Some(session) = sessions.get_mut(&id) {
                session.last_activity = SystemTime::now();
            }
            Ok(())
        }
    }
    
    fn remove_session(&self, id: &str) -> impl Future<Output = Result<()>> + Send {
        let sessions = self.sessions.clone();
        let id = id.to_string();
        async move {
            let mut sessions = sessions.write().await;
            sessions.remove(&id);
            Ok(())
        }
    }
} 