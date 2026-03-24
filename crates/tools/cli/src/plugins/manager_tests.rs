// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::commands::registry::CommandRegistry;
use crate::plugins::plugin::{Plugin, PluginFactory};
use clap::Command as ClapCommand;
use squirrel_commands::Command as SquirrelCommand;
use squirrel_commands::error::CommandError;
use std::path::PathBuf;
use std::sync::Arc;

struct RegistryCommand;

impl SquirrelCommand for RegistryCommand {
    fn name(&self) -> &str {
        "reg_cmd"
    }

    fn description(&self) -> &str {
        "registry command"
    }

    fn execute(&self, args: &[String]) -> Result<String, CommandError> {
        Ok(format!("ok:{args:?}"))
    }

    fn parser(&self) -> ClapCommand {
        ClapCommand::new("reg_cmd")
    }

    fn clone_box(&self) -> Box<dyn SquirrelCommand> {
        Box::new(RegistryCommand)
    }
}

struct SimplePlugin {
    name: String,
    cmds: Vec<Arc<dyn SquirrelCommand>>,
}

#[async_trait::async_trait]
impl Plugin for SimplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> Option<&str> {
        Some("simple")
    }

    async fn initialize(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn register_commands(&self, _registry: &CommandRegistry) -> Result<(), PluginError> {
        Ok(())
    }

    fn commands(&self) -> Vec<Arc<dyn SquirrelCommand>> {
        self.cmds.clone()
    }

    async fn execute(&self, _args: &[String]) -> Result<String, PluginError> {
        Ok("exec".to_string())
    }

    async fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

struct FailingInitPlugin;

#[async_trait::async_trait]
impl Plugin for FailingInitPlugin {
    fn name(&self) -> &str {
        "fail-init"
    }

    fn version(&self) -> &str {
        "0.0.1"
    }

    fn description(&self) -> Option<&str> {
        None
    }

    async fn initialize(&self) -> Result<(), PluginError> {
        Err(PluginError::InitError("boom".to_string()))
    }

    fn register_commands(&self, _registry: &CommandRegistry) -> Result<(), PluginError> {
        Ok(())
    }

    fn commands(&self) -> Vec<Arc<dyn SquirrelCommand>> {
        Vec::new()
    }

    async fn execute(&self, _args: &[String]) -> Result<String, PluginError> {
        Ok(String::new())
    }

    async fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

struct FailingStartPlugin;

#[async_trait::async_trait]
impl Plugin for FailingStartPlugin {
    fn name(&self) -> &str {
        "fail-start"
    }

    fn version(&self) -> &str {
        "0.0.1"
    }

    fn description(&self) -> Option<&str> {
        None
    }

    async fn initialize(&self) -> Result<(), PluginError> {
        Ok(())
    }

    async fn start(&self) -> Result<(), PluginError> {
        Err(PluginError::ValidationError("no start".to_string()))
    }

    fn register_commands(&self, _registry: &CommandRegistry) -> Result<(), PluginError> {
        Ok(())
    }

    fn commands(&self) -> Vec<Arc<dyn SquirrelCommand>> {
        Vec::new()
    }

    async fn execute(&self, _args: &[String]) -> Result<String, PluginError> {
        Ok(String::new())
    }

    async fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

struct SimpleFactory {
    plugin: Arc<dyn Plugin>,
}

impl PluginFactory for SimpleFactory {
    fn create(&self) -> Result<Arc<dyn Plugin>, PluginError> {
        Ok(self.plugin.clone())
    }
}

fn sample_metadata(name: &str) -> PluginMetadata {
    PluginMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: Some("desc".to_string()),
        author: None,
        homepage: None,
    }
}

#[test]
fn default_and_helpers() {
    let m = PluginManager::default();
    assert!(m.list_plugins().is_empty());
    let _ = create_plugin_manager();
    assert!(initialize_plugins().is_ok());
}

#[test]
fn discovery_add_list_get_remove() {
    let mut m = PluginManager::new();
    let meta = sample_metadata("p1");
    let path = PathBuf::from("/virtual/plugin");
    m.add_plugin(meta, path.clone(), PluginStatus::Installed)
        .expect("add");

    assert_eq!(m.list_plugins().len(), 1);
    assert_eq!(m.get_plugin("p1").expect("get").metadata().name, "p1");
    assert_eq!(
        m.get_plugin_mut("p1").expect("get_mut").path(),
        path.as_path()
    );

    assert!(m.get_plugin("nope").is_err());
    assert!(
        m.add_plugin(
            sample_metadata("p1"),
            PathBuf::from("/x"),
            PluginStatus::Installed
        )
        .is_err()
    );

    m.remove_plugin("p1").expect("remove");
    assert!(m.get_plugin("p1").is_err());
}

#[test]
fn load_plugin_via_factory_idempotent_and_register_commands() {
    let mut m = PluginManager::new();
    let cmd = Arc::new(RegistryCommand);
    let plugin = Arc::new(SimplePlugin {
        name: "factory-p".to_string(),
        cmds: vec![cmd],
    });
    let factory = Arc::new(SimpleFactory { plugin });
    m.register_plugin_factory("factory-p", factory)
        .expect("register factory");

    m.load_plugin("factory-p").expect("load");
    m.load_plugin("factory-p").expect("idempotent load");

    let registry = Arc::new(CommandRegistry::new());
    m.register_plugin_commands(&registry)
        .expect("register cmds");
    let out = registry.execute("reg_cmd", &[]).expect("exec");
    assert!(out.contains("ok:"));

    m.start_plugins().expect("start all");
    assert!(m.stop_plugins().is_ok());
}

#[test]
fn load_plugin_init_failure_sets_failed_status() {
    let mut m = PluginManager::new();
    let plugin = Arc::new(FailingInitPlugin);
    let factory = Arc::new(SimpleFactory { plugin });
    m.register_plugin_factory("bad", factory).expect("reg");
    let err = m.load_plugin("bad").expect_err("init fails");
    assert!(matches!(err, PluginError::InitError(_)));
    let item = m.get_plugin("bad").expect("still listed");
    assert!(matches!(item.status(), PluginStatus::Failed(_)));
}

#[test]
fn start_stop_single_plugin_and_unload() {
    let mut m = PluginManager::new();
    let plugin = Arc::new(SimplePlugin {
        name: "lifecycle".to_string(),
        cmds: Vec::new(),
    });
    m.register_plugin_factory("lifecycle", Arc::new(SimpleFactory { plugin }))
        .expect("reg");
    m.load_plugin("lifecycle").expect("load");

    assert!(m.start_plugin("lifecycle").is_ok());
    assert!(m.start_plugin("lifecycle").is_ok());
    assert!(m.stop_plugin("lifecycle").is_ok());

    assert!(m.start_plugin("lifecycle").is_err());
    m.unload_plugin("lifecycle").expect("unload");
}

#[test]
fn start_plugin_errors_when_not_loaded_or_wrong_state() {
    let mut m = PluginManager::new();
    m.add_plugin(
        sample_metadata("only-added"),
        PathBuf::from("/x"),
        PluginStatus::Installed,
    )
    .expect("add");
    assert!(m.start_plugin("only-added").is_err());

    let mut m = PluginManager::new();
    let plugin = Arc::new(FailingStartPlugin);
    m.register_plugin_factory("fs", Arc::new(SimpleFactory { plugin }))
        .expect("reg");
    m.load_plugin("fs").expect("load");
    assert!(m.start_plugin("fs").is_err());
}

#[test]
fn remove_plugin_cleans_loaded_instance() {
    let mut m = PluginManager::new();
    let plugin = Arc::new(SimplePlugin {
        name: "rm".to_string(),
        cmds: Vec::new(),
    });
    m.register_plugin_factory("rm", Arc::new(SimpleFactory { plugin }))
        .expect("reg");
    m.load_plugin("rm").expect("load");
    m.remove_plugin("rm").expect("remove");
    assert!(m.get_plugin("rm").is_err());
}

#[test]
fn unload_plugins_async_resets_state() {
    let mut m = PluginManager::new();
    let plugin = Arc::new(SimplePlugin {
        name: "u".to_string(),
        cmds: Vec::new(),
    });
    m.register_plugin_factory("u", Arc::new(SimpleFactory { plugin }))
        .expect("reg");
    m.load_plugin("u").expect("load");
    let rt = tokio::runtime::Runtime::new().expect("rt");
    rt.block_on(m.unload_plugins()).expect("unload all");
    assert!(m.get_plugin("u").is_ok());
}

#[test]
fn register_plugin_commands_empty_ok() {
    let m = PluginManager::new();
    let registry = Arc::new(CommandRegistry::new());
    m.register_plugin_commands(&registry).expect("noop");
}

#[test]
fn stop_plugins_when_nothing_started_ok() {
    let mut m = PluginManager::new();
    let plugin = Arc::new(SimplePlugin {
        name: "s".to_string(),
        cmds: Vec::new(),
    });
    m.register_plugin_factory("s", Arc::new(SimpleFactory { plugin }))
        .expect("reg");
    m.load_plugin("s").expect("load");
    assert!(m.stop_plugins().is_ok());
}
