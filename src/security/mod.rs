//! Security module for MCP (Machine Context Protocol)
//!
//! This module provides authentication, authorization, and encryption services
//! for secure communication between MCP components.

use crate::error::{MCPError, Result, SecurityError};
use crate::types::{EncryptionFormat, SecurityLevel, Credentials, Session};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rand::{RngCore, rngs::OsRng};
use ring::aead::{Aad, Nonce, NonceSequence, UnboundKey, CHACHA20_POLY1305, OpeningKey, SealingKey, BoundKey};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Declare submodules
pub mod encryption;
pub mod types;
pub mod rbac;
// pub mod authentication; // TODO: Implement these modules
// pub mod tokens; // TODO: Implement these modules
// pub mod audit; // TODO: Implement these modules
mod manager;

// Re-export security manager for the application
pub use manager::{SecurityManager, SecurityManagerImpl};

// Re-export types from types module
pub use types::{
    Action, Permission, PermissionCondition, PermissionContext, PermissionScope, Role
};

// Re-export enhanced RBAC components
pub use rbac::{
    RBACManager, ValidationResult, ValidationRule, InheritanceType, ValidationAuditRecord
};

use crate::config::AppConfig;

// Include the EnhancedRBACManager alias
use crate::security::rbac::RBACManager as EnhancedRBACManager; 