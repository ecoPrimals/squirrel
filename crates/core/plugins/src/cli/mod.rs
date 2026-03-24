// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! CLI plugin integration
//!
//! This module provides integration between plugins and CLI commands.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Plugin, Result};

/// CLI command type for plugin integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommand {
    /// Command name (e.g. "install", "list").
    pub name: String,
    /// Human-readable description of what the command does.
    pub description: String,
    /// Usage string showing expected arguments.
    pub usage: String,
    /// Named parameters the command accepts.
    pub parameters: HashMap<String, String>,
}

/// CLI command metadata
#[derive(Clone, Debug)]
pub struct CliCommandMetadata {
    /// Command name
    pub name: String,

    /// Command description
    pub description: String,

    /// Command usage
    pub usage: String,

    /// Command examples
    pub examples: Vec<String>,

    /// Required permissions
    pub permissions: Vec<String>,
}

/// CLI plugin trait
#[async_trait]
pub trait CliPlugin: Plugin {
    /// Get available commands
    fn get_commands(&self) -> Vec<CliCommandMetadata>;

    /// Execute a command
    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<String>;

    /// Check if the plugin supports a command
    fn supports_command(&self, command: &str) -> bool {
        self.get_commands().iter().any(|cmd| cmd.name == command)
    }

    /// Get command help
    fn get_command_help(&self, command: &str) -> Option<String>;

    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{CliCommand, CliCommandMetadata, CliPlugin};
    use crate::Plugin;
    use crate::Result as PluginResult;
    use crate::plugin::PluginMetadata;
    use async_trait::async_trait;
    use std::any::Any;
    use std::collections::HashMap;

    struct TestCliPlugin {
        metadata: PluginMetadata,
    }

    impl TestCliPlugin {
        fn new() -> Self {
            let mut m = PluginMetadata::new("cli", "1.0", "d", "a");
            m.capabilities = vec!["cap-a".into(), "cap-b".into()];
            Self { metadata: m }
        }
    }

    #[async_trait]
    impl Plugin for TestCliPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn initialize(&self) -> anyhow::Result<()> {
            Ok(())
        }

        async fn shutdown(&self) -> anyhow::Result<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[async_trait]
    impl CliPlugin for TestCliPlugin {
        fn get_commands(&self) -> Vec<CliCommandMetadata> {
            vec![CliCommandMetadata {
                name: "list".into(),
                description: "list things".into(),
                usage: "list [opts]".into(),
                examples: vec!["list -a".into()],
                permissions: vec!["read".into()],
            }]
        }

        async fn execute_command(&self, command: &str, args: Vec<String>) -> PluginResult<String> {
            Ok(format!("ran {command} {args:?}"))
        }

        fn get_command_help(&self, command: &str) -> Option<String> {
            (command == "list").then(|| "help for list".into())
        }
    }

    #[test]
    fn cli_command_serializes_and_clones() {
        let c = CliCommand {
            name: "install".into(),
            description: "install plugin".into(),
            usage: "install <id>".into(),
            parameters: HashMap::from([("force".into(), "bool".into())]),
        };
        let json = serde_json::to_string(&c).expect("should succeed");
        let back: CliCommand = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(back.name, c.name);
        let cloned = c.clone();
        assert_eq!(format!("{cloned:?}"), format!("{c:?}"));
    }

    #[test]
    fn cli_command_metadata_clones_and_debugs() {
        let m = CliCommandMetadata {
            name: "n".into(),
            description: "d".into(),
            usage: "u".into(),
            examples: vec!["e".into()],
            permissions: vec!["p".into()],
        };
        let _ = format!("{m:?}");
        assert_eq!(m.clone().name, m.name);
    }

    #[test]
    fn cli_plugin_supports_command_and_capabilities() {
        let p = TestCliPlugin::new();
        assert!(p.supports_command("list"));
        assert!(!p.supports_command("missing"));
        assert_eq!(p.get_capabilities(), vec!["cap-a", "cap-b"]);
    }

    #[tokio::test]
    async fn cli_plugin_execute_and_help() {
        let p = TestCliPlugin::new();
        let out = p
            .execute_command("list", vec!["a".into()])
            .await
            .expect("should succeed");
        assert!(out.contains("list"));
        assert_eq!(p.get_command_help("list"), Some("help for list".into()));
        assert!(p.get_command_help("other").is_none());
    }
}
