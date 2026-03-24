// SPDX-License-Identifier: AGPL-3.0-or-later
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

/// User identity information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    /// Primary key for the user record.
    pub id: Uuid,
    /// Unique login or display name.
    pub username: String,
    /// Optional email address.
    pub email: Option<String>,
    /// Role names assigned to the user.
    pub roles: Vec<String>,
    /// When the identity was created.
    pub created_at: DateTime<Utc>,
    /// Last successful login time, if any.
    pub last_login: Option<DateTime<Utc>>,
    /// Whether the account may authenticate.
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

    /// Creates and stores a new user with default role `user`.
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

    /// Returns the identity for the given id, if it exists.
    pub async fn get_identity(&self, id: &Uuid) -> Result<Option<UserIdentity>> {
        let identities = self.identities.read().await;
        Ok(identities.get(id).cloned())
    }

    /// Looks up an identity by username.
    pub async fn get_identity_by_username(&self, username: &str) -> Result<Option<UserIdentity>> {
        let identities = self.identities.read().await;
        Ok(identities
            .values()
            .find(|i| i.username == username)
            .cloned())
    }

    /// Replaces the stored identity with the given record.
    pub async fn update_identity(&self, identity: UserIdentity) -> Result<()> {
        let mut identities = self.identities.write().await;
        identities.insert(identity.id, identity);
        Ok(())
    }

    /// Removes the identity from the store.
    pub async fn delete_identity(&self, id: &Uuid) -> Result<()> {
        let mut identities = self.identities.write().await;
        identities.remove(id);
        Ok(())
    }

    /// Resolves credentials to an identity (placeholder until BearDog integration).
    pub async fn authenticate(
        &self,
        username: &str,
        _password: &str,
    ) -> Result<Option<UserIdentity>> {
        // Placeholder implementation - delegate to BearDog
        self.get_identity_by_username(username).await
    }

    /// Sets `last_login` to the current time for the given user id.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_identity_serde_round_trip() {
        let i = UserIdentity {
            id: Uuid::new_v4(),
            username: "u".to_string(),
            email: None,
            roles: vec![],
            created_at: chrono::Utc::now(),
            last_login: None,
            active: false,
        };
        let json = serde_json::to_string(&i).expect("identity serializes");
        let back: UserIdentity = serde_json::from_str(&json).expect("identity deserializes");
        assert_eq!(back.id, i.id);
        assert_eq!(back.username, i.username);
        assert_eq!(back.email, i.email);
        assert_eq!(back.active, i.active);
    }

    #[tokio::test]
    async fn create_get_update_delete_authenticate_record_login() {
        let m = DefaultIdentityManager::new();
        let _ = DefaultIdentityManager::default();

        let id = m
            .create_identity("alice".to_string(), Some("a@b.c".to_string()))
            .await
            .expect("create_identity");
        assert_eq!(id.username, "alice");
        assert_eq!(id.roles, vec!["user".to_string()]);
        assert!(id.active);

        let empty = m
            .create_identity(String::new(), None)
            .await
            .expect("create_identity empty username");
        assert_eq!(empty.username, "");

        assert_eq!(
            m.get_identity(&id.id)
                .await
                .expect("get_identity")
                .expect("identity exists")
                .username,
            "alice"
        );
        assert!(
            m.get_identity(&Uuid::new_v4())
                .await
                .expect("get_identity")
                .is_none()
        );

        assert_eq!(
            m.get_identity_by_username("alice")
                .await
                .expect("get_identity_by_username")
                .expect("alice exists")
                .id,
            id.id
        );
        assert!(
            m.get_identity_by_username("nobody")
                .await
                .expect("get_identity_by_username")
                .is_none()
        );

        let mut upd = m
            .get_identity(&id.id)
            .await
            .expect("get_identity")
            .expect("identity exists");
        upd.email = Some("new@x.y".to_string());
        m.update_identity(upd).await.expect("update_identity");
        assert_eq!(
            m.get_identity(&id.id)
                .await
                .expect("get_identity")
                .expect("identity exists")
                .email
                .as_deref(),
            Some("new@x.y")
        );

        assert_eq!(
            m.authenticate("alice", "any")
                .await
                .expect("authenticate")
                .expect("alice authenticated")
                .id,
            id.id
        );
        assert!(
            m.authenticate("unknown", "pw")
                .await
                .expect("authenticate")
                .is_none()
        );

        m.record_login(&id.id).await.expect("record_login");
        assert!(
            m.get_identity(&id.id)
                .await
                .expect("get_identity")
                .expect("identity exists")
                .last_login
                .is_some()
        );

        m.record_login(&Uuid::new_v4())
            .await
            .expect("record_login noop");

        m.delete_identity(&id.id).await.expect("delete_identity");
        assert!(
            m.get_identity(&id.id)
                .await
                .expect("get_identity")
                .is_none()
        );
        assert!(
            m.authenticate("alice", "x")
                .await
                .expect("authenticate")
                .is_none()
        );
    }
}
