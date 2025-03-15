use std::error::Error;
use crate::commands::Command;

/// The current version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use clap::{Parser, CommandFactory};

// Include the built info
include!(concat!(env!("OUT_DIR"), "/built.rs"));

/// Arguments for the version command
#[derive(Parser)]
#[command(name = "version", about = "Display version information")]
pub struct VersionArgs {
    /// Show detailed version information
    #[arg(short, long)]
    verbose: bool,
}

/// A command that displays version information
#[derive(Clone)]
pub struct VersionCommand;

impl Command for VersionCommand {
    fn name(&self) -> &'static str {
        "version"
    }

    fn description(&self) -> &'static str {
        "Displays version information"
    }

    fn execute(&self) -> Result<(), Box<dyn Error>> {
        if cfg!(debug_assertions) {
            println!("Version: {VERSION}");
            if let Ok(profile) = std::env::var("PROFILE") {
                println!("Profile: {profile}");
            }
            if let Ok(commit) = std::env::var("GIT_HASH") {
                println!("Git Commit: {commit}");
            }
        } else {
            println!("{VERSION}");
        }
        Ok(())
    }

    fn parser(&self) -> clap::Command {
        VersionArgs::command()
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_command() {
        let cmd = VersionCommand;
        assert_eq!(cmd.name(), "version");
        assert!(cmd.execute().is_ok());
    }

    #[test]
    fn test_version_parser() {
        let cmd = VersionCommand;
        let parser = cmd.parser();
        assert_eq!(parser.get_name(), "version");
    }
} 