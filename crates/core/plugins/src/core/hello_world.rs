// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use crate::plugin::{Plugin, PluginMetadata, PluginStatus};
use anyhow::Result;
use std::any::Any;
use std::fmt;

/// A simple Hello World plugin that demonstrates basic functionality
#[derive(Clone)]
pub struct HelloWorldPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
}

impl fmt::Debug for HelloWorldPlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HelloWorldPlugin")
            .field("metadata", &self.metadata)
            .field("status", &self.status)
            .finish()
    }
}

impl Default for HelloWorldPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl HelloWorldPlugin {
    /// Create a new instance of the `HelloWorldPlugin`
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "hello_world",
                "1.0.0",
                "A simple Hello World plugin",
                "SquirrelLabs",
            )
            .with_capability("core"),
            status: PluginStatus::Registered,
        }
    }
}

impl Plugin for HelloWorldPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async {
            println!("Initializing HelloWorldPlugin");
            Ok(())
        })
    }

    fn shutdown(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async {
            println!("Shutting down HelloWorldPlugin");
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
