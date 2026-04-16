// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Enum dispatch for [`crate::security::traits::UniversalSecurityProvider`] and
//! [`UniversalSecurityService`] (replaces `dyn` + `async_trait`).

use std::sync::Arc;

use crate::security::context::SecurityContext;
use crate::security::errors::SecurityError;
use crate::security::traits::UniversalSecurityProvider;
use crate::traits::{AuthResult, Credentials, Principal};

use super::types::{
    SecurityCapability, SecurityHealth as ServiceHealth, SecurityRequest, SecurityResponse,
    SecurityServiceConfig, SecurityServiceInfo, UniversalSecurityService,
};
use super::{LocalSecurityProvider, SecurityProviderIntegration};

/// Runtime-selected universal security provider (security-provider primary, local fallback, etc.).
///
/// The same enum also implements [`UniversalSecurityService`] — use [`UniversalSecurityServiceBox`].
#[derive(Clone)]
pub enum UniversalSecurityProviderBox {
    /// Security-provider integration (IPC or local fallback).
    SecurityProvider(Arc<SecurityProviderIntegration>),
    /// Local fallback provider.
    Local(Arc<LocalSecurityProvider>),
}

/// Type alias for [`UniversalSecurityProviderBox`] when used as a [`UniversalSecurityService`].
pub type UniversalSecurityServiceBox = UniversalSecurityProviderBox;

impl UniversalSecurityProvider for UniversalSecurityProviderBox {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthResult, SecurityError> {
        match self {
            Self::SecurityProvider(p) => p.authenticate(credentials).await,
            Self::Local(p) => p.authenticate(credentials).await,
        }
    }

    async fn authorize(
        &self,
        principal: &Principal,
        action: &str,
        resource: &str,
    ) -> Result<bool, SecurityError> {
        match self {
            Self::SecurityProvider(p) => p.authorize(principal, action, resource).await,
            Self::Local(p) => p.authorize(principal, action, resource).await,
        }
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        match self {
            Self::SecurityProvider(p) => p.encrypt(data).await,
            Self::Local(p) => p.encrypt(data).await,
        }
    }

    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        match self {
            Self::SecurityProvider(p) => p.decrypt(encrypted_data).await,
            Self::Local(p) => p.decrypt(encrypted_data).await,
        }
    }

    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        match self {
            Self::SecurityProvider(p) => p.sign(data).await,
            Self::Local(p) => p.sign(data).await,
        }
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SecurityError> {
        match self {
            Self::SecurityProvider(p) => p.verify(data, signature).await,
            Self::Local(p) => p.verify(data, signature).await,
        }
    }

    async fn audit_log(
        &self,
        operation: &str,
        context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        match self {
            Self::SecurityProvider(p) => p.audit_log(operation, context).await,
            Self::Local(p) => p.audit_log(operation, context).await,
        }
    }

    async fn health_check(
        &self,
    ) -> Result<crate::security::context::SecurityHealth, SecurityError> {
        match self {
            Self::SecurityProvider(p) => UniversalSecurityProvider::health_check(p.as_ref()).await,
            Self::Local(p) => UniversalSecurityProvider::health_check(p.as_ref()).await,
        }
    }
}

impl UniversalSecurityService for UniversalSecurityProviderBox {
    fn get_capabilities(&self) -> Vec<SecurityCapability> {
        match self {
            Self::SecurityProvider(p) => p.get_capabilities(),
            Self::Local(p) => p.get_capabilities(),
        }
    }

    fn get_service_info(&self) -> SecurityServiceInfo {
        match self {
            Self::SecurityProvider(p) => p.get_service_info(),
            Self::Local(p) => p.get_service_info(),
        }
    }

    async fn handle_security_request(
        &self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, SecurityError> {
        match self {
            Self::SecurityProvider(p) => p.handle_security_request(request).await,
            Self::Local(p) => p.handle_security_request(request).await,
        }
    }

    async fn health_check(&self) -> Result<ServiceHealth, SecurityError> {
        match self {
            Self::SecurityProvider(p) => UniversalSecurityService::health_check(p.as_ref()).await,
            Self::Local(p) => UniversalSecurityService::health_check(p.as_ref()).await,
        }
    }

    async fn initialize(&mut self, config: SecurityServiceConfig) -> Result<(), SecurityError> {
        match self {
            Self::SecurityProvider(p) => {
                let Some(inner) = Arc::get_mut(p) else {
                    return Err(SecurityError::configuration(
                        "Cannot initialize shared security provider integration",
                    ));
                };
                inner.initialize(config).await
            }
            Self::Local(p) => {
                let Some(inner) = Arc::get_mut(p) else {
                    return Err(SecurityError::configuration(
                        "Cannot initialize shared local security service",
                    ));
                };
                inner.initialize(config).await
            }
        }
    }
}
