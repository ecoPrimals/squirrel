use std::fmt;
use serde::{Deserialize, Serialize};
use crate::core::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolVersion {
    V1,
    V2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub version: ProtocolVersion,
    pub compression: bool,
    pub encryption: bool,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            version: ProtocolVersion::V2,
            compression: true,
            encryption: true,
        }
    }
}

pub struct ProtocolManager {
    config: ProtocolConfig,
}

impl ProtocolManager {
    pub fn new(config: ProtocolConfig) -> Self {
        Self { config }
    }

    pub fn get_config(&self) -> &ProtocolConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: ProtocolConfig) {
        self.config = config;
    }

    pub fn is_compression_enabled(&self) -> bool {
        self.config.compression
    }

    pub fn is_encryption_enabled(&self) -> bool {
        self.config.encryption
    }
} 