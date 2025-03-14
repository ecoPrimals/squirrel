use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use std::env;
use serde::{Serialize, Deserialize};
use sysinfo::{System, SystemExt};
use crate::ai::mcp_tools::{
    types::MCPError,
    persistence::ContextSnapshot,
};
use std::collections::HashSet;

/// System information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInfo {
    pub os_type: String,
    pub os_version: String,
    pub cpu_count: usize,
    pub memory_total: u64,
}

/// Argument information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArgInfo {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Command information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub args: Vec<ArgInfo>,
}

/// Context event types
#[derive(Debug, Clone)]
pub enum ContextEvent {
    WorkingDirChanged(PathBuf),
    EnvVarChanged(String, String),
    CommandAdded(CommandInfo),
    CommandRemoved(String),
}

/// Machine context state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextState {
    pub working_dir: PathBuf,
    pub env_vars: HashMap<String, String>,
    pub commands: HashMap<String, CommandInfo>,
    pub system_info: SystemInfo,
    pub last_update: SystemTime,
    pub change_count: u64,
    pub subscriber_count: u64,
    pub registered_tools: HashSet<String>,
}

impl Default for ContextState {
    fn default() -> Self {
        Self {
            working_dir: PathBuf::new(),
            env_vars: HashMap::new(),
            commands: HashMap::new(),
            system_info: SystemInfo {
                os_type: env::consts::OS.to_string(),
                os_version: "Unknown".to_string(),
                cpu_count: 0,
                memory_total: 0,
            },
            last_update: SystemTime::now(),
            change_count: 0,
            subscriber_count: 0,
            registered_tools: HashSet::new(),
        }
    }
}

/// Machine context
pub struct MachineContext {
    working_dir: PathBuf,
    env_vars: HashMap<String, String>,
    commands: HashMap<String, CommandInfo>,
    system_info: SystemInfo,
    state: Arc<RwLock<ContextState>>,
    subscribers: Vec<Arc<dyn Fn(ContextEvent) + Send + Sync>>,
}

impl MachineContext {
    /// Create a new machine context
    pub fn new() -> Result<Self, MCPError> {
        let sys = System::new_all();

        let system_info = SystemInfo {
            os_type: env::consts::OS.to_string(),
            os_version: sys.os_version()
                .unwrap_or_else(|| "Unknown".to_string()),
            cpu_count: sys.cpus().len(),
            memory_total: sys.total_memory(),
        };

        Ok(Self {
            working_dir: env::current_dir()
                .map_err(MCPError::IoError)?,
            env_vars: env::vars().collect(),
            commands: HashMap::new(),
            system_info,
            state: Arc::new(RwLock::new(ContextState::default())),
            subscribers: Vec::new(),
        })
    }

    /// Get the current working directory
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// Set the working directory
    pub fn set_working_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<(), MCPError> {
        let path = path.as_ref().to_path_buf();
        env::set_current_dir(&path)
            .map_err(MCPError::IoError)?;
        self.working_dir = path.clone();
        self.notify_subscribers(ContextEvent::WorkingDirChanged(path));
        Ok(())
    }

    /// Get an environment variable
    pub fn get_env_var(&self, key: &str) -> Option<String> {
        self.env_vars.get(key).cloned()
    }

    /// Get all environment variables
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        self.env_vars.clone()
    }

    /// Set an environment variable
    pub fn set_env_var(&mut self, key: String, value: String) -> Result<(), MCPError> {
        env::set_var(&key, &value);
        self.env_vars.insert(key.clone(), value.clone());
        self.notify_subscribers(ContextEvent::EnvVarChanged(key, value));
        Ok(())
    }

    /// Add a command
    pub fn add_command(&mut self, info: CommandInfo) -> Result<(), MCPError> {
        let name = info.name.clone();
        let info_clone = info.clone();
        self.commands.insert(name.clone(), info);
        self.notify_subscribers(ContextEvent::CommandAdded(info_clone));
        Ok(())
    }

    /// Remove a command
    pub fn remove_command(&mut self, name: &str) -> Result<(), MCPError> {
        if self.commands.remove(name).is_some() {
            self.notify_subscribers(ContextEvent::CommandRemoved(name.to_string()));
        }
        Ok(())
    }

    /// Get a command by name
    pub fn get_command(&self, name: &str) -> Option<&CommandInfo> {
        self.commands.get(name)
    }

    /// Get all commands
    pub fn get_commands(&self) -> HashMap<String, CommandInfo> {
        self.commands.clone()
    }

    /// List all commands
    pub fn list_commands(&self) -> Vec<&CommandInfo> {
        self.commands.values().collect()
    }

    /// Get system information
    pub fn system_info(&self) -> &SystemInfo {
        &self.system_info
    }

    /// Subscribe to context events
    pub fn subscribe<F>(&mut self, subscriber: F) -> Result<(), MCPError>
    where
        F: Fn(ContextEvent) + Send + Sync + 'static,
    {
        self.subscribers.push(Arc::new(subscriber));
        let mut state = self.state.write()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire lock".to_string()))?;
        state.subscriber_count += 1;
        state.last_update = SystemTime::now();
        Ok(())
    }

    /// Notify subscribers of a context event
    fn notify_subscribers(&self, event: ContextEvent) {
        for subscriber in &self.subscribers {
            subscriber(event.clone());
        }

        if let Ok(mut state) = self.state.write() {
            state.change_count += 1;
            state.last_update = SystemTime::now();
        }
    }

    /// Get the current state
    pub fn get_state(&self) -> Result<ContextState, MCPError> {
        self.state.read()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire lock".to_string()))
            .map(|state| state.clone())
    }

    /// Restore from a snapshot
    pub fn restore_from_snapshot(&mut self, snapshot: &ContextSnapshot) -> Result<(), MCPError> {
        self.set_working_dir(&snapshot.working_dir)?;
        self.env_vars = snapshot.env_vars.clone();
        self.commands = snapshot.commands.clone();
        self.system_info = snapshot.system_info.clone();
        Ok(())
    }

    /// Add a tool registration
    pub fn add_tool_registration(&mut self, tool_id: &str) -> Result<(), MCPError> {
        let mut state = self.state.write()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire lock".to_string()))?;
        
        state.registered_tools.insert(tool_id.to_string());
        state.last_update = SystemTime::now();
        state.change_count += 1;
        
        Ok(())
    }

    /// Remove a tool registration
    pub fn remove_tool_registration(&mut self, tool_id: &str) -> Result<(), MCPError> {
        let mut state = self.state.write()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire lock".to_string()))?;
        
        state.registered_tools.remove(tool_id);
        state.last_update = SystemTime::now();
        state.change_count += 1;
        
        Ok(())
    }

    /// Get registered tools
    pub fn get_registered_tools(&self) -> Result<HashSet<String>, MCPError> {
        let state = self.state.read()
            .map_err(|_| MCPError::ProtocolError("Failed to acquire lock".to_string()))?;
        
        Ok(state.registered_tools.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_context_state() {
        let mut context = MachineContext::new().unwrap();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Test working directory
        context.set_working_dir(&temp_path).unwrap();
        assert_eq!(context.working_dir(), temp_path.as_path());

        // Test environment variables
        context.set_env_var("TEST_VAR".to_string(), "test_value".to_string()).unwrap();
        assert_eq!(context.get_env_var("TEST_VAR").unwrap(), "test_value");

        // Test commands
        let command = CommandInfo {
            name: "test_cmd".to_string(),
            description: "Test command".to_string(),
            args: vec![ArgInfo {
                name: "arg1".to_string(),
                description: "First argument".to_string(),
                required: true,
            }],
        };
        context.add_command(command.clone()).unwrap();
        assert_eq!(context.get_command("test_cmd").unwrap().name, "test_cmd");
        assert_eq!(context.list_commands().len(), 1);

        context.remove_command("test_cmd").unwrap();
        assert!(context.get_command("test_cmd").is_none());
        assert_eq!(context.list_commands().len(), 0);
    }

    #[test]
    fn test_context_events() {
        let mut context = MachineContext::new().unwrap();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        let (tx, rx) = std::sync::mpsc::channel();
        context.subscribe(move |event| {
            tx.send(event).unwrap();
        }).unwrap();

        // Test working directory event
        context.set_working_dir(&temp_path).unwrap();
        if let ContextEvent::WorkingDirChanged(path) = rx.try_recv().unwrap() {
            assert_eq!(path, temp_path);
        } else {
            panic!("Expected WorkingDirChanged event");
        }

        // Test environment variable event
        context.set_env_var("TEST_VAR".to_string(), "test_value".to_string()).unwrap();
        if let ContextEvent::EnvVarChanged(key, value) = rx.try_recv().unwrap() {
            assert_eq!(key, "TEST_VAR");
            assert_eq!(value, "test_value");
        } else {
            panic!("Expected EnvVarChanged event");
        }

        // Test command events
        let command = CommandInfo {
            name: "test_cmd".to_string(),
            description: "Test command".to_string(),
            args: vec![ArgInfo {
                name: "arg1".to_string(),
                description: "First argument".to_string(),
                required: true,
            }],
        };
        context.add_command(command.clone()).unwrap();
        if let ContextEvent::CommandAdded(cmd) = rx.try_recv().unwrap() {
            assert_eq!(cmd.name, "test_cmd");
        } else {
            panic!("Expected CommandAdded event");
        }

        context.remove_command("test_cmd").unwrap();
        if let ContextEvent::CommandRemoved(name) = rx.try_recv().unwrap() {
            assert_eq!(name, "test_cmd");
        } else {
            panic!("Expected CommandRemoved event");
        }
    }
}