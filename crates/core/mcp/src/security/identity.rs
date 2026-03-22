// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Identity management for MCP security
//!
//! This module provides identity management functionality for the MCP system.
//! Actual identity operations are delegated to the BearDog framework.

use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// User identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub active: bool,
}

/// Default identity manager implementation
///
/// This provides basic identity management that can be extended
/// or replaced with BearDog integration in the future.
#[derive(Debug, Clone)]
pub struct DefaultIdentityManager {
    identities: Arc<RwLock<HashMap<Uuid, UserIdentity>>>,
}

impl DefaultIdentityManager {
    /// Create a new identity manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            identities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new user identity
    pub async fn create_identity(
        &self,
        username: String,
        email: Option<String>,
    ) -> Result<UserIdentity> {
        let identity = UserIdentity {
            id: Uuid::new_v4(),
            username,
            email,
            roles: vec!["user".to_string()],
            created_at: Utc::now(),
            last_login: None,
            active: true,
        };

        let mut identities = self.identities.write().await;
        identities.insert(identity.id, identity.clone());
        Ok(identity)
    }

    /// Get user identity by ID
    pub async fn get_identity(&self, id: &Uuid) -> Result<Option<UserIdentity>> {
        let identities = self.identities.read().await;
        Ok(identities.get(id).cloned())
    }

    /// Get user identity by username
    pub async fn get_identity_by_username(&self, username: &str) -> Result<Option<UserIdentity>> {
        let identities = self.identities.read().await;
        Ok(identities
            .values()
            .find(|i| i.username == username)
            .cloned())
    }

    /// Update user identity
    pub async fn update_identity(&self, identity: UserIdentity) -> Result<()> {
        let mut identities = self.identities.write().await;
        identities.insert(identity.id, identity);
        Ok(())
    }

    /// Delete user identity
    pub async fn delete_identity(&self, id: &Uuid) -> Result<()> {
        let mut identities = self.identities.write().await;
        identities.remove(id);
        Ok(())
    }

    /// Authenticate user
    pub async fn authenticate(
        &self,
        username: &str,
        _password: &str,
    ) -> Result<Option<UserIdentity>> {
        // Placeholder implementation - delegate to BearDog
        self.get_identity_by_username(username).await
    }

    /// Record login
    pub async fn record_login(&self, id: &Uuid) -> Result<()> {
        let mut identities = self.identities.write().await;
        if let Some(identity) = identities.get_mut(id) {
            identity.last_login = Some(Utc::now());
        }
        Ok(())
    }
}

impl Default for DefaultIdentityManager {
    fn default() -> Self {
        Self::new()
    }
}
