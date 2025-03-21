use serde::Serialize;
use clap::Command as ClapCommand;
use squirrel_commands::{Command, CommandError};
use crate::formatter::{FormatterFactory, OutputFormat};

#[derive(Serialize, Debug)]
struct VersionInfo {
    version: String,
    core_version: String,
    build_date: String,
}

/// Version command implementation
pub struct VersionCommand;

impl VersionCommand {
    /// Create a new version command
    pub fn new() -> Self {
        Self
    }

    fn get_version_info(&self) -> VersionInfo {
        VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            core_version: squirrel_core::build_info::version().to_string(),
            build_date: squirrel_core::build_info::build_date().to_string(),
        }
    }
}

impl Command for VersionCommand {
    fn name(&self) -> &str {
        "version"
    }
    
    fn description(&self) -> &str {
        "Show version information"
    }

    fn help(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }
    
    fn parser(&self) -> ClapCommand {
        ClapCommand::new("version")
            .about("Show version information")
            .arg(clap::Arg::new("check")
                .long("check")
                .help("Check if current version meets requirement")
                .value_name("VERSION"))
    }
    
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(Self)
    }

    fn execute(&self, args: &[String]) -> Result<String, CommandError> {
        let version_info = self.get_version_info();

        // Create formatter based on output format
        let formatter = if args.contains(&"--json".to_string()) {
            FormatterFactory::create(OutputFormat::Json)
        } else if args.contains(&"--yaml".to_string()) {
            FormatterFactory::create(OutputFormat::Yaml)
        } else if args.contains(&"--table".to_string()) {
            FormatterFactory::create(OutputFormat::Table)
        } else {
            FormatterFactory::create(OutputFormat::Text)
        };

        // Check if version requirement check is requested
        if let Some(pos) = args.iter().position(|arg| arg == "--check") {
            if let Some(check_version) = args.get(pos + 1) {
                if version_info.version < check_version.to_string() {
                    return Err(CommandError::ValidationError(format!(
                        "Version requirement not met: {} < {}",
                        version_info.version,
                        check_version
                    )));
                }
                return formatter.format(serde_json::json!({
                    "status": "ok",
                    "message": format!(
                        "Version requirement met: {} >= {}",
                        version_info.version,
                        check_version
                    ),
                    "version": version_info
                })).map_err(|e| CommandError::ExecutionError(e.to_string()));
            }
        }

        // Format and return version info
        formatter.format(version_info).map_err(|e| CommandError::ExecutionError(e.to_string()))
    }
} 