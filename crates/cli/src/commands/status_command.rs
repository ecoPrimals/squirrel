use std::error::Error;
use std::time::Duration;
use serde::Serialize;
use clap::Command as ClapCommand;
use squirrel_commands::{Command, CommandError};
use squirrel_core::Core;
use tokio::time;
use crate::formatter::{FormatterFactory, OutputFormat};

#[derive(Serialize, Debug)]
struct SystemStatus {
    status: String,
    uptime: u64,
    memory_usage: u64,
    active_commands: u32,
    connected_clients: u32,
}

/// Status command implementation
pub struct StatusCommand {
    core: Core,
}

impl StatusCommand {
    /// Create a new status command
    pub fn new() -> Self {
        Self {
            core: Core::new(),
        }
    }

    async fn display_status(&self, format: OutputFormat) -> Result<String, Box<dyn Error>> {
        let status = self.core.get_status()?;
        
        let system_status = SystemStatus {
            status: status.status,
            uptime: status.uptime,
            memory_usage: status.memory_usage,
            active_commands: status.active_commands,
            connected_clients: status.connected_clients,
        };

        let formatter = FormatterFactory::create(format);
        
        if format == OutputFormat::Table {
            let headers = &["Metric", "Value"];
            let rows = vec![
                vec!["Status".to_string(), system_status.status.clone()],
                vec!["Uptime".to_string(), format!("{} seconds", system_status.uptime)],
                vec!["Memory Usage".to_string(), format!("{} MB", system_status.memory_usage)],
                vec!["Active Commands".to_string(), system_status.active_commands.to_string()],
                vec!["Connected Clients".to_string(), system_status.connected_clients.to_string()],
            ];
            Ok(formatter.format_table(headers, &rows))
        } else {
            formatter.format(system_status)
        }
    }
}

impl Command for StatusCommand {
    fn name(&self) -> &str {
        "status"
    }
    
    fn description(&self) -> &str {
        "Show system status"
    }

    fn help(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }
    
    fn parser(&self) -> ClapCommand {
        ClapCommand::new("status")
            .about("Show system status")
            .arg(clap::Arg::new("watch")
                .long("watch")
                .help("Continuously monitor status"))
            .arg(clap::Arg::new("interval")
                .long("interval")
                .help("Update interval in seconds when watching")
                .value_name("SECONDS")
                .default_value("5"))
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self {
            core: self.core.clone(),
        })
    }

    fn execute(&self, args: &[String]) -> Result<String, CommandError> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CommandError::ExecutionError(format!("Failed to create runtime: {}", e)))?;

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

        // Check if watch mode is enabled
        if args.contains(&"--watch".to_string()) {
            let interval = args.iter()
                .position(|arg| arg == "--interval")
                .and_then(|pos| args.get(pos + 1))
                .and_then(|val| val.parse::<u64>().ok())
                .unwrap_or(5);

            rt.block_on(async {
                loop {
                    // Clear screen
                    print!("\x1B[2J\x1B[1;1H");
                    
                    // Display status
                    match self.display_status(format).await {
                        Ok(status) => println!("{}", status),
                        Err(e) => {
                            let formatter = FormatterFactory::create(format);
                            eprintln!("{}", formatter.format_error(e.as_ref()));
                        }
                    }
                    
                    // Wait for interval
                    time::sleep(Duration::from_secs(interval)).await;
                }
            });
            
            Ok("".to_string()) // Never reached in watch mode
        } else {
            // Single status display
            rt.block_on(self.display_status(format))
                .map_err(|e| CommandError::ExecutionError(format!("Status error: {}", e)))
        }
    }
} 