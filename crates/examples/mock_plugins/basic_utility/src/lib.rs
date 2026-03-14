// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab
#![forbid(unsafe_code)]

use squirrel_app::plugin::{
    Plugin, PluginContext, PluginResult, PluginError,
    command::{Command, CommandResult},
    capability::{Capability, CapabilityRequest},
};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use chrono::Utc;

/// Basic Utility Plugin
/// 
/// A simple utility plugin for demonstrating and testing the CLI integration.
/// Provides basic analysis, optimization, and reporting commands.
#[derive(Default)]
pub struct BasicUtilityPlugin {
    context: Option<PluginContext>,
    settings: Settings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Settings {
    report_format: String,
    optimization_level: String,
    auto_update_check: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            report_format: "markdown".to_string(),
            optimization_level: "balanced".to_string(),
            auto_update_check: true,
        }
    }
}

impl Plugin for BasicUtilityPlugin {
    fn initialize(&mut self, context: PluginContext) -> PluginResult<()> {
        // Store the plugin context for later use
        self.context = Some(context);
        
        // Load settings if available
        if let Some(ctx) = &self.context {
            if let Ok(settings_str) = ctx.get_config("settings") {
                if !settings_str.is_empty() {
                    match serde_json::from_str::<Settings>(&settings_str) {
                        Ok(settings) => {
                            self.settings = settings;
                        },
                        Err(e) => {
                            log::warn!("Failed to parse settings: {}", e);
                            // Continue with default settings
                        }
                    }
                }
            }
        }
        
        // Register capabilities this plugin requires
        self.request_capabilities()?;
        
        // Check for updates if enabled
        if self.settings.auto_update_check {
            self.check_for_updates()?;
        }
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        // Save any state if needed
        if let Some(ctx) = &self.context {
            let settings_json = serde_json::to_string(&self.settings)?;
            ctx.set_config("settings", &settings_json)?;
        }
        
        Ok(())
    }
    
    fn get_commands(&self) -> Vec<Command> {
        vec![
            Command::new("analyze")
                .with_description("Analyze project structure and provide recommendations")
                .with_argument("path", "Path to project to analyze", true),
            Command::new("optimize")
                .with_description("Optimize resource usage based on analysis")
                .with_argument("target", "Target to optimize", true)
                .with_flag("aggressive", "Use aggressive optimization"),
            Command::new("report")
                .with_description("Generate a comprehensive report")
                .with_argument("output", "Output file path", false)
                .with_option("format", "Report format (markdown, html, json)", false),
        ]
    }
    
    fn execute_command(&self, command: &str, args: &[&str]) -> CommandResult {
        match command {
            "analyze" => self.cmd_analyze(args),
            "optimize" => self.cmd_optimize(args),
            "report" => self.cmd_report(args),
            _ => Err(PluginError::Command(format!("Unknown command: {}", command)).into()),
        }
    }
    
    fn get_metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "Basic Utility Plugin",
            "version": "1.0.0",
            "author": "DataScienceBioLab",
            "commands": ["analyze", "optimize", "report"],
            "settings": {
                "report_format": self.settings.report_format,
                "optimization_level": self.settings.optimization_level,
            }
        })
    }
}

impl BasicUtilityPlugin {
    /// Request the capabilities needed by this plugin
    fn request_capabilities(&self) -> PluginResult<()> {
        if let Some(ctx) = &self.context {
            // File system read capability
            let read_request = CapabilityRequest::new(
                Capability::FilesystemRead,
                "Read project files for analysis",
                &["${workspace}/**/*"],
            );
            ctx.request_capability(read_request)?;
            
            // File system write capability
            let write_request = CapabilityRequest::new(
                Capability::FilesystemWrite,
                "Write analysis reports and optimization results",
                &["${workspace}/reports/**/*"],
            );
            ctx.request_capability(write_request)?;
            
            // Network capability for update checks
            if self.settings.auto_update_check {
                let network_request = CapabilityRequest::new(
                    Capability::NetworkOutbound,
                    "Check for plugin updates",
                    &["https://api.datasciencebiolab.com/updates/*"],
                );
                ctx.request_capability(network_request)?;
            }
        }
        
        Ok(())
    }
    
    /// Check for updates
    fn check_for_updates(&self) -> PluginResult<()> {
        if let Some(ctx) = &self.context {
            // Check if we have network capability
            if !ctx.has_capability(Capability::NetworkOutbound)? {
                log::info!("Network capability not available, skipping update check");
                return Ok(());
            }
            
            // In a real plugin, we would make an HTTP request here
            // For this mock, just log that we tried
            log::info!("Checking for updates (mock implementation)");
        }
        
        Ok(())
    }
    
    /// Analyze command implementation
    fn cmd_analyze(&self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return Err(PluginError::Command("Missing required path argument".to_string()).into());
        }
        
        let path = args[0];
        
        if let Some(ctx) = &self.context {
            // Check if we have filesystem read capability
            if !ctx.has_capability(Capability::FilesystemRead)? {
                return Err(PluginError::Security("Missing filesystem read capability".to_string()).into());
            }
            
            // In a real plugin, we would analyze files here
            // For this mock, just return fake results
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let result = serde_json::json!({
                "timestamp": timestamp,
                "path": path,
                "files_analyzed": 42,
                "recommendations": [
                    "Optimize imports to reduce build time",
                    "Consider breaking large modules into smaller ones",
                    "Add more comprehensive tests for core modules"
                ]
            });
            
            return Ok(result);
        }
        
        Err(PluginError::System("Plugin not properly initialized".to_string()).into())
    }
    
    /// Optimize command implementation
    fn cmd_optimize(&self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return Err(PluginError::Command("Missing required target argument".to_string()).into());
        }
        
        let target = args[0];
        let aggressive = args.contains(&"--aggressive");
        
        if let Some(ctx) = &self.context {
            // Check if we have both read and write capabilities
            if !ctx.has_capability(Capability::FilesystemRead)? {
                return Err(PluginError::Security("Missing filesystem read capability".to_string()).into());
            }
            
            if !ctx.has_capability(Capability::FilesystemWrite)? {
                return Err(PluginError::Security("Missing filesystem write capability".to_string()).into());
            }
            
            // In a real plugin, we would perform optimization here
            // For this mock, just return fake results
            let optimization_level = if aggressive {
                "aggressive"
            } else {
                &self.settings.optimization_level
            };
            
            let result = serde_json::json!({
                "target": target,
                "optimization_level": optimization_level,
                "changes_made": 12,
                "estimated_improvement": "15%",
                "details": {
                    "memory_saved": "45MB",
                    "startup_time_improvement": "0.3s",
                    "disk_space_saved": "12MB"
                }
            });
            
            return Ok(result);
        }
        
        Err(PluginError::System("Plugin not properly initialized".to_string()).into())
    }
    
    /// Report command implementation
    fn cmd_report(&self, args: &[&str]) -> CommandResult {
        let mut output_path = None;
        let mut format = self.settings.report_format.clone();
        
        // Parse arguments
        if !args.is_empty() {
            output_path = Some(args[0]);
        }
        
        for i in 0..args.len() {
            if args[i] == "--format" && i + 1 < args.len() {
                format = args[i + 1].to_string();
            }
        }
        
        if let Some(ctx) = &self.context {
            // Check capabilities
            if !ctx.has_capability(Capability::FilesystemRead)? {
                return Err(PluginError::Security("Missing filesystem read capability".to_string()).into());
            }
            
            if output_path.is_some() && !ctx.has_capability(Capability::FilesystemWrite)? {
                return Err(PluginError::Security("Missing filesystem write capability".to_string()).into());
            }
            
            // In a real plugin, we would generate a report here
            // For this mock, just return fake results
            let result = serde_json::json!({
                "report_generated": true,
                "format": format,
                "output_path": output_path,
                "sections": [
                    "Project Overview",
                    "Performance Analysis",
                    "Resource Usage",
                    "Recommendations"
                ],
                "generation_time": "0.5s"
            });
            
            return Ok(result);
        }
        
        Err(PluginError::System("Plugin not properly initialized".to_string()).into())
    }
}

// Export a function that creates the plugin
#[no_mangle]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(BasicUtilityPlugin::default())
} 