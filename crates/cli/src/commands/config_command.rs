//! Configuration command for the Squirrel CLI
//!
//! This module provides a CLI command for managing configuration settings.

use clap::{Args, Subcommand, FromArgMatches};
use std::path::{PathBuf, Path};
use log::debug;
use serde::Serialize;

use squirrel_commands::{Command, CommandError};
use crate::config::{ConfigManager, ConfigError};
use crate::formatter::{FormatterFactory, OutputFormat, Formatter};

/// Configuration command arguments
#[derive(Debug, Args)]
pub struct ConfigArgs {
    /// Configuration subcommand
    #[clap(subcommand)]
    pub subcommand: ConfigSubcommand,
    
    /// Path to configuration file (default: search standard locations)
    #[clap(long, short = 'c')]
    pub config_file: Option<PathBuf>,
}

/// Configuration subcommands
#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// Get a configuration value
    #[clap(name = "get")]
    Get {
        /// Configuration key
        key: String,
    },
    
    /// Set a configuration value
    #[clap(name = "set")]
    Set {
        /// Configuration key
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// List all configuration values
    #[clap(name = "list")]
    List {
        /// Filter by key prefix
        #[clap(long, short = 'f')]
        filter: Option<String>,
    },
    
    /// Edit configuration in your default editor
    #[clap(name = "edit")]
    Edit,
    
    /// Import configuration from a file
    #[clap(name = "import")]
    Import {
        /// Path to configuration file to import
        path: PathBuf,
    },
    
    /// Export configuration to a file
    #[clap(name = "export")]
    Export {
        /// Path to export configuration to
        path: PathBuf,
    },
}

#[derive(Debug, Serialize)]
struct ConfigValue {
    key: String,
    value: String,
}

#[derive(Debug, Serialize)]
struct ConfigList {
    values: Vec<ConfigValue>,
}

/// Command for managing configuration
#[derive(Debug, Clone)]
pub struct ConfigCommand;

impl Default for ConfigCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ConfigCommand {
    fn name(&self) -> &str {
        "config"
    }
    
    fn description(&self) -> &str {
        "Manage CLI configuration"
    }
    
    fn execute(&self, args: &[String]) -> Result<String, CommandError> {
        debug!("Executing config command with args: {:?}", args);
        
        // Parse arguments using clap
        let config_matches = self.parser().get_matches_from(
            std::iter::once(String::from("config")).chain(args.iter().cloned())
        );
        
        let config_args = ConfigArgs::from_arg_matches(&config_matches)
            .map_err(|err| CommandError::ExecutionError(err.to_string()))?;
        
        // Load configuration
        let mut config_manager = match ConfigManager::load(config_args.config_file.clone()) {
            Ok(manager) => manager,
            Err(e) => {
                return Err(CommandError::ExecutionError(
                    format!("Failed to load configuration: {}", e)
                ));
            }
        };
        
        // Determine output format
        let format = if args.contains(&"--json".to_string()) {
            OutputFormat::Json
        } else if args.contains(&"--yaml".to_string()) {
            OutputFormat::Yaml
        } else if args.contains(&"--table".to_string()) {
            OutputFormat::Table
        } else {
            OutputFormat::Text
        };
        
        let formatter = FormatterFactory::create(format);
        
        // Process subcommand
        match &config_args.subcommand {
            ConfigSubcommand::Get { key } => {
                self.handle_get(&config_manager, key, &formatter)
            },
            ConfigSubcommand::Set { key, value } => {
                self.handle_set(&mut config_manager, key, value)
            },
            ConfigSubcommand::List { filter } => {
                self.handle_list(&config_manager, &formatter, filter)
            },
            ConfigSubcommand::Edit => {
                self.handle_edit(&config_manager)
            },
            ConfigSubcommand::Import { path } => {
                self.handle_import(&mut config_manager, path)
            },
            ConfigSubcommand::Export { path } => {
                self.handle_export(&config_manager, path)
            },
        }
    }
    
    fn parser(&self) -> clap::Command {
        let mut cmd = ConfigArgs::augment_args(clap::Command::new("config").about("Manage configuration"));
        
        // Add output format arguments
        cmd = cmd
            .arg(clap::Arg::new("json")
                .long("json")
                .help("Output in JSON format")
                .conflicts_with_all(["yaml", "table"]))
            .arg(clap::Arg::new("yaml")
                .long("yaml")
                .help("Output in YAML format")
                .conflicts_with_all(["json", "table"]))
            .arg(clap::Arg::new("table")
                .long("table")
                .help("Output in table format")
                .conflicts_with_all(["json", "yaml"]));
        
        cmd
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

impl ConfigCommand {
    /// Create a new ConfigCommand instance
    pub fn new() -> Self {
        ConfigCommand
    }
    
    /// Handle the 'get' subcommand
    fn handle_get(
        &self,
        config_manager: &ConfigManager,
        key: &str,
        formatter: &Formatter,
    ) -> Result<String, CommandError> {
        debug!("Getting configuration value for key: {}", key);
        
        match config_manager.get(key) {
            Ok(value) => {
                let config_value = ConfigValue {
                    key: key.to_string(),
                    value: value.clone(),
                };
                formatter.format(config_value)
                    .map_err(|e| CommandError::ExecutionError(e.to_string()))
            },
            Err(ConfigError::KeyNotFound(_)) => {
                Err(CommandError::ExecutionError(
                    format!("Configuration key '{}' not found", key)
                ))
            },
            Err(e) => {
                Err(CommandError::ExecutionError(
                    format!("Error getting configuration: {}", e)
                ))
            },
        }
    }
    
    /// Handle the 'set' subcommand
    fn handle_set(
        &self,
        config_manager: &mut ConfigManager,
        key: &str,
        value: &str,
    ) -> Result<String, CommandError> {
        debug!("Setting configuration value: {} = {}", key, value);
        
        match config_manager.set(key, value.to_string()) {
            Ok(()) => {
                // Save the configuration
                if let Err(e) = config_manager.save(None) {
                    return Err(CommandError::ExecutionError(
                        format!("Failed to save configuration: {}", e)
                    ));
                }
                
                Ok(format!("Set {} = {}", key, value))
            },
            Err(e) => {
                Err(CommandError::ExecutionError(
                    format!("Failed to set configuration value: {}", e)
                ))
            },
        }
    }
    
    /// Handle the 'list' subcommand
    fn handle_list(
        &self,
        config_manager: &ConfigManager,
        formatter: &Formatter,
        filter: &Option<String>,
    ) -> Result<String, CommandError> {
        debug!("Listing configuration values with filter: {:?}", filter);
        
        // Get all configuration values
        let all_values = config_manager.list();
        
        // Apply filter if specified
        let filtered_values: Vec<ConfigValue> = if let Some(prefix) = filter {
            all_values
                .into_iter()
                .filter(|(key, _)| key.starts_with(prefix))
                .map(|(key, value)| ConfigValue { key, value })
                .collect()
        } else {
            all_values
                .into_iter()
                .map(|(key, value)| ConfigValue { key, value })
                .collect()
        };
        
        let config_list = ConfigList {
            values: filtered_values,
        };
        
        formatter.format(config_list)
            .map_err(|e| CommandError::ExecutionError(e.to_string()))
    }
    
    /// Handle the 'edit' subcommand
    fn handle_edit(
        &self,
        _config_manager: &ConfigManager, // Prefix with underscore since unused
    ) -> Result<String, CommandError> {
        // TODO: Implement edit functionality
        Err(CommandError::ExecutionError("Edit command not yet implemented".to_string()))
    }
    
    /// Handle the 'import' subcommand
    fn handle_import(
        &self,
        config_manager: &mut ConfigManager,
        path: &Path,
    ) -> Result<String, CommandError> {
        debug!("Importing configuration from: {:?}", path);
        
        match config_manager.import(path.to_path_buf()) {
            Ok(()) => Ok(format!("Successfully imported configuration from {:?}", path)),
            Err(e) => Err(CommandError::ExecutionError(
                format!("Failed to import configuration: {}", e)
            )),
        }
    }
    
    /// Handle the 'export' subcommand
    fn handle_export(
        &self,
        config_manager: &ConfigManager,
        path: &Path,
    ) -> Result<String, CommandError> {
        debug!("Exporting configuration to: {:?}", path);
        
        match config_manager.export(path.to_path_buf()) {
            Ok(()) => Ok(format!("Successfully exported configuration to {:?}", path)),
            Err(e) => Err(CommandError::ExecutionError(
                format!("Failed to export configuration: {}", e)
            )),
        }
    }
}

/// Register configuration-related commands
pub fn register_config_commands(registry: &mut squirrel_commands::CommandRegistry) {
    registry.register("config", std::sync::Arc::new(ConfigCommand))
        .expect("Failed to register config command");
} 