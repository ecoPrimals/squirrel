//! Configuration command for the Squirrel CLI
//!
//! This module provides a CLI command for managing configuration settings.

use clap::{Args, Subcommand, FromArgMatches};
use std::path::{PathBuf, Path};
use log::debug;

use squirrel_commands::{Command, CommandResult, CommandError};
use crate::config::{ConfigManager, ConfigError};
use crate::formatter::{OutputFormatter, FormatterError};

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
    
    fn execute(&self, args: &[String]) -> CommandResult<String> {
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
        
        // Create an output formatter using the configuration's output format
        let output_format = match config_manager.get("output_format") {
            Ok(format) => format,
            Err(_) => "text".to_string(), // Default to text if not found
        };
        
        let formatter = match OutputFormatter::from_format_str(&output_format) {
            Ok(fmt) => fmt,
            Err(e) => {
                return Err(CommandError::ExecutionError(
                    format!("Failed to create output formatter: {}", e)
                ));
            }
        };
        
        // Process subcommand
        match &config_args.subcommand {
            ConfigSubcommand::Get { .. } => {
                self.handle_get(&config_args, &config_manager, &formatter)
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
        ConfigArgs::augment_args(clap::Command::new("config").about("Manage configuration"))
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
        args: &ConfigArgs,
        config_manager: &ConfigManager,
        formatter: &OutputFormatter,
    ) -> CommandResult<String> {
        if let ConfigSubcommand::Get { key } = &args.subcommand {
            debug!("Getting configuration value for key: {}", key);
            
            match config_manager.get(key) {
                Ok(value) => {
                    let formatted = formatter.format_value(&serde_json::Value::String(value))
                        .map_err(|e: FormatterError| CommandError::ExecutionError(e.to_string()))?;
                    Ok(format!("{}={}", key, formatted))
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
        } else {
            // This should never happen if called correctly
            Err(CommandError::ExecutionError(
                "Invalid subcommand for handle_get".to_string()
            ))
        }
    }
    
    /// Handle the 'set' subcommand
    fn handle_set(
        &self,
        config_manager: &mut ConfigManager,
        key: &str,
        value: &str,
    ) -> CommandResult<String> {
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
        formatter: &OutputFormatter,
        filter: &Option<String>,
    ) -> CommandResult<String> {
        debug!("Listing configuration values with filter: {:?}", filter);
        
        // Get all configuration values
        let all_values = config_manager.list();
        
        // Apply filter if specified
        let filtered_values: Vec<(String, String)> = if let Some(prefix) = filter {
            all_values
                .into_iter()
                .filter(|(key, _)| key.starts_with(prefix))
                .collect()
        } else {
            all_values.into_iter().collect()
        };
        
        // Format the results
        let mut result = String::new();
        for (key, value) in filtered_values {
            let formatted_value = formatter.format_value(&serde_json::Value::String(value))
                .map_err(|e: FormatterError| CommandError::ExecutionError(e.to_string()))?;
            result.push_str(&format!("{}={}\n", key, formatted_value));
        }
        
        if result.is_empty() {
            if let Some(filter_value) = filter {
                result = format!("No configuration values found with prefix '{}'", filter_value);
            } else {
                result = "No configuration values found".to_string();
            }
        } else {
            // Remove trailing newline
            result.pop();
        }
        
        Ok(result)
    }
    
    /// Handle the 'edit' subcommand
    fn handle_edit(
        &self,
        config_manager: &ConfigManager,
    ) -> CommandResult<String> {
        debug!("Editing configuration file");
        
        // Check if config file exists
        let config_path = if let Some(path) = config_manager.config_path() {
            path.clone()
        } else {
            return Err(CommandError::ExecutionError(
                "No configuration file found to edit".to_string()
            ));
        };
        
        // Open the editor
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });
        
        debug!("Using editor: {}", editor);
        
        let status = std::process::Command::new(editor)
            .arg(&config_path)
            .status()
            .map_err(|e| CommandError::ExecutionError(
                format!("Failed to launch editor: {}", e)
            ))?;
        
        if !status.success() {
            return Err(CommandError::ExecutionError(
                format!("Editor exited with non-zero status: {}", status)
            ));
        }
        
        Ok(format!("Edited configuration at {}", config_path.display()))
    }
    
    /// Handle the 'import' subcommand
    fn handle_import(
        &self,
        config_manager: &mut ConfigManager,
        path: &Path,
    ) -> CommandResult<String> {
        debug!("Importing configuration from {}", path.display());
        
        match config_manager.import(path.to_path_buf()) {
            Ok(()) => {
                // Save the configuration
                if let Err(e) = config_manager.save(None) {
                    return Err(CommandError::ExecutionError(
                        format!("Failed to save imported configuration: {}", e)
                    ));
                }
                
                Ok(format!("Imported configuration from {}", path.display()))
            },
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
    ) -> CommandResult<String> {
        debug!("Exporting configuration to {}", path.display());
        
        match config_manager.export(path.to_path_buf()) {
            Ok(()) => Ok(format!("Exported configuration to {}", path.display())),
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