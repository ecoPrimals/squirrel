use serde::Serialize;
use clap::Command as ClapCommand;
use squirrel_commands::{Command, CommandRegistry, CommandError};
use crate::formatter::{FormatterFactory, OutputFormat};

#[derive(Serialize, Debug)]
struct CommandHelp {
    name: String,
    description: String,
}

#[derive(Serialize, Debug)]
struct HelpOutput {
    title: String,
    commands: Vec<CommandHelp>,
}

/// Help command implementation
pub struct HelpCommand {
    registry: CommandRegistry,
}

impl HelpCommand {
    /// Create a new help command
    pub fn new() -> Self {
        Self {
            registry: CommandRegistry::new(),
        }
    }
}

impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    
    fn description(&self) -> &str {
        "Show help information for available commands"
    }

    fn help(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }
    
    fn parser(&self) -> ClapCommand {
        ClapCommand::new("help")
            .about("Show help information for available commands")
            .arg(clap::Arg::new("command")
                .help("Command to show help for")
                .required(false))
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {
            registry: self.registry.clone(),
        })
    }

    fn execute(&self, args: &[String]) -> Result<String, CommandError> {
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

        if let Some(cmd) = args.first().filter(|&arg| !arg.starts_with("--")) {
            // Show help for specific command
            match self.registry.get_command(cmd) {
                Ok(command) => {
                    let help = CommandHelp {
                        name: cmd.to_string(),
                        description: command.help(),
                    };
                    formatter.format(help).map_err(|e| CommandError::ExecutionError(e.to_string()))
                }
                Err(_) => Err(CommandError::CommandNotFound(cmd.to_string())),
            }
        } else {
            // Show general help
            match self.registry.list_commands() {
                Ok(command_names) => {
                    let commands: Vec<CommandHelp> = command_names
                        .iter()
                        .filter_map(|cmd| {
                            match self.registry.get_command(cmd) {
                                Ok(command) => Some(CommandHelp {
                                    name: cmd.to_string(),
                                    description: command.help(),
                                }),
                                Err(_) => None,
                            }
                        })
                        .collect();

                    let help_output = HelpOutput {
                        title: "Squirrel CLI - Available Commands".to_string(),
                        commands,
                    };

                    if format == OutputFormat::Table {
                        let headers = &["Command", "Description"];
                        let rows: Vec<Vec<String>> = help_output.commands
                            .iter()
                            .map(|cmd| vec![cmd.name.clone(), cmd.description.clone()])
                            .collect();
                        Ok(formatter.format_table(headers, &rows))
                    } else {
                        formatter.format(help_output).map_err(|e| CommandError::ExecutionError(e.to_string()))
                    }
                },
                Err(e) => Err(e),
            }
        }
    }
} 