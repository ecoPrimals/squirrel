// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Minimal `Plugin` implementation for [`crate::plugins::manager::PluginManager::create_test_plugin`].

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use crate::commands::registry::CommandRegistry;
use crate::plugins::error::PluginError;
use crate::plugins::plugin::Plugin;
use squirrel_commands::Command;
use tracing::debug;

struct TestPlugin {
    name: String,
    _path: PathBuf,
}

impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn description(&self) -> Option<&str> {
        Some("A test plugin")
    }

    fn initialize(&self) -> Pin<Box<dyn Future<Output = Result<(), PluginError>> + Send + '_>> {
        let name = self.name.clone();
        Box::pin(async move {
            debug!("Test plugin {} initialized", name);
            Ok(())
        })
    }

    fn register_commands(&self, _registry: &CommandRegistry) -> Result<(), PluginError> {
        debug!("Test plugin {} registered commands", self.name);
        Ok(())
    }

    fn commands(&self) -> Vec<Arc<dyn Command>> {
        Vec::new()
    }

    fn execute(
        &self,
        args: &[String],
    ) -> Pin<Box<dyn Future<Output = Result<String, PluginError>> + Send + '_>> {
        let name = self.name.clone();
        let args = args.to_vec();
        Box::pin(async move {
            debug!("Test plugin {} executed with args: {:?}", name, args);
            Ok(format!("Test plugin {name} executed"))
        })
    }

    fn cleanup(&self) -> Pin<Box<dyn Future<Output = Result<(), PluginError>> + Send + '_>> {
        let name = self.name.clone();
        Box::pin(async move {
            debug!("Test plugin {} cleaned up", name);
            Ok(())
        })
    }
}

pub(super) fn test_plugin_arc(name: String, path: PathBuf) -> Result<Arc<dyn Plugin>, PluginError> {
    Ok(Arc::new(TestPlugin { name, _path: path }))
}
