use std::fmt;
use serde::{Deserialize, Serialize};
use crate::core::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub level: SecurityLevel,
    pub encryption_enabled: bool,
    pub authentication_enabled: bool,
    pub authorization_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            level: SecurityLevel::Medium,
            encryption_enabled: true,
            authentication_enabled: true,
            authorization_enabled: true,
        }
    }
}

pub struct SecurityManager {
    config: SecurityConfig,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: SecurityConfig) {
        self.config = config;
    }

    pub fn is_encryption_enabled(&self) -> bool {
        self.config.encryption_enabled
    }

    pub fn is_authentication_enabled(&self) -> bool {
        self.config.authentication_enabled
    }

    pub fn is_authorization_enabled(&self) -> bool {
        self.config.authorization_enabled
    }
} 